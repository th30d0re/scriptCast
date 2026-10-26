//! Python-owned structural operations and registry metadata.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOp {
    Insert {
        position: usize,
        speaker_id: String,
        timestamp: Option<String>,
        body: String,
    },
    Delete {
        index: usize,
    },
    Move {
        index: usize,
        to: usize,
    },
    SetSpeaker {
        index: usize,
        speaker_id: String,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Registry {
    pub speakers: Vec<Speaker>,
    pub clips: Vec<Clip>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Speaker {
    pub id: String,
    pub name: String,
    pub engine: String,
    pub has_expressive: bool,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Clip {
    pub id: String,
    pub source: PathBuf,
    pub start: f64,
    pub end: f64,
    #[serde(default)]
    pub citation: String,
}

pub fn edit_error(code: Option<i32>, stderr: &str) -> String {
    match code {
        Some(3) => "The script changed on disk. Reload to edit.".into(),
        Some(1) => format!("Structure not saved: {}", stderr.trim()),
        _ => format!("Structure edit failed: {}", stderr.trim()),
    }
}

pub fn archive_ready(engine: &str, text: &str, registry: &Registry) -> bool {
    engine != "archive"
        || registry
            .clips
            .iter()
            .any(|c| text.starts_with(&format!("[clip:{}]", c.id)))
}
