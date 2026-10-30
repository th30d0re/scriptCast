//! Make room for dialogue in a music or background bed.
//!
//! Two processes, ported from HyperFrames (Apache-2.0, see NOTICE):
//!
//! - [`duck`]: a broadband gain envelope that drops the bed to `target_db`
//!   while speech is active (`duck.mjs` / `audio-duck.mjs`).
//! - [`carve`]: nine third-octave peaking dips, weighted toward the bands
//!   that carry speech intelligibility, applied only while the voice speaks
//!   (`audioCarve.ts` / `carve.mjs`).
//!
//! Speech comes from ScriptCast's `episode_manifest.json` (`turns[].start_ms`,
//! `turns[].end_ms`), a plain `[[start_ms, end_ms], ...]` list, or (carve only)
//! the voice WAV itself.

use std::path::Path;

use serde::{Deserialize, Serialize};
use soundcraft_audio_io::{AudioBuffer, BitDepth, EncodeOptions};
use soundcraft_dsp::biquad::{Biquad, FilterType};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

/// One stretch of speech, in milliseconds from the start of the bed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start_ms: f64,
    pub end_ms: f64,
}

/// Sort spans and bridge gaps shorter than `merge_gap_ms`, as `mergeIntervals`
/// in `duck.mjs` does. Overlapping and touching spans always merge.
pub fn merge_spans(spans: &[Span], merge_gap_ms: f64) -> Vec<Span> {
    let mut sorted: Vec<Span> = spans.iter().copied().filter(|s| s.end_ms > s.start_ms).collect();
    sorted.sort_by(|a, b| a.start_ms.total_cmp(&b.start_ms).then(a.end_ms.total_cmp(&b.end_ms)));
    let mut merged: Vec<Span> = Vec::new();
    for s in sorted {
        match merged.last_mut() {
            Some(prev) if s.start_ms <= prev.end_ms || s.start_ms - prev.end_ms < merge_gap_ms => {
                prev.end_ms = prev.end_ms.max(s.end_ms);
            }
            _ => merged.push(s),
        }
    }
    merged
}

/// Read speech spans from JSON: either a ScriptCast episode manifest
/// (`{"turns": [{"start_ms": .., "end_ms": ..}, ..]}`) or `[[start_ms, end_ms], ..]`.
pub fn spans_from_json(text: &str) -> Result<Vec<Span>> {
    #[derive(Deserialize)]
    struct Turn {
        start_ms: f64,
        end_ms: f64,
    }
    #[derive(Deserialize)]
    struct Manifest {
        turns: Vec<Turn>,
    }
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        Manifest(Manifest),
        Pairs(Vec<[f64; 2]>),
    }
    Ok(match serde_json::from_str::<Input>(text)? {
        Input::Manifest(m) => m.turns.into_iter().map(|t| Span { start_ms: t.start_ms, end_ms: t.end_ms }).collect(),
        Input::Pairs(p) => p.into_iter().map(|[a, b]| Span { start_ms: a, end_ms: b }).collect(),
    })
}

pub fn db_to_gain(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

pub fn gain_to_db(g: f64) -> f64 {
    20.0 * g.max(1e-12).log10()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DuckParams {
    /// Bed level during speech, dB. HyperFrames' default multiplier 0.25 is -12.04 dB.
    pub target_db: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub merge_gap_ms: f64,
}

impl Default for DuckParams {
    fn default() -> Self {
        Self { target_db: -12.0, attack_ms: 150.0, release_ms: 400.0, merge_gap_ms: 300.0 }
    }
}

/// Linear gain of the duck envelope at `t_ms`.
///
/// Like `duckKeyframes` + `duckLane`: each merged span starts a linear ramp
/// (in amplitude) down to the target over `attack_ms` at its start, and a
/// linear ramp back to unity over `release_ms` at its end. A ramp starts from
/// wherever the previous one left the gain.
pub fn duck_gain_at(spans: &[Span], p: &DuckParams, t_ms: f64) -> f64 {
    let ducked = db_to_gain(p.target_db);
    let mut g = 1.0;
    for s in spans {
        if t_ms < s.start_ms {
            break;
        }
        let from = g;
        let a = ramp(from, ducked, (t_ms - s.start_ms) / p.attack_ms);
        if t_ms < s.end_ms {
            return a;
        }
        let at_end = ramp(from, ducked, (s.end_ms - s.start_ms) / p.attack_ms);
        g = ramp(at_end, 1.0, (t_ms - s.end_ms) / p.release_ms);
    }
    g
}

fn ramp(from: f64, to: f64, frac: f64) -> f64 {
    let f = if frac.is_finite() { frac.clamp(0.0, 1.0) } else { 1.0 };
    from + (to - from) * f
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckReport {
    pub spans: Vec<Span>,
    pub params: DuckParams,
    pub frames: usize,
    pub sample_rate: u32,
}

/// Apply the duck envelope to every channel of `bed` in place.
pub fn duck(bed: &mut AudioBuffer, spans: &[Span], p: &DuckParams) -> DuckReport {
    let merged = merge_spans(spans, p.merge_gap_ms);
    let sr = f64::from(bed.sample_rate);
    let frames = bed.frames();
    let env: Vec<f32> = (0..frames).map(|i| duck_gain_at(&merged, p, i as f64 * 1000.0 / sr) as f32).collect();
    for ch in &mut bed.channels {
        for (x, g) in ch.iter_mut().zip(&env) {
            *x *= g;
        }
    }
    DuckReport { spans: merged, params: *p, frames, sample_rate: bed.sample_rate }
}

/// Third-octave centres from `audioCarve.ts` (`CANDIDATE_CENTERS_HZ`).
pub const CARVE_CENTERS_HZ: [f64; 9] = [160.0, 250.0, 400.0, 630.0, 1000.0, 1600.0, 2500.0, 4000.0, 6000.0];

/// Speech-intelligibility weight of a band: 1.0 from 1.25 to 2.5 kHz, falling
/// off as a Gaussian in octaves outside it (the same shape as
/// `intelligibilityPenaltyDb`, widened to a plateau).
pub fn intelligibility_weight(center_hz: f64) -> f64 {
    let oct = if center_hz < 1250.0 {
        (1250.0 / center_hz).log2()
    } else if center_hz > 2500.0 {
        (center_hz / 2500.0).log2()
    } else {
        0.0
    };
    (-(oct * oct) / 2.0).exp()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CarveParams {
    /// Cut at weight 1.0, dB (positive number). `carveProfile(0.8).maxCutDb` = 14.8.
    pub max_cut_db: f64,
    /// `carveProfile(0.8).q` = 2.06.
    pub q: f64,
    /// RMS analysis window for the voice.
    pub window_ms: f64,
    /// Voice level below its own peak that counts as silence (`DYNAMIC_RANGE_DB`).
    pub dynamic_range_db: f64,
    /// Envelope smoothing (`ATTACK_S`, `RELEASE_S`).
    pub attack_ms: f64,
    pub release_ms: f64,
}

impl Default for CarveParams {
    fn default() -> Self {
        Self { max_cut_db: 14.8, q: 2.06, window_ms: 50.0, dynamic_range_db: 30.0, attack_ms: 50.0, release_ms: 250.0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CarveBand {
    pub freq_hz: f64,
    pub gain_db: f64,
    pub q: f64,
}

/// The bands and their full-depth cuts.
pub fn carve_bands(p: &CarveParams) -> Vec<CarveBand> {
    CARVE_CENTERS_HZ
        .iter()
        .map(|&f| CarveBand { freq_hz: f, gain_db: -p.max_cut_db * intelligibility_weight(f), q: p.q })
        .collect()
}

/// Where speech is active, as a 0..1 target per analysis window.
pub enum Activity<'a> {
    /// Mono-summed voice samples at the bed's sample rate.
    Voice(&'a [f32]),
    Spans(&'a [Span]),
}

/// Per-window target depth (0..1). For a voice: 50 ms RMS relative to the
/// voice's loudest window, mapped linearly from `-dynamic_range_db` (0) to the
/// peak (1), as `analyseCarveDynamics` does per band.
pub fn activity_targets(act: &Activity, sample_rate: u32, frames: usize, p: &CarveParams) -> Vec<f64> {
    let win = ((p.window_ms / 1000.0) * f64::from(sample_rate)).round().max(1.0) as usize;
    let n = frames.div_ceil(win);
    match act {
        Activity::Spans(spans) => (0..n)
            .map(|w| {
                let mid = (w as f64 + 0.5) * win as f64 * 1000.0 / f64::from(sample_rate);
                f64::from(u8::from(spans.iter().any(|s| mid >= s.start_ms && mid < s.end_ms)))
            })
            .collect(),
        Activity::Voice(v) => {
            let dbs: Vec<f64> = (0..n)
                .map(|w| {
                    let chunk = v.get(w * win..((w + 1) * win).min(v.len())).unwrap_or(&[]);
                    if chunk.is_empty() {
                        return f64::NEG_INFINITY;
                    }
                    let ms = chunk.iter().map(|&x| f64::from(x) * f64::from(x)).sum::<f64>() / chunk.len() as f64;
                    if ms > 0.0 { 10.0 * ms.log10() } else { f64::NEG_INFINITY }
                })
                .collect();
            let peak = dbs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            dbs.iter()
                .map(|&d| if d.is_finite() && peak.is_finite() { (1.0 + (d - peak) / p.dynamic_range_db).max(0.0) } else { 0.0 })
                .collect()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CarveReport {
    pub bands: Vec<CarveBand>,
    pub params: CarveParams,
    pub active_fraction: f64,
    pub frames: usize,
    pub sample_rate: u32,
}

/// Coefficient update interval, samples.
const BLOCK: usize = 32;

/// Dip the bed under the voice in place. Each band's gain follows the smoothed
/// activity envelope, so the bed is flat (bit-exact passthrough at level 0)
/// between turns.
pub fn carve(bed: &mut AudioBuffer, act: &Activity, p: &CarveParams) -> CarveReport {
    let sr = bed.sample_rate;
    let frames = bed.frames();
    let win = ((p.window_ms / 1000.0) * f64::from(sr)).round().max(1.0) as usize;
    let targets = activity_targets(act, sr, frames, p);
    let bands = carve_bands(p);
    let step = BLOCK as f64 / f64::from(sr);
    let att = 1.0 - (-step / (p.attack_ms / 1000.0)).exp();
    let rel = 1.0 - (-step / (p.release_ms / 1000.0)).exp();

    let mut levels = Vec::with_capacity(frames.div_ceil(BLOCK));
    let mut level = 0.0f64;
    for b in 0..frames.div_ceil(BLOCK) {
        let t = targets.get(b * BLOCK / win).copied().unwrap_or(0.0);
        level += (if t > level { att } else { rel }) * (t - level);
        levels.push(level);
    }

    for ch in &mut bed.channels {
        let mut filters: Vec<Biquad> =
            bands.iter().map(|b| Biquad::design(FilterType::Peak, b.freq_hz as f32, b.q as f32, 0.0, sr as f32)).collect();
        for (b, block) in ch.chunks_mut(BLOCK).enumerate() {
            let l = levels[b];
            for (f, band) in filters.iter_mut().zip(&bands) {
                f.set(FilterType::Peak, band.freq_hz as f32, band.q as f32, (band.gain_db * l) as f32, sr as f32);
                f.process_block(block);
            }
        }
    }
    let active_fraction = if targets.is_empty() { 0.0 } else { targets.iter().filter(|&&t| t > 0.0).count() as f64 / targets.len() as f64 };
    CarveReport { bands, params: *p, active_fraction, frames, sample_rate: sr }
}

/// Average all channels into one.
pub fn mono(buf: &AudioBuffer) -> Vec<f32> {
    let n = buf.num_channels().max(1) as f32;
    (0..buf.frames()).map(|i| buf.channels.iter().map(|c| c[i]).sum::<f32>() / n).collect()
}

pub fn read_wav(path: &Path) -> Result<AudioBuffer> {
    let bytes = std::fs::read(path)?;
    let ext = path.extension().and_then(|e| e.to_str());
    Ok(soundcraft_audio_io::decode(&bytes, ext)?.1)
}

/// Write 32-bit float WAV, so ducked tails never lose resolution.
pub fn write_wav(path: &Path, buf: &AudioBuffer) -> Result<()> {
    let opts = EncodeOptions { bit_depth: BitDepth::Float32, ..EncodeOptions::default() };
    std::fs::write(path, soundcraft_audio_io::encode(buf, &opts)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const SR: u32 = 48_000;

    fn sine(freq: f64, secs: f64, amp: f64) -> Vec<f32> {
        (0..(secs * f64::from(SR)) as usize).map(|i| (amp * (2.0 * PI * freq * i as f64 / f64::from(SR)).sin()) as f32).collect()
    }

    fn rms_db(x: &[f32]) -> f64 {
        let ms = x.iter().map(|&v| f64::from(v) * f64::from(v)).sum::<f64>() / x.len() as f64;
        10.0 * ms.log10()
    }

    fn at(ms: f64) -> usize {
        (ms * f64::from(SR) / 1000.0) as usize
    }

    fn span(a: f64, b: f64) -> Span {
        Span { start_ms: a, end_ms: b }
    }

    #[test]
    fn merges_short_gaps_only() {
        let m = merge_spans(&[span(1000.0, 2000.0), span(0.0, 500.0), span(2200.0, 3000.0)], 300.0);
        assert_eq!(m, vec![span(0.0, 500.0), span(1000.0, 3000.0)]);
    }

    #[test]
    fn reads_manifest_and_pairs() {
        let m = spans_from_json(r#"{"turns":[{"turn_index":0,"start_ms":10,"end_ms":20}]}"#).unwrap();
        assert_eq!(m, vec![span(10.0, 20.0)]);
        assert_eq!(spans_from_json("[[1,2],[3,4]]").unwrap().len(), 2);
    }

    #[test]
    fn duck_hits_target_and_recovers_with_attack_and_release() {
        let p = DuckParams::default();
        let mut bed = AudioBuffer { sample_rate: SR, channels: vec![vec![1.0; at(6000.0)]] };
        duck(&mut bed, &[span(1000.0, 2000.0), span(3500.0, 4500.0)], &p);
        let g = |ms: f64| gain_to_db(f64::from(bed.channels[0][at(ms)]));

        assert!(g(500.0).abs() < 1e-6, "before speech");
        // Attack: halfway (linear amplitude) at 75 ms, at target from 150 ms.
        let mid = gain_to_db((1.0 + db_to_gain(p.target_db)) / 2.0);
        assert!((g(1075.0) - mid).abs() < 0.05);
        assert!(g(1140.0) > p.target_db + 0.1, "still ramping");
        for ms in [1150.0, 1500.0, 1999.0, 3700.0, 4400.0] {
            assert!((g(ms) - p.target_db).abs() < 0.05, "at {ms}: {}", g(ms));
        }
        // Release: back at 0 dB 400 ms after the end, not before.
        assert!(g(2390.0) < -0.05);
        for ms in [2400.0, 3000.0, 3499.0, 5000.0] {
            assert!(g(ms).abs() < 1e-6, "at {ms}: {}", g(ms));
        }
    }

    #[test]
    fn carve_dips_mids_during_speech_and_leaves_lows() {
        let p = CarveParams::default();
        let secs = 4.0;
        let low = sine(100.0, secs, 0.3);
        let mid = sine(2000.0, secs, 0.3);
        // Voice: speech 1.0..3.0 s.
        let voice: Vec<f32> =
            sine(1500.0, secs, 0.5).iter().enumerate().map(|(i, &v)| if (at(1000.0)..at(3000.0)).contains(&i) { v } else { 0.0 }).collect();

        let run = |src: &[f32]| {
            let mut b = AudioBuffer { sample_rate: SR, channels: vec![src.to_vec()] };
            carve(&mut b, &Activity::Voice(&voice), &p);
            b.channels.remove(0)
        };
        let lo_out = run(&low);
        let mid_out = run(&mid);
        let speech = at(1500.0)..at(2900.0);
        let quiet = at(200.0)..at(900.0);

        let mid_drop = rms_db(&mid[speech.clone()]) - rms_db(&mid_out[speech.clone()]);
        assert!(mid_drop > 10.0, "2 kHz dropped only {mid_drop} dB");
        assert!((rms_db(&mid[quiet.clone()]) - rms_db(&mid_out[quiet.clone()])).abs() < 0.01, "2 kHz touched outside speech");
        let lo_change = (rms_db(&low[speech.clone()]) - rms_db(&lo_out[speech])).abs();
        assert!(lo_change < 0.5, "100 Hz changed {lo_change} dB");

        // The same with turn windows instead of a voice track.
        let mut b = AudioBuffer { sample_rate: SR, channels: vec![mid.clone()] };
        carve(&mut b, &Activity::Spans(&[span(1000.0, 3000.0)]), &p);
        assert!(rms_db(&mid[at(1500.0)..at(2900.0)]) - rms_db(&b.channels[0][at(1500.0)..at(2900.0)]) > 10.0);
    }

    #[test]
    fn weights_peak_in_intelligibility_band() {
        for f in [1600.0, 2500.0] {
            assert_eq!(intelligibility_weight(f), 1.0);
        }
        assert!(intelligibility_weight(160.0) < 0.02);
        assert!(intelligibility_weight(1000.0) < 1.0);
    }
}
