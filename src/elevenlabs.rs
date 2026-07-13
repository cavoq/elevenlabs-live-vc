use std::{io::Read, time::Instant};

use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, multipart};

use crate::{
    audio::AudioPlayer,
    config::Config,
    dsp::{StreamingResampler, resample_clip},
};

pub struct AudioPayload {
    pub bytes: Vec<u8>,
    pub file_name: &'static str,
    pub mime: &'static str,
    pub file_format: &'static str,
}

pub struct ElevenLabsClient {
    client: Client,
    api_key: String,
    voice_id: String,
    model_id: String,
    remove_background_noise: bool,
    output_format: String,
    output_sample_rate: u32,
    api_base_url: String,
}

impl ElevenLabsClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(300))
            .user_agent(concat!("elevenlabs-live-vc/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to create HTTP client")?;
        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            voice_id: config.voice_id.clone(),
            model_id: config.model_id.clone(),
            remove_background_noise: config.remove_background_noise,
            output_format: config.output_format.clone(),
            output_sample_rate: config.api_output_sample_rate,
            api_base_url: config.api_base_url.trim_end_matches('/').to_owned(),
        })
    }

    pub fn convert_and_play(&self, payload: AudioPayload, player: &AudioPlayer) -> Result<()> {
        let url = format!(
            "{}/v1/speech-to-speech/{}/stream",
            self.api_base_url,
            urlencoding::encode(&self.voice_id)
        );
        let part = multipart::Part::bytes(payload.bytes)
            .file_name(payload.file_name)
            .mime_str(payload.mime)?;
        let form = multipart::Form::new()
            .part("audio", part)
            .text("model_id", self.model_id.clone())
            .text(
                "remove_background_noise",
                self.remove_background_noise.to_string(),
            )
            .text("file_format", payload.file_format.to_owned());

        let started = Instant::now();
        let mut response = self
            .client
            .post(url)
            .query(&[("output_format", self.output_format.as_str())])
            .header("xi-api-key", &self.api_key)
            .multipart(form)
            .send()
            .context("ElevenLabs request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .unwrap_or_else(|_| "<response body unavailable>".into());
            bail!("ElevenLabs returned {status}: {}", truncate(&body, 500));
        }

        player.clear();
        let mut resampler = StreamingResampler::new(self.output_sample_rate, player.sample_rate())?;
        let mut buffer = [0_u8; 8192];
        let mut trailing_byte = None;
        let mut first_audio = true;

        loop {
            let read = response
                .read(&mut buffer)
                .context("failed while reading ElevenLabs audio stream")?;
            if read == 0 {
                break;
            }
            if first_audio {
                println!(
                    "First audio in {:.0} ms",
                    started.elapsed().as_secs_f64() * 1000.0
                );
                first_audio = false;
            }

            let mut bytes = Vec::with_capacity(read + usize::from(trailing_byte.is_some()));
            if let Some(byte) = trailing_byte.take() {
                bytes.push(byte);
            }
            bytes.extend_from_slice(&buffer[..read]);
            if bytes.len() % 2 != 0 {
                trailing_byte = bytes.pop();
            }
            let samples: Vec<f32> = bytes
                .chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32768.0)
                .collect();
            player.enqueue(&resampler.push(&samples)?);
        }

        player.enqueue(&resampler.finish()?);
        player.wait_until_empty();
        Ok(())
    }
}

pub fn prepare_payload(
    samples: &[f32],
    sample_rate: u32,
    low_latency: bool,
) -> Result<AudioPayload> {
    if low_latency {
        let samples = resample_clip(samples, sample_rate, 16_000)?;
        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            let pcm = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
            bytes.extend_from_slice(&pcm.to_le_bytes());
        }
        return Ok(AudioPayload {
            bytes,
            file_name: "input.pcm",
            mime: "application/octet-stream",
            file_format: "pcm_s16le_16",
        });
    }

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for sample in samples {
            writer.write_sample((sample.clamp(-1.0, 1.0) * 32767.0).round() as i16)?;
        }
        writer.finalize()?;
    }
    Ok(AudioPayload {
        bytes: cursor.into_inner(),
        file_name: "input.wav",
        mime: "audio/wav",
        file_format: "other",
    })
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let result: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{result}…")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_payload_has_riff_header() {
        let payload = prepare_payload(&[0.0; 480], 48_000, false).unwrap();
        assert_eq!(&payload.bytes[..4], b"RIFF");
        assert_eq!(payload.file_format, "other");
    }

    #[test]
    fn raw_payload_is_16_bit_pcm() {
        let payload = prepare_payload(&[0.0; 4800], 48_000, true).unwrap();
        assert_eq!(payload.file_format, "pcm_s16le_16");
        assert_eq!(payload.bytes.len() % 2, 0);
    }
}
