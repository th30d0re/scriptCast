//! Linear crossfade of two mono PCM wav files.

use std::fs;
use std::path::{Path, PathBuf};

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

struct Wav {
    rate: u32,
    bits: u16,
    samples: Vec<i32>,
}

fn read_mono(path: &Path) -> Result<Wav, String> {
    let name = path.display();
    let mut reader = WavReader::open(path).map_err(|e| format!("{name}: {e}"))?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err(format!(
            "{name}: expected mono, found {} channels",
            spec.channels
        ));
    }
    if spec.sample_format != SampleFormat::Int || !matches!(spec.bits_per_sample, 16 | 32) {
        return Err(format!(
            "{name}: expected 16-bit or 32-bit PCM, found {}-bit {:?}",
            spec.bits_per_sample, spec.sample_format
        ));
    }
    let samples = reader
        .samples::<i32>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(Wav {
        rate: spec.sample_rate,
        bits: spec.bits_per_sample,
        samples,
    })
}

/// Join `a` and `b`, overlapping `fade` frames with a linear ramp.
/// Both slices must hold at least `fade` frames.
pub fn crossfade(a: &[i32], b: &[i32], fade: usize, bits: u16) -> Vec<i32> {
    let (lo, hi) = if bits == 16 {
        (f64::from(i16::MIN), f64::from(i16::MAX))
    } else {
        (f64::from(i32::MIN), f64::from(i32::MAX))
    };
    let tail = a.len() - fade;
    let mut out = Vec::with_capacity(a.len() + b.len() - fade);
    out.extend_from_slice(&a[..tail]);
    for i in 0..fade {
        let t = (i + 1) as f64 / (fade + 1) as f64;
        let mixed = f64::from(a[tail + i]) * (1.0 - t) + f64::from(b[i]) * t;
        out.push(mixed.round().clamp(lo, hi) as i32);
    }
    out.extend_from_slice(&b[fade..]);
    out
}

/// Crossfade two wav files into `out`. Nothing is written on error.
pub fn crossfade_files(a: &Path, b: &Path, out: &Path, fade_ms: f64) -> Result<(), String> {
    if !fade_ms.is_finite() || fade_ms < 0.0 {
        return Err(format!("--fade-ms must be zero or more, got {fade_ms}"));
    }
    let a = read_mono(a)?;
    let b = read_mono(b)?;
    if a.rate != b.rate {
        return Err(format!(
            "sample rates differ: {} Hz and {} Hz",
            a.rate, b.rate
        ));
    }
    let fade = (fade_ms * f64::from(a.rate) / 1000.0).round() as usize;
    if a.samples.len() < fade || b.samples.len() < fade {
        return Err(format!(
            "a file is shorter than the {fade}-frame fade ({} and {} frames)",
            a.samples.len(),
            b.samples.len()
        ));
    }

    // Mixed depths are joined at the wider one; 16-bit samples are scaled up.
    let bits = a.bits.max(b.bits);
    let widen = |wav: &Wav| -> Vec<i32> {
        if wav.bits < bits {
            wav.samples.iter().map(|&s| s << 16).collect()
        } else {
            wav.samples.clone()
        }
    };
    let joined = crossfade(&widen(&a), &widen(&b), fade, bits);

    let spec = WavSpec {
        channels: 1,
        sample_rate: a.rate,
        bits_per_sample: bits,
        sample_format: SampleFormat::Int,
    };
    let mut tmp_name = out.as_os_str().to_owned();
    tmp_name.push(".part");
    let tmp = PathBuf::from(tmp_name);
    let write = || -> Result<(), hound::Error> {
        let mut writer = WavWriter::create(&tmp, spec)?;
        for &sample in &joined {
            writer.write_sample(sample)?;
        }
        writer.finalize()
    };
    if let Err(e) = write() {
        let _ = fs::remove_file(&tmp);
        return Err(format!("{}: {e}", out.display()));
    }
    fs::rename(&tmp, out).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", out.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_is_sum_minus_fade() {
        let out = crossfade(&[1000; 100], &[-1000; 80], 20, 16);
        assert_eq!(out.len(), 160);
    }

    #[test]
    fn ramp_moves_from_a_to_b() {
        let out = crossfade(&[1000; 10], &[0; 10], 4, 16);
        assert_eq!(&out[..6], &[1000; 6]);
        assert_eq!(&out[6..10], &[800, 600, 400, 200]);
        assert_eq!(&out[10..], &[0; 6]);
    }

    #[test]
    fn zero_fade_concatenates() {
        assert_eq!(crossfade(&[1, 2], &[3], 0, 16), [1, 2, 3]);
    }
}
