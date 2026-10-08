//! Dry-run schema and render arguments; no synthesis is performed by parsing a preview.
use crate::{cli::Resolved, runner::JobSpec};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Preview {
    pub schema: u32,
    pub episode_id: String,
    pub transcript: PathBuf,
    pub voices: PathBuf,
    pub turns: Vec<PlannedTurn>,
    pub speakers: Vec<Speaker>,
    pub warnings: Vec<Warning>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlannedTurn {
    pub index: usize,
    pub turn_id: String,
    pub speaker_id: String,
    pub speaker_name: String,
    pub timestamp_ms: u64,
    pub first_words: String,
    pub clip_id: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Speaker {
    pub id: String,
    pub name: String,
    pub engine: Option<String>,
    pub turns: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Warning {
    pub kind: String,
    pub line: Option<usize>,
    pub speaker_id: Option<String>,
    pub blocking: bool,
    pub message: String,
}
impl Preview {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let preview: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if preview.schema != 1 {
            return Err("Unsupported dry-run schema".into());
        }
        Ok(preview)
    }
    pub fn can_render(&self) -> bool {
        !self.turns.is_empty() && !self.warnings.iter().any(|w| w.blocking)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Options {
    pub script: String,
    pub episode_id: String,
    pub voices: String,
    pub output: String,
    pub gap_ms: u32,
    pub tail_ms: u32,
    pub speech_threshold: f64,
    pub overwrite: bool,
    pub no_trim: bool,
    pub skip_als: bool,
    pub fcpxml: bool,
    pub max_turns: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            script: String::new(),
            episode_id: String::new(),
            voices: "voices.yaml".into(),
            output: "outputs".into(),
            gap_ms: 250,
            tail_ms: 100,
            speech_threshold: 0.01,
            overwrite: false,
            no_trim: false,
            skip_als: false,
            fcpxml: false,
            max_turns: 0,
        }
    }
}
impl Options {
    pub fn dir(&self) -> PathBuf {
        Path::new(&self.output).join(&self.episode_id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.episode_id.is_empty()
            || Path::new(&self.episode_id).components().count() != 1
            || !matches!(
                Path::new(&self.episode_id).components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err("Episode id must be a single folder name".into());
        }
        if !Path::new(&self.script).is_file() {
            return Err("Choose a script file".into());
        }
        if !Path::new(&self.voices).is_file() {
            return Err("Choose a voices file".into());
        }
        Ok(())
    }
    pub fn spec(
        &self,
        resolved: Resolved,
        cwd: PathBuf,
        dry: Option<&Path>,
        resume: bool,
    ) -> JobSpec {
        let mut args = resolved.prefix.clone();
        for (flag, value) in [
            ("--transcript", self.script.clone()),
            ("--episode-id", self.episode_id.clone()),
            ("--voices", self.voices.clone()),
            ("--out-dir", self.output.clone()),
            ("--gap-ms", self.gap_ms.to_string()),
            ("--tail-ms", self.tail_ms.to_string()),
            ("--speech-threshold", self.speech_threshold.to_string()),
        ] {
            args.extend([flag.into(), value.into()]);
        }
        if let Some(path) = dry {
            args.extend([
                "--dry-run".into(),
                "--json-out".into(),
                path.as_os_str().into(),
            ]);
        } else if resume {
            args.push("--resume".into());
        } else if self.overwrite {
            args.push("--overwrite".into());
        }
        for (enabled, flag) in [
            (self.no_trim, "--no-trim"),
            (self.skip_als, "--skip-als"),
            (self.fcpxml, "--fcpxml"),
        ] {
            if enabled {
                args.push(flag.into());
            }
        }
        if self.max_turns > 0 {
            args.extend(["--max-turns".into(), self.max_turns.to_string().into()]);
        }
        JobSpec {
            program: resolved.program.clone(),
            args,
            cwd,
            synthesizes: dry.is_none(),
            overwrites: vec![],
            resolved,
        }
    }
}

impl Preview {
    /// Planned positions come from the dry run; only completed turns can expose WAVs.
    pub fn episode(
        &self,
        dir: &Path,
        completed: &std::collections::BTreeSet<usize>,
    ) -> crate::episode::Episode {
        use crate::episode::{Episode, Manifest, Speaker, Turn, VerificationState};
        let audio_dir = dir.join("Samples/Processed");
        let mut files = vec![];
        if let Ok(entries) = std::fs::read_dir(&audio_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                } else if entry.file_type().is_ok_and(|t| t.is_dir())
                    && let Ok(chunks) = std::fs::read_dir(path)
                {
                    files.extend(chunks.flatten().map(|e| e.path()));
                }
            }
        }
        let speakers: Vec<_> = self
            .speakers
            .iter()
            .enumerate()
            .map(|(i, speaker)| Speaker {
                id: speaker.id.clone(),
                name: speaker.name.clone(),
                engine: speaker.engine.clone().unwrap_or_default(),
                colour_idx: i,
            })
            .collect();
        let turns: Vec<_> = self
            .turns
            .iter()
            .enumerate()
            .map(|(i, turn)| {
                let stable_name = |path: &Path| {
                    path.file_stem().is_some_and(|stem| {
                        let stem = stem.to_string_lossy();
                        stem == turn.turn_id
                            || stem.starts_with(&format!("{}_chunk_", turn.turn_id))
                    })
                };
                let has_stable = files.iter().any(|p| stable_name(p));

                let mut wav_paths: Vec<_> = files
                    .iter()
                    .filter(|path| {
                        completed.contains(&turn.index)
                            && path.extension().is_some_and(|e| e == "wav")
                            && path.file_stem().is_some_and(|stem| {
                                let stem = stem.to_string_lossy();
                                if has_stable {
                                    stable_name(path)
                                } else {
                                    stem == format!("turn_{}", turn.index)
                                        || stem.starts_with(&format!("turn_{}_", turn.index))
                                }
                            })
                    })
                    .cloned()
                    .collect();
                wav_paths.sort_by_key(|p| {
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .and_then(|s| s.rsplit('_').next()?.parse::<usize>().ok())
                        .unwrap_or_default()
                });
                let duration: u64 = wav_paths
                    .iter()
                    .filter_map(|p| crate::audio::probe(p)?.duration)
                    .map(|d| d.as_millis() as u64)
                    .sum();
                let end_ms = if duration > 0 {
                    turn.timestamp_ms.saturating_add(duration)
                } else {
                    self.turns
                        .get(i + 1)
                        .map(|t| t.timestamp_ms)
                        .unwrap_or(
                            turn.timestamp_ms
                                + turn.first_words.split_whitespace().count() as u64 * 400,
                        )
                        .max(turn.timestamp_ms + 1)
                };
                Turn {
                    index: turn.index,
                    id: turn.turn_id.clone(),
                    speaker_idx: speakers
                        .iter()
                        .position(|s| s.id == turn.speaker_id)
                        .unwrap_or(0),
                    start_ms: turn.timestamp_ms,
                    end_ms,
                    first_words: turn.first_words.clone(),
                    verification: if wav_paths.is_empty() {
                        VerificationState::MissingAudio
                    } else {
                        VerificationState::NotVerified
                    },
                    wav_paths,
                    verify: None,
                }
            })
            .collect();
        Episode {
            dir: dir.to_owned(),
            als_path: None,
            project_root: crate::episode::find_project_root(dir),
            script_path: Some(self.transcript.clone()),
            script_in_sync: None,
            voices_path: Some(self.voices.clone()),
            manifest: Manifest {
                schema_version: "1.0.0".into(),
                episode_id: self.episode_id.clone(),
                source_file: self.transcript.display().to_string(),
                engine: "planned".into(),
                sample_rate: 48000,
                created_at: String::new(),
                speakers: vec![],
                turns: vec![],
            },
            rendered_at: None,
            total_ms: turns.iter().map(|t| t.end_ms).max().unwrap_or_default(),
            speakers,
            turns,
            warnings: vec!["Planned timing · final turn positions are set after rendering".into()],
            stress_mtime: None,
            read_along: Default::default(),
            clip_transcripts: Default::default(),
            stress_ensemble: Default::default(),
        }
    }
}
/// Reopen a running episode before its first manifest has been published.
pub fn load_live(dir: &Path) -> Result<crate::episode::Episode, crate::episode::LoadError> {
    let loaded = crate::episode::Episode::load(dir);
    for (path, record) in crate::detached::discover(dir) {
        if record.preset != "render" {
            continue;
        }
        if record.exit.is_some() && loaded.is_ok() {
            break;
        }
        let preview = std::fs::read(path.with_extension("preview.json"))
            .ok()
            .and_then(|b| Preview::parse(&b).ok());
        if let Some(preview) = preview {
            let mut progress = crate::progress::Progress::default();
            for event in crate::progress::Tail::default()
                .read(&record.progress)
                .unwrap_or_default()
            {
                progress.apply(&event);
            }
            let mut planned = preview.episode(dir, &progress.completed);
            if let Ok(existing) = &loaded {
                for turn in &mut planned.turns {
                    if !progress.completed.contains(&turn.index)
                        && let Some(old) = existing.turns.iter().find(|old| old.id == turn.id)
                        && old.verification != crate::episode::VerificationState::MissingAudio
                    {
                        turn.wav_paths = old.wav_paths.clone();
                        turn.verification = old.verification;
                        turn.verify = old.verify.clone();
                    }
                }
            }
            return Ok(planned);
        }
    }
    loaded
}
