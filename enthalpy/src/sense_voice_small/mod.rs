use crate::audio::resample::Resampler;
use crate::audio::silero_vad::{VadConfig, VadProcessor};
use crate::audio::{WavFrontend, WavFrontendConfig};
use crate::var_builder::VarBuilder;
use crate::Res;
use anyhow::Error;
use candle_core::{Device, Module, Tensor};
use candle_nn::Embedding;
use decoder::{Decoder, Token};
use encoder::{Encoder, EncoderConfig};
use std::path::PathBuf;

mod decoder;
mod encoder;

pub struct SenseVoiceSmallConfig {
    pub cmvn_file: PathBuf,
    pub weight_file: PathBuf,
    pub tokens_file: PathBuf,
    pub vad: Option<VadConfig>,
    pub resample: Option<(u32, u32)>,
}

pub struct SenseVoiceSmall {
    device: Device,
    resampler: Option<Resampler>,
    vad: Option<VadProcessor>,
    embed: Embedding,
    frontend: WavFrontend,
    encoder: Encoder,
    decoder: Decoder,
}

impl SenseVoiceSmall {
    pub fn new(cfg: SenseVoiceSmallConfig, device: &Device) -> Res<Self> {
        let device = device.clone();

        // embedding
        let lid_dict_len = 7;
        let textnorm_dict_len = 2;
        let embedding_dim = 560;
        let num_embeddings = 7 + lid_dict_len + textnorm_dict_len;
        let weight =
            Tensor::randn::<_, f32>(0.0, 1.0, (num_embeddings, embedding_dim), &Device::Cpu)?;
        let embed = Embedding::new(weight, embedding_dim);

        // audio frontend
        let frontend = WavFrontend::new(WavFrontendConfig {
            cmvn_file: Some(cfg.cmvn_file),
            ..WavFrontendConfig::default()
        })?;

        let vb = VarBuilder::from_file(&cfg.weight_file, &device)?;

        // encoder
        let encoder = Encoder::new_with_config(
            EncoderConfig {
                input_size: embedding_dim,
                output_size: 512,
                attention_heads: 4,
                linear_units: 2048,
                num_blocks: 50,
                tp_blocks: 20,
                dropout_rate: 0.1,
                attention_dropout_rate: 0.1,
                kernel_size: 11,
                sanm_shfit: 0,
                normalize_before: true,
                concat_after: false,
            },
            vb.clone(),
        )?;

        // decoder
        let decoder = Decoder::new(&cfg.tokens_file, vb)?;

        // vad
        let vad = match cfg.vad {
            Some(cfg) if cfg.enable => Some(VadProcessor::new(cfg)?),
            _ => None,
        };

        // resample
        let resampler = match cfg.resample {
            Some((from, to)) => Some(Resampler::new(from, to)?),
            None => None,
        };

        Ok(Self {
            device,
            resampler,
            vad,
            embed,
            frontend,
            encoder,
            decoder,
        })
    }

    pub fn transpose(&mut self, waveform: &mut [f32]) -> Res<Vec<Token>> {
        let waveform = match &self.resampler {
            None => waveform,
            Some(sampler) => &mut sampler.apply_resample(&waveform)?,
        };

        let out = match &mut self.vad {
            None => {
                let mut out = Vec::with_capacity(1);
                let text = self.process(waveform)?;
                out.push(Token {
                    text,
                    start: 0,
                    end: 0,
                });
                out
            }
            Some(vad) => {
                let segments = vad.process(&waveform);

                let mut out = Vec::with_capacity(segments.len());
                for mut seg in segments {
                    let text = self.process(&mut seg.data)?;
                    out.push(Token {
                        text,
                        start: seg.start,
                        end: seg.end,
                    });
                }
                out
            }
        };

        Ok(out)
    }
    fn process(&mut self, waveform: &mut [f32]) -> Res<String> {
        let mut text = String::with_capacity(1024);
        let features = self.frontend(waveform)?;
        let encoder_out = self.encoder.forward(&features)?;
        let out = self.decoder.decode(&encoder_out)?;

        for item in out.iter() {
            text += &item.text;
        }

        Ok(text)
    }

    fn frontend(&self, waveform: &mut [f32]) -> Res<Tensor> {
        let cpu = Device::Cpu;
        let speech = self
            .frontend
            .extract_features_f32(waveform)
            .map_err(|e| Error::msg(e.to_string()))?
            .to_device(&cpu)?
            .unsqueeze(0)?;

        let language_query = Tensor::new(&[[0i64]], &cpu)?;
        let language_query = self.embed.forward(&language_query)?;

        let text_norm_query = Tensor::new(&[[15i64]], &cpu)?;
        let text_norm_query = self.embed.forward(&text_norm_query)?;

        let event_emo_query = Tensor::new(&[[1i64, 2]], &cpu)?;
        let event_emo_query = self.embed.forward(&event_emo_query)?;

        let speech = Tensor::cat(&[&text_norm_query, &speech], 1)?;
        let input_query = Tensor::cat(&[&language_query, &event_emo_query], 1)?;
        let speech = Tensor::cat(&[&input_query, &speech], 1)?;

        let speech = speech.to_device(&self.device)?;

        Ok(speech)
    }
}
