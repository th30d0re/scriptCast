//! Phase M tool contracts: word ranges, argv, audition scoring, outliers, stress.
//! The Python tools own the files. This module only builds arguments and reads JSON.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::{path::Path, time::SystemTime};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TimedWord {
    pub i: usize,
    pub word: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WordsJson {
    pub source: String,
    pub duration: f64,
    pub words: Vec<TimedWord>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CandidateRun {
    pub rank: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub score: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CandidatesJson {
    pub source: String,
    pub candidates: Vec<CandidateRun>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Take {
    pub wav: String,
    pub text: String,
    pub heard: String,
    pub score: f64,
    pub run: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AuditionCandidate {
    pub reference: String,
    pub temperature: f64,
    pub mean: f64,
    pub min: f64,
    pub clean: u32,
    #[serde(rename = "of")]
    pub of_takes: u32,
    pub takes: Vec<Take>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AuditionJson {
    pub speaker: String,
    pub engine: String,
    pub candidates: Vec<AuditionCandidate>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalibrateTake {
    pub wav: String,
    #[serde(default)]
    pub sentence: String,
    pub ok: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalibrateRow {
    pub spelling: String,
    pub score: f64,
    pub takes: Vec<CalibrateTake>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalibrateJson {
    pub word: String,
    pub reading: String,
    pub candidates: Vec<CalibrateRow>,
    pub best: String,
    pub written: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Outlier {
    pub turn_index: usize,
    pub turn_id: String,
    pub speaker_id: String,
    pub predicted_s: f64,
    pub actual_s: f64,
    pub ratio: f64,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OutliersJson {
    pub outliers: Vec<Outlier>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Heteronym {
    pub word: String,
    pub expected: String,
    pub heard: String,
    pub ok: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct StressJson {
    pub heteronyms: Vec<Heteronym>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ClipRegistered {
    pub id: String,
    pub registry: String,
    pub turn: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ReferenceWritten {
    pub wav: String,
    pub text: String,
    pub duration: f64,
    pub peak: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordSpan {
    pub anchor: usize,
    pub focus: usize,
}

impl WordSpan {
    pub fn click(self, index: usize, shift: bool) -> Self {
        if shift {
            Self {
                anchor: self.anchor,
                focus: index,
            }
        } else {
            Self {
                anchor: index,
                focus: index,
            }
        }
    }

    pub fn inclusive(self) -> (usize, usize) {
        (self.anchor.min(self.focus), self.anchor.max(self.focus))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: &'static str,
    pub args: Vec<String>,
}

impl Invocation {
    pub fn argv(&self) -> Vec<String> {
        std::iter::once(self.command.to_owned())
            .chain(self.args.iter().cloned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatioClass {
    Danger,
    Within,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeardWord {
    pub text: String,
    pub differs: bool,
}

pub fn source_cache_key(path: &Path) -> Result<(std::path::PathBuf, SystemTime), String> {
    let modified = path
        .metadata()
        .map_err(|e| e.to_string())?
        .modified()
        .map_err(|e| e.to_string())?;
    Ok((path.to_path_buf(), modified))
}

pub fn id_collides(existing: &[&str], id: &str) -> bool {
    let id = id.trim();
    !id.is_empty() && existing.contains(&id)
}

pub fn snap_start_index(words: &[TimedWord], seconds: f64, end_index: usize) -> usize {
    if words.is_empty() {
        return 0;
    }
    let end_index = end_index.min(words.len() - 1);
    let mut best = 0;
    let mut best_distance = f64::MAX;
    for (index, word) in words.iter().enumerate().take(end_index + 1) {
        let distance = (word.start - seconds).abs();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    best
}

pub fn snap_end_index(words: &[TimedWord], seconds: f64, start_index: usize) -> usize {
    if words.is_empty() {
        return 0;
    }
    let start_index = start_index.min(words.len() - 1);
    let mut best = start_index;
    let mut best_distance = f64::MAX;
    for (index, word) in words.iter().enumerate().skip(start_index) {
        let distance = (word.end - seconds).abs();
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    best
}

pub fn span_times(words: &[TimedWord], span: WordSpan) -> Option<(f64, f64)> {
    let (start, end) = span.inclusive();
    let first = words.get(start)?;
    let last = words.get(end)?;
    Some((first.start, last.end))
}

pub fn pad_seconds(pad_ms: u32) -> String {
    format!("{:.2}", f64::from(pad_ms) / 1000.0)
}

pub fn seconds_arg(seconds: f64) -> String {
    format!("{seconds:.2}")
}

pub fn clip_words_invocation(source: &str) -> Invocation {
    Invocation {
        command: "scriptcast-clip",
        args: vec![
            source.to_owned(),
            "--words-json".into(),
            "--model".into(),
            "small".into(),
        ],
    }
}

pub struct ClipRegister {
    pub source: String,
    pub id: String,
    pub start: f64,
    pub end: f64,
    pub speaker: String,
    pub citation: String,
    pub url: String,
    pub content_note: String,
    pub pad_ms: u32,
}

pub fn clip_register_invocation(clip: &ClipRegister) -> Invocation {
    let mut args = vec![
        clip.source.clone(),
        "--id".into(),
        clip.id.trim().to_owned(),
        "--start".into(),
        seconds_arg(clip.start),
        "--end".into(),
        seconds_arg(clip.end),
    ];
    if !clip.speaker.trim().is_empty() {
        args.push("--speaker".into());
        args.push(clip.speaker.trim().to_owned());
    }
    if !clip.citation.trim().is_empty() {
        args.push("--citation".into());
        args.push(clip.citation.trim().to_owned());
    }
    if !clip.url.trim().is_empty() {
        args.push("--url".into());
        args.push(clip.url.trim().to_owned());
    }
    if !clip.content_note.trim().is_empty() {
        args.push("--content-note".into());
        args.push(clip.content_note.trim().to_owned());
    }
    args.push("--pad".into());
    args.push(pad_seconds(clip.pad_ms));
    args.push("--json".into());
    Invocation {
        command: "scriptcast-clip",
        args,
    }
}

pub fn reference_candidates_invocation(source: &str, seconds: f64) -> Invocation {
    Invocation {
        command: "scriptcast-reference",
        args: vec![
            source.to_owned(),
            "--candidates-json".into(),
            "--seconds".into(),
            seconds_arg(seconds),
        ],
    }
}

pub fn reference_write_invocation(
    source: &str,
    name: &str,
    start: f64,
    end: f64,
    peak: f64,
) -> Invocation {
    Invocation {
        command: "scriptcast-reference",
        args: vec![
            source.to_owned(),
            "--name".into(),
            name.trim().to_owned(),
            "--start".into(),
            seconds_arg(start),
            "--end".into(),
            seconds_arg(end),
            "--peak".into(),
            format!("{peak:.2}"),
            "--json".into(),
        ],
    }
}

pub fn audition_invocation(
    references: &[String],
    speaker: &str,
    engine: &str,
    keep: &str,
    temperature: Option<&str>,
) -> Invocation {
    let mut args = references.to_vec();
    args.push("--speaker".into());
    args.push(speaker.to_owned());
    args.push("--engine".into());
    args.push(engine.to_owned());
    if let Some(temperature) = temperature.filter(|value| !value.is_empty()) {
        args.push("--temperature".into());
        args.push(temperature.to_owned());
    }
    args.push("--json".into());
    args.push("--keep".into());
    args.push(keep.to_owned());
    Invocation {
        command: "scriptcast-audition",
        args,
    }
}

pub fn calibrate_invocation(
    word: &str,
    reading: &str,
    speaker: &str,
    candidates: &str,
    keep: &str,
    write: bool,
) -> Invocation {
    let mut args = vec![
        word.trim().to_owned(),
        "--reading".into(),
        reading.trim().to_owned(),
        "--speaker".into(),
        speaker.to_owned(),
    ];
    if !candidates.trim().is_empty() {
        args.push("--candidates".into());
        args.push(candidates.trim().to_owned());
    }
    args.push("--json".into());
    args.push("--keep".into());
    args.push(keep.to_owned());
    if write {
        args.push("--write".into());
    }
    Invocation {
        command: "scriptcast-calibrate",
        args,
    }
}

pub fn outliers_invocation(episode: &str, transcript: &str) -> Invocation {
    Invocation {
        command: "scriptcast-outliers",
        args: vec![
            episode.to_owned(),
            "--transcript".into(),
            transcript.to_owned(),
            "--json".into(),
        ],
    }
}

pub fn stress_invocation(wav: &str, text: &str) -> Invocation {
    Invocation {
        command: "scriptcast-stress",
        args: vec![
            wav.to_owned(),
            "--text".into(),
            text.to_owned(),
            "--json".into(),
        ],
    }
}

pub fn winner_index(means: &[f64]) -> Option<usize> {
    let mut best = None;
    for (index, mean) in means.iter().enumerate() {
        if !mean.is_finite() {
            continue;
        }
        if best.is_none_or(|(_, current)| *mean > current) {
            best = Some((index, *mean));
        }
    }
    best.map(|(index, _)| index)
}

pub fn heard_diff(sentence: &str, heard: &str) -> Vec<HeardWord> {
    let left: Vec<String> = sentence.split_whitespace().map(normalize_word).collect();
    let right: Vec<String> = heard.split_whitespace().map(normalize_word).collect();
    let display: Vec<String> = heard.split_whitespace().map(str::to_owned).collect();
    let rows = left.len();
    let cols = right.len();
    let mut scores = vec![vec![0usize; cols + 1]; rows + 1];
    for row in 1..=rows {
        for col in 1..=cols {
            if left[row - 1] == right[col - 1] {
                scores[row][col] = scores[row - 1][col - 1] + 1;
            } else {
                scores[row][col] = scores[row - 1][col].max(scores[row][col - 1]);
            }
        }
    }
    let mut differs = vec![true; cols];
    let mut row = rows;
    let mut col = cols;
    while row > 0 && col > 0 {
        if left[row - 1] == right[col - 1] {
            differs[col - 1] = false;
            row -= 1;
            col -= 1;
        } else if scores[row - 1][col] >= scores[row][col - 1] {
            row -= 1;
        } else {
            col -= 1;
        }
    }
    display
        .into_iter()
        .zip(differs)
        .map(|(text, differs)| HeardWord { text, differs })
        .collect()
}

fn normalize_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

pub fn classify_ratio(ratio: f64) -> RatioClass {
    if !(0.6..=1.5).contains(&ratio) {
        RatioClass::Danger
    } else {
        RatioClass::Within
    }
}

pub fn parse_tool<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        return Err(error.to_owned());
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}

pub fn last_json_line(log: &str) -> Option<&str> {
    log.lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with('{'))
}

pub fn progress_line(log: &str) -> String {
    log.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('{'))
        .unwrap_or("")
        .to_owned()
}

pub fn assign_reference(fields: &mut Map<String, Value>, audio: &str, text: &str) {
    fields.insert("reference_audio".into(), Value::String(audio.to_owned()));
    fields.insert("reference_text".into(), Value::String(text.to_owned()));
}

pub fn clip_insert_body(turn: &str) -> String {
    turn.lines().nth(1).unwrap_or(turn).to_owned()
}

pub fn clock(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor().max(0.0) as u64;
    let rest = (seconds - minutes as f64 * 60.0).max(0.0);
    format!("{minutes:02}:{rest:05.2}")
}

pub fn mean_percent(mean: f64) -> String {
    format!("{:.1}", mean * 100.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeanBand {
    High,
    Mid,
    Low,
}

pub fn mean_band(mean: f64) -> MeanBand {
    let score = mean * 100.0;
    if score >= 90.0 {
        MeanBand::High
    } else if score >= 80.0 {
        MeanBand::Mid
    } else {
        MeanBand::Low
    }
}

/// Keep `current` when it is one of `people`. Otherwise use the Voices-tab
/// selection, then the first person (the same default the Voices list uses).
pub fn default_audition_speaker(people: &[&str], selected: &str, current: &str) -> String {
    if people.contains(&current) {
        current.to_owned()
    } else if people.contains(&selected) {
        selected.to_owned()
    } else {
        people.first().copied().unwrap_or("").to_owned()
    }
}
