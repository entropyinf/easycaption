// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::config::{config_get, config_update};
use crate::server::serve;

mod common;
mod config;
mod server;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![serve, config_get, config_update])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
