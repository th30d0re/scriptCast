//! Runs the scriptcast-fx binary on audio made in the test.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use soundcraft_audio_io::{AudioBuffer, BitDepth, EncodeOptions, FileFormat};

const SR: u32 = 48_000;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_scriptcast-fx"))
}

/// A fresh directory for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scriptcast-fx-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_float_wav(path: &Path, buf: &AudioBuffer) {
    let opts = EncodeOptions { format: FileFormat::Wav, bit_depth: BitDepth::Float32, dither: false, bwf: None };
    std::fs::write(path, soundcraft_audio_io::encode(buf, &opts).unwrap()).unwrap();
}

fn read_wav(path: &Path) -> AudioBuffer {
    soundcraft_audio_io::decode(&std::fs::read(path).unwrap(), Some("wav")).unwrap().1
}

fn silence(secs: f32) -> AudioBuffer {
    AudioBuffer::new(SR, 2, (SR as f32 * secs) as usize)
}

fn sine(freq: f32, amp: f32, secs: f32) -> AudioBuffer {
    let frames = (SR as f32 * secs) as usize;
    let ch: Vec<f32> = (0..frames).map(|i| amp * (std::f32::consts::TAU * freq * i as f32 / SR as f32).sin()).collect();
    AudioBuffer { sample_rate: SR, channels: vec![ch.clone(), ch] }
}

/// A speech-like test signal: a 200 Hz tone with a 6 kHz burst in the middle.
fn voice(secs: f32) -> AudioBuffer {
    let mut buf = sine(200.0, 0.3, secs);
    let frames = buf.frames();
    for ch in &mut buf.channels {
        for (i, s) in ch.iter_mut().enumerate().skip(frames / 3).take(frames / 3) {
            *s += 0.2 * (std::f32::consts::TAU * 6000.0 * i as f32 / SR as f32).sin();
        }
    }
    buf
}

fn run(cmd: &mut Command) -> Output {
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "exit {:?}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// The JSON value after `"key":`, up to the next comma or closing brace.
fn field<'a>(json: &'a str, key: &str) -> &'a str {
    let tag = format!("\"{key}\":");
    let start = json.find(&tag).unwrap_or_else(|| panic!("{key} missing in {json}")) + tag.len();
    let rest = &json[start..];
    let end = if rest.starts_with('[') { rest.find(']').unwrap() + 1 } else { rest.find([',', '}']).unwrap() };
    rest[..end].trim()
}

fn num(json: &str, key: &str) -> f64 {
    field(json, key).parse().unwrap_or_else(|_| panic!("{key} is not a number in {json}"))
}

fn meter(path: &Path) -> String {
    let out = run(bin().args(["meter", "--in"]).arg(path).arg("--json"));
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn meter_silence_is_quiet_and_finite() {
    let dir = scratch("meter-silence");
    let path = dir.join("silence.wav");
    write_float_wav(&path, &silence(1.0));
    let json = meter(&path);
    let keys: Vec<&str> = json.trim_matches(['{', '}']).split(',').map(|kv| kv.split(':').next().unwrap()).collect();
    assert_eq!(keys, ["\"integrated_lufs\"", "\"short_term_lufs\"", "\"momentary_lufs\"", "\"true_peak_db\""]);
    let lufs = num(&json, "integrated_lufs");
    assert!(lufs.is_finite() && lufs <= -70.0, "{json}");
}

#[test]
fn meter_sine_has_a_true_peak() {
    let dir = scratch("meter-sine");
    let path = dir.join("sine.wav");
    write_float_wav(&path, &sine(1000.0, 0.5, 1.0));
    let json = meter(&path);
    let tp = num(&json, "true_peak_db");
    assert!(tp.is_finite() && tp > -40.0, "{json}");
    assert!(num(&json, "integrated_lufs").is_finite(), "{json}");
}

fn chain_json(input: &Path, output: &Path, extra: &[&str]) -> String {
    let mut cmd = bin();
    cmd.args(["chain", "--in"]).arg(input).arg("--out").arg(output).args(extra).arg("--json");
    String::from_utf8(run(&mut cmd).stdout).unwrap().trim().to_string()
}

#[test]
fn speaker_chain_keeps_frame_count() {
    let dir = scratch("speaker");
    let input = dir.join("in.wav");
    let src = voice(1.5);
    write_float_wav(&input, &src);
    for (name, extra) in [("plain", &[][..]), ("full", &["--room", "--eq7", "--set", "channel_strip.hpf_on=1"][..])] {
        let output = dir.join(format!("{name}.wav"));
        let mut args = vec!["--preset", "speaker"];
        args.extend_from_slice(extra);
        let json = chain_json(&input, &output, &args);
        assert_eq!(field(&json, "preset"), "\"speaker\"");
        assert_eq!(num(&json, "frames") as usize, src.frames(), "{json}");
        assert_eq!(field(&json, "bypass"), "false");
        let out = read_wav(&output);
        assert_eq!(out.frames(), src.frames());
        assert_eq!((out.sample_rate, out.num_channels()), (SR, 2));
        assert!(out.channels.iter().flatten().all(|s| s.is_finite()));
    }
    let json = chain_json(&input, &dir.join("x.wav"), &["--preset", "speaker", "--room", "--eq7"]);
    assert_eq!(field(&json, "plugins"), "[\"channel_strip\",\"de_esser\",\"room_reverb\",\"eq_7band\"]");
}

#[test]
fn master_chain_keeps_frame_count() {
    let dir = scratch("master");
    let input = dir.join("in.wav");
    let src = sine(440.0, 0.9, 1.0);
    write_float_wav(&input, &src);
    let output = dir.join("out.wav");
    let json = chain_json(&input, &output, &["--preset", "master"]);
    assert_eq!(field(&json, "plugins"), "[\"maximizer\",\"dither\",\"dc_offset_removal\"]");
    assert_eq!(num(&json, "frames") as usize, src.frames());
    assert_eq!(num(&json, "sample_rate") as u32, SR);
    assert_eq!(read_wav(&output).frames(), src.frames());
}

#[test]
fn bypass_writes_input_samples() {
    let dir = scratch("bypass");
    let input = dir.join("in.wav");
    let output = dir.join("out.wav");
    let src = voice(1.0);
    write_float_wav(&input, &src);
    let json = chain_json(&input, &output, &["--preset", "speaker", "--room", "--bypass"]);
    assert_eq!(field(&json, "bypass"), "true");
    assert_eq!(read_wav(&output), src);
}

#[test]
fn bad_arguments_fail_with_one_line() {
    let dir = scratch("errors");
    let input = dir.join("in.wav");
    write_float_wav(&input, &silence(0.5));
    let output = dir.join("out.wav");
    let cases: &[&[&str]] = &[
        &["--preset", "speaker", "--set", "channel_strip.no_such_param=1"],
        &["--preset", "speaker", "--set", "no_such_plugin.gain=1"],
        &["--preset", "master", "--room"],
        &["--preset", "master", "--eq7"],
        &["--preset", "radio"],
    ];
    for extra in cases {
        let out = bin().args(["chain", "--in"]).arg(&input).arg("--out").arg(&output).args(*extra).output().unwrap();
        assert!(!out.status.success(), "{extra:?} succeeded");
        let err = String::from_utf8(out.stderr).unwrap();
        assert_eq!(err.lines().count(), 1, "{extra:?}: {err}");
        assert!(!output.exists(), "{extra:?} wrote output");
    }
}
