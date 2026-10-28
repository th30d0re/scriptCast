//! Apply a speech envelope to a bed: a fixed level duck, plus peaking cuts.

use soundcraft_dsp::biquad::{Biquad, FilterType};
use soundcraft_dsp::db_to_gain;

use crate::MediaError;
use crate::envelope::{DEFAULT_ATTACK_MS, DEFAULT_RELEASE_MS, SpeechDriver};
use crate::profile::{CarveBand, DEFAULT_STRENGTH, carve_profile};
use crate::span::SpeechSpan;

/// Level and timing for one render. Depth defaults to quarter strength (6 dB).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarveSettings {
    pub duck_db: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
}

impl Default for CarveSettings {
    fn default() -> Self {
        Self {
            duck_db: carve_profile(DEFAULT_STRENGTH).duck_db,
            attack_ms: DEFAULT_ATTACK_MS,
            release_ms: DEFAULT_RELEASE_MS,
        }
    }
}

/// Carve `channels` in place.
///
/// `spans` are episode-clock milliseconds. Overlaps are collapsed; a non-zero
/// merge gap is the caller's decision ([`crate::merge_spans`]). Outside speech
/// the bed is unchanged, including the return to flat after the release.
pub fn carve_bed(
    channels: &mut [Vec<f32>],
    sample_rate: u32,
    spans: &[SpeechSpan],
    bands: &[CarveBand],
    settings: &CarveSettings,
) -> Result<(), MediaError> {
    if channels.is_empty() {
        return Ok(());
    }
    if sample_rate == 0 {
        return Err(MediaError::SampleRate);
    }
    let frames = channels[0].len();
    if channels.iter().any(|channel| channel.len() != frames) {
        return Err(MediaError::RaggedChannels);
    }
    if frames == 0 {
        return Ok(());
    }

    let ducked = ducked_amplitude(settings.duck_db);
    let mut filters: Vec<Vec<Biquad>> = (0..channels.len())
        .map(|_| {
            bands
                .iter()
                .map(|band| design(band, 0.0, sample_rate))
                .collect()
        })
        .collect();
    let mut applied = 0.0f64;
    let driver = SpeechDriver::new(
        frames,
        sample_rate,
        spans,
        settings.attack_ms,
        settings.release_ms,
    );
    for (frame, level) in driver.enumerate() {
        if (level - applied).abs() > 0.002 || endpoint(level, applied) {
            applied = level;
            for state in &mut filters {
                for (filter, band) in state.iter_mut().zip(bands) {
                    filter.set(
                        FilterType::Peak,
                        band.freq_hz as f32,
                        band.q as f32,
                        (band.gain_db * level) as f32,
                        sample_rate as f32,
                    );
                }
            }
        }
        let amp = 1.0 + (ducked - 1.0) * (level.clamp(0.0, 1.0) as f32);
        for (channel, state) in channels.iter_mut().zip(filters.iter_mut()) {
            let mut sample = channel[frame] * amp;
            for filter in state {
                sample = filter.process_sample(sample);
            }
            channel[frame] = sample;
        }
    }
    Ok(())
}

fn endpoint(level: f64, applied: f64) -> bool {
    (level <= 0.0 || level >= 1.0) && (level - applied).abs() > 1e-12
}

fn ducked_amplitude(duck_db: f64) -> f32 {
    let db = if duck_db.is_finite() {
        duck_db.max(0.0)
    } else {
        0.0
    };
    db_to_gain(-(db as f32))
}

fn design(band: &CarveBand, level: f64, sample_rate: u32) -> Biquad {
    Biquad::design(
        FilterType::Peak,
        band.freq_hz as f32,
        band.q as f32,
        (band.gain_db * level) as f32,
        sample_rate as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::intelligibility_bands;
    use crate::profile::carve_profile;
    use soundcraft_dsp::biquad::{Coeffs, FilterType};

    const SR: u32 = 48_000;

    #[test]
    fn no_speech_leaves_the_bed_untouched_even_with_bands() {
        let input = tone(2_000.0, SR, SR as usize);
        let bands = intelligibility_bands(&carve_profile(0.25));
        let mut channels = vec![input.clone()];
        carve_bed(&mut channels, SR, &[], &bands, &CarveSettings::default()).unwrap();
        assert_eq!(channels[0], input);
    }

    #[test]
    fn speech_dips_2_khz_harder_than_100_hz_and_a_pause_recovers() {
        let bands = intelligibility_bands(&carve_profile(DEFAULT_STRENGTH));
        let settings = CarveSettings::default();
        let spans = [SpeechSpan {
            start_ms: 400,
            end_ms: 1200,
        }];
        let frames = SR as usize * 2;

        let (low_in, low_out) = render(100.0, frames, &spans, &bands, &settings);
        let (high_in, high_out) = render(2_000.0, frames, &spans, &bands, &settings);

        let speech = frame(800)..frame(1100);
        let pause = frame(1800)..frame(2000);
        let pre = ..frame(400);

        assert_eq!(&high_out[pre.clone()], &high_in[pre]);

        let low_db = ratio_db(&low_out[speech.clone()], &low_in[speech.clone()]);
        let high_db = ratio_db(&high_out[speech.clone()], &high_in[speech.clone()]);
        let expect_low = predicted_db(&bands, 100.0, settings.duck_db);
        let expect_high = predicted_db(&bands, 2_000.0, settings.duck_db);
        assert!(
            (low_db - expect_low).abs() < 1.0,
            "100 Hz measured {low_db:.2} dB, expected {expect_low:.2}"
        );
        assert!(
            (high_db - expect_high).abs() < 1.0,
            "2 kHz measured {high_db:.2} dB, expected {expect_high:.2}"
        );
        assert!(
            high_db < low_db - 3.0,
            "2 kHz {high_db:.2} should be well under 100 Hz {low_db:.2}"
        );

        let recovered = ratio_db(&high_out[pause.clone()], &high_in[pause]);
        assert!(
            recovered.abs() < 1.0,
            "pause was {recovered:.2} dB, want about 0"
        );
    }

    #[test]
    fn stereo_matches_a_mono_render() {
        let bands = intelligibility_bands(&carve_profile(0.25));
        let spans = [SpeechSpan {
            start_ms: 0,
            end_ms: 500,
        }];
        let input = tone(2_000.0, SR, SR as usize);
        let mut mono = vec![input.clone()];
        let mut stereo = vec![input.clone(), input];
        let settings = CarveSettings::default();
        carve_bed(&mut mono, SR, &spans, &bands, &settings).unwrap();
        carve_bed(&mut stereo, SR, &spans, &bands, &settings).unwrap();
        assert_eq!(stereo[0], mono[0]);
        assert_eq!(stereo[1], mono[0]);
    }

    #[test]
    fn ragged_channels_are_refused() {
        let mut channels = vec![vec![0.0; 8], vec![0.0; 4]];
        let err = carve_bed(&mut channels, SR, &[], &[], &CarveSettings::default()).unwrap_err();
        assert_eq!(err, MediaError::RaggedChannels);
    }

    fn render(
        freq: f32,
        frames: usize,
        spans: &[SpeechSpan],
        bands: &[CarveBand],
        settings: &CarveSettings,
    ) -> (Vec<f32>, Vec<f32>) {
        let input = tone(freq, SR, frames);
        let mut channels = vec![input.clone()];
        carve_bed(&mut channels, SR, spans, bands, settings).unwrap();
        (input, channels.pop().unwrap())
    }

    fn tone(freq: f32, sample_rate: u32, frames: usize) -> Vec<f32> {
        (0..frames)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                0.25 * (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect()
    }

    fn frame(ms: usize) -> usize {
        ms * SR as usize / 1000
    }

    fn ratio_db(out: &[f32], input: &[f32]) -> f64 {
        20.0 * (rms(out) / rms(input)).log10()
    }

    fn rms(samples: &[f32]) -> f64 {
        let energy: f64 = samples
            .iter()
            .map(|sample| f64::from(*sample).powi(2))
            .sum();
        (energy / samples.len() as f64).sqrt()
    }

    fn predicted_db(bands: &[CarveBand], freq: f32, duck_db: f64) -> f64 {
        let mut db = -duck_db;
        for band in bands {
            let coeffs = Coeffs::design(
                FilterType::Peak,
                band.freq_hz as f32,
                band.q as f32,
                band.gain_db as f32,
                SR as f32,
            );
            db += f64::from(coeffs.magnitude_db(freq, SR as f32));
        }
        db
    }
}
