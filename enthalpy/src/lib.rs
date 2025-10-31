pub mod audio;
mod config;
#[allow(dead_code)]
mod quantized_nn;
mod quantized_var_builder;
pub mod sense_voice_small;
pub mod util;
pub mod var_builder;

pub use config::ConfigRefresher;

pub type Res<T> = anyhow::Result<T>;
