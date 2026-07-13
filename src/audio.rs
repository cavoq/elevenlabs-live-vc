use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::config::Config;

struct CaptureState {
    samples: Vec<f32>,
    pre_buffer: VecDeque<f32>,
    pre_buffer_samples: usize,
    recording: bool,
    voice_detected: bool,
    started: Instant,
    last_voice: Instant,
    automatic: bool,
    threshold: f32,
    enabled: bool,
}

pub struct AudioRecorder {
    _stream: Stream,
    state: Arc<Mutex<CaptureState>>,
    sample_rate: u32,
    min_recording: Duration,
    silence_duration: Duration,
}

impl AudioRecorder {
    pub fn new(config: &Config) -> Result<Self> {
        let host = cpal::default_host();
        let device = select_input_device(&host, config.input_device.as_deref())?;
        let supported = device
            .default_input_config()
            .context("input device has no default format")?;
        let sample_format = supported.sample_format();
        let stream_config = supported.config();
        let sample_rate = stream_config.sample_rate;
        let channels = usize::from(stream_config.channels);
        let now = Instant::now();
        let state = Arc::new(Mutex::new(CaptureState {
            samples: Vec::new(),
            pre_buffer: VecDeque::new(),
            pre_buffer_samples: (u64::from(sample_rate) * config.vad_pre_buffer_ms / 1000) as usize,
            recording: false,
            voice_detected: false,
            started: now,
            last_voice: now,
            automatic: config.mode == crate::config::Mode::Automatic,
            threshold: config.vad_threshold,
            enabled: true,
        }));

        let stream = match sample_format {
            SampleFormat::F32 => build_input::<f32>(&device, stream_config, channels, &state),
            SampleFormat::F64 => build_input::<f64>(&device, stream_config, channels, &state),
            SampleFormat::I16 => build_input::<i16>(&device, stream_config, channels, &state),
            SampleFormat::I32 => build_input::<i32>(&device, stream_config, channels, &state),
            SampleFormat::U16 => build_input::<u16>(&device, stream_config, channels, &state),
            SampleFormat::U32 => build_input::<u32>(&device, stream_config, channels, &state),
            other => bail!("unsupported input sample format: {other}"),
        }?;
        stream.play().context("failed to start input stream")?;

        let name = device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_else(|_| device.to_string());
        println!(
            "Input: {name} ({sample_rate} Hz, {} channel{})",
            channels,
            if channels == 1 { "" } else { "s" }
        );

        Ok(Self {
            _stream: stream,
            state,
            sample_rate,
            min_recording: Duration::from_millis(config.vad_min_recording_ms),
            silence_duration: Duration::from_millis(config.vad_silence_ms),
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn start_manual(&self) {
        let mut state = self.state.lock().expect("capture state poisoned");
        state.samples.clear();
        state.pre_buffer.clear();
        state.recording = true;
        state.voice_detected = true;
        state.started = Instant::now();
        state.last_voice = state.started;
    }

    pub fn stop_manual(&self) -> Vec<f32> {
        let mut state = self.state.lock().expect("capture state poisoned");
        state.recording = false;
        state.voice_detected = false;
        std::mem::take(&mut state.samples)
    }

    pub fn poll_automatic(&self) -> Option<Vec<f32>> {
        let mut state = self.state.lock().expect("capture state poisoned");
        if !state.recording
            || !state.voice_detected
            || state.started.elapsed() < self.min_recording
            || state.last_voice.elapsed() < self.silence_duration
        {
            return None;
        }
        state.recording = false;
        state.voice_detected = false;
        Some(std::mem::take(&mut state.samples))
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.lock().expect("capture state poisoned");
        state.enabled = enabled;
        if !enabled {
            state.recording = false;
            state.voice_detected = false;
            state.samples.clear();
            state.pre_buffer.clear();
        }
    }
}

fn build_input<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    state: &Arc<Mutex<CaptureState>>,
) -> Result<Stream, cpal::Error>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    let state = Arc::clone(state);
    device.build_input_stream(
        config,
        move |data: &[T], _| capture(data, channels, &state),
        |error| eprintln!("Input stream error: {error}"),
        None,
    )
}

fn capture<T>(data: &[T], channels: usize, shared: &Arc<Mutex<CaptureState>>)
where
    T: Sample,
    f32: FromSample<T>,
{
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|frame| {
            frame
                .iter()
                .map(|sample| f32::from_sample(*sample))
                .sum::<f32>()
                / frame.len() as f32
        })
        .collect();
    if mono.is_empty() {
        return;
    }
    let rms = (mono.iter().map(|sample| sample * sample).sum::<f32>() / mono.len() as f32).sqrt();
    let mut state = shared.lock().expect("capture state poisoned");

    if !state.enabled {
        return;
    }

    if !state.automatic {
        if state.recording {
            state.samples.extend_from_slice(&mono);
        }
        return;
    }

    if !state.voice_detected {
        state.pre_buffer.extend(mono.iter().copied());
        while state.pre_buffer.len() > state.pre_buffer_samples {
            state.pre_buffer.pop_front();
        }
        if rms > state.threshold {
            state.voice_detected = true;
            state.recording = true;
            state.started = Instant::now();
            state.last_voice = state.started;
            state.samples = state.pre_buffer.iter().copied().collect();
            println!("Voice detected, recording...");
        }
    } else if state.recording {
        state.samples.extend_from_slice(&mono);
        if rms > state.threshold {
            state.last_voice = Instant::now();
        }
    }
}

struct PlaybackState {
    samples: VecDeque<f32>,
}

pub struct AudioPlayer {
    _stream: Stream,
    state: Arc<Mutex<PlaybackState>>,
    sample_rate: u32,
    max_buffered_samples: usize,
}

impl AudioPlayer {
    pub fn new(config: &Config) -> Result<Self> {
        let host = cpal::default_host();
        let device = select_output_device(&host, config.output_device.as_deref())?;
        let supported = device
            .default_output_config()
            .context("output device has no default format")?;
        let sample_format = supported.sample_format();
        let stream_config = supported.config();
        let sample_rate = stream_config.sample_rate;
        let channels = usize::from(stream_config.channels);
        let state = Arc::new(Mutex::new(PlaybackState {
            samples: VecDeque::new(),
        }));

        let stream = match sample_format {
            SampleFormat::F32 => build_output::<f32>(&device, stream_config, channels, &state),
            SampleFormat::F64 => build_output::<f64>(&device, stream_config, channels, &state),
            SampleFormat::I16 => build_output::<i16>(&device, stream_config, channels, &state),
            SampleFormat::I32 => build_output::<i32>(&device, stream_config, channels, &state),
            SampleFormat::U16 => build_output::<u16>(&device, stream_config, channels, &state),
            SampleFormat::U32 => build_output::<u32>(&device, stream_config, channels, &state),
            other => bail!("unsupported output sample format: {other}"),
        }?;
        stream.play().context("failed to start output stream")?;

        let name = device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_else(|_| device.to_string());
        println!(
            "Output: {name} ({sample_rate} Hz, {} channel{})",
            channels,
            if channels == 1 { "" } else { "s" }
        );

        Ok(Self {
            _stream: stream,
            state,
            sample_rate,
            max_buffered_samples: sample_rate as usize * 2,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn clear(&self) {
        self.state
            .lock()
            .expect("playback state poisoned")
            .samples
            .clear();
    }

    pub fn enqueue(&self, samples: &[f32]) {
        let mut offset = 0;
        while offset < samples.len() {
            let written = {
                let mut state = self.state.lock().expect("playback state poisoned");
                let available = self
                    .max_buffered_samples
                    .saturating_sub(state.samples.len());
                let count = available.min(samples.len() - offset);
                state.samples.extend(&samples[offset..offset + count]);
                count
            };
            offset += written;
            if written == 0 {
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    pub fn wait_until_empty(&self) {
        while !self
            .state
            .lock()
            .expect("playback state poisoned")
            .samples
            .is_empty()
        {
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn build_output<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    state: &Arc<Mutex<PlaybackState>>,
) -> Result<Stream, cpal::Error>
where
    T: Sample + SizedSample + FromSample<f32>,
{
    let state = Arc::clone(state);
    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            let mut state = state.lock().expect("playback state poisoned");
            for frame in data.chunks_mut(channels) {
                let value = state.samples.pop_front().unwrap_or(0.0);
                for sample in frame {
                    *sample = T::from_sample(value);
                }
            }
        },
        |error| eprintln!("Output stream error: {error}"),
        None,
    )
}

pub fn list_devices() -> Result<()> {
    let host = cpal::default_host();
    println!("Input devices:");
    for (index, device) in host.input_devices()?.enumerate() {
        println!("  {index}: {}", device_name(&device));
    }
    println!("\nOutput devices:");
    for (index, device) in host.output_devices()?.enumerate() {
        println!("  {index}: {}", device_name(&device));
    }
    Ok(())
}

fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_owned())
        .unwrap_or_else(|_| device.to_string())
}

fn select_input_device(host: &cpal::Host, selector: Option<&str>) -> Result<cpal::Device> {
    if let Some(selector) = selector {
        return select_device(host.input_devices()?, selector, "input");
    }
    host.default_input_device()
        .context("no default input device")
}

fn select_output_device(host: &cpal::Host, selector: Option<&str>) -> Result<cpal::Device> {
    if let Some(selector) = selector {
        return select_device(host.output_devices()?, selector, "output");
    }
    host.default_output_device()
        .context("no default output device")
}

fn select_device(
    devices: impl Iterator<Item = cpal::Device>,
    selector: &str,
    kind: &str,
) -> Result<cpal::Device> {
    let devices: Vec<_> = devices.collect();
    if let Ok(index) = selector.parse::<usize>() {
        return devices
            .into_iter()
            .nth(index)
            .with_context(|| format!("{kind} device index {index} does not exist"));
    }
    let selector = selector.to_lowercase();
    devices
        .into_iter()
        .find(|device| device_name(device).to_lowercase().contains(&selector))
        .with_context(|| format!("no {kind} device name contains {selector:?}"))
}
