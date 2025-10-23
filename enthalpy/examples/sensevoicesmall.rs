use candle_core::Device;
use enthalpy::Res;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::load_data;
use enthalpy::audio::resample::Resampler;
use enthalpy::audio::silero_vad::{VadConfig, VadProcessor};
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use std::path::Path;
use tracing::Level;

fn main() -> Res<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=TRACE")
        .compact()
        .init();

    // transpose_file()?;
    transpose_stream()?;

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

    let mp3 = "/Users/entropy/Documents/NCE1-英音-(MP3+LRC)/005&006－Nice to Meet You..mp3";
    let mut data = load_data(mp3)?;

    let mut vad = VadProcessor::new(VadConfig::default())?;
    for mut seg in vad.process(&mut data) {
        let features = model.frontend(&mut seg.data)?;
        let encoder_out = model.encode(&features)?;
        let out = model.decode(&encoder_out)?;

        print!(
            "[{:.1}s,{:.1}s]:",
            seg.start as f32 / 1000.0,
            seg.end as f32 / 1000.0
        );
        for item in out.iter() {
            print!("{}", item.text);
        }
        println!();
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

    let mic = AudioInput::from_screen_capture_kit()?;

    while let Ok(chunk) = mic.rx.recv() {
        let resampled_chunk = resampler.apply_resample(&chunk)?;
        let segments = vad.process(&resampled_chunk);

        for mut seg in segments {
            let features = model.frontend(&mut seg.data)?;
            let encoder_out = model.encode(&features)?;
            let out = model.decode(&encoder_out)?;

            print!(
                "[{:.1}s,{:.1}s]:",
                seg.start as f32 / 1000.0,
                seg.end as f32 / 1000.0
            );
            for item in out.iter() {
                print!("{}", item.text);
            }
            println!();
        }
    }

    Ok(())
}
