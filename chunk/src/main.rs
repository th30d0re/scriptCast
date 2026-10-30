use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;

use scriptcast_chunk::{crossfade, pieces_json, split};

const USAGE: &str = "usage:
  scriptcast-chunk split --text TEXT --max-chars N --json
  scriptcast-chunk halve --text TEXT --json
  scriptcast-chunk crossfade --a A.wav --b B.wav --out OUT.wav [--fade-ms 50]";

/// Parse `--name value` / `--name=value` options and bare flags.
fn parse_args(
    args: &[String],
    valued: &[&str],
    flags: &[&str],
) -> Result<(HashMap<String, String>, Vec<String>), String> {
    let mut values = HashMap::new();
    let mut seen_flags = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name, Some(value.to_string())),
            _ => (arg.as_str(), None),
        };
        let name = name.strip_prefix("--").unwrap_or(name);
        if flags.contains(&name) && inline.is_none() && arg.starts_with("--") {
            seen_flags.push(name.to_string());
        } else if valued.contains(&name) && arg.starts_with("--") {
            let value = match inline {
                Some(value) => value,
                None => iter
                    .next()
                    .cloned()
                    .ok_or_else(|| format!("--{name} needs a value"))?,
            };
            if values.insert(name.to_string(), value).is_some() {
                return Err(format!("--{name} given twice"));
            }
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }
    Ok((values, seen_flags))
}

fn require<'a>(values: &'a HashMap<String, String>, name: &str) -> Result<&'a String, String> {
    values
        .get(name)
        .ok_or_else(|| format!("--{name} is required"))
}

fn run(args: &[String]) -> Result<Option<String>, String> {
    let Some((command, rest)) = args.split_first() else {
        return Err("missing command".into());
    };
    match command.as_str() {
        "split" => {
            let (values, _) = parse_args(rest, &["text", "max-chars"], &["json"])?;
            let max_chars: usize = require(&values, "max-chars")?
                .parse()
                .map_err(|_| "--max-chars must be a whole number".to_string())?;
            let pieces = split::split(require(&values, "text")?, max_chars)?;
            Ok(Some(pieces_json(&pieces)))
        }
        "halve" => {
            let (values, _) = parse_args(rest, &["text"], &["json"])?;
            let pieces = split::halve(require(&values, "text")?)?;
            Ok(Some(pieces_json(&pieces)))
        }
        "crossfade" => {
            let (values, _) = parse_args(rest, &["a", "b", "out", "fade-ms"], &[])?;
            let fade_ms: f64 = match values.get("fade-ms") {
                Some(text) => text
                    .parse()
                    .map_err(|_| "--fade-ms must be a number".to_string())?,
                None => 50.0,
            };
            crossfade::crossfade_files(
                Path::new(require(&values, "a")?),
                Path::new(require(&values, "b")?),
                Path::new(require(&values, "out")?),
                fade_ms,
            )?;
            Ok(None)
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(output) => {
            if let Some(output) = output {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("scriptcast-chunk: {message}\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}
