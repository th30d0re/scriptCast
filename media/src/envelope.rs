//! How open the carve is at each sample.
//!
//! HyperFrames' keyframe helper (`duck.mjs`) ramps linearly to the ducked
//! level over the attack, then linearly home over the release. A new span
//! that starts during a release restarts the attack from wherever the ramp
//! is. The keyframe list in that file can drop such an attack when the
//! previous release point was already pushed; this follower does not.

use crate::MediaError;
use crate::span::{SpeechSpan, merge_spans};

/// Ramp into a speech window. From `duck.mjs`'s default attack.
pub const DEFAULT_ATTACK_MS: f64 = 150.0;
/// Ramp back out after the window. From `duck.mjs`'s default release.
pub const DEFAULT_RELEASE_MS: f64 = 400.0;

/// Per-sample engagement, 0 = bed open, 1 = fully ducked and carved.
///
/// Allocates one value per frame. [`crate::carve_bed`] does not store this;
/// it exists so a caller can draw the lane, and so the ramp can be tested
/// without audio.
pub fn speech_engagement(
    frames: usize,
    sample_rate: u32,
    spans: &[SpeechSpan],
    attack_ms: f64,
    release_ms: f64,
) -> Result<Vec<f32>, MediaError> {
    if sample_rate == 0 {
        return Err(MediaError::SampleRate);
    }
    Ok(
        SpeechDriver::new(frames, sample_rate, spans, attack_ms, release_ms)
            .map(|level| level as f32)
            .collect(),
    )
}

pub(crate) struct SpeechDriver {
    frames: usize,
    frame: usize,
    sample_rate: u32,
    spans: Vec<SpeechSpan>,
    span_i: usize,
    speaking: bool,
    attack_samples: f64,
    release_samples: f64,
    env: Ramp,
}

impl SpeechDriver {
    pub(crate) fn new(
        frames: usize,
        sample_rate: u32,
        spans: &[SpeechSpan],
        attack_ms: f64,
        release_ms: f64,
    ) -> Self {
        Self {
            frames,
            frame: 0,
            sample_rate,
            spans: merge_spans(spans, 0),
            span_i: 0,
            speaking: false,
            attack_samples: ms_to_samples(attack_ms, sample_rate),
            release_samples: ms_to_samples(release_ms, sample_rate),
            env: Ramp::open(),
        }
    }

    fn advance(&mut self) -> f64 {
        let t = self.frame as u128 * 1000;
        let sr = u128::from(self.sample_rate);
        while self.span_i < self.spans.len() && u128::from(self.spans[self.span_i].end_ms) * sr <= t
        {
            self.span_i += 1;
        }
        let speaking = self.span_i < self.spans.len()
            && u128::from(self.spans[self.span_i].start_ms) * sr <= t;
        if speaking != self.speaking {
            self.speaking = speaking;
            let duration = if speaking {
                self.attack_samples
            } else {
                self.release_samples
            };
            self.env
                .set_target(if speaking { 1.0 } else { 0.0 }, duration);
        }
        self.env.next_value()
    }
}

impl Iterator for SpeechDriver {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if self.frame >= self.frames {
            return None;
        }
        let level = self.advance();
        self.frame += 1;
        Some(level)
    }
}

/// Linear ramp that always takes the full attack or release to reach its new target.
struct Ramp {
    level: f64,
    target: f64,
    step: f64,
}

impl Ramp {
    fn open() -> Self {
        Self {
            level: 0.0,
            target: 0.0,
            step: 0.0,
        }
    }

    fn set_target(&mut self, target: f64, duration_samples: f64) {
        let target = target.clamp(0.0, 1.0);
        if !duration_samples.is_finite()
            || duration_samples <= 0.0
            || (target - self.level).abs() < 1e-12
        {
            self.level = target;
            self.target = target;
            self.step = 0.0;
            return;
        }
        self.target = target;
        self.step = (target - self.level) / duration_samples;
    }

    fn next_value(&mut self) -> f64 {
        if self.step == 0.0 {
            return self.level;
        }
        let next = self.level + self.step;
        let crossed = (self.target - self.level).signum() != (self.target - next).signum()
            || (self.target - next).abs() < 1e-12;
        if crossed {
            self.level = self.target;
            self.step = 0.0;
        } else {
            self.level = next;
        }
        self.level
    }
}

fn ms_to_samples(ms: f64, sample_rate: u32) -> f64 {
    let ms = if ms.is_finite() { ms.max(0.0) } else { 0.0 };
    ms * f64::from(sample_rate) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(ms: u64, sr: u32) -> usize {
        (ms as usize) * (sr as usize) / 1000
    }

    #[test]
    fn attack_reaches_full_and_a_long_pause_opens_again() {
        let sr = 48_000u32;
        let spans = [
            SpeechSpan {
                start_ms: 0,
                end_ms: 500,
            },
            SpeechSpan {
                start_ms: 2000,
                end_ms: 2500,
            },
        ];
        let level = speech_engagement((sr * 3) as usize, sr, &spans, 150.0, 400.0).unwrap();
        assert!(level[0] < 0.01, "first sample {}", level[0]);
        assert!(
            level[at(180, sr)] > 0.99,
            "after attack {}",
            level[at(180, sr)]
        );
        assert!(level[at(1000, sr)] < 0.02, "pause {}", level[at(1000, sr)]);
        assert!(
            level[at(2200, sr)] > 0.99,
            "second span {}",
            level[at(2200, sr)]
        );
    }

    #[test]
    fn zero_sample_rate_is_an_error() {
        assert_eq!(
            speech_engagement(10, 0, &[], 150.0, 400.0).unwrap_err(),
            MediaError::SampleRate
        );
    }
}
