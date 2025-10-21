use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use enthalpy::Res;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::load_data;
use enthalpy::audio::resample::Resampler;
use enthalpy::audio::silero_vad::{VadConfig, VadProcessor};
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use tracing::{Level, event};

fn main() -> Res<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=TRACE")
        .compact()
        .init();

    ctc()?;

    Ok(())
}

fn ctc() -> Res<()> {
    let mut cmd = Command::new("pwd");
    let output = cmd.output()?;
    let pwd = String::from_utf8(output.stdout)?;
    println!("pwd: {}", pwd);

    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let model_path = Path::new("/Users/entropy/.cache/modelscope/hub/models/iic/SenseVoiceSmall");

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: Box::new(model_path.join("am.mvn")),
        weight_file: Box::new(model_path.join("model.pt")),
        tokens_file: Box::new(model_path.join("tokens.json")),
    };

    let model = SenseVoiceSmall::new(cfg, &device)?;
    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(
            &["/Users/entropy/workspace/easycaption/enthalpy/encoder_out.safetensor"],
            DType::F32,
            &device,
        )?
    };
    let encoder_out = vb.get((1, 124, 512), "encoder_out")?;
    println!("encoder_out: {:?}", encoder_out);
    let out = model.decode(&encoder_out)?;

    for item in out.iter() {
        println!(
            "[{}s,{}s]:{}",
            item.timestamp.0 as f32 / 1000.0,
            item.timestamp.1 as f32 / 1000.0,
            item.text
        );
    }

    Ok(())
}

fn transpose_file() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let model_path = Path::new("/Users/entropy/.cache/modelscope/hub/models/iic/SenseVoiceSmall");

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: Box::new(model_path.join("am.mvn")),
        weight_file: Box::new(model_path.join("model.pt")),
        tokens_file: Box::new(model_path.join("tokens.json")),
    };

    let model = SenseVoiceSmall::new(cfg, &device)?;

    let mp3 = "/Users/entropy/Documents/NCE1-英音-(MP3+LRC)/001&002－Excuse Me.mp3";
    let mut data = load_data(mp3)?;

    let start = Instant::now();
    let features = model.frontend(&mut data)?;
    let encoder_out = model.encode(&features)?;
    let out = model.decode(&encoder_out)?;
    let cost = start.elapsed();

    println!("cost: {:?}", cost);

    for item in out.iter() {
        println!(
            "[{}s,{}s]:{}",
            item.timestamp.0 as f32 / 1000.0,
            item.timestamp.1 as f32 / 1000.0,
            item.text
        );
    }

    Ok(())
}

fn transpose_stream() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let model_path = Path::new("/Users/entropy/.cache/modelscope/hub/models/iic/SenseVoiceSmall");

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: Box::new(model_path.join("am.mvn")),
        weight_file: Box::new(model_path.join("model.pt")),
        tokens_file: Box::new(model_path.join("tokens.json")),
    };

    let model = SenseVoiceSmall::new(cfg, &device)?;

    let mut vad = VadProcessor::new(VadConfig::default())?;

    let resampler = Resampler::new(48000, 16000)?;

    let mic = AudioInput::new()?;

    while let Ok(chunk) = mic.rec.recv() {
        let resampled_chunk = resampler.apply_resample(&chunk)?;
        let segment = vad.process(&resampled_chunk);

        if let Some(seg) = segment {
            let out = match seg.data {
                None => Vec::with_capacity(0),
                Some(mut data) => {
                    event!(Level::TRACE, "in frontend");
                    let features = model.frontend(&mut data)?;

                    event!(Level::TRACE, "encoding");
                    let encoder_out = model.encode(&features)?;

                    event!(Level::TRACE, "decoding");
                    let o = model.decode(&encoder_out)?;

                    event!(Level::TRACE, "decoded");
                    o
                }
            };

            print!(
                "[{}s,{}s]:",
                seg.start as f32 / 1000f32,
                seg.end as f32 / 1000f32
            );
            for item in out.iter() {
                print!("{}", item.text);
            }
            println!();
        }
    }

    Ok(())
}
