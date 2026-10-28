//! `scriptcast-media carve` command line.

use std::path::{Path, PathBuf};

use serde::Serialize;
use soundcraft_audio_io::{AudioBuffer, AudioInfo, BitDepth, EncodeOptions, FileFormat, encode};

use crate::analyse::{CarveSource, analyse_carve_bands, intelligibility_bands, mix_carve_sources};
use crate::profile::{CarveBand, DEFAULT_STRENGTH, carve_profile};
use crate::render::{CarveSettings, carve_bed};
use crate::span::{DEFAULT_MERGE_GAP_MS, SpeechSpan, spans_from_manifest};
use crate::{DEFAULT_ATTACK_MS, DEFAULT_RELEASE_MS};

const HELP: &str = "\
scriptcast-media carve --manifest episode_manifest.json --bed bed.wav --out carved.wav

Duck a music bed under speech, and dip the bands that mask a voice while
someone is talking. Speech times come from the manifest's turn windows.
Turns separated by less than --merge-gap-ms are one window. The bed returns
to flat between windows.

  --manifest PATH       episode_manifest.json
  --bed PATH            music bed (wav, flac, mp3, ...)
  --out PATH            output wav, 32-bit float
  --strength N          0..1, how hard to carve (default 0.25, which is 6 dB)
  --merge-gap-ms N      join turns closer than this (default 300)
  --attack-ms N         ramp into the duck (default 150)
  --release-ms N        ramp back out (default 400)
  --voice PATH          optional recording; the dipped bands are chosen from it
  --voice-offset-ms N   where the preceding --voice starts, in ms (default 0)
  --json                print the plan as JSON
  --dry-run             print the plan and write nothing

With no --voice, the dips follow the centres nearest 2 kHz. At the
default strength those are 1, 1.6 and 2.5 kHz. Pass --voice to pick
the bands from a recording instead.
";

#[derive(Debug)]
struct CarveArgs {
    manifest: PathBuf,
    bed: PathBuf,
    out: PathBuf,
    strength: f64,
    merge_gap_ms: u64,
    attack_ms: f64,
    release_ms: f64,
    voices: Vec<VoiceArg>,
    json: bool,
    dry_run: bool,
}

#[derive(Debug)]
struct VoiceArg {
    path: PathBuf,
    offset_ms: f64,
}

#[derive(Serialize)]
struct Report {
    turns: usize,
    merge_gap_ms: u64,
    spans: Vec<SpeechSpan>,
    strength: f64,
    duck_db: f64,
    attack_ms: f64,
    release_ms: f64,
    bands: Vec<CarveBand>,
    band_source: String,
    sample_rate: Option<u32>,
    frames: Option<u64>,
    dry_run: bool,
    wrote: Option<String>,
}

/// Run the CLI. `args` is everything after the program name.
pub fn run(args: &[String]) -> Result<(), String> {
    match parse(args)? {
        None => {
            println!("{HELP}");
            Ok(())
        }
        Some(carve) => {
            let report = execute(&carve)?;
            if carve.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                );
            } else {
                print_report(&report);
            }
            Ok(())
        }
    }
}

fn execute(args: &CarveArgs) -> Result<Report, String> {
    let json = std::fs::read_to_string(&args.manifest)
        .map_err(|err| format!("could not read {}: {err}", args.manifest.display()))?;
    let plan = spans_from_manifest(&json, args.merge_gap_ms).map_err(|err| err.to_string())?;
    let profile = carve_profile(args.strength);
    let (bands, band_source) = choose_bands(args, &profile)?;
    let settings = CarveSettings {
        duck_db: profile.duck_db,
        attack_ms: args.attack_ms,
        release_ms: args.release_ms,
    };

    if args.dry_run {
        return Ok(report(
            args,
            &plan.spans,
            plan.turn_count,
            &bands,
            band_source,
            None,
            None,
            None,
        ));
    }
    if !is_wav(&args.out) {
        return Err(format!(
            "output must be a .wav file, got {}",
            args.out.display()
        ));
    }

    let (info, mut bed) = read_audio(&args.bed)?;
    if bed.channels.is_empty() || bed.frames() == 0 {
        return Err(format!("{} has no audio", args.bed.display()));
    }
    let sample_rate = info.sample_rate;
    let frames = bed.frames() as u64;
    carve_bed(
        &mut bed.channels,
        sample_rate,
        &plan.spans,
        &bands,
        &settings,
    )
    .map_err(|err| err.to_string())?;
    write_wav(&args.out, sample_rate, bed.channels)?;
    Ok(report(
        args,
        &plan.spans,
        plan.turn_count,
        &bands,
        band_source,
        Some(sample_rate),
        Some(frames),
        Some(args.out.display().to_string()),
    ))
}

fn report(
    args: &CarveArgs,
    spans: &[SpeechSpan],
    turns: usize,
    bands: &[CarveBand],
    band_source: String,
    sample_rate: Option<u32>,
    frames: Option<u64>,
    wrote: Option<String>,
) -> Report {
    let profile = carve_profile(args.strength);
    Report {
        turns,
        merge_gap_ms: args.merge_gap_ms,
        spans: spans.to_vec(),
        strength: args.strength,
        duck_db: profile.duck_db,
        attack_ms: args.attack_ms,
        release_ms: args.release_ms,
        bands: bands.to_vec(),
        band_source,
        sample_rate,
        frames,
        dry_run: args.dry_run,
        wrote,
    }
}

fn choose_bands(
    args: &CarveArgs,
    profile: &crate::CarveProfile,
) -> Result<(Vec<CarveBand>, String), String> {
    if args.voices.is_empty() {
        return Ok((
            intelligibility_bands(profile),
            "intelligibility".to_string(),
        ));
    }
    let mut monos = Vec::with_capacity(args.voices.len());
    let mut sample_rate: Option<u32> = None;
    for voice in &args.voices {
        let (info, buffer) = read_audio(&voice.path)?;
        if info.sample_rate == 0 {
            return Err(format!("{}: sample rate is 0", voice.path.display()));
        }
        if let Some(expected) = sample_rate
            && expected != info.sample_rate
        {
            return Err(format!(
                "{} is {} Hz, but the first voice is {expected} Hz",
                voice.path.display(),
                info.sample_rate
            ));
        }
        sample_rate = Some(info.sample_rate);
        monos.push((mono(&buffer), voice.offset_ms));
    }
    let sample_rate = sample_rate.unwrap_or(0);
    let sources: Vec<CarveSource<'_>> = monos
        .iter()
        .map(|(samples, offset_ms)| CarveSource {
            samples,
            offset_seconds: offset_ms / 1000.0,
        })
        .collect();
    let mixed = mix_carve_sources(&sources, sample_rate);
    let bands = analyse_carve_bands(&mixed, sample_rate, profile);
    if bands.is_empty() {
        Ok((bands, "voice (silent, no bands)".to_string()))
    } else {
        Ok((bands, "voice".to_string()))
    }
}

fn print_report(report: &Report) {
    println!(
        "turns  {}, merged to {} span{} (gap < {} ms)",
        report.turns,
        report.spans.len(),
        if report.spans.len() == 1 { "" } else { "s" },
        report.merge_gap_ms
    );
    println!(
        "duck   {:.2} dB, attack {:.0} ms, release {:.0} ms, strength {}",
        report.duck_db, report.attack_ms, report.release_ms, report.strength
    );
    if report.bands.is_empty() {
        println!("bands  none ({})", report.band_source);
    } else {
        let bands: Vec<String> = report
            .bands
            .iter()
            .map(|band| {
                format!(
                    "{:.0} Hz {:+.2} dB Q {:.2}",
                    band.freq_hz, band.gain_db, band.q
                )
            })
            .collect();
        println!("bands  {} ({})", bands.join(", "), report.band_source);
    }
    match &report.wrote {
        Some(path) => println!("wrote  {path}"),
        None if report.dry_run => println!("dry run: nothing written"),
        None => {}
    }
}

fn read_audio(path: &Path) -> Result<(AudioInfo, AudioBuffer), String> {
    let bytes =
        std::fs::read(path).map_err(|err| format!("could not read {}: {err}", path.display()))?;
    let hint = path.extension().and_then(|ext| ext.to_str());
    soundcraft_audio_io::decode(&bytes, hint)
        .map_err(|err| format!("could not decode {}: {err}", path.display()))
}

fn write_wav(path: &Path, sample_rate: u32, channels: Vec<Vec<f32>>) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("could not create {}: {err}", parent.display()))?;
    }
    let buffer = AudioBuffer {
        sample_rate,
        channels,
    };
    let options = EncodeOptions {
        format: FileFormat::Wav,
        bit_depth: BitDepth::Float32,
        dither: false,
        bwf: None,
    };
    let bytes = encode(&buffer, &options).map_err(|err| format!("could not encode wav: {err}"))?;
    std::fs::write(path, bytes).map_err(|err| format!("could not write {}: {err}", path.display()))
}

fn mono(buffer: &AudioBuffer) -> Vec<f32> {
    let frames = buffer.frames();
    let channels = buffer.num_channels().max(1) as f32;
    let mut mixed = vec![0.0f32; frames];
    for channel in &buffer.channels {
        for (frame, sample) in channel.iter().take(frames).enumerate() {
            mixed[frame] += sample / channels;
        }
    }
    mixed
}

fn is_wav(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => {
            let ext = ext.to_ascii_lowercase();
            ext == "wav" || ext == "wave"
        }
        None => false,
    }
}

/// `Ok(None)` means the caller asked for help.
fn parse(args: &[String]) -> Result<Option<CarveArgs>, String> {
    if args.is_empty() {
        return Err(HELP.trim_end().to_string());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(None);
    }
    if args[0] != "carve" {
        return Err(format!("unknown command '{}'\n\n{HELP}", args[0]));
    }
    let mut manifest = None;
    let mut bed = None;
    let mut out = None;
    let mut strength = DEFAULT_STRENGTH;
    let mut merge_gap_ms = DEFAULT_MERGE_GAP_MS;
    let mut attack_ms = DEFAULT_ATTACK_MS;
    let mut release_ms = DEFAULT_RELEASE_MS;
    let mut voices: Vec<VoiceArg> = Vec::new();
    let mut json = false;
    let mut dry_run = false;

    let mut index = 1;
    while index < args.len() {
        let (flag, inline) = split_flag(&args[index]);
        let mut take = |name: &str| -> Result<String, String> {
            if let Some(value) = inline {
                return Ok(value.to_string());
            }
            index += 1;
            args.get(index)
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match flag {
            "--manifest" => manifest = Some(PathBuf::from(take(flag)?)),
            "--bed" => bed = Some(PathBuf::from(take(flag)?)),
            "--out" => out = Some(PathBuf::from(take(flag)?)),
            "--strength" => strength = parse_strength(&take(flag)?)?,
            "--merge-gap-ms" => merge_gap_ms = parse_u64(&take(flag)?, flag)?,
            "--attack-ms" => attack_ms = parse_nonneg(&take(flag)?, flag)?,
            "--release-ms" => release_ms = parse_nonneg(&take(flag)?, flag)?,
            "--voice" => voices.push(VoiceArg {
                path: PathBuf::from(take(flag)?),
                offset_ms: 0.0,
            }),
            "--voice-offset-ms" => {
                let offset_ms = parse_f64(&take(flag)?, flag)?;
                let voice = voices
                    .last_mut()
                    .ok_or("--voice-offset-ms needs a preceding --voice")?;
                voice.offset_ms = offset_ms;
            }
            "--json" => json = true,
            "--dry-run" => dry_run = true,
            other => return Err(format!("unknown flag {other}\n\n{HELP}")),
        }
        index += 1;
    }

    let manifest = manifest.ok_or("missing --manifest")?;
    let bed = bed.ok_or("missing --bed")?;
    let out = out.ok_or("missing --out")?;
    Ok(Some(CarveArgs {
        manifest,
        bed,
        out,
        strength,
        merge_gap_ms,
        attack_ms,
        release_ms,
        voices,
        json,
        dry_run,
    }))
}

fn split_flag(arg: &str) -> (&str, Option<&str>) {
    match arg.split_once('=') {
        Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
        _ => (arg, None),
    }
}

fn parse_strength(value: &str) -> Result<f64, String> {
    let number = parse_f64(value, "--strength")?;
    if (0.0..=1.0).contains(&number) {
        Ok(number)
    } else {
        Err("--strength must be a number from 0 to 1".to_string())
    }
}

fn parse_nonneg(value: &str, flag: &str) -> Result<f64, String> {
    let number = parse_f64(value, flag)?;
    if number >= 0.0 {
        Ok(number)
    } else {
        Err(format!("{flag} must be zero or positive"))
    }
}

fn parse_f64(value: &str, flag: &str) -> Result<f64, String> {
    let number: f64 = value
        .parse()
        .map_err(|_| format!("{flag} needs a number, got {value:?}"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("{flag} needs a finite number"))
    }
}

fn parse_u64(value: &str, flag: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} needs a whole number of milliseconds, got {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn strength_outside_the_knob_is_rejected() {
        let err = parse(&args(&[
            "carve",
            "--manifest",
            "m.json",
            "--bed",
            "b.wav",
            "--out",
            "o.wav",
            "--strength",
            "2",
        ]))
        .unwrap_err();
        assert!(err.contains("0 to 1"), "{err}");
    }

    #[test]
    fn dry_run_merges_turns_and_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("scriptcast-media-dry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = dir.join("episode_manifest.json");
        std::fs::write(
            &manifest,
            r#"{"turns":[{"start_ms":0,"end_ms":1000},{"start_ms":1200,"end_ms":2000}]}"#,
        )
        .unwrap();
        let out = dir.join("carved.wav");
        let parsed = parse(&args(&[
            "carve",
            "--manifest",
            manifest.to_str().unwrap(),
            "--bed",
            "missing.wav",
            "--out",
            out.to_str().unwrap(),
            "--dry-run",
        ]))
        .unwrap()
        .unwrap();
        let report = execute(&parsed).unwrap();
        assert!(report.dry_run);
        assert_eq!(report.turns, 2);
        assert_eq!(report.spans.len(), 1);
        assert_eq!(report.spans[0].end_ms, 2000);
        assert_eq!(report.band_source, "intelligibility");
        assert!(!out.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn writes_a_bed_that_is_quieter_while_speech_is_open() {
        let dir = std::env::temp_dir().join(format!("scriptcast-media-wav-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = dir.join("episode_manifest.json");
        std::fs::write(&manifest, r#"{"turns":[{"start_ms":0,"end_ms":1000}]}"#).unwrap();
        let bed = dir.join("bed.wav");
        let out = dir.join("carved.wav");
        let frames = 48_000usize;
        let tone: Vec<f32> = (0..frames)
            .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 2000.0 * i as f32 / 48_000.0).sin())
            .collect();
        write_wav(&bed, 48_000, vec![tone.clone()]).unwrap();

        let parsed = parse(&args(&[
            "carve",
            "--manifest",
            manifest.to_str().unwrap(),
            "--bed",
            bed.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--attack-ms",
            "20",
        ]))
        .unwrap()
        .unwrap();
        let report = execute(&parsed).unwrap();
        assert_eq!(report.wrote.as_deref(), Some(out.to_str().unwrap()));

        let (_info, carved) = read_audio(&out).unwrap();
        let late = frames / 2;
        let input_rms = rms(&tone[late..]);
        let output_rms = rms(&carved.channels[0][late..]);
        let ratio = output_rms / input_rms;
        assert!(
            ratio < 0.5,
            "expected a clear dip, rms ratio was {ratio:.3}"
        );
        assert!(ratio > 0.05, "bed was wiped out, rms ratio was {ratio:.3}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn rms(samples: &[f32]) -> f64 {
        let energy: f64 = samples
            .iter()
            .map(|sample| f64::from(*sample).powi(2))
            .sum();
        (energy / samples.len() as f64).sqrt()
    }
}
