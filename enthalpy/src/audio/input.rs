use crate::Res;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{HostId, Stream};
use std::sync::mpsc::Receiver;

pub struct AudioInput {
    _stream: Stream,
    pub rec: Receiver<Vec<f32>>,
}

impl AudioInput {
    pub fn new() -> Res<AudioInput> {
        // Set up the input device and stream with the default input config.
        let host = cpal::host_from_id(HostId::ScreenCaptureKit)?;
        let device = host
            .default_input_device()
            .ok_or_else(|| Error::msg("No default input device"))?;

        let config = device
            .default_input_config()
            .expect("Failed to get default input config");

        let channel_count = config.channels() as usize;

        let (tx, rx) = std::sync::mpsc::channel();
        let stream = device.build_input_stream(
            &config.config(),
            move |pcm: &[f32], _: &cpal::InputCallbackInfo| {
                let pcm = pcm
                    .iter()
                    .step_by(channel_count)
                    .copied()
                    .collect::<Vec<f32>>();
                
                if !pcm.is_empty() {
                    if let Err(_) = tx.send(pcm) {
                        return;
                    }
                }
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            None,
        )?;
        stream.play()?;

        Ok(AudioInput {
            _stream: stream,
            rec: rx,
        })
    }
}
