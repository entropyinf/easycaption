use candle_core::Device;
use enthalpy::Res;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use enthalpy::util::modelscope::ModelScopeRepo;
use tracing::Level;

#[tokio::main]
async fn main() -> Res<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=TRACE")
        .compact()
        .init();

    transpose_stream().await?;

    Ok(())
}

async fn transpose_stream() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let repo = ModelScopeRepo::new(
        "iic/SenseVoiceSmall",
        "/Users/entropy/.cache/modelscope/hub/models/",
    );

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: repo.get("am.mvn").await?,
        weight_file: repo.get("model.pt").await?,
        tokens_file: repo.get("tokens.json").await?,
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
