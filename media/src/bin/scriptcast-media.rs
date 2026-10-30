//! `scriptcast-media duck|carve|lane`: make room for dialogue in a music bed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use scriptcast_media::{
    Activity, CarveParams, DuckParams, Result, VoiceSource, analyse_carve_bands, carve, carve_profile, carve_with, duck, duck_lane,
    intelligibility_bands, merge_spans, mix_voices, mono, read_wav, spans_from_json, write_wav,
};
use serde_json::json;

const USAGE: &str = "usage:
  scriptcast-media duck  --bed BED.wav --spans MANIFEST_OR_SPANS.json --out OUT.wav
                         [--target-db -12 | --strength 0..1] [--attack-ms 150] [--release-ms 400] [--merge-gap-ms 300]
                         [--json] [--dry-run]
  scriptcast-media carve --bed BED.wav (--voice VOICE.wav [--voice-offset-ms MS] ... | --spans MANIFEST_OR_SPANS.json)
                         --out OUT.wav [--max-cut-db 14.8 --q 2.06 | --strength 0..1] [--merge-gap-ms 300] [--json] [--dry-run]
  scriptcast-media lane  --spans MANIFEST_OR_SPANS.json --duration-ms MS [--step-ms 10]
                         [--attack-ms 150] [--release-ms 400] [--merge-gap-ms 300]

--spans takes episode_manifest.json (turns[].start_ms/end_ms) or [[start_ms, end_ms], ...].
--voice repeats; each --voice-offset-ms places the --voice before it on the bed's clock.
--strength sets depth from one knob (0.25 is a gentle default). On carve it also dips only the
  bands the voices occupy (or, with no voice, the ones nearest 2 kHz) instead of all nine.
--dry-run prints the plan (spans, bands, settings) and writes nothing.
lane prints the duck envelope, 0 (open) to 1 (ducked), every --step-ms, as JSON.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("scriptcast-media: {e}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

struct Voice {
    path: PathBuf,
    offset_ms: f64,
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().ok_or("missing command")?;
    let mut opts: HashMap<String, String> = HashMap::new();
    let mut voices: Vec<Voice> = Vec::new();
    let (mut json, mut dry_run) = (false, false);
    while let Some(a) = args.next() {
        let key = a.strip_prefix("--").ok_or_else(|| format!("unexpected argument {a}"))?;
        match key {
            "json" => json = true,
            "dry-run" => dry_run = true,
            _ => {
                let value = args.next().ok_or_else(|| format!("--{key} needs a value"))?;
                match key {
                    "voice" => voices.push(Voice { path: value.into(), offset_ms: 0.0 }),
                    "voice-offset-ms" => voices.last_mut().ok_or("--voice-offset-ms needs a --voice before it")?.offset_ms = value.parse()?,
                    _ => {
                        opts.insert(key.to_string(), value);
                    }
                }
            }
        }
    }
    let path = |k: &str| opts.get(k).map(PathBuf::from).ok_or_else(|| format!("--{k} is required"));
    let num = |k: &str, d: f64| -> Result<f64> { Ok(opts.get(k).map(|v| v.parse()).transpose()?.unwrap_or(d)) };
    let spans = || -> Result<Vec<_>> { spans_from_json(&std::fs::read_to_string(path("spans")?)?) };
    let strength = opts.get("strength").map(|v| v.parse::<f64>()).transpose()?;
    if let Some(s) = strength
        && !(0.0..=1.0).contains(&s)
    {
        return Err("--strength must be a number from 0 to 1".into());
    }
    let d = DuckParams::default();
    let timing = || -> Result<DuckParams> {
        Ok(DuckParams {
            target_db: match strength {
                Some(_) if opts.contains_key("target-db") => return Err("use --target-db or --strength, not both".into()),
                Some(s) => -carve_profile(s).duck_db,
                None => num("target-db", d.target_db)?,
            },
            attack_ms: num("attack-ms", d.attack_ms)?,
            release_ms: num("release-ms", d.release_ms)?,
            merge_gap_ms: num("merge-gap-ms", d.merge_gap_ms)?,
        })
    };

    if cmd == "lane" {
        let p = timing()?;
        let step_ms = num("step-ms", 10.0)?;
        let lane = duck_lane(&spans()?, &p, num("duration-ms", f64::NAN)?, step_ms);
        if lane.is_empty() {
            return Err("--duration-ms and a positive --step-ms are required".into());
        }
        println!("{}", json!({"step_ms": step_ms, "spans": merge_spans(&spans()?, p.merge_gap_ms), "lane": lane}));
        return Ok(());
    }

    let out = if dry_run { opts.get("out").map(PathBuf::from) } else { Some(path("out")?) };
    let (report, bed) = match cmd.as_str() {
        "duck" => {
            let p = timing()?;
            if dry_run {
                (json!({"spans": merge_spans(&spans()?, p.merge_gap_ms), "params": p}), None)
            } else {
                let mut bed = read_wav(&path("bed")?)?;
                let r = serde_json::to_value(duck(&mut bed, &spans()?, &p))?;
                (r, Some(bed))
            }
        }
        "carve" => {
            let c = CarveParams::default();
            if strength.is_some() && (opts.contains_key("max-cut-db") || opts.contains_key("q")) {
                return Err("use --max-cut-db/--q or --strength, not both".into());
            }
            let p = CarveParams { max_cut_db: num("max-cut-db", c.max_cut_db)?, q: num("q", c.q)?, ..c };
            let mut bed = if dry_run { None } else { Some(read_wav(&path("bed")?)?) };
            // All voices on one clock, mono, at one sample rate (the bed's, when there is a bed).
            let voice = if voices.is_empty() {
                None
            } else {
                let mut rate = bed.as_ref().map(|b| b.sample_rate);
                let mut monos = Vec::new();
                for v in &voices {
                    let buf = read_wav(&v.path)?;
                    match rate {
                        Some(r) if r != buf.sample_rate => {
                            return Err(format!("{} is {} Hz, expected {r} Hz", v.path.display(), buf.sample_rate).into());
                        }
                        _ => rate = Some(buf.sample_rate),
                    }
                    monos.push((mono(&buf), v.offset_ms));
                }
                let rate = rate.unwrap_or(0);
                let sources: Vec<VoiceSource> = monos.iter().map(|(s, o)| VoiceSource { samples: s, offset_ms: *o }).collect();
                Some((mix_voices(&sources, rate), rate))
            };
            let merged;
            let act = match &voice {
                Some((samples, _)) => Activity::Voice(samples),
                None => {
                    merged = merge_spans(&spans()?, num("merge-gap-ms", d.merge_gap_ms)?);
                    Activity::Spans(&merged)
                }
            };
            let (bands, source) = match (strength, &voice) {
                (None, _) => (scriptcast_media::carve_bands(&p), "weights"),
                (Some(s), Some((samples, rate))) => (analyse_carve_bands(samples, *rate, &carve_profile(s)), "voice"),
                (Some(s), None) => (intelligibility_bands(&carve_profile(s)), "intelligibility"),
            };
            let mut r = match bed.as_mut() {
                Some(b) if strength.is_some() => serde_json::to_value(carve_with(b, &act, &bands, &p))?,
                Some(b) => serde_json::to_value(carve(b, &act, &p))?,
                None => json!({"bands": bands, "params": p}),
            };
            r["band_source"] = json!(source);
            if let Some(s) = strength {
                r["strength"] = json!(s);
                r["profile"] = json!(carve_profile(s));
            }
            if let Activity::Spans(s) = act {
                r["spans"] = json!(s);
            }
            (r, bed)
        }
        other => return Err(format!("unknown command {other}").into()),
    };
    let mut report = report;
    report["dry_run"] = json!(dry_run);
    if let (Some(out), Some(bed)) = (&out, &bed) {
        write_wav(out, bed)?;
        report["out"] = json!(out);
    }
    if json || dry_run {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if let Some(out) = &out {
        println!("wrote {}", out.display());
    }
    Ok(())
}
