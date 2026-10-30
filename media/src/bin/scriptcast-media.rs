//! `scriptcast-media duck|carve`: make room for dialogue in a music bed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use scriptcast_media::{Activity, CarveParams, DuckParams, Result, carve, duck, merge_spans, mono, read_wav, spans_from_json, write_wav};

const USAGE: &str = "usage:
  scriptcast-media duck  --bed BED.wav --spans MANIFEST_OR_SPANS.json --out OUT.wav
                         [--target-db -12] [--attack-ms 150] [--release-ms 400] [--merge-gap-ms 300] [--json]
  scriptcast-media carve --bed BED.wav (--voice VOICE.wav | --spans MANIFEST_OR_SPANS.json) --out OUT.wav
                         [--max-cut-db 14.8] [--q 2.06] [--merge-gap-ms 300] [--json]

--spans takes episode_manifest.json (turns[].start_ms/end_ms) or [[start_ms, end_ms], ...].";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("scriptcast-media: {e}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().ok_or("missing command")?;
    let mut opts: HashMap<String, String> = HashMap::new();
    let mut json = false;
    while let Some(a) = args.next() {
        let key = a.strip_prefix("--").ok_or_else(|| format!("unexpected argument {a}"))?;
        if key == "json" {
            json = true;
        } else {
            opts.insert(key.to_string(), args.next().ok_or_else(|| format!("--{key} needs a value"))?);
        }
    }
    let path = |k: &str| opts.get(k).map(PathBuf::from).ok_or_else(|| format!("--{k} is required"));
    let num = |k: &str, d: f64| -> Result<f64> { Ok(opts.get(k).map(|v| v.parse()).transpose()?.unwrap_or(d)) };
    let spans = || -> Result<Vec<_>> { spans_from_json(&std::fs::read_to_string(path("spans")?)?) };

    let out = path("out")?;
    let mut bed = read_wav(&path("bed")?)?;
    let report = match cmd.as_str() {
        "duck" => {
            let d = DuckParams::default();
            let p = DuckParams {
                target_db: num("target-db", d.target_db)?,
                attack_ms: num("attack-ms", d.attack_ms)?,
                release_ms: num("release-ms", d.release_ms)?,
                merge_gap_ms: num("merge-gap-ms", d.merge_gap_ms)?,
            };
            serde_json::to_value(duck(&mut bed, &spans()?, &p))?
        }
        "carve" => {
            let d = CarveParams::default();
            let p = CarveParams { max_cut_db: num("max-cut-db", d.max_cut_db)?, q: num("q", d.q)?, ..d };
            let r = if opts.contains_key("voice") {
                let v = read_wav(&path("voice")?)?;
                if v.sample_rate != bed.sample_rate {
                    return Err(format!("voice is {} Hz, bed is {} Hz", v.sample_rate, bed.sample_rate).into());
                }
                carve(&mut bed, &Activity::Voice(&mono(&v)), &p)
            } else {
                let merged = merge_spans(&spans()?, num("merge-gap-ms", DuckParams::default().merge_gap_ms)?);
                carve(&mut bed, &Activity::Spans(&merged), &p)
            };
            serde_json::to_value(r)?
        }
        other => return Err(format!("unknown command {other}").into()),
    };
    write_wav(&out, &bed)?;
    if json {
        let mut v = report;
        v["out"] = serde_json::json!(out);
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("wrote {}", out.display());
    }
    Ok(())
}
