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
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| e.to_string())?;

        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        // 10 ms bucket
        let items_per_bucket = ((sample_rate.get() as usize) * (channels.get() as usize)) / 100;

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
    }
    Ok(Envelope { peaks })
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
}
