use crate::Res;
use std::collections::VecDeque;
use std::time;
use std::time::Instant;
use time::Duration;
use voice_activity_detector::{IteratorExt, VoiceActivityDetector};

const CHUNK_SIZE: usize = 512;

#[derive(Debug, Clone, Copy)]
pub struct VadConfig {
    pub sample_rate: u32,         // Sample rate, e.g., 16000 Hz
    pub silence_duration_ms: u32, // Silence duration (milliseconds), e.g., 500 ms
    pub speech_threshold: f32,
    pub window_ms: u32,
    pub interval_ms: u64,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            silence_duration_ms: 450,
            speech_threshold: 0.5,
            window_ms: 10000,
            interval_ms: 1000,
        }
    }
}

#[derive(Debug)]
pub struct VadProcessor {
    vad: VoiceActivityDetector,
    config: VadConfig,
    chunk_ms: u32,

    last_process: Instant,

    cache: VecDeque<f32>,
    chunks: VecDeque<Chunk>,

    epoch: Option<Instant>,
    chunks_count: usize,
}

impl VadProcessor {
    pub fn new(config: VadConfig) -> Res<Self> {
        let vad = VoiceActivityDetector::builder()
            .sample_rate(config.sample_rate)
            .chunk_size(CHUNK_SIZE)
            .build()?;

        let chunk_ms = ((CHUNK_SIZE as f32 / config.sample_rate as f32) * 1000.0) as u32;
        let chunks_capacity = (config.window_ms as f32 / chunk_ms as f32 * 2.0).ceil() as usize;

        Ok(Self {
            vad,
            config,
            chunk_ms,
            cache: VecDeque::with_capacity(CHUNK_SIZE * 2),
            last_process: Instant::now(),
            epoch: None,
            chunks: VecDeque::with_capacity(chunks_capacity),
            chunks_count: 0,
        })
    }

    pub fn process(&mut self, samples: &[f32]) -> Option<Segment> {
        let now = Instant::now();

        if self.epoch.is_none() {
            self.epoch = Some(now);
        }

        // Merge samples and chunks
        self.cache.extend(samples);
        let len = self.cache.len();
        let chunk_count = len / CHUNK_SIZE * CHUNK_SIZE;
        if chunk_count == 0 {
            return None;
        }
        let samples = self.cache.drain(0..chunk_count);

        // VAD
        for (data, pred) in samples.predict(&mut self.vad) {
            self.chunks.push_back(Chunk {
                index: self.chunks_count,
                data,
                pred,
            });
            self.chunks_count += 1;
        }
        if self.chunks.is_empty() {
            return None;
        }

        // Sleep
        if now - self.last_process < Duration::from_millis(self.config.interval_ms) {
            return None;
        }
        self.last_process = now;

        // Extract output
        let mut data: Vec<f32> = Vec::with_capacity(self.chunks.len() * CHUNK_SIZE);
        let start = self.chunks.get(0)?.index * self.chunk_ms as usize;
        let end = self.chunks.get(self.chunks.len() - 1)?.index * self.chunk_ms as usize;
        let speech = self
            .chunks
            .iter()
            .any(|chunk| chunk.pred >= self.config.speech_threshold);
        if !speech {
            self.chunks.clear();
            return Some(Segment {
                start,
                end,
                data: None,
            });
        }
        for chunk in &self.chunks {
            data.extend(&chunk.data)
        }

        // Slide window to the biggest silence area
        let chunks_ms = end - start;
        if chunks_ms > self.config.window_ms as usize {
            self.slide_to_silence_area();
        }

        Some(Segment {
            start,
            end,
            data: Some(data),
        })
    }

    fn slide_to_silence_area(&mut self) {
        if self.chunks.is_empty() {
            return;
        }

        let kernel_size =
            (self.config.silence_duration_ms as f32 / self.chunk_ms as f32).ceil() as usize;
        if kernel_size >= self.chunks.len() {
            return;
        }

        let mut min_avg_pred = f32::MAX;
        let mut min_index = 0;

        for i in 0..=(self.chunks.len() - kernel_size) {
            let sum: f32 = self
                .chunks
                .range(i..(i + kernel_size))
                .map(|chunk| chunk.pred)
                .sum();
            let avg = sum / kernel_size as f32;

            if avg < min_avg_pred {
                min_avg_pred = avg;
                min_index = i + kernel_size;
            }
        }

        if min_index > 0 {
            self.chunks.drain(0..min_index);
        }
    }
}

pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub data: Option<Vec<f32>>,
}

#[derive(Debug)]
struct Chunk {
    index: usize,
    data: Vec<f32>,
    pred: f32,
}
