//! scriptcast-fx command line.
//!
//!   scriptcast-fx meter --in FILE [--json]
//!   scriptcast-fx chain --in FILE --out FILE --preset speaker|master [--room] [--eq7]
//!                       [--bypass] [--set plugin.param=value]... [--json]

use std::path::PathBuf;
use std::process::ExitCode;

use scriptcast_fx::{ChainReport, Preset, Setting, build_chain, chain_ids, measure, read_audio, run_chain, write_wav};

const USAGE: &str = "usage: scriptcast-fx meter --in FILE [--json] | chain --in FILE --out FILE --preset speaker|master [--room] [--eq7] [--bypass] [--set plugin.param=value]... [--json]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("scriptcast-fx: {}", e.replace('\n', " "));
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let (cmd, rest) = args.split_first().ok_or(USAGE)?;
    match cmd.as_str() {
        "meter" => meter(rest),
        "chain" => chain(rest),
        "-h" | "--help" => Ok(USAGE.to_string()),
        other => Err(format!("unknown command '{other}'; {USAGE}")),
    }
}

#[derive(Default)]
struct Args {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    preset: Option<String>,
    room: bool,
    eq7: bool,
    bypass: bool,
    json: bool,
    sets: Vec<Setting>,
}

/// Parses flags. `allowed` lists the flags the command takes.
fn parse(rest: &[String], allowed: &[&str]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        if !allowed.contains(&flag.as_str()) {
            return Err(format!("unknown argument '{flag}'"));
        }
        let mut value = || it.next().cloned().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--in" => set_once(&mut a.input, PathBuf::from(value()?), flag)?,
            "--out" => set_once(&mut a.output, PathBuf::from(value()?), flag)?,
            "--preset" => set_once(&mut a.preset, value()?, flag)?,
            "--set" => a.sets.push(Setting::parse(&value()?)?),
            "--room" => a.room = true,
            "--eq7" => a.eq7 = true,
            "--bypass" => a.bypass = true,
            "--json" => a.json = true,
            _ => return Err(format!("unknown argument '{flag}'")),
        }
    }
    Ok(a)
}

fn set_once<T>(slot: &mut Option<T>, v: T, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{flag} given twice"));
    }
    *slot = Some(v);
    Ok(())
}

fn meter(rest: &[String]) -> Result<String, String> {
    let a = parse(rest, &["--in", "--json"])?;
    let input = a.input.ok_or("meter needs --in FILE")?;
    let l = measure(&read_audio(&input)?)?;
    Ok(if a.json {
        l.to_json()
    } else {
        format!(
            "integrated_lufs {}\nshort_term_lufs {}\nmomentary_lufs {}\ntrue_peak_db {}",
            l.integrated_lufs, l.short_term_lufs, l.momentary_lufs, l.true_peak_db
        )
    })
}

fn chain(rest: &[String]) -> Result<String, String> {
    let a = parse(rest, &["--in", "--out", "--preset", "--room", "--eq7", "--bypass", "--set", "--json"])?;
    let input = a.input.ok_or("chain needs --in FILE")?;
    let output = a.output.ok_or("chain needs --out FILE")?;
    let preset = Preset::parse(&a.preset.ok_or("chain needs --preset speaker|master")?)?;
    let ids = chain_ids(preset, a.room, a.eq7)?;
    // Build the chain before reading audio, so a bad --set fails fast, also with --bypass.
    let mut plugins = build_chain(&ids, &a.sets)?;

    let mut buf = read_audio(&input)?;
    if !a.bypass {
        run_chain(&mut buf, &mut plugins);
    }
    write_wav(&output, &buf)?;

    let report = ChainReport {
        preset,
        plugins: ids,
        bypass: a.bypass,
        frames: buf.frames(),
        sample_rate: buf.sample_rate,
        wrote: output.display().to_string(),
    };
    Ok(if a.json { report.to_json() } else { format!("wrote {} ({} frames)", report.wrote, report.frames) })
}
