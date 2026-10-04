//! Optional, manifest-bound read-along metadata. Loading never writes episode files.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io::ErrorKind, path::Path};

#[derive(Clone, Debug, Deserialize)]
pub struct Word {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Timings {
    pub manifest_sha256: String,
    pub model: String,
    pub turns: HashMap<String, Vec<Word>>,
}

#[derive(Clone, Debug, Default)]
pub struct ReadAlong {
    pub timings: Option<Timings>,
    pub notice: Option<String>,
}

fn manifest_hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl ReadAlong {
    pub fn load(dir: &Path, manifest: &[u8]) -> Self {
        let bytes = match fs::read(dir.join("word_timings.json")) {
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
            timings: None,
            notice: Some(format!("Read-along timings unavailable: {error}")),
        }
    }
    pub fn from_bytes(bytes: &[u8], manifest: &[u8]) -> Result<Self, String> {
        let timings: Timings = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if timings.manifest_sha256 != manifest_hash(manifest) {
            return Ok(Self {
                timings: None,
                notice: Some("Read-along timings are out of date (episode re-rendered)".into()),
            });
        }
        for words in timings.turns.values() {
            if words.iter().any(|w| w.end_ms < w.start_ms)
                || words.windows(2).any(|w| w[1].start_ms < w[0].start_ms)
            {
                return Err("Invalid word time bounds or order".into());
            }
        }
        Ok(Self {
            timings: Some(timings),
            notice: None,
        })
    }
    pub fn words(&self, turn_id: &str) -> Option<&[Word]> {
        self.timings
            .as_ref()?
            .turns
            .get(turn_id)
            .filter(|words| !words.is_empty())
            .map(Vec::as_slice)
    }
}

/// The player concatenates WAVs, while the manifest can leave gaps between segments.
/// Convert its cursor back to the timeline used by the per-segment transcriber.
pub fn manifest_position(turn: &crate::episode::TurnEntry, mut position_ms: u64) -> u64 {
    let mut segments: Vec<_> = turn.segments.iter().collect();
    segments.sort_by_key(|segment| segment.chunk_index);
    for segment in segments {
        if position_ms < segment.duration_ms {
            return segment.start_ms.saturating_sub(turn.start_ms) + position_ms;
        }
        position_ms = position_ms.saturating_sub(segment.duration_ms);
    }
    turn.end_ms.saturating_sub(turn.start_ms)
}

/// Gaps and silence do not highlight a neighbour. End boundaries are exclusive.
pub fn current_word(words: &[Word], position_ms: u64) -> Option<usize> {
    words
        .iter()
        .position(|word| word.start_ms <= position_ms && position_ms < word.end_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_boundaries_and_gaps() {
        let words = vec![
            Word {
                text: "One".into(),
                start_ms: 100,
                end_ms: 300,
            },
            Word {
                text: "two".into(),
                start_ms: 400,
                end_ms: 500,
            },
        ];
        for (position, expected) in [
            (0, None),
            (99, None),
            (100, Some(0)),
            (299, Some(0)),
            (300, None),
            (350, None),
            (400, Some(1)),
            (500, None),
            (1000, None),
        ] {
            assert_eq!(current_word(&words, position), expected);
        }
        assert_eq!(current_word(&[], 100), None);
    }

    #[test]
    fn playback_cursor_tracks_manifest_offsets_across_concatenated_segments() {
        let turn = serde_json::from_value(serde_json::json!({
            "turn_index": 0, "turn_id": "one", "speaker_id": "speaker",
            "start_ms": 10000, "end_ms": 13000,
            "segments": [
                {"chunk_index": 1, "segment_wav": "b.wav", "start_ms": 12000, "duration_ms": 1000},
                {"chunk_index": 0, "segment_wav": "a.wav", "start_ms": 10000, "duration_ms": 1000}
            ]
        }))
        .unwrap();
        assert_eq!(manifest_position(&turn, 500), 500);
        assert_eq!(manifest_position(&turn, 1000), 2000);
        assert_eq!(manifest_position(&turn, 1500), 2500);
        assert_eq!(manifest_position(&turn, 2000), 3000);
    }

    #[test]
    fn matching_manifest_loads_and_changed_bytes_hide_timings() {
        let manifest = b"manifest bytes";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "manifest_sha256": manifest_hash(manifest),
            "model": "fake",
            "turns": {"turn": [{"text": "Hello", "start_ms": 0, "end_ms": 100}]}
        }))
        .unwrap();
        let loaded = ReadAlong::from_bytes(&bytes, manifest).unwrap();
        assert_eq!(loaded.words("turn").unwrap()[0].text, "Hello");
        assert!(loaded.words("clip").is_none());
        let stale = ReadAlong::from_bytes(&bytes, b"changed manifest").unwrap();
        assert!(stale.words("turn").is_none());
        assert_eq!(
            stale.notice.as_deref(),
            Some("Read-along timings are out of date (episode re-rendered)")
        );
        assert!(ReadAlong::from_bytes(b"{", manifest).is_err());
    }
}
