use candle_core::Device;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use enthalpy::Res;
use serde_json::json;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, LazyLock, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

pub type CmdResult<T = ()> = Result<T, String>;

#[tauri::command]
pub fn transcribe(app: AppHandle) -> CmdResult<bool> {
    if let Ok(mut guard) = State::global().lock(){
        if guard.handle.is_some(){
            guard.handle = None;
            return Ok(false)
        }


        let window = app.get_window("caption").ok_or("No window found")?;

        let tx = Transpose::new()
            .map_err(|e| e.to_string())?
            .run(window)
            .map_err(|e| e.to_string())?;

        guard.handle = Some(tx);

        return Ok(true)
    }

    Ok(false)

}


struct State {
    handle: Option<Sender<()>>,
}

impl State {
    pub fn global() -> &'static Mutex<State> {
        static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State { handle: None }));
        &STATE
    }
}

struct Transpose {
    model: SenseVoiceSmall,
    mic: AudioInput,
}

impl Transpose {
    pub fn new() -> Res<Transpose> {
        let device = Device::Cpu;

        let model_path =
            Path::new("/Users/entropy/.cache/modelscope/hub/models/iic/SenseVoiceSmall");

        let cfg = SenseVoiceSmallConfig {
            cmvn_file: model_path.join("am.mvn"),
            weight_file: model_path.join("model.pt"),
            tokens_file: model_path.join("tokens.json"),
            vad: Some(VadConfig::default()),
            resample: Some((48000, 16000)),
        };

        let model = SenseVoiceSmall::new(cfg, &device)?;
        let mic = AudioInput::from_screen_capture_kit()?;

        Ok(Transpose { model, mic })
    }

    pub fn run(self, window: tauri::Window) -> Res<Sender<()>> {
        let (tx, rx) = mpsc::channel::<()>();

        println!("Transpose started");

        thread::spawn(|| {
            let rx = rx;
            let window = window;
            let mut transpose = self;

            loop {
                if let Ok(mut chunk) = transpose.mic.rx.recv() {
                    if let Err(e) = transpose.transpose(&window, &mut chunk) {
                        println!("Error transcribing: {}", e);
                        break;
                    }
                }

                if let Err(mpsc::TryRecvError::Disconnected) = rx.try_recv() {
                    println!("Transpose exiting...");
                    break;
                }
            }
        });

        Ok(tx)
    }

    fn transpose(&mut self, window: &tauri::Window, chunk: &mut [f32]) -> Res<()> {
        let tokens = self.model.transpose(chunk)?;

        for token in tokens {
            let start = token.start;
            let end = token.end;
            let text = token.text;

            let emit_out = window.emit(
                "caption",
                json!({
                    "start": start,
                    "end": end,
                    "text": text,
                }),
            );

            if let Err(e) = emit_out {
                println!("Error emitting event: {}", e);
                break;
            }
        }

        Ok(())
    }
}
