//! Read-only filesystem model. No UI types or audio decoding on the load path.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::SystemTime,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub schema_version: String,
    pub episode_id: String,
    pub source_file: String,
    pub engine: String,
    pub sample_rate: u32,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub speakers: Vec<SpeakerEntry>,
    #[serde(default)]
    pub turns: Vec<TurnEntry>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct SpeakerEntry {
    pub speaker_id: String,
    pub display_name: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub turn_count: usize,
}
#[derive(Clone, Debug, Deserialize)]
pub struct TurnEntry {
    pub turn_index: usize,
    pub turn_id: String,
    pub speaker_id: String,
    #[serde(default)]
    pub source_timestamp: String,
    #[serde(default)]
    pub start_ms: u64,
    #[serde(default)]
    pub end_ms: u64,
    #[serde(default)]
    pub segments: Vec<SegmentEntry>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct SegmentEntry {
    pub chunk_index: usize,
    pub segment_wav: String,
    pub duration_ms: u64,
    #[serde(default)]
    pub start_ms: u64,
    #[serde(default)]
    pub end_ms: u64,
    #[serde(default)]
    pub speech_duration_ms: Option<u64>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct RenderState {
    #[serde(default)]
    pub als_path: Option<PathBuf>,
    #[serde(default)]
    pub schema_version: String,
    #[serde(default)]
    pub source_file: String,
    #[serde(default)]
    pub source_hash: String,
    #[serde(default)]
    pub rendered_at: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct StressCheck {
    pub word: String,
    #[serde(default)]
    pub reading: String,
    #[serde(default)]
    pub verdict: String,
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct VerifyResult {
    pub turn_id: String,
    pub failed: bool,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub expected: String,
    #[serde(default)]
    pub heard: String,
    #[serde(default)]
    pub repetition: Option<String>,
    #[serde(default)]
    pub stress: Vec<StressCheck>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationState {
    Passed,
    Failed,
    NotVerified,
    MissingAudio,
}
#[derive(Clone, Debug)]
pub struct Speaker {
    pub id: String,
    pub name: String,
    pub engine: String,
    pub colour_idx: usize,
}
#[derive(Clone, Debug)]
pub struct Turn {
    pub index: usize,
    pub id: String,
    pub speaker_idx: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub first_words: String,
    pub wav_paths: Vec<PathBuf>,
    pub verify: Option<VerifyResult>,
    pub verification: VerificationState,
}
#[derive(Clone, Debug)]
pub struct Episode {
    pub dir: PathBuf,
    pub als_path: Option<PathBuf>,
    pub project_root: Option<PathBuf>,
    pub script_path: Option<PathBuf>,
    pub script_in_sync: Option<bool>,
    pub voices_path: Option<PathBuf>,
    pub manifest: Manifest,
    pub rendered_at: Option<String>,
    pub total_ms: u64,
    pub speakers: Vec<Speaker>,
    pub turns: Vec<Turn>,
    pub warnings: Vec<String>,
    pub stress_mtime: Option<SystemTime>,
}
#[derive(Debug)]
pub struct LoadError {
    pub path: PathBuf,
    pub cause: String,
}
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.cause)
    }
}
impl std::error::Error for LoadError {}
fn error(path: &Path, cause: impl fmt::Display) -> LoadError {
    LoadError {
        path: path.to_owned(),
        cause: cause.to_string(),
    }
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, LoadError> {
    let bytes = fs::read(path).map_err(|e| error(path, e))?;
    serde_json::from_slice(&bytes).map_err(|e| error(path, e))
}
fn optional<T: serde::de::DeserializeOwned>(path: &Path, warnings: &mut Vec<String>) -> Option<T> {
    match read_json(path) {
        Ok(v) => Some(v),
        Err(e) => {
            warnings.push(e.to_string());
            None
        }
    }
}
fn absolute(path: &Path) -> PathBuf {
    let expanded = if let Ok(tail) = path.strip_prefix("~") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(tail)
    } else {
        path.to_owned()
    };
    fs::canonicalize(&expanded)
        .unwrap_or_else(|_| std::path::absolute(&expanded).unwrap_or(expanded))
}
pub fn find_project_root(dir: &Path) -> Option<PathBuf> {
    find_root_with_override(
        dir,
        std::env::var_os("SCRIPTCAST_PROJECT")
            .as_deref()
            .map(Path::new),
    )
}
/// Explicit override also makes the environment rule testable without mutating process globals.
pub fn find_root_with_override(dir: &Path, root: Option<&Path>) -> Option<PathBuf> {
    if let Some(root) = root.filter(|p| !p.as_os_str().is_empty()) {
        return Some(absolute(root));
    }
    absolute(dir)
        .ancestors()
        .find(|p| p.join("scriptcast.toml").is_file())
        .map(Path::to_owned)
}
impl Episode {
    pub fn load(dir: &Path) -> Result<Self, LoadError> {
        Self::load_cancellable(dir, &AtomicBool::new(false))
    }

    /// Cooperative cancellation between metadata reads and per-turn path checks.
    /// In-flight filesystem reads complete normally; no episode files are written.
    pub fn load_cancellable(dir: &Path, canceled: &AtomicBool) -> Result<Self, LoadError> {
        let check = || {
            if canceled.load(Ordering::Relaxed) {
                Err(error(dir, "Loading canceled"))
            } else {
                Ok(())
            }
        };
        check()?;
        let dir = absolute(dir);
        let manifest_path = dir.join("episode_manifest.json");
        let manifest: Manifest = read_json(&manifest_path)?;
        let mut warnings = Vec::new();
        check()?;
        let state: Option<RenderState> = optional(&dir.join("render_state.json"), &mut warnings);
        check()?;
        let verification: Option<Vec<VerifyResult>> =
            optional(&dir.join("stress.json"), &mut warnings);
        let mut verification: HashMap<_, _> = verification
            .unwrap_or_default()
            .into_iter()
            .map(|v| (v.turn_id.clone(), v))
            .collect();
        check()?;
        let csv_path = dir.join("TURN_INDEX.csv");
        let words = match fs::read_to_string(&csv_path)
            .map_err(|e| e.to_string())
            .and_then(|s| parse_turn_index(&s))
        {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.turn_index, r.first_words))
                .collect::<HashMap<_, _>>(),
            Err(e) => {
                warnings.push(format!("{}: {e}", csv_path.display()));
                HashMap::new()
            }
        };
        let project_root = find_project_root(&dir);
        let script_path = state
            .as_ref()
            .filter(|s| Path::new(&s.source_file).is_absolute())
            .map(|s| absolute(Path::new(&s.source_file)))
            .or_else(|| {
                (!manifest.source_file.is_empty())
                    .then(|| {
                        if Path::new(&manifest.source_file).is_absolute() {
                            Some(absolute(Path::new(&manifest.source_file)))
                        } else {
                            project_root
                                .as_ref()
                                .map(|root| absolute(&root.join(&manifest.source_file)))
                        }
                    })
                    .flatten()
            });
        let script_in_sync = match (
            script_path.as_ref(),
            state.as_ref().filter(|s| !s.source_hash.is_empty()),
        ) {
            (Some(path), Some(state)) => match fs::read(path) {
                Ok(bytes) => Some(
                    Sha256::digest(bytes)
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                        .eq_ignore_ascii_case(&state.source_hash),
                ),
                Err(e) => {
                    warnings.push(format!("{}: {e}", path.display()));
                    None
                }
            },
            _ => None,
        };
        let voices_path = project_root.as_ref().and_then(|root| {
            let path = root.join("scriptcast.toml");
            match fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|s| toml::from_str::<toml::Value>(&s).map_err(|e| e.to_string()))
            {
                Ok(config) => config
                    .get("voices")
                    .and_then(|v| v.as_str())
                    .map(|v| absolute(&root.join(v))),
                Err(e) => {
                    warnings.push(format!("{}: {e}", path.display()));
                    None
                }
            }
        });
        let mut speakers = Vec::new();
        // Manifest speaker order is authoritative, including speakers with no current turns.
        for s in &manifest.speakers {
            if speakers.iter().any(|v: &Speaker| v.id == s.speaker_id) {
                return Err(error(&manifest_path, "duplicate speaker_id"));
            }
            speakers.push(Speaker {
                id: s.speaker_id.clone(),
                name: s.display_name.clone(),
                engine: s.engine.clone(),
                colour_idx: speakers.len(),
            });
        }
        let mut turns = Vec::new();
        let mut ids = HashSet::new();
        let mut indices = HashSet::new();
        for t in &manifest.turns {
            check()?;
            if t.end_ms < t.start_ms || !ids.insert(&t.turn_id) || !indices.insert(t.turn_index) {
                return Err(error(
                    &manifest_path,
                    format!(
                        "turn {} has invalid time bounds or duplicate identity",
                        t.turn_index
                    ),
                ));
            }
            let speaker_idx = speakers
                .iter()
                .position(|s| s.id == t.speaker_id)
                .unwrap_or_else(|| {
                    warnings.push(format!(
                        "Unknown speaker {}; using its ID as the label",
                        t.speaker_id
                    ));
                    let idx = speakers.len();
                    speakers.push(Speaker {
                        id: t.speaker_id.clone(),
                        name: t.speaker_id.clone(),
                        engine: String::new(),
                        colour_idx: idx,
                    });
                    idx
                });
            let mut segments = t.segments.iter().collect::<Vec<_>>();
            segments.sort_by_key(|s| s.chunk_index);
            let wav_paths: Vec<_> = segments
                .iter()
                .map(|s| absolute(&dir.join(&s.segment_wav)))
                .collect();
            let verify = verification.remove(&t.turn_id);
            let status = if wav_paths.is_empty() || wav_paths.iter().any(|p| !p.is_file()) {
                VerificationState::MissingAudio
            } else {
                match &verify {
                    Some(v) if v.failed => VerificationState::Failed,
                    Some(_) => VerificationState::Passed,
                    None => VerificationState::NotVerified,
                }
            };
            turns.push(Turn {
                index: t.turn_index,
                id: t.turn_id.clone(),
                speaker_idx,
                start_ms: t.start_ms,
                end_ms: t.end_ms,
                first_words: words.get(&t.turn_index).cloned().unwrap_or_default(),
                wav_paths,
                verify,
                verification: status,
            });
        }
        if !verification.is_empty() {
            let mut stale_ids: Vec<_> = verification.keys().map(String::as_str).collect();
            stale_ids.sort_unstable();
            warnings.push(format!(
                "{}: {} stale verification entries did not match any manifest turn_id: {}",
                dir.join("stress.json").display(),
                stale_ids.len(),
                stale_ids.join(", ")
            ));
        }
        let total_ms = turns.iter().map(|t| t.end_ms).max().unwrap_or(0);
        let stress_mtime = fs::metadata(dir.join("stress.json"))
            .and_then(|m| m.modified())
            .ok();
        Ok(Self {
            dir,
            als_path: state.as_ref().and_then(|s| s.als_path.clone()),
            project_root,
            script_path,
            script_in_sync,
            voices_path,
            manifest,
            rendered_at: state.map(|s| s.rendered_at).filter(|s| !s.is_empty()),
            total_ms,
            speakers,
            turns,
            warnings,
            stress_mtime,
        })
    }

    /// Dominance is measured by segment speech overlap, excluding inter-turn/segment gaps.
    /// Silent slices have no dominant speaker.
    pub fn overview_sections(&self, n: usize) -> Vec<Option<usize>> {
        (0..n)
            .map(|i| {
                let start = self.total_ms as f64 * i as f64 / n as f64;
                let end = self.total_ms as f64 * (i + 1) as f64 / n as f64;
                let mut durations = vec![0.0_f64; self.speakers.len()];
                for (turn, raw) in self.turns.iter().zip(&self.manifest.turns) {
                    for segment in &raw.segments {
                        let speech = segment.speech_duration_ms.unwrap_or(segment.duration_ms);
                        let seg_start = if segment.end_ms > segment.start_ms {
                            segment.start_ms
                        } else {
                            turn.start_ms
                        };
                        durations[turn.speaker_idx] += (end
                            .min((seg_start.saturating_add(speech)) as f64)
                            - start.max(seg_start as f64))
                        .max(0.0);
                    }
                }
                durations
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| **d > 0.0)
                    .max_by(|(ia, a), (ib, b)| a.total_cmp(b).then_with(|| ib.cmp(ia)))
                    .map(|(i, _)| i)
            })
            .collect()
    }
    pub fn count(&self, state: VerificationState) -> usize {
        self.turns
            .iter()
            .filter(|t| t.verification == state)
            .count()
    }
}

/// Sections in the timeline overview strip: about one per three minutes,
/// clamped to the design's 12–32 range.
pub fn overview_section_count(total_ms: u64) -> usize {
    (total_ms as f64 / 180_000.0).round().clamp(12.0, 32.0) as usize
}

#[derive(Debug, PartialEq, Eq)]
pub struct TurnIndexRow {
    pub turn_index: usize,
    pub first_words: String,
}
/// Strict CSV parser supporting commas, escaped quotes, CRLF, and newlines inside quotes.
pub fn parse_turn_index(input: &str) -> Result<Vec<TurnIndexRow>, String> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut closed = false;
    let mut chars = input.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' if field.is_empty() && !closed => quoted = true,
                ',' => {
                    row.push(std::mem::take(&mut field));
                    closed = false;
                }
                '\r' | '\n' => {
                    if c == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    closed = false;
                }
                _ if closed || c == '"' => return Err("invalid character after CSV quote".into()),
                _ => field.push(c),
            }
        }
    }
    if quoted {
        return Err("unterminated quoted CSV field".into());
    }
    if !row.is_empty() || !field.is_empty() || closed {
        row.push(field);
        rows.push(row);
    }
    if rows.is_empty() {
        return Err("missing CSV header".into());
    }
    let header = rows.remove(0);
    let index = header
        .iter()
        .position(|v| v == "turn_index")
        .ok_or("missing turn_index column")?;
    let words = header
        .iter()
        .position(|v| v == "first_words")
        .ok_or("missing first_words column")?;
    let mut seen = HashSet::new();
    rows.into_iter()
        .enumerate()
        .map(|(line, row)| {
            if row.len() != header.len() {
                return Err(format!("CSV record {} has wrong field count", line + 2));
            }
            let turn_index = row[index]
                .parse::<usize>()
                .map_err(|e| format!("CSV record {}: {e}", line + 2))?;
            if !seen.insert(turn_index) {
                return Err(format!("duplicate CSV turn_index {turn_index}"));
            }
            Ok(TurnIndexRow {
                turn_index,
                first_words: row[words].clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::overview_section_count;
    #[test]
    fn overview_sections_scale_with_episode_length() {
        assert_eq!(overview_section_count(20 * 60_000), 12);
        assert_eq!(overview_section_count(64 * 60_000), 21);
        assert_eq!(overview_section_count(3 * 3_600_000), 32);
        assert_eq!(overview_section_count(0), 12);
    }
}
