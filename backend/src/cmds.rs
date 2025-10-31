use crate::download::Downloads;
use crate::transpose::{TransposeConfig, TransposeService};
use enthalpy::audio::input::{AudioInput, HostDevice};
use enthalpy::sense_voice_small::SenseVoiceSmall;
use enthalpy::util::modelscope::FileInfo;
use serde_json::Value;

pub type CmdResult<T = ()> = Result<T, String>;

#[tauri::command]
pub async fn update_transcribe_config(patch: Value) -> CmdResult<()> {
    TransposeService::get()
        .await
        .update_config(patch)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_transcribe_config() -> CmdResult<TransposeConfig> {
    Ok(TransposeService::get().await.get_config().await)
}

#[tauri::command]
pub async fn get_required_files(model_dir: String) -> CmdResult<Vec<FileInfo>> {
    SenseVoiceSmall::get_required_files(&model_dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_required_file(model_dir: String, file_name: String) -> CmdResult<()> {
    Downloads::get()
        .await
        .download(model_dir, file_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_download_required_file(file_name: String) -> CmdResult<()> {
    Downloads::get()
        .await
        .stop_download(file_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_devices() -> CmdResult<Vec<HostDevice>> {
    AudioInput::all_inputs().map_err(|e| e.to_string())
}
