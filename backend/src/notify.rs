use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct Notifier {
    app_handle: AppHandle,
}

impl Notifier {
    fn new(app_handle: AppHandle) -> Notifier {
        Self { app_handle }
    }
}

static INSTANCE: OnceCell<Notifier> = OnceCell::const_new();

#[derive(Serialize, Clone)]
struct Message {
    r#type: MessageType,
    content: String,
}

#[derive(Serialize, Clone)]
pub enum MessageType {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl Notifier {
    pub async fn init(app_handle: AppHandle) -> &'static Notifier {
        INSTANCE
            .get_or_init(|| async { Notifier::new(app_handle) })
            .await
    }

    pub async fn get() -> &'static Notifier {
        INSTANCE.get().unwrap()
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        self.app_handle.emit(event, payload)
    }

    pub fn notify(&self, r#type: MessageType, message: &str) {
        let _ = self
            .app_handle
            .emit(
                "notify",
                Message {
                    r#type,
                    content: message.to_string(),
                },
            )
            .is_ok();
    }

    pub fn error(&self, message: &str){
        self.notify(MessageType::Error, message);
    }

    pub fn info(&self, message: &str){
        self.notify(MessageType::Info, message);
    }
}
