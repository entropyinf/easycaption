use crate::common::Res;
use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SupportedStreamConfig};
use hound::WavWriter;
use std::io;

use std::sync::{Arc, Mutex};

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

    let mut config: SupportedStreamConfig = device
        .default_input_config()
        .expect("Failed to get default input config");

    println!("Default input config: {config:?}");

    let spec = wav_spec_from_config(&config);

    let writer = WavWriter::new(w, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    // Run the input stream on a separate thread.
    let writer = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {err}");
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8, W>(data, &writer),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16, W>(data, &writer),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32, W>(data, &writer),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32, W>(data, &writer),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    stream.play()?;

    println!("Started recording");

    Ok(Arc::new(stream))
}

type WavWriterHandle<W> = Arc<Mutex<Option<WavWriter<W>>>>;

fn write_input_data<T, U, W>(input: &[T], writer: &WavWriterHandle<W>)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
    W: io::Write + io::Seek,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
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