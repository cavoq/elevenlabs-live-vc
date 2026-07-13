use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Manual,
    Automatic,
}

fn default_mode() -> Mode {
    Mode::Automatic
}
fn default_model() -> String {
    "eleven_multilingual_sts_v2".into()
}
fn default_true() -> bool {
    true
}
fn default_output_format() -> String {
    "pcm_22050".into()
}
fn default_api_output_sample_rate() -> u32 {
    22_050
}
fn default_vad_threshold() -> f32 {
    0.01
}
fn default_vad_silence_ms() -> u64 {
    350
}
fn default_vad_min_recording_ms() -> u64 {
    300
}
fn default_vad_pre_buffer_ms() -> u64 {
    250
}
fn default_min_audio_ms() -> u64 {
    150
}
fn default_trim_threshold() -> f32 {
    0.005
}
fn default_trim_padding_ms() -> u64 {
    100
}
fn default_api_base_url() -> String {
    "https://api.elevenlabs.io".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub voice_id: String,
    #[serde(default = "default_mode")]
    pub mode: Mode,
    #[serde(default = "default_model")]
    pub model_id: String,
    #[serde(default = "default_true")]
    pub remove_background_noise: bool,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    #[serde(default = "default_true")]
    pub low_latency_input: bool,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_api_output_sample_rate")]
    pub api_output_sample_rate: u32,
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,
    #[serde(default = "default_vad_silence_ms")]
    pub vad_silence_ms: u64,
    #[serde(default = "default_vad_min_recording_ms")]
    pub vad_min_recording_ms: u64,
    #[serde(default = "default_vad_pre_buffer_ms")]
    pub vad_pre_buffer_ms: u64,
    #[serde(default = "default_min_audio_ms")]
    pub min_audio_ms: u64,
    #[serde(default = "default_trim_threshold")]
    pub trim_threshold: f32,
    #[serde(default = "default_trim_padding_ms")]
    pub trim_padding_ms: u64,
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("invalid TOML in {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.api_key.trim().is_empty() || self.api_key.starts_with("your_") {
            bail!("api_key is missing in config.toml");
        }
        if self.voice_id.trim().is_empty() || self.voice_id.starts_with("your_") {
            bail!("voice_id is missing in config.toml");
        }
        if self.api_output_sample_rate == 0 {
            bail!("api_output_sample_rate must be greater than zero");
        }
        let expected_format = format!("pcm_{}", self.api_output_sample_rate);
        if self.output_format != expected_format {
            bail!(
                "output_format must be {expected_format:?} when api_output_sample_rate is {}",
                self.api_output_sample_rate
            );
        }
        if !(0.0..=1.0).contains(&self.vad_threshold) {
            bail!("vad_threshold must be between 0 and 1");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> Config {
        Config {
            api_key: "secret".into(),
            voice_id: "voice".into(),
            mode: Mode::Automatic,
            model_id: default_model(),
            remove_background_noise: true,
            input_device: None,
            output_device: None,
            low_latency_input: true,
            output_format: default_output_format(),
            api_output_sample_rate: default_api_output_sample_rate(),
            vad_threshold: default_vad_threshold(),
            vad_silence_ms: default_vad_silence_ms(),
            vad_min_recording_ms: default_vad_min_recording_ms(),
            vad_pre_buffer_ms: default_vad_pre_buffer_ms(),
            min_audio_ms: default_min_audio_ms(),
            trim_threshold: default_trim_threshold(),
            trim_padding_ms: default_trim_padding_ms(),
            api_base_url: default_api_base_url(),
        }
    }

    #[test]
    fn rejects_mismatched_pcm_rate() {
        let mut config = valid_config();
        config.output_format = "pcm_44100".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn accepts_valid_defaults() {
        assert!(valid_config().validate().is_ok());
    }
}
