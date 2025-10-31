import { invoke } from "@tauri-apps/api/core"
import { FileInfo, TransposeConfig } from "./types"

export async function updateTranscribeConfig(param: TransposeConfig): Promise<void> {
    return await invoke<void>('update_transcribe_config', { patch: param })
}

export async function getTranscribeConfig(): Promise<TransposeConfig> {
    return await invoke<any>('get_transcribe_config')
}

export async function getDevices(): Promise<[{ host: string, device: string }]> {
    return await invoke<any>('get_devices')
}

export async function checkRequiredFiles(modelDir: string): Promise<FileInfo[]> {
    return await invoke<FileInfo[]>('get_required_files', { modelDir: modelDir })
}

export async function downloadRequiredFile(modelDir: string, fileName: string): Promise<void> {
    return await invoke<void>('download_required_file', { modelDir: modelDir, fileName: fileName })
}

export async function stopDownloadRequiredFile(fileName: string): Promise<void> {
    return await invoke<void>('stop_download_required_file', { fileName: fileName })
}