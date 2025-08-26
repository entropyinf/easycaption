use crate::common::Res;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SupportedStreamConfig};
use hound::WavWriter;
use log::error;
use std::io;
use std::sync::Arc;
use tokio::sync::watch::Sender;
use tokio::sync::{watch};

pub fn record<W>(w: W) -> Res<Arc<impl StreamTrait>>
where
    W: io::Write + io::Seek + Send + 'static,
{
    let host = cpal::host_from_id(cpal::HostId::ScreenCaptureKit)?;

    let device = host
        .input_devices()?
        .next()
        .ok_or_else(|| Error::msg("Failed to find a default input device"))?;

    println!("input device: {}", device.name()?);

    let config: SupportedStreamConfig = device
        .default_input_config()
        .expect("Failed to get default input config");

    println!("Default input config: {config:?}");

    let err_fn = move |err| {
        error!("an error occurred on stream: {err}");
    };

    let spec = wav_spec_from_config(&config);

    let (tx, mut rx) = watch::channel(WavSample::I8(0));

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data: &[i8], _: &_| write_input_data::<i8>(data, tx.clone()),
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| write_input_data::<i16>(data, tx.clone()),
            err_fn,
            None,
        ),
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data: &[i32], _: &_| write_input_data::<i32>(data, tx.clone()),
            err_fn,
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| write_input_data::<f32>(data, tx.clone()),
            err_fn,
            None,
        ),
        sample_format => {
            return Err(Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    tokio::spawn(async move {
        let mut writer = WavWriter::new(w, spec).unwrap();
        loop {
            if rx.changed().await.is_err() {
                break;
            };
            let sample = *rx.borrow();

            match sample {
                WavSample::I8(v) => writer.write_sample(i8::from_sample(v)).ok(),
                WavSample::I16(v) => writer.write_sample(i16::from_sample(v)).ok(),
                WavSample::I32(v) => writer.write_sample(i32::from_sample(v)).ok(),
                WavSample::F32(v) => writer.write_sample(f32::from_sample(v)).ok(),
            };
        }
    });

    if let Ok(stream) = stream {
        stream.play()?;
        println!("Started recording");
        return Ok(Arc::new(stream));
    }

    Err(Error::msg("Failed to build input stream"))
}

fn write_input_data<T>(input: &[T], tx: Sender<WavSample>)
where
    T: Into<WavSample> + Copy,
{
    for &sample in input.iter() {
        let _ = tx.send(sample.into());
    }
}

fn wav_spec_from_config(config: &SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

#[derive(Clone, Copy, Debug)]
enum WavSample {
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
}

impl From<i8> for WavSample {
    fn from(value: i8) -> Self {
        WavSample::I8(value)
    }
}

impl From<i16> for WavSample {
    fn from(value: i16) -> Self {
        WavSample::I16(value)
    }
}
impl From<i32> for WavSample {
    fn from(value: i32) -> Self {
        WavSample::I32(value)
    }
}
impl From<f32> for WavSample {
    fn from(value: f32) -> Self {
        WavSample::F32(value)
    }
}
