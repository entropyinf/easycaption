// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::common::ResourceGuard;
use crate::config::{config_get, config_update};
use crate::server::{stop_transcribe, transcribe};
use tauri::Manager;
use tokio::sync::Mutex;

mod audio;
mod common;
mod config;
mod server;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            app.manage(Mutex::new(AppData::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            transcribe,
            stop_transcribe,
            config_get,
            config_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub struct AppData {
    pub audio_stream_handle: Option<ResourceGuard>,
}

impl AppData {
    fn new() -> Self {
        Self {
            audio_stream_handle: None,
        }
    }
}
