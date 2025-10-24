use candle_core::Device;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use enthalpy::Res;
use std::path::Path;
use tracing::Level;

fn main() -> Res<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=TRACE")
        .compact()
        .init();

    transpose_stream()?;

    Ok(())
}

fn transpose_stream() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let model_path = Path::new("/Users/entropy/.cache/modelscope/hub/models/iic/SenseVoiceSmall");

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: model_path.join("am.mvn"),
        weight_file: model_path.join("model.pt"),
        tokens_file: model_path.join("tokens.json"),
        vad: Some(VadConfig::default()),
        resample: Some((48000, 16000)),
    };

    let mut model = SenseVoiceSmall::new(cfg, &device)?;
    let mic = AudioInput::from_screen_capture_kit()?;

    while let Ok(mut chunk) = mic.rx.recv() {
        let tokens = model.transpose(&mut chunk)?;

        for token in tokens {
            println!(
                "[{:.1}s,{:.1}s]:{}",
                token.start as f32 / 1000.0,
                token.end as f32 / 1000.0,
                token.text
            );
        }
    }

    Ok(())
}
