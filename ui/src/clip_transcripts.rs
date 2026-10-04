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
            if turn.words.iter().any(|w| w.end_ms < w.start_ms)
                || turn.words.windows(2).any(|w| w[1].start_ms < w[0].start_ms)
            {
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
