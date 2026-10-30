use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, Stdio};

const USAGE: &str = "usage:\n  scriptcast-tone speak --text TEXT --instruct TEXT --out OUT.wav [--lang english]\n  scriptcast-tone worker";

fn parse_args(args: &[String]) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name, Some(value.to_string())),
            _ => (arg.as_str(), None),
        };
        let Some(name) = name.strip_prefix("--") else {
            return Err(format!("unknown argument: {arg}"));
        };
        if !["text", "instruct", "out", "lang"].contains(&name) {
            return Err(format!("unknown argument: {arg}"));
        }
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
    }
    Ok(values)
}

fn require<'a>(values: &'a HashMap<String, String>, name: &str) -> Result<&'a str, String> {
    values
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("--{name} is required"))
}

fn worker_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tone crate has a parent")
        .join("scriptcast/tone_worker.py")
}

fn spawn_worker(piped: bool) -> Result<Child, String> {
    let python = std::env::var("SCRIPTCAST_TONE_PYTHON").unwrap_or_else(|_| "python3".into());
    let mut command = Command::new(&python);
    command.arg(worker_path());
    if piped {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());
    }
    command
        .spawn()
        .map_err(|error| format!("could not start {python}: {error}"))
}

fn run_worker() -> Result<(), String> {
    let status = spawn_worker(false)?
        .wait()
        .map_err(|error| format!("worker wait failed: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("worker exited with {status}"))
    }
}

fn run_speak(args: &[String]) -> Result<(), String> {
    let values = parse_args(args)?;
    let text = require(&values, "text")?;
    let instruct = require(&values, "instruct")?;
    let out = require(&values, "out")?;
    if instruct.trim().is_empty() {
        return Err("--instruct must not be blank".into());
    }
    let lang = values.get("lang").map(String::as_str).unwrap_or("english");

    let mut child = spawn_worker(true)?;
    let stdout = child.stdout.take().ok_or("worker stdout unavailable")?;
    let mut reader = BufReader::new(stdout);
    let mut greeting = String::new();
    reader
        .read_line(&mut greeting)
        .map_err(|error| format!("could not read worker greeting: {error}"))?;
    let ready: serde_json::Value = serde_json::from_str(&greeting)
        .map_err(|_| format!("worker sent an invalid greeting: {greeting:?}"))?;
    if ready != serde_json::json!({"ready": true}) {
        return Err(format!("worker sent an unexpected greeting: {greeting:?}"));
    }

    let request = serde_json::json!({
        "text": text,
        "instruct": instruct,
        "out": out,
        "lang": lang,
    });
    let mut stdin = child.stdin.take().ok_or("worker stdin unavailable")?;
    writeln!(stdin, "{request}").map_err(|error| format!("could not write request: {error}"))?;
    drop(stdin);

    let mut reply_line = String::new();
    reader
        .read_line(&mut reply_line)
        .map_err(|error| format!("could not read worker reply: {error}"))?;
    let reply: serde_json::Value = serde_json::from_str(&reply_line)
        .map_err(|_| format!("worker sent invalid JSON: {reply_line:?}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("worker wait failed: {error}"))?;
    if reply.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        let error = reply
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown error");
        return Err(error.to_string());
    }
    if !status.success() {
        return Err(format!("worker exited with {status}"));
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return Err("missing command".into());
    };
    match command.as_str() {
        "speak" => run_speak(rest),
        "worker" if rest.is_empty() => run_worker(),
        "worker" => Err("worker takes no arguments".into()),
        other => Err(format!("unknown command: {other}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("scriptcast-tone: {message}\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}
