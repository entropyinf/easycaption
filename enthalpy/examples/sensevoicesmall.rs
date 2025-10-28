use candle_core::Device;
use enthalpy::Res;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::load_audio;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use enthalpy::util::modelscope::ModelScopeRepo;
use tokio::time::Instant;
use tracing::Level;

#[tokio::main]
async fn main() -> Res<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=TRACE")
        .compact()
        .init();

    transpose_stream().await?;
    // transpose_file().await?;

    Ok(())
}

async fn transpose_file() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let (mut data, sample_rate) =
        load_audio("/Users/entropy/Documents/NCE1-英音-(MP3+LRC)/001&002－Excuse Me.mp3")?;

    let repo = ModelScopeRepo::new(
        "iic/SenseVoiceSmall",
        "/Users/entropy/.cache/modelscope/hub/models/",
    );

    let q_repo = ModelScopeRepo::new(
        "lovemefan/SenseVoiceGGUF",
        "/Users/entropy/.cache/modelscope/hub/models/",
    );

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: repo.get("am.mvn").await?,
        // weight_file: q_repo.get("sense-voice-small-q8_0.gguf").await?,
        weight_file: repo.get("model.pt").await?,
        tokens_file: repo.get("tokens.json").await?,
        vad: Some(VadConfig::default()),
        resample: Some((sample_rate, 16000)),
    };

    let mut model = SenseVoiceSmall::new(cfg, &device)?;

    let start = Instant::now();
    let tokens = model.transpose(&mut data)?;
    println!("{:.2}", start.elapsed().as_secs_f32());
    for token in tokens {
        println!(
            "[{:.1}s,{:.1}s]:{}",
            token.start as f32 / 1000.0,
            token.end as f32 / 1000.0,
            token.text
        );
    }

    Ok(())
}

async fn transpose_stream() -> Res<()> {
    // let device = Device::new_metal(0)?;
    let device = Device::Cpu;

    let repo = ModelScopeRepo::new(
        "iic/SenseVoiceSmall",
        "/Users/entropy/.cache/modelscope/hub/models/",
    );

    let mic = AudioInput::from_screen_capture_kit()?;
    let sample_rate = mic.config.sample_rate().0;

    let cfg = SenseVoiceSmallConfig {
        cmvn_file: repo.get("am.mvn").await?,
        weight_file: repo.get("model.pt").await?,
        tokens_file: repo.get("tokens.json").await?,
        vad: Some(VadConfig::default()),
        resample: Some((sample_rate, 16000)),
    };

    let mut model = SenseVoiceSmall::new(cfg, &device)?;

    mic.record()?;
    while let Ok(mut chunk) = mic.rx.recv() {
        let start = Instant::now();
        let tokens = model.transpose(&mut chunk)?;

        for token in tokens {
            print!("cost:{:.2},", start.elapsed().as_secs_f32());
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
