use crate::notify::Notifier;
use chrono::Utc;
use enthalpy::sense_voice_small::SenseVoiceSmall;
use enthalpy::util::modelscope::Progress;
use enthalpy::Res;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use tokio::sync::{OnceCell, RwLock};
use tokio::task::JoinHandle;
use tracing::event;

pub struct Downloads {
    downloads: RwLock<HashMap<String, JoinHandle<Res<()>>>>,
}

impl Downloads {
    pub async fn get() -> &'static Downloads {
        static INSTANCE: OnceCell<Downloads> = OnceCell::const_new();
        INSTANCE.get_or_init(|| async { Downloads::new() }).await
    }

    fn new() -> Downloads {
        Downloads {
            downloads: RwLock::new(HashMap::new()),
        }
    }

    pub async fn download(&self, model_dir: String, file_name: String) -> Res<()> {
        let handle = tokio::spawn(download(model_dir.clone(), file_name.clone()));
        self.downloads.write().await.insert(file_name, handle);
        Ok(())
    }

    pub async fn stop_download(&self, file_name: String) -> Res<()> {
        if let Some(handle) = self.downloads.write().await.remove(&file_name) {
            handle.abort();
        }

        Ok(())
    }
}

async fn download(model_dir: String, file_name: String) -> Res<()> {
    let downloader = DownloadProgress {
        file_name: file_name.clone(),
        notifier: Notifier::get().await.clone(),
        size: AtomicU64::new(0),
        position: AtomicU64::new(0),
        last_report: AtomicI64::new(0),
    };
    let repo = SenseVoiceSmall::model_repo(&model_dir).await;
    repo.download_with_progress(&file_name, downloader).await
}

struct DownloadProgress {
    file_name: String,
    notifier: Notifier,
    size: AtomicU64,
    position: AtomicU64,
    last_report: AtomicI64,
}

#[derive(Serialize, Clone)]
struct ProgressInfo {
    file_name: String,
    size: u64,
    position: u64,
}

impl Progress for DownloadProgress {
    fn inc(&self, delta: u64) {
        self.position.fetch_add(delta, Ordering::Relaxed);
        self.report();
    }

    fn set_position(&self, pos: u64) {
        self.position.store(pos, Ordering::Relaxed);
        self.report();
    }

    fn finish(&self) {
        self.position
            .store(self.size.load(Ordering::Relaxed), Ordering::Relaxed);
        self.report();
    }

    fn set_length(&self, len: u64) {
        self.size.store(len, Ordering::Relaxed);
        self.report();
    }
}

impl DownloadProgress {
    fn report(&self) {
        let now = Utc::now().timestamp_millis();
        let before = self.last_report.load(Ordering::Relaxed);
        if now - before < 500 {
            return;
        }

        event!(
            tracing::Level::DEBUG,
            "download_progress, file name: {}, position: {}, total: {}",
            self.file_name,
            self.position.load(Ordering::Relaxed),
            self.size.load(Ordering::Relaxed)
        );

        self.last_report.swap(now, Ordering::Relaxed);

        let progress = ProgressInfo {
            file_name: self.file_name.clone(),
            size: self.size.load(Ordering::Relaxed),
            position: self.position.load(Ordering::Relaxed),
        };

        let _ = self.notifier.emit("download_progress", progress).is_ok();
    }
}
