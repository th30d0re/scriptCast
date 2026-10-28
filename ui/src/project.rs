//! Project tool report, immutable snapshots, and local draft operations.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub const FILES: [&str; 6] = [
    "toml",
    "voices",
    "clips",
    "pronunciations",
    "accepted_hearings",
    "speaker_rates",
];
pub const TOML_KEYS: [&str; 10] = [
    "voices",
    "pronunciations",
    "speaker_rates",
    "clips",
    "clip_sources",
    "references",
    "scripts",
    "outputs",
    "omnivoice_python",
    "tail_ms",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Report {
    pub schema: u32,
    pub root: PathBuf,
    pub toml: Section,
    pub voices: Section,
    pub clips: Section,
    pub pronunciations: Section,
    pub accepted_hearings: Section,
    pub speaker_rates: Section,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Section {
    pub path: PathBuf,
    pub missing: bool,
    pub problems: Vec<String>,
    pub unknown_keys: Vec<String>,
    pub values: Map<String, Value>,
    pub speakers: Vec<Value>,
    pub clips: Vec<Value>,
    pub entries: Vec<Value>,
    pub rates: Map<String, Value>,
}
impl Report {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let report: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if report.schema != 1 {
            return Err(format!("Unsupported project schema {}", report.schema));
        }
        Ok(report)
    }
    pub fn section(&self, file: &str) -> &Section {
        match file {
            "voices" => &self.voices,
            "clips" => &self.clips,
            "pronunciations" => &self.pronunciations,
            "accepted_hearings" => &self.accepted_hearings,
            "speaker_rates" => &self.speaker_rates,
            _ => &self.toml,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolError {
    Stale,
    Validation(Vec<String>),
    Failed(String),
}
impl ToolError {
    pub fn messages(&self) -> Vec<String> {
        match self {
            Self::Stale => vec!["The file changed on disk. Reload?".into()],
            Self::Validation(lines) => lines.clone(),
            Self::Failed(message) => vec![message.clone()],
        }
    }
}
pub fn tool_result(
    code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
    python: &str,
) -> Result<Report, ToolError> {
    let error = String::from_utf8_lossy(stderr).trim().to_owned();
    if error.contains("ModuleNotFoundError") && error.contains("ruamel") {
        return Err(ToolError::Failed(format!(
            "scriptcast-project needs ruamel.yaml in {python}"
        )));
    }
    match code {
        Some(0) => Report::parse(stdout).map_err(ToolError::Failed),
        Some(3) => Err(ToolError::Stale),
        Some(1) => Err(ToolError::Validation(
            error.lines().map(str::to_owned).collect(),
        )),
        _ => Err(ToolError::Failed(error)),
    }
}
pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Hashes must be captured with the report, never refreshed immediately before saving.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub report: Report,
    pub hashes: BTreeMap<String, String>,
}
impl Snapshot {
    pub fn capture(report: Report) -> Result<Self, String> {
        let mut hashes = BTreeMap::new();
        for file in FILES {
            let section = report.section(file);
            let bytes = match fs::read(&section.path) {
                Ok(bytes) => bytes,
                Err(e) if section.missing && e.kind() == std::io::ErrorKind::NotFound => vec![],
                Err(e) => return Err(format!("{}: {e}", section.path.display())),
            };
            hashes.insert(file.into(), digest(&bytes));
        }
        Ok(Self { report, hashes })
    }
}

pub fn engine_fields(engine: &str) -> &'static [&'static str] {
    match engine {
        "omnivoice" => &[
            "name",
            "engine",
            "reference_audio",
            "reference_text",
            "reference_audio_expressive",
            "reference_text_expressive",
            "speed",
            "character_profile",
        ],
        "kokoro" | "mlx_kokoro" => &["name", "engine", "kokoro_voice", "lang_code", "speed"],
        "mlx_chatterbox" => &[
            "name",
            "engine",
            "reference_audio",
            "exaggeration",
            "temperature",
            "cfg_weight",
        ],
        "tone" => &["name", "engine", "instruct", "character_profile"],
        "elevenlabs" => &["name", "engine", "elevenlabs_voice_id"],
        "archive" => &["name", "engine"],
        _ => &[],
    }
}
pub fn required_errors(fields: &Map<String, Value>) -> BTreeMap<String, String> {
    let engine = fields
        .get("engine")
        .and_then(Value::as_str)
        .unwrap_or("kokoro");
    let required: &[&str] = match engine {
        "omnivoice" => &["reference_audio", "reference_text"],
        "kokoro" | "mlx_kokoro" => &["kokoro_voice", "lang_code"],
        "mlx_chatterbox" => &["reference_audio"],
        "tone" => &["instruct"],
        "elevenlabs" => &["elevenlabs_voice_id"],
        "archive" => &[],
        _ => &["engine"],
    };
    let mut errors = BTreeMap::new();
    for key in required {
        if fields
            .get(*key)
            .and_then(Value::as_str)
            .is_none_or(|s| s.trim().is_empty())
        {
            errors.insert((*key).into(), format!("{engine} needs {key}"));
        }
    }
    if engine_fields(engine).is_empty() {
        errors.insert("engine".into(), format!("Unknown engine: {engine}"));
    }
    if engine == "omnivoice" {
        let audio = fields
            .get("reference_audio_expressive")
            .and_then(Value::as_str)
            .unwrap_or("");
        let text = fields
            .get("reference_text_expressive")
            .and_then(Value::as_str)
            .unwrap_or("");
        if audio.trim().is_empty() != text.trim().is_empty() {
            errors.insert(
                "reference_text_expressive".into(),
                "Provide both expressive audio and text.".into(),
            );
        }
    }
    errors
}

/// Keys whose values differ. TOML `set` ops use this list. Speaker, clip, and pronunciation
/// ops send the complete mapping so unchanged keys stay byte-identical.
pub fn changed_fields(
    before: &Map<String, Value>,
    after: &Map<String, Value>,
) -> Map<String, Value> {
    after
        .iter()
        .filter(|(key, value)| before.get(*key) != Some(*value))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
pub fn removed_engine_fields(fields: &Map<String, Value>, engine: &str) -> Vec<String> {
    fields
        .keys()
        .filter(|key| !engine_fields(engine).contains(&key.as_str()))
        .cloned()
        .collect()
}

pub fn toml_ops(before: &Section, after: &Section) -> Vec<Value> {
    changed_fields(&before.values, &after.values)
        .into_iter()
        .map(|(key, value)| json!({"file":"toml", "op":"set", "key":key, "value":value}))
        .collect()
}
pub fn pronunciation_ops(before: &Section, after: &Section) -> Vec<Value> {
    let mut ops = vec![];
    for old in &before.entries {
        if !after.entries.iter().any(|new| new["word"] == old["word"]) {
            ops.push(json!({"file":"pronunciations", "op":"delete_entry", "word":old["word"]}));
        }
    }
    for new in &after.entries {
        if !before
            .entries
            .iter()
            .any(|old| old["word"] == new["word"] && old["readings"] == new["readings"])
        {
            ops.push(
                json!({"file":"pronunciations", "op":"set_entry", "word":new["word"],
                "readings":new["readings"]}),
            );
        }
    }
    ops
}
pub fn hearing_ops(before: &Section, after: &Section) -> Vec<Value> {
    let mut ops = vec![];
    for old in &before.entries {
        for heard in old["heard"].as_array().into_iter().flatten() {
            let retained = after.entries.iter().any(|new| {
                new["word"] == old["word"]
                    && new["heard"]
                        .as_array()
                        .is_some_and(|values| values.contains(heard))
            });
            if !retained {
                ops.push(json!({"file":"accepted_hearings", "op":"delete", "word":old["word"], "heard":heard}));
            }
        }
    }
    ops
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub root: PathBuf,
    pub opened: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeStatus {
    Rendered,
    Interrupted,
    NotRendered,
}
impl EpisodeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rendered => "Rendered",
            Self::Interrupted => "Interrupted render",
            Self::NotRendered => "Not rendered",
        }
    }
}
/// Manifest `source_file` is relative to the project root. An absolute path is used as given.
pub fn script_from_manifest(root: &std::path::Path, source_file: &str) -> PathBuf {
    let raw = std::path::Path::new(source_file);
    if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        root.join(raw)
    }
}
pub fn same_file(left: &std::path::Path, right: &std::path::Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
/// Local wall time, two lines: `05 Oct 2026` then `10:32`.
pub fn format_rendered(raw: &str) -> String {
    let Ok(stamp) = chrono::DateTime::parse_from_rfc3339(raw) else {
        return raw.to_owned();
    };
    stamp
        .with_timezone(&chrono::Local)
        .format("%d %b %Y\n%H:%M")
        .to_string()
}
/// Calendar date of the timestamp, in UTC so `2026-10-05T01:33:56Z` stays 05 Oct 2026.
pub fn format_review_date(raw: &str) -> String {
    let Ok(stamp) = chrono::DateTime::parse_from_rfc3339(raw) else {
        return raw.to_owned();
    };
    stamp.format("%d %b %Y").to_string()
}
pub fn episode_status(has_manifest: bool, has_unfinished_job: bool) -> EpisodeStatus {
    if has_unfinished_job {
        EpisodeStatus::Interrupted
    } else if has_manifest {
        EpisodeStatus::Rendered
    } else {
        EpisodeStatus::NotRendered
    }
}
#[derive(Clone, Debug)]
pub struct ProjectEpisode {
    pub id: String,
    pub script: PathBuf,
    pub output: Option<PathBuf>,
    pub turns: Option<usize>,
    pub duration_ms: Option<u64>,
    pub rendered: String,
    pub verification: String,
    pub status: EpisodeStatus,
}
pub fn discover_episodes(report: &Report) -> Result<Vec<ProjectEpisode>, String> {
    use crate::episode::{Episode, VerificationState};
    let root = &report.root;
    let outputs = root.join(
        report
            .toml
            .values
            .get("outputs")
            .and_then(Value::as_str)
            .unwrap_or("outputs"),
    );
    let scripts = root.join(
        report
            .toml
            .values
            .get("scripts")
            .and_then(Value::as_str)
            .unwrap_or("scripts"),
    );
    let mut rows = vec![];
    if outputs.is_dir() {
        for entry in fs::read_dir(&outputs).map_err(|e| e.to_string())? {
            let dir = entry.map_err(|e| e.to_string())?.path();
            let unfinished = unfinished_jobs(&dir);
            let has_manifest = dir.join("episode_manifest.json").is_file();
            if !has_manifest && !unfinished {
                continue;
            }
            let episode = match Episode::load(&dir) {
                Ok(episode) => episode,
                Err(error) => {
                    rows.push(ProjectEpisode {
                        id: dir
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        script: PathBuf::new(),
                        output: Some(dir),
                        turns: None,
                        duration_ms: None,
                        rendered: "—".into(),
                        verification: format!("Cannot load episode: {error}"),
                        status: episode_status(has_manifest, unfinished),
                    });
                    continue;
                }
            };
            let passed = episode
                .turns
                .iter()
                .filter(|t| t.verification == VerificationState::Passed)
                .count();
            let failed = episode
                .turns
                .iter()
                .filter(|t| t.verification == VerificationState::Failed)
                .count();
            let unchecked = episode.turns.len() - passed - failed;
            let script = script_from_manifest(root, &episode.manifest.source_file);
            rows.push(ProjectEpisode {
                id: episode.manifest.episode_id,
                script,
                output: Some(dir),
                turns: Some(episode.turns.len()),
                duration_ms: Some(episode.total_ms),
                rendered: episode.rendered_at.unwrap_or(episode.manifest.created_at),
                verification: format!("{passed} passed · {failed} failed · {unchecked} unchecked"),
                status: episode_status(true, unfinished),
            });
        }
    }
    let header = regex::Regex::new(r"(?m)^\s*.+?\s*\(\d{1,2}:\d{2}\)\s*$")
        .expect("constant transcript header expression");
    let mut pending = vec![scripts];
    while let Some(dir) = pending.pop() {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                pending.push(path);
                continue;
            }
            if !path.extension().is_some_and(|e| e == "md" || e == "txt") {
                continue;
            }
            if rows.iter().any(|row| same_file(&row.script, &path)) {
                continue;
            }
            rows.push(ProjectEpisode {
                id: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                script: path.clone(),
                output: None,
                turns: fs::read_to_string(&path)
                    .ok()
                    .map(|text| header.find_iter(&text).count()),
                duration_ms: None,
                rendered: "—".into(),
                verification: "Not run".into(),
                status: EpisodeStatus::NotRendered,
            });
        }
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

impl Report {
    pub fn section_mut(&mut self, file: &str) -> &mut Section {
        match file {
            "voices" => &mut self.voices,
            "clips" => &mut self.clips,
            "pronunciations" => &mut self.pronunciations,
            "accepted_hearings" => &mut self.accepted_hearings,
            "speaker_rates" => &mut self.speaker_rates,
            _ => &mut self.toml,
        }
    }
}
pub fn small_list_ops(file: &str, before: &Section, after: &Section) -> Result<Vec<Value>, String> {
    match file {
        "voices" => voice_ops(before, after),
        "clips" => clip_ops(before, after),
        "toml" => Ok(toml_ops(before, after)),
        "pronunciations" => Ok(pronunciation_ops(before, after)),
        "accepted_hearings" => Ok(hearing_ops(before, after)),
        "speaker_rates" => Err("Speaker rates are read-only.".into()),
        other => Err(format!("Unknown project file {other}")),
    }
}

/// Scratch contains only the operation document; project file writes occur exclusively in `edit`.
pub fn edit(
    resolved: &crate::cli::Resolved,
    snapshot: &Snapshot,
    file: &str,
    ops: &[Value],
) -> Result<Snapshot, ToolError> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "scriptcast-l-ops-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        output
            .write_all(&serde_json::to_vec(ops).map_err(|e| ToolError::Failed(e.to_string()))?)
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let hash = snapshot
            .hashes
            .get(file)
            .ok_or_else(|| ToolError::Failed("Missing loaded hash".into()))?;
        let result = resolved
            .base_command()
            .args(["edit", "--ops"])
            .arg(&path)
            .arg("--expect-sha256")
            .arg(format!("{file}={hash}"))
            .arg("--root")
            .arg(&snapshot.report.root)
            .current_dir(&snapshot.report.root)
            .output()
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let report = tool_result(
            result.status.code(),
            &result.stdout,
            &result.stderr,
            &interpreter_label(resolved),
        )?;
        let snapshot = load(resolved, &snapshot.report.root)?;
        if snapshot.report != report {
            return Err(ToolError::Stale);
        }
        Ok(snapshot)
    })();
    let _ = fs::remove_file(path);
    result
}

/// Read twice around hashing so a concurrent edit cannot pair old values with a newer file hash.
pub fn load(
    resolved: &crate::cli::Resolved,
    root: &std::path::Path,
) -> Result<Snapshot, ToolError> {
    let read = || {
        let output = resolved
            .base_command()
            .args(["show", "--json", "--root"])
            .arg(root)
            .current_dir(root)
            .output()
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let report = tool_result(
            output.status.code(),
            &output.stdout,
            &output.stderr,
            &interpreter_label(resolved),
        )?;
        Snapshot::capture(report).map_err(ToolError::Failed)
    };
    let before = read()?;
    let after = read()?;
    if before.hashes != after.hashes || before.report != after.report {
        return Err(ToolError::Stale);
    }
    Ok(after)
}

/// Id changes become `rename_speaker`. Field edits are separate complete `set_speaker` ops.
pub fn rename_ops(before: &Section, after: &Section) -> Result<Vec<Value>, String> {
    let mut ops = vec![];
    for new in &after.speakers {
        let old_id = new
            .get("original_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| new["id"].as_str().unwrap_or(""));
        let new_id = new["id"].as_str().unwrap_or("");
        let known = before
            .speakers
            .iter()
            .any(|old| old["id"].as_str() == Some(old_id));
        if known && old_id != new_id {
            ops.push(json!({"file":"voices", "op":"rename_speaker", "id":old_id, "to":new_id}));
        }
    }
    Ok(ops)
}

fn loaded_speaker_id(speaker: &Value) -> &str {
    speaker
        .get("original_id")
        .and_then(Value::as_str)
        .unwrap_or_else(|| speaker["id"].as_str().unwrap_or(""))
}

/// `set_speaker` receives every key in `fields`. Absent keys are removed by the tool.
pub fn voice_ops(before: &Section, after: &Section) -> Result<Vec<Value>, String> {
    let mut ops = vec![];
    for old in &before.speakers {
        let id = old["id"].as_str().unwrap_or("");
        if !after
            .speakers
            .iter()
            .any(|speaker| loaded_speaker_id(speaker) == id)
        {
            ops.push(json!({"file":"voices", "op":"delete_speaker", "id":id}));
        }
    }
    ops.extend(rename_ops(before, after)?);
    for new in &after.speakers {
        let id = new["id"].as_str().unwrap_or("");
        let fields = new
            .get("fields")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("Speaker {id} is missing fields"))?;
        let loaded = loaded_speaker_id(new);
        let previous = before
            .speakers
            .iter()
            .find(|old| old["id"].as_str() == Some(loaded));
        if previous.is_none_or(|old| old.get("fields") != new.get("fields")) {
            ops.push(json!({"file":"voices", "op":"set_speaker", "id":id, "fields":fields}));
        }
    }
    Ok(ops)
}

fn clip_fields(clip: &Value) -> Result<Map<String, Value>, String> {
    let id = clip["id"].as_str().unwrap_or("");
    clip.get("fields")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| format!("Clip {id} is missing fields"))
}

/// `set_clip` receives the complete clip mapping from `fields`.
pub fn clip_ops(before: &Section, after: &Section) -> Result<Vec<Value>, String> {
    let mut ops = vec![];
    for old in &before.clips {
        if !after.clips.iter().any(|clip| clip["id"] == old["id"]) {
            ops.push(json!({"file":"clips", "op":"delete_clip", "id":old["id"]}));
        }
    }
    for new in &after.clips {
        let id = new["id"].as_str().unwrap_or("");
        let fields = clip_fields(new)?;
        let previous = before.clips.iter().find(|old| old["id"] == new["id"]);
        let changed = match previous {
            None => true,
            Some(old) => clip_fields(old)? != fields,
        };
        if changed {
            ops.push(json!({"file":"clips", "op":"set_clip", "id":id, "fields":fields}));
        }
    }
    Ok(ops)
}

/// Create `scriptcast.toml` through `scriptcast-project init`. The UI does not write the file.
pub fn init_project(
    resolved: &crate::cli::Resolved,
    root: &std::path::Path,
) -> Result<Snapshot, ToolError> {
    let output = resolved
        .base_command()
        .args(["init", "--root"])
        .arg(root)
        .current_dir(root)
        .output()
        .map_err(|e| ToolError::Failed(e.to_string()))?;
    tool_result(
        output.status.code(),
        &output.stdout,
        &output.stderr,
        &interpreter_label(resolved),
    )?;
    load(resolved, root)
}

pub fn unfinished_jobs(dir: &std::path::Path) -> bool {
    fs::read_dir(dir.join("_jobs"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|e| e == "json"))
        .any(|entry| {
            let Ok(record) = crate::detached::read(&entry.path()) else {
                return true;
            };
            let progress = if record.progress.is_absolute() {
                record.progress
            } else {
                dir.join(record.progress)
            };
            !fs::read_to_string(progress)
                .unwrap_or_default()
                .lines()
                .any(|line| {
                    serde_json::from_str::<Value>(line).is_ok_and(|value| value["event"] == "done")
                })
        })
}

/// Console-script discovery returns the wrapper; dependency diagnostics name its Python interpreter.
pub fn interpreter_label(resolved: &crate::cli::Resolved) -> String {
    use std::io::BufRead;
    let fallback = resolved.program.display().to_string();
    if resolved
        .program
        .file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with("python"))
    {
        return fallback;
    }
    let Ok(file) = fs::File::open(&resolved.program) else {
        return fallback;
    };
    let mut first = String::new();
    if std::io::BufReader::new(file).read_line(&mut first).is_err() {
        return fallback;
    }
    let Some(shebang) = first.strip_prefix("#!") else {
        return fallback;
    };
    let mut words = shebang.split_whitespace();
    let Some(program) = words.next() else {
        return fallback;
    };
    if program.ends_with("/env") {
        let Some(python) = words.find(|word| word.starts_with("python")) else {
            return fallback;
        };
        return std::env::split_paths(&resolved.path)
            .map(|dir| dir.join(python))
            .find(|p| p.is_file())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| python.into());
    }
    program.into()
}
