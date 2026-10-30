//! Strength profiles and which bands to dip.
//!
//! [`carve_profile`] expands one 0..1 strength knob into cut depth, band count,
//! Q and duck depth (`carveProfile` in `audioCarve.ts`). [`analyse_carve_bands`]
//! picks bands from the voice's spectrum (`analyseCarveBands`): a Hann window,
//! a 4096-point FFT, hop 2048, and a third-octave average around each centre,
//! ranked by power less an intelligibility penalty. Without a voice,
//! [`intelligibility_bands`] dips the centres nearest 2 kHz.

use serde::Serialize;

use crate::{CARVE_CENTERS_HZ, CarveBand};

/// Gentle studio default. HyperFrames' own CLI uses 0.8.
pub const DEFAULT_STRENGTH: f64 = 0.25;

const FRAME: usize = 4096;
const HOP: usize = 2048;
const BIAS_AUTHORITY_DB: f64 = 30.0;

/// What one strength setting expands to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarveProfile {
    /// Deepest cut on the strongest band, dB (positive number).
    pub max_cut_db: f64,
    /// How many of the nine centres to dip.
    pub bands: usize,
    pub q: f64,
    /// 0 ranks bands by voice power alone, 1 leans hard toward 2 kHz.
    pub intelligibility_bias: f64,
    /// Broadband duck while speech is active, dB (positive number).
    pub duck_db: f64,
}

/// Map a strength knob onto the profile. Values outside 0..1 clamp; a
/// non-finite strength is 0.5, as in HyperFrames.
pub fn carve_profile(strength: f64) -> CarveProfile {
    let s = if strength.is_finite() { strength.clamp(0.0, 1.0) } else { 0.5 };
    CarveProfile {
        max_cut_db: round2(2.0 + s * 16.0),
        bands: (1.0 + s * 6.0).round().clamp(1.0, 6.0) as usize,
        q: round2(1.1 + s * 1.2),
        intelligibility_bias: round2((0.6 + s * 0.4).min(1.0)),
        duck_db: round2(s * 24.0),
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// One voice recording and where it starts on the bed's clock.
pub struct VoiceSource<'a> {
    pub samples: &'a [f32],
    pub offset_ms: f64,
}

/// Sum voices onto one clock. Audio before time zero is dropped; overlapping
/// voices add.
pub fn mix_voices(parts: &[VoiceSource], sample_rate: u32) -> Vec<f32> {
    let placed: Vec<(i64, &[f32])> = parts
        .iter()
        .filter_map(|p| {
            let at = (p.offset_ms / 1000.0 * f64::from(sample_rate)).round();
            at.is_finite().then_some((at as i64, p.samples))
        })
        .collect();
    let len = placed.iter().map(|&(at, s)| at.saturating_add(s.len() as i64)).max().unwrap_or(0).max(0) as usize;
    let mut out = vec![0.0f32; len];
    for (at, samples) in placed {
        for (i, &x) in samples.iter().enumerate() {
            let t = at + i as i64;
            if t >= 0 && (t as usize) < len {
                out[t as usize] += x;
            }
        }
    }
    out
}

/// The `profile.bands` centres nearest 2 kHz, the two closest at full depth.
pub fn intelligibility_bands(profile: &CarveProfile) -> Vec<CarveBand> {
    let mut ranked: Vec<(f64, f64)> = CARVE_CENTERS_HZ.iter().map(|&c| (c, weight_2k(c))).collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
    ranked.truncate(profile.bands.max(1));
    bands_from_weights(profile, &ranked)
}

/// Pick bands from a mono voice. A silent or empty voice picks none.
pub fn analyse_carve_bands(voice: &[f32], sample_rate: u32, profile: &CarveProfile) -> Vec<CarveBand> {
    if voice.is_empty() || sample_rate == 0 {
        return Vec::new();
    }
    let (freqs, power) = power_spectrum(voice, sample_rate);
    let mut ranked: Vec<(f64, f64)> = CARVE_CENTERS_HZ
        .iter()
        .filter_map(|&c| {
            let e = band_power(&freqs, &power, c);
            (e > 0.0).then(|| (c, 10.0 * e.log10() - profile.intelligibility_bias * BIAS_AUTHORITY_DB * (1.0 - weight_2k(c))))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
    ranked.truncate(profile.bands.max(1));
    // Back to linear power, so a band 3 dB under the leader gets half its cut.
    let linear: Vec<(f64, f64)> = ranked.into_iter().map(|(c, db)| (c, 10f64.powf(db / 10.0))).collect();
    bands_from_weights(profile, &linear)
}

fn bands_from_weights(profile: &CarveProfile, selected: &[(f64, f64)]) -> Vec<CarveBand> {
    let Some(top) = selected.first().map(|b| b.1).filter(|&w| w > 0.0) else {
        return Vec::new();
    };
    let mut bands: Vec<CarveBand> = selected
        .iter()
        .map(|&(f, w)| {
            let depth = (profile.max_cut_db * w / top).clamp(profile.max_cut_db / 2.0, profile.max_cut_db);
            CarveBand { freq_hz: f, gain_db: -round2(depth), q: profile.q }
        })
        .collect();
    bands.sort_by(|a, b| a.freq_hz.total_cmp(&b.freq_hz));
    bands
}

/// Gaussian in octaves around 2 kHz (`intelligibilityPenaltyDb`).
fn weight_2k(center_hz: f64) -> f64 {
    let oct = (center_hz / 2000.0).log2();
    (-(oct * oct) / 2.0).exp()
}

fn power_spectrum(mono: &[f32], sample_rate: u32) -> (Vec<f64>, Vec<f64>) {
    let n = mono.len().max(FRAME);
    let window: Vec<f64> = (0..FRAME).map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (FRAME - 1) as f64).cos()).collect();
    let bins = FRAME / 2 + 1;
    let mut acc = vec![0.0f64; bins];
    let (mut re, mut im) = (vec![0.0f64; FRAME], vec![0.0f64; FRAME]);
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
    let width = f64::from(sample_rate) / FRAME as f64;
    let frames = frames.max(1) as f64;
    ((0..bins).map(|k| k as f64 * width).collect(), acc.into_iter().map(|s| s / frames).collect())
}

/// Mean power over the third octave around `center`.
fn band_power(freqs: &[f64], power: &[f64], center: f64) -> f64 {
    let half = 2f64.powf(1.0 / 6.0);
    let (lo, hi) = (center / half, center * half);
    let picked: Vec<f64> = freqs.iter().zip(power).filter(|(f, _)| **f >= lo && **f < hi).map(|(_, p)| *p).collect();
    if picked.is_empty() { 0.0 } else { picked.iter().sum::<f64>() / picked.len() as f64 }
}

/// In-place radix-2 forward FFT, unnormalised (`fft.ts`).
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
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
        let ang = -2.0 * std::f64::consts::PI / len as f64;
        let (wr, wi) = (ang.cos(), ang.sin());
        for i in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0, 0.0);
            for k in 0..len / 2 {
                let (a, b) = (i + k, i + k + len / 2);
                let vr = re[b] * cr - im[b] * ci;
                let vi = re[b] * ci + im[b] * cr;
                re[b] = re[a] - vr;
                im[b] = im[a] - vi;
                re[a] += vr;
                im[a] += vi;
                let next = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = next;
            }
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const SR: u32 = 48_000;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn quarter_strength_is_the_studio_default() {
        let p = carve_profile(0.25);
        close(p.max_cut_db, 6.0);
        assert_eq!(p.bands, 3);
        close(p.q, 1.4);
        close(p.intelligibility_bias, 0.7);
        close(p.duck_db, 6.0);
    }

    #[test]
    fn knob_ends_clamp_and_nan_is_half() {
        let (off, full) = (carve_profile(0.0), carve_profile(1.0));
        assert_eq!((off.bands, full.bands), (1, 6));
        close(off.duck_db, 0.0);
        close(full.max_cut_db, 18.0);
        close(carve_profile(9.0).max_cut_db, 18.0);
        close(carve_profile(f64::NAN).max_cut_db, carve_profile(0.5).max_cut_db);
    }

    #[test]
    fn fft_of_a_sine_lands_on_bin_one() {
        let n = 8;
        let mut re: Vec<f64> = (0..n).map(|i| (2.0 * PI * i as f64 / n as f64).sin()).collect();
        let mut im = vec![0.0; n];
        fft(&mut re, &mut im);
        assert!((im[1] + n as f64 / 2.0).abs() < 1e-6 && (im[n - 1] - n as f64 / 2.0).abs() < 1e-6);
        assert!(re.iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn without_a_voice_the_bands_sit_around_2_khz() {
        let bands = intelligibility_bands(&carve_profile(0.25));
        assert_eq!(bands.iter().map(|b| b.freq_hz as u32).collect::<Vec<_>>(), vec![1000, 1600, 2500]);
        assert!(bands.iter().all(|b| (-6.0..=-3.0).contains(&b.gain_db) && (b.q - 1.4).abs() < 1e-9));
    }

    #[test]
    fn bias_moves_a_low_heavy_voice_up_to_1600_hz() {
        let quiet = 10f64.powf(-9.0 / 20.0);
        let voice: Vec<f32> = (0..SR as usize / 2)
            .map(|i| {
                let t = i as f64 / f64::from(SR);
                (0.5 * (2.0 * PI * 250.0 * t).sin() + 0.5 * quiet * (2.0 * PI * 1600.0 * t).sin()) as f32
            })
            .collect();
        let mut one = carve_profile(0.25);
        one.bands = 1;
        one.intelligibility_bias = 0.0;
        assert_eq!(analyse_carve_bands(&voice, SR, &one)[0].freq_hz, 250.0);
        one.intelligibility_bias = 0.7;
        assert_eq!(analyse_carve_bands(&voice, SR, &one)[0].freq_hz, 1600.0);
    }

    #[test]
    fn silence_picks_no_bands() {
        assert!(analyse_carve_bands(&[0.0; 10_000], SR, &carve_profile(0.25)).is_empty());
    }

    #[test]
    fn voices_mix_at_their_offsets() {
        let mixed = mix_voices(&[VoiceSource { samples: &[1.0, 2.0, 3.0], offset_ms: 0.0 }, VoiceSource { samples: &[4.0, 5.0], offset_ms: 200.0 }], 10);
        assert_eq!(mixed, vec![1.0, 2.0, 7.0, 5.0]);
        assert_eq!(mix_voices(&[VoiceSource { samples: &[1.0, 2.0, 3.0], offset_ms: -100.0 }], 10), vec![2.0, 3.0]);
    }
}
