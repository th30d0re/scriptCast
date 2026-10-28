use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_scriptcast-chunk"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn write_wav(path: &Path, rate: u32, channels: u16, bits: u16, frames: usize, value: i32) {
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: bits,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for _ in 0..frames * usize::from(channels) {
        writer.write_sample(value).unwrap();
    }
    writer.finalize().unwrap();
}

fn read_wav(path: &Path) -> (hound::WavSpec, Vec<i32>) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let samples = reader.samples::<i32>().map(Result::unwrap).collect();
    (reader.spec(), samples)
}

#[test]
fn split_prints_only_json() {
    let out = run(&[
        "split",
        "--text",
        "Mr. Smith went home. He sat.",
        "--max-chars",
        "20",
        "--json",
    ]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        "{\"pieces\":[{\"text\":\"Mr. Smith went home. \",\"extended_for_tag\":false},\
         {\"text\":\"He sat.\",\"extended_for_tag\":false}]}\n"
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn split_short_text_is_one_piece() {
    let out = run(&["split", "--text", "hi", "--max-chars", "2", "--json"]);
    assert_eq!(
        stdout(&out),
        "{\"pieces\":[{\"text\":\"hi\",\"extended_for_tag\":false}]}\n"
    );
}

#[test]
fn split_tag_sets_extended_for_tag() {
    let out = run(&[
        "split",
        "--text",
        "[pause:500ms] hello there friend",
        "--max-chars",
        "8",
        "--json",
    ]);
    let json = stdout(&out);
    assert!(
        json.starts_with("{\"pieces\":[{\"text\":\"[pause:500ms] \",\"extended_for_tag\":true},")
    );
    assert_eq!(json.matches("true").count(), 1);
}

#[test]
fn split_cjk_roundtrips_as_utf8() {
    let out = run(&["split", "--text", "你好。世界", "--max-chars=3", "--json"]);
    assert_eq!(
        stdout(&out),
        "{\"pieces\":[{\"text\":\"你好。\",\"extended_for_tag\":false},\
         {\"text\":\"世界\",\"extended_for_tag\":false}]}\n"
    );
}

#[test]
fn split_unclosed_tag_fails() {
    let out = run(&[
        "split",
        "--text",
        "hello [pause",
        "--max-chars",
        "5",
        "--json",
    ]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn halve_keeps_tags_whole() {
    let out = run(&["halve", "--text", "left [pause:10ms] right", "--json"]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        "{\"pieces\":[{\"text\":\"left \",\"extended_for_tag\":false},\
         {\"text\":\"[pause:10ms] right\",\"extended_for_tag\":false}]}\n"
    );
}

#[test]
fn bad_arguments_fail() {
    assert!(!run(&[]).status.success());
    assert!(!run(&["nope"]).status.success());
    assert!(
        !run(&["split", "--text", "x", "--max-chars", "abc"])
            .status
            .success()
    );
    assert!(
        !run(&["split", "--text", "x", "--max-chars", "0"])
            .status
            .success()
    );
    assert!(!run(&["split", "--text", "x"]).status.success());
}

#[test]
fn crossfade_length_is_sum_minus_fade() {
    let (a, b, out) = (scratch("a1.wav"), scratch("b1.wav"), scratch("o1.wav"));
    write_wav(&a, 1000, 1, 16, 100, 1000);
    write_wav(&b, 1000, 1, 16, 80, -1000);
    let result = run(&[
        "crossfade",
        "--a",
        a.to_str().unwrap(),
        "--b",
        b.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--fade-ms",
        "20",
    ]);
    assert!(result.status.success());
    let (spec, samples) = read_wav(&out);
    assert_eq!(samples.len(), 160);
    assert_eq!(
        (spec.channels, spec.sample_rate, spec.bits_per_sample),
        (1, 1000, 16)
    );
    assert_eq!(samples[0], 1000);
    assert_eq!(samples[159], -1000);
}

#[test]
fn crossfade_default_fade_is_50ms() {
    let (a, b, out) = (scratch("a2.wav"), scratch("b2.wav"), scratch("o2.wav"));
    write_wav(&a, 1000, 1, 32, 100, 5);
    write_wav(&b, 1000, 1, 32, 100, 5);
    let result = run(&[
        "crossfade",
        "--a",
        a.to_str().unwrap(),
        "--b",
        b.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    let (spec, samples) = read_wav(&out);
    assert_eq!(samples.len(), 150);
    assert_eq!(spec.bits_per_sample, 32);
}

fn assert_refused(name: &str, a: (u32, u16, usize), b: (u32, u16, usize)) {
    let (pa, pb, out) = (
        scratch(&format!("{name}-a.wav")),
        scratch(&format!("{name}-b.wav")),
        scratch(&format!("{name}-o.wav")),
    );
    let _ = fs::remove_file(&out);
    write_wav(&pa, a.0, a.1, 16, a.2, 1);
    write_wav(&pb, b.0, b.1, 16, b.2, 1);
    let result = run(&[
        "crossfade",
        "--a",
        pa.to_str().unwrap(),
        "--b",
        pb.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--fade-ms",
        "20",
    ]);
    assert!(!result.status.success(), "{name} should fail");
    assert!(!out.exists(), "{name} wrote a file");
    assert!(
        !out.with_extension("wav.part").exists(),
        "{name} left a partial"
    );
}

#[test]
fn crossfade_refuses_bad_input_and_writes_nothing() {
    assert_refused("rate", (1000, 1, 100), (2000, 1, 100));
    assert_refused("channels", (1000, 1, 100), (1000, 2, 100));
    assert_refused("short-a", (1000, 1, 19), (1000, 1, 100));
    assert_refused("short-b", (1000, 1, 100), (1000, 1, 19));
}
