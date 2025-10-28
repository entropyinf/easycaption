use crate::Res;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, HostId, Stream, SupportedStreamConfig};
use std::sync::Arc;
use std::sync::mpsc::Receiver;

pub struct AudioInput {
    pub config: SupportedStreamConfig,
    stream: Arc<Stream>,
    pub rx: Receiver<Vec<f32>>,
}

unsafe impl Sync for AudioInput {}
unsafe impl Send for AudioInput {}

impl AudioInput {
    pub fn with_host_device(host_name: &str, device_name: &str) -> Res<AudioInput> {
        let host = Self::host_of_name(host_name)?;
        let device = host
            .input_devices()?
            .into_iter()
            .find(|d| d.name().unwrap_or_default() == device_name)
            .ok_or_else(|| Error::msg("Device not found"))?;

        Ok(Self::try_from(device)?)
    }

    #[cfg(target_os = "macos")]
    pub fn from_screen_capture_kit() -> Res<AudioInput> {
        // Set up the input device and stream with the default input config.
        let host = cpal::host_from_id(HostId::ScreenCaptureKit)?;
        let device = host
            .default_input_device()
            .ok_or_else(|| Error::msg("No default input device"))?;

        Ok(Self::try_from(device)?)
    }

    pub fn record(&self) -> Res<()> {
        self.stream.play()?;
        Ok(())
    }

    pub fn host_names() -> Vec<String> {
        Self::host_ids()
            .iter()
            .map(|h| h.name())
            .map(String::from)
            .collect::<Vec<String>>()
    }

    pub fn devices_of_host(host: &Host) -> Res<Vec<Device>> {
        Ok(host.input_devices()?.into_iter().collect::<Vec<Device>>())
    }

    pub fn host_of_name(host_name: &str) -> Res<Host> {
        let host_id = Self::host_ids()
            .into_iter()
            .find(|h| h.name() == host_name)
            .ok_or_else(|| Error::msg("Host not found"))?;

        Ok(cpal::host_from_id(host_id)?)
    }

    fn host_ids() -> Vec<HostId> {
        cpal::available_hosts()
            .iter()
            .copied()
            .collect::<Vec<HostId>>()
    }
}

impl TryFrom<Device> for AudioInput {
    type Error = Error;
    fn try_from(device: Device) -> Res<Self> {
        let config = device.default_input_config()?;
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

        let out = AudioInput {
            config,
            stream: Arc::new(stream),
            rx,
        };

        Ok(out)
    }
}
