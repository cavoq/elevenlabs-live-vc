use anyhow::{Context, Result};
use rubato::{FftFixedIn, Resampler};

const RESAMPLE_CHUNK: usize = 1024;

pub fn trim_with_padding(
    samples: &[f32],
    sample_rate: u32,
    threshold: f32,
    padding_ms: u64,
) -> Vec<f32> {
    let Some(first) = samples.iter().position(|sample| sample.abs() > threshold) else {
        return Vec::new();
    };
    let last = samples
        .iter()
        .rposition(|sample| sample.abs() > threshold)
        .unwrap_or(first);
    let padding = (u64::from(sample_rate) * padding_ms / 1000) as usize;
    let start = first.saturating_sub(padding);
    let end = (last + padding + 1).min(samples.len());
    samples[start..end].to_vec()
}

pub fn resample_clip(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
    if from_rate == to_rate {
        return Ok(samples.to_vec());
    }

    let mut resampler =
        FftFixedIn::<f32>::new(from_rate as usize, to_rate as usize, RESAMPLE_CHUNK, 2, 1)
            .context("failed to create resampler")?;
    let mut result = Vec::with_capacity(
        (samples.len() as u64 * u64::from(to_rate) / u64::from(from_rate)) as usize + 2048,
    );
    let mut offset = 0;

    while offset + resampler.input_frames_next() <= samples.len() {
        let length = resampler.input_frames_next();
        let input = vec![samples[offset..offset + length].to_vec()];
        let output = resampler.process(&input, None)?;
        result.extend_from_slice(&output[0]);
        offset += length;
    }

    if offset < samples.len() {
        let input = vec![samples[offset..].to_vec()];
        let output = resampler.process_partial(Some(&input), None)?;
        result.extend_from_slice(&output[0]);
    }

    Ok(result)
}

pub struct StreamingResampler {
    inner: Option<FftFixedIn<f32>>,
    pending: Vec<f32>,
}

impl StreamingResampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Result<Self> {
        let inner = if from_rate == to_rate {
            None
        } else {
            Some(FftFixedIn::<f32>::new(
                from_rate as usize,
                to_rate as usize,
                RESAMPLE_CHUNK,
                2,
                1,
            )?)
        };
        Ok(Self {
            inner,
            pending: Vec::new(),
        })
    }

    pub fn push(&mut self, samples: &[f32]) -> Result<Vec<f32>> {
        let Some(resampler) = &mut self.inner else {
            return Ok(samples.to_vec());
        };
        self.pending.extend_from_slice(samples);
        let mut output_samples = Vec::new();

        loop {
            let needed = resampler.input_frames_next();
            if self.pending.len() < needed {
                break;
            }
            let remainder = self.pending.split_off(needed);
            let input = vec![std::mem::replace(&mut self.pending, remainder)];
            let output = resampler.process(&input, None)?;
            output_samples.extend_from_slice(&output[0]);
        }
        Ok(output_samples)
    }

    pub fn finish(&mut self) -> Result<Vec<f32>> {
        let Some(resampler) = &mut self.inner else {
            return Ok(std::mem::take(&mut self.pending));
        };
        if self.pending.is_empty() {
            return Ok(Vec::new());
        }
        let input = vec![std::mem::take(&mut self.pending)];
        let output = resampler.process_partial(Some(&input), None)?;
        Ok(output.into_iter().next().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_keeps_padding() {
        let mut samples = vec![0.0; 100];
        samples[50] = 0.5;
        let trimmed = trim_with_padding(&samples, 1000, 0.1, 10);
        assert_eq!(trimmed.len(), 21);
        assert_eq!(trimmed[10], 0.5);
    }

    #[test]
    fn empty_or_silent_input_is_rejected() {
        assert!(trim_with_padding(&[], 48_000, 0.01, 100).is_empty());
        assert!(trim_with_padding(&[0.0; 10], 48_000, 0.01, 100).is_empty());
    }
}
