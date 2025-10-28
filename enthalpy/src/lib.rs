pub mod audio;
pub mod sense_voice_small;
pub mod util;
pub mod var_builder;
mod quantized_var_builder;
#[allow(dead_code)]
mod quantized_nn;

pub type Res<T> = anyhow::Result<T>;
