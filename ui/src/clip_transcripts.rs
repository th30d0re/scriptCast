//! Optional, manifest-bound ensemble transcripts for archive clip turns.
use crate::word_timings::Word;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io::ErrorKind, path::Path};

#[derive(Clone, Debug, Deserialize)]
pub struct ClipWord {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default)]
    pub engine: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScriptWord {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default)]
    pub matched: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CoverageWarning {
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub words: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClipTurn {
    pub clip_id: String,
    pub words: Vec<ClipWord>,
    #[serde(default)]
    pub script_words: Vec<ScriptWord>,
    #[serde(default)]
    pub coverage_warnings: Vec<CoverageWarning>,
    #[serde(default)]
    pub whisper_disagreement_pct: f64,
    #[serde(default)]
    pub listen_manually: bool,
    #[serde(default)]
    pub engines_used: Vec<String>,
}

impl ClipTurn {
    pub fn ensemble_text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn as_words(&self) -> Vec<Word> {
        self.words
            .iter()
            .map(|w| Word {
                text: w.text.clone(),
                start_ms: w.start_ms,
                end_ms: w.end_ms,
            })
            .collect()
    }

    pub fn as_script_words(&self) -> Vec<Word> {
        self.script_words
            .iter()
            .map(|w| Word {
                text: w.text.clone(),
                start_ms: w.start_ms,
                end_ms: w.end_ms,
            })
            .collect()
    }

    pub fn has_script_words(&self) -> bool {
        !self.script_words.is_empty()
    }

    pub fn all_phonon2(&self) -> bool {
        !self.words.is_empty() && self.words.iter().all(|w| w.engine == "phonon-2")
    }

    pub fn engine_labels(&self) -> Vec<String> {
        let mut seen = Vec::new();
        for word in &self.words {
            if !word.engine.is_empty() && !seen.iter().any(|e| e == &word.engine) {
                seen.push(word.engine.clone());
            }
        }
        if seen.is_empty() {
            for name in &self.engines_used {
                if !seen.iter().any(|e| e == name) {
                    seen.push(name.clone());
                }
            }
        }
        seen
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClipTranscriptsFile {
    pub manifest_sha256: String,
    pub turns: HashMap<String, ClipTurn>,
}

#[derive(Clone, Debug, Default)]
pub struct ClipTranscripts {
    pub file: Option<ClipTranscriptsFile>,
    pub notice: Option<String>,
}

fn manifest_hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn times_ok(words: impl Iterator<Item = (u64, u64)>) -> bool {
    let mut previous: Option<u64> = None;
    for (start, end) in words {
        if end < start {
            return false;
        }
        if previous.is_some_and(|prev| start < prev) {
            return false;
        }
        previous = Some(start);
    }
    true
}

impl ClipTranscripts {
    pub fn load(dir: &Path, manifest: &[u8]) -> Self {
        let bytes = match fs::read(dir.join("clip_transcripts.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Self::default(),
            Err(error) => return Self::unavailable(error),
        };
        match Self::from_bytes(&bytes, manifest) {
            Ok(value) => value,
            Err(error) => Self::unavailable(error),
        }
    }

    fn unavailable(error: impl std::fmt::Display) -> Self {
        Self {
            file: None,
            notice: Some(format!("Clip transcripts unavailable: {error}")),
        }
    }

    pub fn from_bytes(bytes: &[u8], manifest: &[u8]) -> Result<Self, String> {
        let file: ClipTranscriptsFile = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if file.manifest_sha256 != manifest_hash(manifest) {
            return Ok(Self {
                file: None,
                notice: Some("Clip transcripts are out of date (episode re-rendered)".into()),
            });
        }
        for turn in file.turns.values() {
            let ensemble_ok = times_ok(turn.words.iter().map(|w| (w.start_ms, w.end_ms)));
            let script_ok = times_ok(turn.script_words.iter().map(|w| (w.start_ms, w.end_ms)));
            if !ensemble_ok || !script_ok {
                return Err("Invalid clip word time bounds or order".into());
            }
        }
        Ok(Self {
            file: Some(file),
            notice: None,
        })
    }

    pub fn get(&self, turn_id: &str) -> Option<&ClipTurn> {
        self.file.as_ref()?.turns.get(turn_id)
    }

    pub fn words(&self, turn_id: &str) -> Option<Vec<Word>> {
        let turn = self.get(turn_id)?;
        if turn.words.is_empty() {
            None
        } else {
            Some(turn.as_words())
        }
    }

    /// Prefer script-aligned timings for in-row highlighting when present (v2).
    pub fn row_words(&self, turn_id: &str) -> Option<Vec<Word>> {
        let turn = self.get(turn_id)?;
        if !turn.script_words.is_empty() {
            Some(turn.as_script_words())
        } else if !turn.words.is_empty() {
            Some(turn.as_words())
        } else {
            None
        }
    }
}

/// Apply ensemble text after the clip tag; keep the tag (transcript-only).
pub fn use_ensemble_transcript(body: &str, ensemble: &str) -> Result<String, String> {
    let tag = crate::script::clip_tag(body)
        .ok_or_else(|| "Use ensemble transcript needs a [clip:<id>] tag".to_string())?;
    let ensemble = ensemble.trim();
    if ensemble.is_empty() {
        return Err("Ensemble transcript is empty".into());
    }
    Ok(format!("{tag} {ensemble}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_timings::{current_word, manifest_position};

    #[test]
    fn matching_manifest_loads_and_changed_bytes_hide_transcripts() {
        let manifest = b"manifest bytes";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "manifest_sha256": manifest_hash(manifest),
            "turns": {
                "t1": {
                    "clip_id": "atwater_1981",
                    "words": [
                        {"text": "Hello", "start_ms": 0, "end_ms": 100, "engine": "phonon-2"}
                    ],
                    "coverage_warnings": [],
                    "whisper_disagreement_pct": 11.8,
                    "listen_manually": false,
                    "engines_used": ["phonon-2"]
                }
            }
        }))
        .unwrap();
        let loaded = ClipTranscripts::from_bytes(&bytes, manifest).unwrap();
        assert_eq!(loaded.get("t1").unwrap().clip_id, "atwater_1981");
        assert_eq!(loaded.words("t1").unwrap()[0].text, "Hello");
        assert!(loaded.words("missing").is_none());
        let stale = ClipTranscripts::from_bytes(&bytes, b"changed").unwrap();
        assert!(stale.get("t1").is_none());
        assert_eq!(
            stale.notice.as_deref(),
            Some("Clip transcripts are out of date (episode re-rendered)")
        );
        assert!(ClipTranscripts::from_bytes(b"{", manifest).is_err());
    }

    #[test]
    fn v1_without_script_words_keeps_ensemble_row_lookup() {
        let manifest = b"manifest v1";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "manifest_sha256": manifest_hash(manifest),
            "turns": {
                "t1": {
                    "clip_id": "demo",
                    "words": [
                        {"text": "Hello", "start_ms": 0, "end_ms": 100, "engine": "phonon-2"}
                    ]
                }
            }
        }))
        .unwrap();
        let loaded = ClipTranscripts::from_bytes(&bytes, manifest).unwrap();
        let turn = loaded.get("t1").unwrap();
        assert!(!turn.has_script_words());
        assert_eq!(loaded.row_words("t1").unwrap()[0].text, "Hello");
    }

    #[test]
    fn v2_row_words_prefer_script_words() {
        let manifest = b"manifest v2";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "version": 2,
            "manifest_sha256": manifest_hash(manifest),
            "turns": {
                "t1": {
                    "clip_id": "demo",
                    "words": [
                        {"text": "nineteen", "start_ms": 100, "end_ms": 200, "engine": "x"},
                        {"text": "fifty", "start_ms": 200, "end_ms": 300, "engine": "x"},
                        {"text": "four", "start_ms": 300, "end_ms": 400, "engine": "x"}
                    ],
                    "script_words": [
                        {"text": "1954", "start_ms": 100, "end_ms": 400, "matched": true}
                    ]
                }
            }
        }))
        .unwrap();
        let loaded = ClipTranscripts::from_bytes(&bytes, manifest).unwrap();
        let row = loaded.row_words("t1").unwrap();
        assert_eq!(row.len(), 1);
        assert_eq!(row[0].text, "1954");
        assert_eq!(row[0].start_ms, 100);
        assert_eq!(row[0].end_ms, 400);
        assert_eq!(loaded.words("t1").unwrap().len(), 3);
    }

    #[test]
    fn clip_read_along_word_lookup_uses_turn_relative_times() {
        let words = vec![
            Word {
                text: "You".into(),
                start_ms: 480,
                end_ms: 560,
            },
            Word {
                text: "start".into(),
                start_ms: 640,
                end_ms: 880,
            },
        ];
        assert_eq!(current_word(&words, 400), None);
        assert_eq!(current_word(&words, 500), Some(0));
        assert_eq!(current_word(&words, 700), Some(1));
        assert_eq!(current_word(&words, 880), None);
        let turn = serde_json::from_value(serde_json::json!({
            "turn_index": 220, "turn_id": "clip", "speaker_id": "s",
            "start_ms": 0, "end_ms": 1000,
            "segments": [
                {"chunk_index": 0, "segment_wav": "a.wav",
                 "start_ms": 0, "duration_ms": 1000}
            ]
        }))
        .unwrap();
        assert_eq!(manifest_position(&turn, 500), 500);
    }

    #[test]
    fn use_ensemble_keeps_clip_tag() {
        let body = "[clip:atwater_1981] Operator text.";
        let out = use_ensemble_transcript(body, "Ensemble heard this.").unwrap();
        assert!(out.starts_with("[clip:atwater_1981] "));
        assert!(out.contains("Ensemble heard this."));
        assert!(!out.contains("Operator text."));
        assert!(use_ensemble_transcript("plain", "x").is_err());
        assert!(use_ensemble_transcript(body, "  ").is_err());
    }
}
