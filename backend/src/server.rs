#[tauri::command]
pub async fn transcribe() -> Result<(), ()> {
    Ok(())
}

#[tauri::command]
pub async fn stop_transcribe() -> Result<(), ()> {
    Ok(())
}