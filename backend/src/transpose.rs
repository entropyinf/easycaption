use crate::notify::Notifier;
use anyhow::bail;
use enthalpy::audio::input::AudioInput;
use enthalpy::audio::silero_vad::VadConfig;
use enthalpy::sense_voice_small::{SenseVoiceSmall, SenseVoiceSmallConfig};
use enthalpy::{ConfigRefresher, Res};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::{watch, OnceCell, RwLock};
use tokio::time::Instant;
use tracing::event;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TransposeConfig {
    enable: bool,
    input_host: String,
    input_device: String,
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
        self.config.read().await.get_curr()
    }

    pub async fn new(app_handle: AppHandle) -> Res<Self> {
        let model_dir = PathBuf::from("/Users/entropy/.cache/modelscope/hub/models");

        let config = TransposeConfig {
            enable: false,
            input_host: String::default(),
            input_device: String::default(),
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
        event!(tracing::Level::INFO, "Updating config {}", patch);

        let mut config = self.config.write().await;
        let old = config.get_curr();
        let mut new = serde_json::to_value(old.clone())?;
        json_patch::merge(&mut new, &patch);
        let new = serde_json::from_value::<TransposeConfig>(new)?;

        config.update(new)?;
        config.watch_curr().await?;

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
            };

            loop {
                select! {
                    Ok(new) = transpose.config.watch_new() => {
                        event!(tracing::Level::DEBUG, "Received config");
                        transpose.update_config(new).await;
                    },
                    Some(mut pcm) = pcm_rx.recv() => {
                        if let Err(e) = transpose.transpose(&mut pcm).await{
                            Notifier::get().await.error(&e.to_string());
                        }
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

        let window = self.app_handle.get_window("caption");
        if window.is_none() {
            return Ok(());
        }
        let window = window.unwrap();

        let model = self.model.as_mut().unwrap();
        let tokens = model.transpose(pcm)?;
        for token in tokens {
            event!(tracing::Level::DEBUG, "Token: {}", token.text);

            let emit_out = window.emit(
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

        Ok(())
    }

    async fn update_config(&mut self, new: TransposeConfig) {
        match self.do_update_config(new).await {
            Ok(_) => {
                let _ = self.config.commit(true).await;
            }
            Err(e) => {
                Notifier::get().await.error(&e.to_string());
                let _ = self.config.commit(true).await;
            }
        }
    }

    async fn do_update_config(&mut self, new: TransposeConfig) -> Res<()> {
        if !new.enable {
            self.model.take();
            self.input.take();
            return Ok(());
        }

        let old = self.config.get_curr();

        let should_reload_input = new.input_host != old.input_host
            || new.input_device != old.input_device
            || self.input.is_none();

        if should_reload_input {
            self.notifier.info(&format!(
                "Record input {} - {}",
                new.input_host, new.input_device
            ));
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
                self.notifier.info("Loading model");
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

#[derive(Clone)]
struct ConfigSync<T: Clone> {
    curr_rx: watch::Receiver<T>,
    curr_tx: watch::Sender<T>,
    new_rx: watch::Receiver<T>,
    new_tx: watch::Sender<T>,
}

impl<T: Clone + Send + Sync + 'static> ConfigSync<T> {
    pub fn new(initial: T) -> Self {
        println!("Creating config sync");
        let (curr_tx, curr_rx) = watch::channel(initial.clone());
        let (new_tx, new_rx) = watch::channel(initial);
        Self {
            curr_rx,
            curr_tx,
            new_rx,
            new_tx,
        }
    }

    pub fn update(&self, value: T) -> Res<()> {
        self.new_tx.send(value)?;
        Ok(())
    }

    /// Commit the new value to the current value
    pub async fn commit(&mut self, success: bool) -> Res<()> {
        if success {
            let new = self.get_new();
            self.curr_tx.send(new)?;
        } else {
            self.curr_rx.mark_changed();
        }
        Ok(())
    }

    pub async fn watch_new(&mut self) -> Res<T> {
        self.new_rx.changed().await?;
        println!("New value received");
        Ok(self.get_new())
    }

    pub fn get_new(&mut self) -> T {
        println!("Getting new value");
        self.new_rx.borrow().clone()
    }

    pub async fn watch_curr(&mut self) -> Res<T> {
        self.curr_rx.changed().await?;
        println!("Current value received");
        Ok(self.get_curr())
    }

    pub fn get_curr(&self) -> T {
        println!("Getting current value");
        self.curr_rx.borrow().clone()
    }
}
