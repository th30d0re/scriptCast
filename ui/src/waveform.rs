use crate::episode::Episode;
use eframe::egui;
use soundcraft_audio_io::peaks::Peaks;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

/// One decoded file, mixed to mono, with a multi-resolution overview.
///
/// `samples` and `peaks` describe the same channel. Range queries pass `samples`
/// back into [`Peaks`] so each drawn column is an exact min/max.
#[derive(Clone)]
pub struct Envelope {
    pub samples: Arc<Vec<f32>>,
    pub peaks: Arc<Peaks>,
    pub sample_rate: u32,
    /// Source time of sample 0. A full-file decode uses `0`.
    pub origin: f64,
}

impl Envelope {
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn duration_secs(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f64 / f64::from(self.sample_rate)
        }
    }
}

/// How many times a path was decoded into the in-memory cache.
pub fn cache_builds() -> u64 {
    builds().load(Ordering::Relaxed)
}

pub fn envelope(paths: &[PathBuf]) -> Result<Envelope, String> {
    if paths.is_empty() {
        return Ok(empty_envelope());
    }
    if paths.len() == 1 {
        return load(&paths[0]).map(|file| envelope_of(&file));
    }
    let mut rate = 0u32;
    let mut mono = Vec::new();
    for path in paths {
        let file = load(path)?;
        if rate == 0 {
            rate = file.sample_rate;
            mono.extend(file.mono.iter().copied());
            continue;
        }
        if file.sample_rate == rate {
            mono.extend(file.mono.iter().copied());
        } else {
            let piece = soundcraft_dsp::offline::resample(
                &[file.mono.as_ref().clone()],
                file.sample_rate,
                rate,
            );
            mono.extend(piece.into_iter().next().unwrap_or_default());
        }
    }
    Ok(envelope_from_mono(mono, rate, 0.0))
}

struct CachedFile {
    sample_rate: u32,
    channels: Arc<Vec<Vec<f32>>>,
    mono: Arc<Vec<f32>>,
    peaks: Arc<Peaks>,
    #[cfg_attr(not(test), allow(dead_code))]
    via_ffmpeg: bool,
}

fn builds() -> &'static AtomicU64 {
    static BUILDS: AtomicU64 = AtomicU64::new(0);
    &BUILDS
}

type PeakCache = HashMap<(PathBuf, SystemTime), Arc<CachedFile>>;

fn cache() -> &'static Mutex<PeakCache> {
    static CACHE: OnceLock<Mutex<PeakCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Decode `path` once per `(path, mtime)` and keep the samples and [`Peaks`].
fn load(path: &Path) -> Result<Arc<CachedFile>, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let key = (path.to_path_buf(), mtime);
    let mut guard = cache().lock().unwrap_or_else(|err| err.into_inner());
    if let Some(hit) = guard.get(&key) {
        return Ok(Arc::clone(hit));
    }
    let decoded = decode_path(path)?;
    builds().fetch_add(1, Ordering::Relaxed);
    guard.insert(key, Arc::clone(&decoded));
    Ok(decoded)
}

fn decode_path(path: &Path) -> Result<Arc<CachedFile>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let hint = path.extension().and_then(|ext| ext.to_str());
    match decode_native(&bytes, hint) {
        Ok((rate, channels)) => Ok(finish(rate, channels, false)),
        Err(error) => {
            // soundcraft refused the file. ffmpeg is the last resort, and it is logged.
            eprintln!(
                "scriptcast-ui: ffmpeg fallback for {}: {error}",
                path.display()
            );
            let pcm = ffmpeg_f32le_all(path).map_err(|ffmpeg| format!("{error}; {ffmpeg}"))?;
            let samples = f32le_samples(&pcm);
            if samples.is_empty() {
                return Err(format!("{error}; ffmpeg produced no audio"));
            }
            Ok(finish(48_000, vec![samples], true))
        }
    }
}

fn decode_native(bytes: &[u8], hint: Option<&str>) -> Result<(u32, Vec<Vec<f32>>), String> {
    let (_info, buf) = soundcraft_audio_io::decode(bytes, hint).map_err(|e| e.to_string())?;
    if buf.sample_rate == 0 || buf.channels.is_empty() || buf.frames() == 0 {
        return Err("decoded audio contains no samples".into());
    }
    Ok((buf.sample_rate, buf.channels))
}

fn finish(sample_rate: u32, channels: Vec<Vec<f32>>, via_ffmpeg: bool) -> Arc<CachedFile> {
    let mono = Arc::new(mono_mix(&channels));
    let peaks = Arc::new(Peaks::build(&mono));
    Arc::new(CachedFile {
        sample_rate,
        channels: Arc::new(channels),
        mono,
        peaks,
        via_ffmpeg,
    })
}

fn mono_mix(channels: &[Vec<f32>]) -> Vec<f32> {
    let frames = frames_of(channels);
    if channels.len() <= 1 {
        return channels
            .first()
            .map(|c| c[..frames].to_vec())
            .unwrap_or_default();
    }
    let n = channels.len() as f32;
    (0..frames)
        .map(|i| channels.iter().map(|c| c[i]).sum::<f32>() / n)
        .collect()
}

fn frames_of(channels: &[Vec<f32>]) -> usize {
    channels.iter().map(Vec::len).min().unwrap_or(0)
}

fn envelope_from_mono(samples: Vec<f32>, sample_rate: u32, origin: f64) -> Envelope {
    let peaks = Arc::new(Peaks::build(&samples));
    Envelope {
        samples: Arc::new(samples),
        peaks,
        sample_rate,
        origin,
    }
}

fn empty_envelope() -> Envelope {
    envelope_from_mono(Vec::new(), 0, 0.0)
}

fn envelope_of(file: &CachedFile) -> Envelope {
    Envelope {
        samples: Arc::clone(&file.mono),
        peaks: Arc::clone(&file.peaks),
        sample_rate: file.sample_rate,
        origin: 0.0,
    }
}

/// Seconds of source audio kept on either side of a clip when ffmpeg builds the waveform.
pub const CLIP_CONTEXT_SECS: f64 = 2.0;

/// Decode `[start, start + len)` to a mono 48 kHz WAV in memory.
/// `ffmpeg is not available` when the binary is missing.
pub fn ffmpeg_wav(path: &Path, start: f64, len: f64) -> Result<Vec<u8>, String> {
    if !start.is_finite() || !len.is_finite() || start < 0.0 || len <= 0.0 {
        return Err("Invalid audio range".into());
    }
    let output = match std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-ss"])
        .arg(format!("{start:.3}"))
        .arg("-t")
        .arg(format!("{len:.3}"))
        .arg("-i")
        .arg(path)
        .args(["-f", "wav", "-ac", "1", "-ar", "48000", "-"])
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err("ffmpeg is not available".into());
        }
        Err(error) => return Err(error.to_string()),
    };
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            format!("ffmpeg exited {}", output.status)
        } else {
            format!("ffmpeg: {detail}")
        });
    }
    if output.stdout.is_empty() {
        return Err("ffmpeg produced no audio".into());
    }
    Ok(output.stdout)
}

// True when `peaks` cover `seconds` (10 ms buckets). A few buckets of
// container padding still count; a decode that stops a third of the way does not.
// Drawing no longer uses that check. Columns come from `Peaks` over the decoded buffer.

fn f32le_samples(bytes: &[u8]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(bytes.len() / 4);
    let mut index = 0;
    while index + 4 <= bytes.len() {
        let mut chunk = [0u8; 4];
        chunk.copy_from_slice(&bytes[index..index + 4]);
        samples.push(f32::from_le_bytes(chunk));
        index += 4;
    }
    samples
}

#[cfg_attr(not(test), allow(dead_code))]
fn peaks_of_f32le(bytes: &[u8]) -> Vec<f32> {
    const RATE: usize = 48_000;
    let per = RATE / 100;
    let mut peaks = Vec::new();
    let mut peak = 0.0_f32;
    let mut count = 0usize;
    for sample in f32le_samples(bytes) {
        peak = peak.max(sample.abs());
        count += 1;
        if count == per {
            peaks.push(peak.min(1.0));
            peak = 0.0;
            count = 0;
        }
    }
    if count > 0 {
        peaks.push(peak.min(1.0));
    }
    peaks
}

/// Mono 48 kHz f32le for `[start, start + len)`. `-t` is an output option so the
/// requested length is decoded, including sources rodio stops early on.
#[cfg_attr(not(test), allow(dead_code))]
fn ffmpeg_f32le(path: &Path, start: f64, len: f64) -> Result<Vec<u8>, String> {
    if !start.is_finite() || !len.is_finite() || start < 0.0 || len <= 0.0 {
        return Err("Invalid audio range".into());
    }
    ffmpeg_pcm(path, Some(start), Some(len))
}

fn ffmpeg_f32le_all(path: &Path) -> Result<Vec<u8>, String> {
    ffmpeg_pcm(path, None, None)
}

fn ffmpeg_pcm(path: &Path, start: Option<f64>, len: Option<f64>) -> Result<Vec<u8>, String> {
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-v", "error"]);
    if let Some(start) = start {
        cmd.arg("-ss").arg(format!("{start:.3}"));
    }
    cmd.arg("-i").arg(path);
    if let Some(len) = len {
        // `-t` is an output option so the requested length is decoded.
        cmd.arg("-t").arg(format!("{len:.3}"));
    }
    cmd.args(["-vn", "-f", "f32le", "-ac", "1", "-ar", "48000", "-"])
        .stdin(std::process::Stdio::null());
    let output = match cmd.output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err("ffmpeg is not available".into());
        }
        Err(error) => return Err(error.to_string()),
    };
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            format!("ffmpeg exited {}", output.status)
        } else {
            format!("ffmpeg: {detail}")
        });
    }
    if output.stdout.len() < 4 {
        return Err("ffmpeg produced no audio".into());
    }
    Ok(output.stdout)
}

/// Full-file overview. The `f64` is the window's start in the source, `0` for a
/// full-file decode. `start` and `end` are checked, then the cached file is returned
/// so a short native decode cannot clip the drawing window.
pub fn clip_envelope(path: &Path, start: f64, end: f64) -> Result<(Envelope, f64), String> {
    if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
        return Err("Invalid audio range".into());
    }
    let file = load(path)?;
    Ok((envelope_of(&file), 0.0))
}

/// Interleaved PCM at `out_rate`. A `start`/`end` pair is an exact sample range
/// of the decoded buffer, taken before resampling.
pub struct OutputPcm {
    pub channels: u16,
    pub sample_rate: u32,
    pub interleaved: Vec<f32>,
}

pub fn output_samples(
    path: &Path,
    start: Option<f64>,
    end: Option<f64>,
    out_rate: u32,
) -> Result<OutputPcm, String> {
    let file = load(path)?;
    let frames = frames_of(&file.channels);
    let (from, to) = match (start, end) {
        (None, None) => (0, frames),
        (Some(start), Some(end)) => sample_bounds(file.sample_rate, frames, start, end)?,
        _ => return Err("Invalid audio range".into()),
    };
    render_output(&file.channels, file.sample_rate, from, to, out_rate)
}

pub fn output_samples_bytes(bytes: &[u8], ext: &str, out_rate: u32) -> Result<OutputPcm, String> {
    let (rate, channels) = decode_native(bytes, Some(ext))?;
    let frames = frames_of(&channels);
    render_output(&channels, rate, 0, frames, out_rate)
}

fn sample_bounds(rate: u32, frames: usize, start: f64, end: f64) -> Result<(usize, usize), String> {
    if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start || rate == 0 {
        return Err("Invalid audio range".into());
    }
    let frames_f = frames as f64;
    let rate_f = f64::from(rate);
    let from = (start * rate_f).round().clamp(0.0, frames_f) as usize;
    let to = (end * rate_f).round().clamp(0.0, frames_f) as usize;
    if to <= from {
        return Err("Invalid audio range".into());
    }
    Ok((from, to))
}

fn render_output(
    channels: &[Vec<f32>],
    from_rate: u32,
    start: usize,
    end: usize,
    out_rate: u32,
) -> Result<OutputPcm, String> {
    if end <= start {
        return Err("decoded audio contains no samples".into());
    }
    let sliced = slice_channels(channels, start, end);
    let rate = if out_rate == 0 { from_rate } else { out_rate };
    let rendered = if from_rate == rate {
        sliced
    } else {
        soundcraft_dsp::offline::resample(&sliced, from_rate, rate)
    };
    let count = rendered.len();
    if count == 0 || count > u16::MAX as usize {
        return Err("audio has no channels".into());
    }
    Ok(OutputPcm {
        channels: count as u16,
        sample_rate: rate,
        interleaved: interleave(&rendered),
    })
}

fn slice_channels(channels: &[Vec<f32>], start: usize, end: usize) -> Vec<Vec<f32>> {
    channels
        .iter()
        .map(|channel| {
            let to = end.min(channel.len());
            let from = start.min(to);
            channel[from..to].to_vec()
        })
        .collect()
}

fn interleave(channels: &[Vec<f32>]) -> Vec<f32> {
    let frames = frames_of(channels);
    let mut out = Vec::with_capacity(frames.saturating_mul(channels.len()));
    for frame in 0..frames {
        for channel in channels {
            out.push(channel[frame]);
        }
    }
    out
}

/// Absolute peak of each of `n` equal columns across the whole envelope.
pub fn bars(env: &Envelope, n: usize) -> Vec<f32> {
    bars_between(env, env.origin, env.origin + env.duration_secs(), n)
}

/// Absolute peak of each of `n` columns covering source time `[t0, t1)`.
pub fn bars_between(env: &Envelope, t0: f64, t1: f64, n: usize) -> Vec<f32> {
    if n == 0
        || env.samples.is_empty()
        || env.sample_rate == 0
        || !t0.is_finite()
        || !t1.is_finite()
    {
        return Vec::new();
    }
    let rate = f64::from(env.sample_rate);
    let start = (t0 - env.origin) * rate;
    let end = (t1 - env.origin) * rate;
    let step = (end - start) / n as f64;
    if !step.is_finite() || step <= 0.0 {
        return vec![0.0; n];
    }
    env.peaks
        .spans(&env.samples, start, step, 0.0, n)
        .into_iter()
        .map(|span| span.map(|(lo, hi)| lo.abs().max(hi.abs())).unwrap_or(0.0))
        .collect()
}

/// Seconds covered by one legacy envelope bucket. Drawing uses [`Peaks`] instead.
pub const BUCKET_SECS: f32 = 0.01;

/// Max peak in `[t0, t1)` seconds, or `None` when that range is outside the turn.
pub fn peak_in_range(env: &Envelope, t0: f32, t1: f32, dur_sec: f32) -> Option<f32> {
    if t1 <= 0.0 || t0 >= dur_sec || env.samples.is_empty() || dur_sec <= 0.0 {
        return None;
    }
    let n = env.samples.len();
    let start = ((t0.max(0.0) / dur_sec) * n as f32) as usize;
    let end = ((t1.min(dur_sec) / dur_sec) * n as f32).ceil() as usize;
    let end = end.max(start.saturating_add(1)).min(n);
    let start = start.min(n.saturating_sub(1));
    if start >= end {
        return None;
    }
    let (lo, hi) = env.peaks.minmax(&env.samples, start, end);
    Some(lo.abs().max(hi.abs()))
}

pub struct Waveforms {
    receiver: std::sync::mpsc::Receiver<(usize, Envelope)>,
    envelopes: HashMap<usize, Envelope>,
    canceled: Arc<std::sync::atomic::AtomicBool>,
}

impl Waveforms {
    pub fn start(episode: &Episode, ctx: egui::Context) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let canceled = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let canceled_clone = canceled.clone();
        let mut jobs = Vec::new();
        for (i, turn) in episode.turns.iter().enumerate() {
            if !turn.wav_paths.is_empty() {
                jobs.push((i, turn.wav_paths.clone()));
            }
        }

        std::thread::spawn(move || {
            let mut computed = 0;
            for (i, audio_paths) in jobs {
                if canceled_clone.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(env) = envelope(&audio_paths) {
                    if sender.send((i, env)).is_err() {
                        break;
                    }
                    computed += 1;
                    if computed % 50 == 0 {
                        ctx.request_repaint();
                    }
                }
            }
            ctx.request_repaint();
        });

        Self {
            receiver,
            envelopes: HashMap::new(),
            canceled,
        }
    }

    pub fn poll(&mut self) {
        for (idx, env) in self.receiver.try_iter() {
            self.envelopes.insert(idx, env);
        }
    }

    pub fn get(&self, turn: usize) -> Option<&Envelope> {
        self.envelopes.get(&turn)
    }
}

impl Drop for Waveforms {
    fn drop(&mut self) {
        self.canceled.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_of(samples: Vec<f32>, rate: u32) -> Envelope {
        envelope_from_mono(samples, rate, 0.0)
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio")
            .join(name)
    }

    #[test]
    fn test_bars_empty() {
        let env = empty_envelope();
        assert_eq!(bars(&env, 0), Vec::<f32>::new());
        assert_eq!(bars(&env, 5), Vec::<f32>::new());
    }

    #[test]
    fn test_bars_n_0() {
        let env = env_of(vec![0.1, 0.2], 100);
        assert_eq!(bars(&env, 0), Vec::<f32>::new());
    }

    #[test]
    fn test_bars_n_larger() {
        let env = env_of(vec![0.1, 0.2], 100);
        let b = bars(&env, 5);
        assert_eq!(b.len(), 5);
        assert!(b[0] > 0.0);
        assert_eq!(b[4], 0.0);
    }

    #[test]
    fn test_bars_max_per_bar() {
        let env = env_of(vec![0.1, 0.9, 0.2, 0.5, 0.1, 0.1], 100);
        let b = bars(&env, 2);
        assert_eq!(b.len(), 2);
        assert!((b[0] - 0.9).abs() < 1e-5, "{b:?}");
        assert!((b[1] - 0.5).abs() < 1e-5, "{b:?}");
    }

    #[test]
    fn test_peak_in_range() {
        let env = env_of(vec![0.1, 0.9, 0.2, 0.5], 100);
        assert_eq!(peak_in_range(&env, -1.0, -0.1, 0.04), None);
        assert_eq!(peak_in_range(&env, 0.04, 0.05, 0.04), None);
        assert_eq!(peak_in_range(&env, 0.0, 0.02, 0.04), Some(0.9));
        assert_eq!(peak_in_range(&env, 0.02, 0.04, 0.04), Some(0.5));
    }

    #[test]
    fn test_envelope_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/episode_small/Samples/Processed/turn_1_0.wav");
        let bytes = std::fs::read(&path).unwrap();
        let info = soundcraft_audio_io::probe(&bytes, Some("wav")).unwrap();
        let env = envelope(&[path]).unwrap();
        assert!(!env.is_empty());
        let diff = (env.duration_secs() - info.duration_secs()).abs();
        assert!(
            diff < 0.02,
            "duration {} vs {}",
            env.duration_secs(),
            info.duration_secs()
        );
        assert_eq!(BUCKET_SECS, 0.01);
    }

    #[test]
    fn pcm_peaks_cover_one_second() {
        let bytes = vec![0u8; 48_000 * 4];
        let peaks = peaks_of_f32le(&bytes);
        assert_eq!(peaks.len(), 100);
    }

    #[test]
    fn ffmpeg_names_a_missing_binary_or_a_missing_source() {
        let error =
            ffmpeg_wav(Path::new("/tmp/scriptcast-no-such-clip.mp4"), 0.0, 1.0).unwrap_err();
        assert!(
            error.contains("ffmpeg is not available") || error.contains("ffmpeg"),
            "{error}"
        );
        let ranged = ffmpeg_f32le(Path::new("/tmp/scriptcast-no-such-clip.mp4"), 0.0, 0.1);
        assert!(ranged.is_err());
    }

    #[test]
    fn decodes_m4a_and_mp4_fixtures() {
        for name in ["tone.m4a", "tone.mp4"] {
            let path = fixture(name);
            let (env, origin) = clip_envelope(&path, 0.0, 0.05).unwrap();
            assert_eq!(origin, 0.0, "{name}");
            assert!(
                env.duration_secs() > 0.2,
                "{name} decoded {}s",
                env.duration_secs()
            );
            assert!(env.sample_rate >= 8_000, "{name}");
        }
    }

    #[test]
    fn peaks_match_brute_force_minmax() {
        let samples: Vec<f32> = (0..4_000)
            .map(|i| ((i as f32) * 0.017).sin() + if i % 250 == 0 { 0.4 } else { 0.0 })
            .collect();
        let peaks = Peaks::build(&samples);
        for (start, end) in [(0, 32), (10, 3_900), (31, 33), (100, 101), (0, 4_000)] {
            let got = peaks.minmax(&samples, start, end);
            let window = &samples[start..end];
            let want = (
                window.iter().copied().fold(f32::INFINITY, f32::min),
                window.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            );
            assert_eq!(got, want, "{start}..{end}");
        }
    }

    #[test]
    fn cache_is_built_once_across_repeated_frames() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/episode_small/Samples/Processed/turn_1_0.wav");
        let mut seen = Vec::new();
        let before = cache_builds();
        for _ in 0..8 {
            let env = envelope(std::slice::from_ref(&path)).unwrap();
            let _ = bars(&env, 16);
            let _ = peak_in_range(&env, 0.0, 0.05, env.duration_secs() as f32);
            seen.push((env.samples, env.peaks));
        }
        for (samples, peaks) in &seen[1..] {
            assert!(Arc::ptr_eq(&seen[0].0, samples));
            assert!(Arc::ptr_eq(&seen[0].1, peaks));
        }
        assert!(cache_builds() >= before);
    }

    #[test]
    fn clip_maker_envelope_spans_the_whole_window() {
        let path = fixture("tone.mp4");
        let (env, origin) = clip_envelope(&path, 0.0, 0.05).unwrap();
        assert_eq!(origin, 0.0);
        let dur = env.duration_secs();
        assert!(dur > 0.2, "decoded {dur}s of a 0.25s source");
        let cols = bars_between(&env, 0.0, dur, 30);
        assert_eq!(cols.len(), 30);
        let late = cols[20..].iter().copied().fold(0.0_f32, f32::max);
        assert!(
            late > 0.05,
            "the last third of the window is silent: {cols:?}"
        );
    }

    #[test]
    fn output_range_is_an_exact_sample_slice() {
        let samples: Vec<f32> = (0..1_000).map(|i| i as f32).collect();
        let rendered = render_output(&[samples], 1_000, 100, 110, 1_000).unwrap();
        assert_eq!(rendered.channels, 1);
        assert_eq!(rendered.sample_rate, 1_000);
        assert_eq!(
            rendered.interleaved,
            (100..110).map(|i| i as f32).collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "decodes every clips.yaml source and prints native vs ffmpeg timings"]
    fn archive_clip_sources() {
        let roots = [
            PathBuf::from("/tmp/scriptcast-k/proj"),
            PathBuf::from("/Users/emmanuel/Documents/Theory/TheOriginalPower"),
        ];
        let root = roots
            .into_iter()
            .find(|root| {
                root.join("Architecting_the_operation/archive/clips.yaml")
                    .is_file()
            })
            .expect("clips.yaml");
        let yaml =
            std::fs::read_to_string(root.join("Architecting_the_operation/archive/clips.yaml"))
                .unwrap();
        let mut native = 0u32;
        let mut ffmpeg = 0u32;
        let mut failed = 0u32;
        let mut seen = std::collections::HashSet::new();
        for line in yaml.lines() {
            let Some(source) = line.trim().strip_prefix("source:") else {
                continue;
            };
            let rel = source.trim().trim_matches('"');
            if !seen.insert(rel.to_owned()) {
                continue;
            }
            let path = root.join(rel);
            let started = std::time::Instant::now();
            match decode_path(&path) {
                Ok(file) => {
                    let elapsed = started.elapsed();
                    let seconds = file.mono.len() as f64 / f64::from(file.sample_rate.max(1));
                    if file.via_ffmpeg {
                        ffmpeg += 1;
                    } else {
                        native += 1;
                    }
                    eprintln!(
                        "{} {:.3}s audio in {:.3}s {}",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or(rel),
                        seconds,
                        elapsed.as_secs_f64(),
                        if file.via_ffmpeg { "ffmpeg" } else { "native" }
                    );
                }
                Err(error) => {
                    failed += 1;
                    eprintln!(
                        "{} FAILED in {:.3}s: {error}",
                        rel,
                        started.elapsed().as_secs_f64()
                    );
                }
            }
        }
        eprintln!("native {native} ffmpeg {ffmpeg} failed {failed}");
        assert_eq!(failed, 0);
        assert!(native + ffmpeg > 0);
    }
}
