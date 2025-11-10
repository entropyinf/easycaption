use std::mem;
use crate::config::ConfigSync;
use crate::notify::Notifier;
use anyhow::bail;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig, Token};
use enthalpy::{ConfigRefresher, Res};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::{OnceCell, RwLock};
use tokio::{select, time};
use tracing::event;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TransposeConfig {
    enable: bool,
    input_host: String,
    input_device: String,
    realtime: bool,
    realtime_rate: u64,
    model_config: SenseVoiceSmallConfig,
}

pub struct TransposeService {
    config: RwLock<ConfigSync<TransposeConfig>>,
}

static INSTANCE: OnceCell<TransposeService> = OnceCell::const_new();

impl TransposeService {
    pub async fn init(app_handle: AppHandle) -> &'static TransposeService {
        INSTANCE
            .get_or_init(|| async { TransposeService::new(app_handle).await.unwrap() })
            .await
    }

    pub async fn get() -> &'static TransposeService {
        INSTANCE.get().unwrap()
    }

    pub async fn get_config(&self) -> TransposeConfig {
        self.config.read().await.curr().clone()
    }

    pub async fn new(app_handle: AppHandle) -> Res<Self> {
        let model_dir = PathBuf::from("/Users/entropy/.cache/modelscope/hub/models");

        let config = TransposeConfig {
            enable: false,
            input_host: String::default(),
            input_device: String::default(),
            realtime: false,
            realtime_rate: 800,
            model_config: SenseVoiceSmallConfig {
                model_dir,
                vad: VadConfig::default(),
                resample: Some((48000, 16000)),
                use_gpu: false,
            },
        };

        let config = ConfigSync::new(config);

        Transpose::init(config.clone(), app_handle).await?;

        Ok(TransposeService {
            config: RwLock::new(config),
        })
    }

    pub async fn update_config(&self, patch: Value) -> Res<()> {
        event!(tracing::Level::INFO, "Check lock {}", patch);
        let mut config = self.config.write().await;
        let old = config.curr().clone();
        let mut new = serde_json::to_value(old)?;
        json_patch::merge(&mut new, &patch);
        let new = serde_json::from_value::<TransposeConfig>(new)?;

        config.update_sync(new).await?;
        event!(tracing::Level::INFO, "Release lock {}", patch);

        Ok(())
    }
}

struct Transpose {
    config: ConfigSync<TransposeConfig>,
    model: Option<SenseVoiceSmall>,
    input: Option<AudioInput>,
    pcm_tx: Sender<Vec<f32>>,
    app_handle: AppHandle,
    notifier: Notifier,
    realtime_interval: time::Interval,
}

impl Transpose {
    async fn init(config: ConfigSync<TransposeConfig>, app_handle: AppHandle) -> Res<()> {
        tokio::spawn(async move {
            let (pcm_tx, mut pcm_rx) = channel::<Vec<f32>>(100);
            let notifier = Notifier::get().await.clone();
            let mut transpose = Transpose {
                config,
                model: None,
                input: None,
                pcm_tx,
                app_handle,
                notifier,
                realtime_interval: time::interval(Duration::from_hours(999999)),
            };

            loop {
                select! {
                    Ok(_) = transpose.config.wait_update() => {
                        event!(tracing::Level::DEBUG, "Received config");
                        transpose.update_config().await;
                    },
                    Some(mut pcm) = pcm_rx.recv() => {
                        if let Err(e) = transpose.transpose(&mut pcm).await{
                            Notifier::get().await.error(&e.to_string());
                        }
                    },
                    _ = transpose.realtime_interval.tick() => {
                        let _ = transpose.transpose_vad_cache().await;
                    },
                    else => {
                        return;
                    },
                }
            }
        });

        Ok(())
    }

    async fn transpose(&mut self, pcm: &mut [f32]) -> Res<()> {
        if self.model.is_none() {
            event!(tracing::Level::DEBUG, "Model is none");
            return Ok(());
        }
        let model = self.model.as_mut().unwrap();
        let mut segments = model.segment(pcm)?;
        let tokens: Vec<Token> = model.transpose(&mut segments)?;

        self.emit_tokens(tokens);

        Ok(())
    }

    async fn transpose_vad_cache(&mut self) -> Res<()> {
        if self.model.is_none() {
            event!(tracing::Level::DEBUG, "Model is none");
            return Ok(());
        }
        let model = self.model.as_mut().unwrap();

        if self.config.curr().realtime {
            let tokens = model.transpose_vad_cache()?;
            self.emit_tokens(tokens);
        }

        Ok(())
    }

    fn emit_tokens(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            event!(tracing::Level::DEBUG, "Token: {}", token.text);

            let emit_out = self.app_handle.emit(
                "caption",
                json!({
                    "start": token.start,
                    "end": token.end,
                    "text": token.text,
                }),
            );

            if let Err(e) = emit_out {
                event!(tracing::Level::ERROR, "Error emitting event {}", e);
            }
        }
    }

    async fn update_config(&mut self) {
        match self.do_update_config().await {
            Ok(_) => {
                let _ = self.config.finish(true).await;
            }
            Err(e) => {
                Notifier::get().await.error(&e.to_string());
                let _ = self.config.finish(false).await;
            }
        }
    }

    async fn do_update_config(&mut self) -> Res<()> {
        let new = self.config.fresh().clone();
        if !new.enable {
            self.model.take();
            self.input.take();
            return Ok(());
        }

        let old = self.config.curr().clone();

        let should_reload_input = new.input_host != old.input_host
            || new.input_device != old.input_device
            || self.input.is_none();

        if should_reload_input {
            let pcm_tx = self.pcm_tx.clone();
            let mut input = AudioInput::with_host_device(&new.input_host, &new.input_device)?;
            let rx = input.play()?;
            tokio::task::spawn_blocking(move || loop {
                match rx.recv() {
                    Ok(pcm) => {
                        if let Err(e) = pcm_tx.blocking_send(pcm) {
                            event!(tracing::Level::ERROR, "Error sending pcm: {}", e);
                            break;
                        }
                    }
                    Err(_) => {
                        event!(tracing::Level::DEBUG, "Close channel");
                        break;
                    }
                }
            });
            self.input.replace(input);
        };

        match (old.realtime, new.realtime) {
            (true, false) => {
                let mut interval= time::interval(Duration::from_hours(999999));
                mem::swap(&mut self.realtime_interval, &mut interval);
            }
            (_, true) => {
                let mut interval= time::interval(Duration::from_millis(new.realtime_rate));
                mem::swap(&mut self.realtime_interval, &mut interval);
            }
            _ => {}
        }

        let device_changed = old.model_config.use_gpu != new.model_config.use_gpu;
        let model_dir_changed = old.model_config.model_dir != new.model_config.model_dir;
        let should_reload = model_dir_changed || device_changed;

        match (&mut self.model, should_reload) {
            (None, _) | (Some(_), true) => {
                let failed =
                    SenseVoiceSmall::check_required_files(&new.model_config.model_dir).await;

                if failed {
                    self.notifier.error("Missing required files");
                    bail!("Missing required files");
                }

                event!(tracing::Level::DEBUG, "Loading model");
                self.notifier.info("Loading");
                match SenseVoiceSmall::with_config(new.model_config).await {
                    Ok(new_model) => {
                        self.model.replace(new_model);
                    }
                    Err(e) => {
                        event!(tracing::Level::ERROR, "Error loading model {}", e);
                        self.notifier.error(&format!("Error loading model {}", e));
                    }
                }
            }
            (Some(model), false) => {
                event!(tracing::Level::DEBUG, "Refreshing model");
                model.refresh(&new.model_config, &old.model_config)?;
            }
        }

        Ok(())
    }
}
