// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::notify::Notifier;
use crate::transpose::TransposeService;
use tauri::Manager;
use tracing::Level;

mod cmds;
mod notify;
mod transpose;
mod download;
mod config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("enthalpy=DEBUG")
        .with_env_filter("backend=TRACE")
        .compact()
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.app_handle().clone();
            tokio::spawn(async move {
                TransposeService::init(handle.clone()).await;
                Notifier::init(handle).await;
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handlers())
        .run(tauri::generate_context!())
        .expect("error while running backend application");
}

pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        cmds::update_transcribe_config,
        cmds::get_transcribe_config,
        cmds::get_devices,
        cmds::get_required_files,
        cmds::download_required_file,
        cmds::stop_download_required_file
    ]
}