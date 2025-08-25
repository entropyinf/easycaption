use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {

}


#[tauri::command]
pub fn config_update(config: Config) {
    println!("更新配置: {:?}", config)
}

#[tauri::command]
pub fn config_get() -> Config {
    Config::default()
}
