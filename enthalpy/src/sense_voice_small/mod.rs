use crate::Res;
use crate::audio::{WavFrontend, WavFrontendConfig};
use anyhow::Error;
use candle_core::{DType, Device, Module, Tensor, Var};
use candle_nn::{Embedding, VarBuilder, VarMap};
use decoder::{DecodeResult, Decoder};
use encoder::{Encoder, EncoderConfig};
use std::ops::Deref;
use std::path::Path;

mod decoder;
mod encoder;

pub struct SenseVoiceSmallConfig {
    pub cmvn_file: Box<dyn AsRef<Path>>,
    pub weight_file: Box<dyn AsRef<Path>>,
    pub tokens_file: Box<dyn AsRef<Path>>,
}

pub struct SenseVoiceSmall {
    device: Device,
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
        let weight = Tensor::randn::<_, f32>(0.0, 1.0, (num_embeddings, embedding_dim), &Device::Cpu)?;
        let embed = Embedding::new(weight, embedding_dim);

        // audio
        let frontend = WavFrontend::new(WavFrontendConfig {
            cmvn_file: Some(cfg.cmvn_file),
            ..WavFrontendConfig::default()
        })?;

        let vb = load_model(cfg.weight_file.deref(), &device)?;

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
        let decoder = Decoder::new(cfg.tokens_file.deref(), vb)?;

        Ok(Self {
            device,
            embed,
            frontend,
            encoder,
            decoder,
        })
    }

    pub fn encode(&self, x: &Tensor) -> Res<Tensor> {
        let (_, len, _) = x.dims3()?;
        let len = &Tensor::new(&[len as f32], &self.device)?;
        let (encoder_out, _) = self.encoder.forward(&x, len)?;

        Ok(encoder_out)
    }

    pub fn decode(&self, encode_out: &Tensor) -> Res<Vec<DecodeResult>> {
        self.decoder.decode(encode_out)
    }

    pub fn embed(&self, x: &Tensor) -> Res<Tensor> {
        Ok(self.embed.forward(&x)?)
    }

    pub fn frontend(&self, waveform: &mut [f32]) -> Res<Tensor> {
        let cpu = Device::Cpu;
        let speech = self
            .frontend
            .extract_features_f32(waveform)
            .map_err(|e| Error::msg(e.to_string()))?
            .to_device(&cpu)?
            .unsqueeze(0)?;

        let language_query = Tensor::new(&[[0i64]], &cpu)?;
        let language_query = self.embed(&language_query)?;

        let text_norm_query = Tensor::new(&[[15i64]], &cpu)?;
        let text_norm_query = self.embed(&text_norm_query)?;

        let event_emo_query = Tensor::new(&[[1i64, 2]], &cpu)?;
        let event_emo_query = self.embed(&event_emo_query)?;

        let speech = Tensor::cat(&[&text_norm_query, &speech], 1)?;
        let input_query = Tensor::cat(&[&language_query, &event_emo_query], 1)?;
        let speech = Tensor::cat(&[&input_query, &speech], 1)?;

        let speech = speech.to_device(&self.device)?;

        Ok(speech)
    }
}

pub fn load_model(path: &dyn AsRef<Path>, device: &Device) -> Res<VarBuilder<'static>> {
    let tensors = candle_core::pickle::read_all(path)?;
    let vm = VarMap::new();

    let mut vm_data_map = vm.data().lock().map_err(|e| Error::msg(e.to_string()))?;
    for (name, tensor) in tensors.iter() {
        vm_data_map.insert(
            String::from(name),
            Var::from_tensor(&tensor.to_device(&device)?)?,
        );
    }

    Ok(VarBuilder::from_varmap(&vm, DType::F32, &device))
}
