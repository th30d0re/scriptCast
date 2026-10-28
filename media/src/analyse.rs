//! Which bands to dip.
//!
//! The spectrum path is a port of HyperFrames `analyseCarveBands`: a Hann
//! window, a 4096-point FFT, hop 2048, and a third-octave average around each
//! candidate centre. Centres are about two-thirds of an octave apart and each
//! band is only a third of an octave wide, so a tone that sits between them
//! (2 kHz, for example) falls in a gap and does not score.
//!
//! With no voice recording, [`intelligibility_bands`] dips the centres closest
//! to 2 kHz. That is where a bed usually masks speech, and it does not require
//! decoding every turn.

use crate::profile::{CANDIDATE_CENTERS_HZ, CarveBand, CarveProfile, round2};

const FRAME: usize = 4096;
const HOP: usize = 2048;
const BIAS_AUTHORITY_DB: f64 = 30.0;

/// One voice buffer and where it starts relative to the bed, in seconds.
pub struct CarveSource<'a> {
    pub samples: &'a [f32],
    pub offset_seconds: f64,
}

/// Sum voice buffers onto one clock.
///
/// Audio that starts before time zero contributes only the part that overlaps.
/// Two voices at once add, they are not averaged.
pub fn mix_carve_sources(parts: &[CarveSource<'_>], sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let placed: Vec<(i64, &[f32])> = parts
        .iter()
        .filter_map(|part| {
            Some((
                offset_frames(part.offset_seconds, sample_rate)?,
                part.samples,
            ))
        })
        .collect();
    let mut length: i64 = 0;
    for &(at, samples) in &placed {
        length = length.max(at.saturating_add(samples.len() as i64));
    }
    if length <= 0 {
        return Vec::new();
    }
    let mut mixed = vec![0.0f32; length as usize];
    for (at, samples) in placed {
        let from = if at < 0 { (-at) as usize } else { 0 };
        for (offset, sample) in samples.iter().enumerate().skip(from) {
            let target = at + offset as i64;
            if target < 0 || target >= length {
                continue;
            }
            mixed[target as usize] += sample;
        }
    }
    mixed
}

/// Bands ranked by proximity to 2 kHz, deepest on the closest centres.
///
/// 2 kHz itself is not one of the nine candidates. They step by about
/// two-thirds of an octave, so the closest are 1.6 and 2.5 kHz.
pub fn intelligibility_bands(profile: &CarveProfile) -> Vec<CarveBand> {
    let mut ranked: Vec<(f64, f64)> = CANDIDATE_CENTERS_HZ
        .into_iter()
        .map(|center| (center, intelligibility_weight(center)))
        .collect();
    ranked.sort_by(|a, b| cmp_desc(a.1, b.1).then(cmp_asc(a.0, b.0)));
    let selected: Vec<(f64, f64)> = ranked.into_iter().take(profile.bands.max(1)).collect();
    bands_from_weights(profile, &selected)
}

/// Pick bands from a voice recording. An empty or silent voice returns no bands.
pub fn analyse_carve_bands(
    voice: &[f32],
    sample_rate: u32,
    profile: &CarveProfile,
) -> Vec<CarveBand> {
    if voice.is_empty() || sample_rate == 0 {
        return Vec::new();
    }
    let (freqs, power) = power_spectrum(voice, sample_rate);
    let mut ranked: Vec<(f64, f64, bool)> = CANDIDATE_CENTERS_HZ
        .into_iter()
        .map(|center| {
            let energy = band_power(&freqs, &power, center);
            let score = if energy > 0.0 {
                10.0 * energy.log10()
                    - intelligibility_penalty_db(center, profile.intelligibility_bias)
            } else {
                f64::NEG_INFINITY
            };
            (center, score, energy > 0.0)
        })
        .collect();
    ranked.sort_by(|a, b| cmp_desc(a.1, b.1).then(cmp_asc(a.0, b.0)));
    let selected: Vec<(f64, f64)> = ranked
        .into_iter()
        .take(profile.bands.max(1))
        .filter(|band| band.2)
        .map(|band| {
            // `10*log10` already applied. Convert back to a linear weight so a
            // band 3 dB under the leader gets half its cut.
            let linear = if band.1.is_finite() {
                10f64.powf(band.1 / 10.0)
            } else {
                0.0
            };
            (band.0, linear)
        })
        .collect();
    bands_from_weights(profile, &selected)
}

fn bands_from_weights(profile: &CarveProfile, selected: &[(f64, f64)]) -> Vec<CarveBand> {
    let Some(top) = selected
        .first()
        .map(|band| band.1)
        .filter(|weight| *weight > 0.0)
    else {
        return Vec::new();
    };
    let mut bands: Vec<CarveBand> = selected
        .iter()
        .map(|(freq, weight)| {
            let relative = weight / top;
            let depth =
                (profile.max_cut_db * relative).clamp(profile.max_cut_db / 2.0, profile.max_cut_db);
            CarveBand {
                freq_hz: *freq,
                gain_db: -round2(depth),
                q: profile.q,
            }
        })
        .collect();
    bands.sort_by(|a, b| cmp_asc(a.freq_hz, b.freq_hz));
    bands
}

fn intelligibility_weight(center: f64) -> f64 {
    let octaves = (center / 2000.0).log2();
    (-(octaves * octaves) / 2.0).exp()
}

fn intelligibility_penalty_db(center: f64, bias: f64) -> f64 {
    bias * BIAS_AUTHORITY_DB * (1.0 - intelligibility_weight(center))
}

fn power_spectrum(mono: &[f32], sample_rate: u32) -> (Vec<f64>, Vec<f64>) {
    let n = mono.len().max(FRAME);
    let window: Vec<f64> = (0..FRAME)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (FRAME - 1) as f64).cos())
        .collect();
    let bins = FRAME / 2 + 1;
    let mut acc = vec![0.0f64; bins];
    let mut re = vec![0.0f64; FRAME];
    let mut im = vec![0.0f64; FRAME];
    let mut frames = 0usize;
    let mut start = 0;
    while start + FRAME <= n {
        im.fill(0.0);
        for i in 0..FRAME {
            re[i] = f64::from(mono.get(start + i).copied().unwrap_or(0.0)) * window[i];
        }
        fft(&mut re, &mut im);
        for k in 0..bins {
            acc[k] += re[k] * re[k] + im[k] * im[k];
        }
        frames += 1;
        start += HOP;
    }
    let frames = frames.max(1) as f64;
    let width = f64::from(sample_rate) / FRAME as f64;
    let freqs: Vec<f64> = (0..bins).map(|k| k as f64 * width).collect();
    let power: Vec<f64> = acc.into_iter().map(|sum| sum / frames).collect();
    (freqs, power)
}

fn band_power(freqs: &[f64], power: &[f64], center: f64) -> f64 {
    let half = 2f64.powf(1.0 / 6.0);
    let lo = center / half;
    let hi = center * half;
    let mut sum = 0.0;
    let mut count = 0usize;
    for (freq, bin) in freqs.iter().zip(power) {
        if *freq >= lo && *freq < hi {
            sum += bin;
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// In-place radix-2 FFT. Ported from HyperFrames `fft.ts` (unnormalized, forward).
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two() && n == im.len());
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = (-2.0 * std::f64::consts::PI) / len as f64;
        let wr = ang.cos();
        let wi = ang.sin();
        let mut i = 0;
        while i < n {
            let mut cr = 1.0;
            let mut ci = 0.0;
            for k in 0..(len / 2) {
                let ur = re[i + k];
                let ui = im[i + k];
                let idx = i + k + len / 2;
                let vr = re[idx] * cr - im[idx] * ci;
                let vi = re[idx] * ci + im[idx] * cr;
                re[i + k] = ur + vr;
                im[i + k] = ui + vi;
                re[idx] = ur - vr;
                im[idx] = ui - vi;
                let next_cr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = next_cr;
            }
            i += len;
        }
        len <<= 1;
    }
}

fn offset_frames(seconds: f64, sample_rate: u32) -> Option<i64> {
    let frames = seconds * f64::from(sample_rate);
    if !frames.is_finite() {
        return None;
    }
    let frames = frames.round();
    if !(i64::MIN as f64..=i64::MAX as f64).contains(&frames) {
        return None;
    }
    Some(frames as i64)
}

fn cmp_desc(left: f64, right: f64) -> std::cmp::Ordering {
    right
        .partial_cmp(&left)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn cmp_asc(left: f64, right: f64) -> std::cmp::Ordering {
    left.partial_cmp(&right)
        .unwrap_or(std::cmp::Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::carve_profile;

    const SR: u32 = 48_000;

    #[test]
    fn forward_fft_of_a_sine_lands_on_bin_one() {
        let n = 8;
        let mut re: Vec<f64> = (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * i as f64 / n as f64).sin())
            .collect();
        let mut im = vec![0.0; n];
        fft(&mut re, &mut im);
        assert!((im[1] + n as f64 / 2.0).abs() < 1e-6, "im[1]={}", im[1]);
        assert!(
            (im[n - 1] - n as f64 / 2.0).abs() < 1e-6,
            "im[last]={}",
            im[n - 1]
        );
        assert!(re.iter().all(|value| value.abs() < 1e-6));
    }

    #[test]
    fn silence_has_no_bands() {
        let profile = carve_profile(0.25);
        assert!(analyse_carve_bands(&[], SR, &profile).is_empty());
        assert!(analyse_carve_bands(&[0.0; 1000], SR, &profile).is_empty());
    }

    #[test]
    fn bias_moves_a_low_heavy_pair_up_to_1600_hz() {
        // The low tone is 9 dB louder. Unbiased, the 250 Hz centre wins.
        // At the default bias the 1600 Hz centre wins, which is the case
        // HyperFrames uses to prove the weighting has enough authority.
        let quiet = 10f64.powf(-9.0 / 20.0);
        let n = SR as usize / 2;
        let mut voice = vec![0.0f32; n];
        for (i, sample) in voice.iter_mut().enumerate() {
            let t = i as f64 / f64::from(SR);
            *sample = (0.5 * (2.0 * std::f64::consts::PI * 250.0 * t).sin()
                + 0.5 * quiet * (2.0 * std::f64::consts::PI * 1600.0 * t).sin())
                as f32;
        }
        let mut one = carve_profile(0.25);
        one.bands = 1;
        one.intelligibility_bias = 0.0;
        assert_eq!(analyse_carve_bands(&voice, SR, &one)[0].freq_hz, 250.0);
        one.intelligibility_bias = carve_profile(0.25).intelligibility_bias;
        assert_eq!(analyse_carve_bands(&voice, SR, &one)[0].freq_hz, 1600.0);
    }

    #[test]
    fn a_tilted_voice_is_not_carved_only_at_the_fundamental() {
        let voice = tilted_voice(120.0);
        let mut flat = carve_profile(0.25);
        flat.bands = 1;
        flat.intelligibility_bias = 0.0;
        assert!(analyse_carve_bands(&voice, SR, &flat)[0].freq_hz <= 400.0);

        let profile = carve_profile(0.25);
        let bands = analyse_carve_bands(&voice, SR, &profile);
        assert_eq!(bands.len(), 3);
        assert!(bands.iter().any(|band| band.freq_hz >= 1000.0));
        assert!(
            bands
                .windows(2)
                .all(|pair| pair[0].freq_hz < pair[1].freq_hz)
        );
        assert!(bands.iter().all(|band| band.gain_db < 0.0));
    }

    #[test]
    fn default_bands_without_a_voice_sit_around_2_khz() {
        let bands = intelligibility_bands(&carve_profile(0.25));
        let freqs: Vec<u32> = bands.iter().map(|band| band.freq_hz as u32).collect();
        // 2 kHz is in the gap between candidates, so the top three are
        // 1, 1.6 and 2.5 kHz. The two nearest 2 kHz take the full cut.
        assert_eq!(freqs, vec![1000, 1600, 2500]);
        for freq in [1600.0, 2500.0] {
            let gain = bands
                .iter()
                .find(|band| band.freq_hz == freq)
                .unwrap()
                .gain_db;
            assert!((gain + 6.0).abs() < 1e-9, "{freq} Hz gain {gain}");
        }
        assert!(
            bands
                .iter()
                .all(|band| (-6.0..=-3.0).contains(&band.gain_db))
        );
        assert!(bands.iter().all(|band| (band.q - 1.4).abs() < 1e-9));
    }

    #[test]
    fn mix_places_offsets_and_drops_audio_before_zero() {
        let mixed = mix_carve_sources(
            &[
                CarveSource {
                    samples: &[1.0, 2.0, 3.0],
                    offset_seconds: 0.0,
                },
                CarveSource {
                    samples: &[4.0, 5.0],
                    offset_seconds: 0.2,
                },
            ],
            10,
        );
        assert_eq!(mixed, vec![1.0, 2.0, 7.0, 5.0]);

        let trimmed = mix_carve_sources(
            &[CarveSource {
                samples: &[10.0, 20.0, 30.0],
                offset_seconds: -0.1,
            }],
            10,
        );
        assert_eq!(trimmed, vec![20.0, 30.0]);
    }

    fn tilted_voice(f0: f64) -> Vec<f32> {
        let n = (SR / 2) as usize;
        let mut out = vec![0.0f32; n];
        for (i, sample) in out.iter_mut().enumerate() {
            let t = i as f64 / f64::from(SR);
            let mut value = 0.0;
            let mut harmonic = 1.0;
            while harmonic * f0 < f64::from(SR) / 2.0 && harmonic <= 64.0 {
                value += (1.0 / harmonic) * (2.0 * std::f64::consts::PI * f0 * harmonic * t).sin();
                harmonic += 1.0;
            }
            *sample = (0.5 * value) as f32;
        }
        out
    }
}
