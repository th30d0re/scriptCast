use crate::episode::Episode;
use eframe::egui;
use rodio::{Decoder, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct Envelope {
    pub peaks: Vec<f32>,
}

pub fn envelope(paths: &[PathBuf]) -> Result<Envelope, String> {
    let mut peaks = Vec::new();
    for path in paths {
        peaks.extend(peaks_of_file(path)?);
    }
    Ok(Envelope { peaks })
}

fn peaks_of_file(path: &std::path::Path) -> Result<Vec<f32>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    peaks_of_reader(BufReader::new(file)).map_err(|e| format!("{}: {e}", path.display()))
}

fn peaks_of_reader(
    reader: impl std::io::Read + std::io::Seek + Send + Sync + 'static,
) -> Result<Vec<f32>, String> {
    let decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    // 10 ms bucket
    let items_per_bucket = ((sample_rate.get() as usize) * (channels.get() as usize)) / 100;
    let mut peaks = Vec::new();
    let mut current_peak = 0.0_f32;
    let mut count = 0;
    for sample in decoder {
        let val = sample.abs();
        if val > current_peak {
            current_peak = val;
        }
        count += 1;
        if count == items_per_bucket {
            peaks.push(current_peak);
            current_peak = 0.0;
            count = 0;
        }
    }
    if count > 0 {
        peaks.push(current_peak);
    }
    Ok(peaks)
}

/// Seconds of source audio kept on either side of a clip when ffmpeg builds the waveform.
pub const CLIP_CONTEXT_SECS: f64 = 2.0;

/// Decode `[start, start + len)` to a mono 48 kHz WAV in memory.
/// `ffmpeg is not available` when the binary is missing.
pub fn ffmpeg_wav(path: &std::path::Path, start: f64, len: f64) -> Result<Vec<u8>, String> {
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

/// True when `peaks` cover `seconds` (10 ms buckets). A few buckets of
/// container padding still count; a decode that stops a third of the way does not.
pub fn covers_window(peaks: &[f32], seconds: f64) -> bool {
    if seconds <= f64::from(BUCKET_SECS) {
        return !peaks.is_empty();
    }
    let got = peaks.len() as f64 * f64::from(BUCKET_SECS);
    got + 0.1 >= seconds * 0.98
}

fn peaks_of_f32le(bytes: &[u8]) -> Vec<f32> {
    const RATE: usize = 48_000;
    let per = RATE / 100;
    let mut peaks = Vec::new();
    let mut peak = 0.0_f32;
    let mut count = 0usize;
    let mut index = 0;
    while index + 4 <= bytes.len() {
        let mut chunk = [0u8; 4];
        chunk.copy_from_slice(&bytes[index..index + 4]);
        let sample = f32::from_le_bytes(chunk);
        peak = peak.max(sample.abs());
        count += 1;
        index += 4;
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
fn ffmpeg_f32le(path: &std::path::Path, start: f64, len: f64) -> Result<Vec<u8>, String> {
    if !start.is_finite() || !len.is_finite() || start < 0.0 || len <= 0.0 {
        return Err("Invalid audio range".into());
    }
    let output = match std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-ss"])
        .arg(format!("{start:.3}"))
        .arg("-i")
        .arg(path)
        .arg("-t")
        .arg(format!("{len:.3}"))
        .args(["-vn", "-f", "f32le", "-ac", "1", "-ar", "48000", "-"])
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
    if output.stdout.len() < 4 {
        return Err("ffmpeg produced no audio".into());
    }
    Ok(output.stdout)
}

/// Full-file peaks when rodio covers the requested window. Otherwise ffmpeg PCM
/// for that window. The `f64` is the window's start in the source, or `0` for a
/// full-file decode.
pub fn clip_envelope(
    path: &std::path::Path,
    start: f64,
    end: f64,
) -> Result<(Envelope, f64), String> {
    let window = (end - start).max(f64::from(BUCKET_SECS));
    let native = match peaks_of_file(path) {
        Ok(peaks) if covers_window(&peaks, window) => {
            return Ok((Envelope { peaks }, 0.0));
        }
        Ok(_) => "decode ended before the end of the source".to_owned(),
        Err(error) => error,
    };
    let ss = (start - CLIP_CONTEXT_SECS).max(0.0);
    let len = (end - ss).max(window) + CLIP_CONTEXT_SECS;
    let bytes = ffmpeg_f32le(path, ss, len).map_err(|ffmpeg| format!("{native}; {ffmpeg}"))?;
    let peaks = peaks_of_f32le(&bytes);
    if !covers_window(&peaks, window) {
        return Err(format!(
            "{native}; ffmpeg decoded {:.2} s of the requested {:.2} s",
            peaks.len() as f64 * f64::from(BUCKET_SECS),
            window
        ));
    }
    Ok((Envelope { peaks }, ss))
}

pub fn bars(env: &Envelope, n: usize) -> Vec<f32> {
    if n == 0 || env.peaks.is_empty() {
        return Vec::new();
    }
    let mut res = Vec::with_capacity(n);
    let peaks_len = env.peaks.len();
    for i in 0..n {
        let start = (i * peaks_len) / n;
        let end = ((i + 1) * peaks_len) / n;

        // Ensure we look at at least 1 peak if possible, but keep end >= start
        let actual_end = std::cmp::max(start + 1, end);
        let actual_end = std::cmp::min(actual_end, peaks_len);
        let actual_start = std::cmp::min(start, peaks_len - 1);

        let mut max = 0.0_f32;
        for v in &env.peaks[actual_start..actual_end] {
            if *v > max {
                max = *v;
            }
        }
        res.push(max);
    }
    res
}

/// Seconds covered by one [`Envelope`] peak bucket (see [`envelope`]).
pub const BUCKET_SECS: f32 = 0.01;

/// Max peak in `[t0, t1)` seconds, or `None` when that range is outside the turn.
pub fn peak_in_range(env: &Envelope, t0: f32, t1: f32, dur_sec: f32) -> Option<f32> {
    if t1 <= 0.0 || t0 >= dur_sec || env.peaks.is_empty() {
        return None;
    }
    let start = ((t0.max(0.0) / BUCKET_SECS) as usize).min(env.peaks.len());
    let end = ((t1.min(dur_sec) / BUCKET_SECS).ceil() as usize)
        .max(start + 1)
        .min(env.peaks.len());
    if start >= end {
        return None;
    }
    Some(
        env.peaks[start..end]
            .iter()
            .copied()
            .fold(0.0_f32, f32::max),
    )
}

pub struct Waveforms {
    receiver: Receiver<(usize, Envelope)>,
    envelopes: HashMap<usize, Envelope>,
    canceled: Arc<AtomicBool>,
}

impl Waveforms {
    pub fn start(episode: &Episode, ctx: egui::Context) -> Self {
        let (sender, receiver) = mpsc::channel();
        let canceled = Arc::new(AtomicBool::new(false));

        let canceled_clone = canceled.clone();
        let mut jobs = Vec::new();
        for (i, turn) in episode.turns.iter().enumerate() {
            if !turn.wav_paths.is_empty() {
                jobs.push((i, turn.wav_paths.clone()));
            }
        }

        thread::spawn(move || {
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

    #[test]
    fn test_bars_empty() {
        let env = Envelope { peaks: vec![] };
        assert_eq!(bars(&env, 0), Vec::<f32>::new());
        assert_eq!(bars(&env, 5), Vec::<f32>::new());
    }

    #[test]
    fn test_bars_n_0() {
        let env = Envelope {
            peaks: vec![0.1, 0.2],
        };
        assert_eq!(bars(&env, 0), Vec::<f32>::new());
    }

    #[test]
    fn test_bars_n_larger() {
        let env = Envelope {
            peaks: vec![0.1, 0.2],
        };
        let b = bars(&env, 5);
        assert_eq!(b.len(), 5);
        assert_eq!(b[0], 0.1);
        assert_eq!(b[1], 0.1);
        assert_eq!(b[2], 0.1);
        assert_eq!(b[3], 0.2);
        assert_eq!(b[4], 0.2);
    }

    #[test]
    fn test_bars_max_per_bar() {
        let env = Envelope {
            peaks: vec![0.1, 0.9, 0.2, 0.5, 0.1, 0.1],
        };
        let b = bars(&env, 2);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0], 0.9);
        assert_eq!(b[1], 0.5);
    }

    #[test]
    fn test_peak_in_range() {
        let env = Envelope {
            peaks: vec![0.1, 0.9, 0.2, 0.5],
        };
        assert_eq!(peak_in_range(&env, -1.0, -0.1, 0.04), None);
        assert_eq!(peak_in_range(&env, 0.04, 0.05, 0.04), None);
        assert_eq!(peak_in_range(&env, 0.0, 0.02, 0.04), Some(0.9));
        assert_eq!(peak_in_range(&env, 0.02, 0.04, 0.04), Some(0.5));
    }

    #[test]
    fn test_envelope_fixture() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/episode_small/Samples/Processed");
        let path = dir.join("turn_1_0.wav");
        let file = File::open(&path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        let dur = decoder.total_duration().unwrap().as_millis() as usize;
        let expected_buckets = dur / 10;

        let env = envelope(&[path]).unwrap();
        for &p in &env.peaks {
            assert!((0.0..=1.0).contains(&p));
        }
        let diff = (env.peaks.len() as isize - expected_buckets as isize).abs();
        assert!(
            diff <= 1,
            "Expected ~{} buckets, got {}",
            expected_buckets,
            env.peaks.len()
        );
    }

    #[test]
    fn pcm_peaks_cover_one_second_and_a_short_decode_does_not() {
        let bytes = vec![0u8; 48_000 * 4];
        let peaks = peaks_of_f32le(&bytes);
        assert_eq!(peaks.len(), 100);
        assert!(covers_window(&peaks, 1.0));
        assert!(!covers_window(&peaks[..30], 1.0));
    }

    #[test]
    fn ffmpeg_names_a_missing_binary_or_a_missing_source() {
        let error = ffmpeg_wav(
            std::path::Path::new("/tmp/scriptcast-no-such-clip.mp4"),
            0.0,
            1.0,
        )
        .unwrap_err();
        assert!(
            error.contains("ffmpeg is not available") || error.contains("ffmpeg"),
            "{error}"
        );
    }
}
