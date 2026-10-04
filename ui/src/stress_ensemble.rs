//! Advisory Phonon-2 second opinion for failed verification turns.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io::ErrorKind, path::Path};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct EnsembleSpan {
    pub start_token: usize,
    pub end_token: usize,
    #[serde(default)]
    pub expected: String,
    #[serde(default)]
    pub whisper: String,
    #[serde(default)]
    pub phonon: String,
    pub verdict: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EnsembleTurn {
    pub score_whisper: f64,
    pub score_phonon: f64,
    #[serde(default)]
    pub spans: Vec<EnsembleSpan>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StressEnsembleFile {
    pub manifest_sha256: String,
    pub stress_sha256: String,
    pub turns: HashMap<String, EnsembleTurn>,
}

#[derive(Clone, Debug, Default)]
pub struct StressEnsemble {
    pub file: Option<StressEnsembleFile>,
    pub notice: Option<String>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl StressEnsemble {
    pub fn load(dir: &Path, manifest: &[u8], stress: Option<&[u8]>) -> Self {
        let bytes = match fs::read(dir.join("stress_ensemble.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Self::default(),
            Err(error) => return Self::unavailable(error),
        };
        match Self::from_bytes(&bytes, manifest, stress) {
            Ok(value) => value,
            Err(error) => Self::unavailable(error),
        }
    }

    fn unavailable(error: impl std::fmt::Display) -> Self {
        Self {
            file: None,
            notice: Some(format!("Second opinion unavailable: {error}")),
        }
    }

    pub fn from_bytes(
        bytes: &[u8],
        manifest: &[u8],
        stress: Option<&[u8]>,
    ) -> Result<Self, String> {
        let file: StressEnsembleFile = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if file.manifest_sha256 != sha256_hex(manifest) {
            return Ok(Self {
                file: None,
                notice: Some("Second opinion is out of date (episode re-rendered)".into()),
            });
        }
        if let Some(stress_bytes) = stress
            && file.stress_sha256 != sha256_hex(stress_bytes)
        {
            return Ok(Self {
                file: None,
                notice: Some("Second opinion is out of date (verification re-ran)".into()),
            });
        }
        Ok(Self {
            file: Some(file),
            notice: None,
        })
    }

    pub fn get(&self, turn_id: &str) -> Option<&EnsembleTurn> {
        self.file.as_ref()?.turns.get(turn_id)
    }

    pub fn summary_line(summary: &str) -> &'static str {
        match summary {
            "likely_audio" => {
                "Engines agree the audio differs from the script: likely an audio problem"
            }
            "likely_recognition" => {
                "Engines disagree with each other: likely a recognition error, listen first"
            }
            "matches_script" => {
                "Both engines now hear the script exactly: the failure did not reproduce"
            }
            _ => "Mixed: listen",
        }
    }

    pub fn span_verdict_label(verdict: &str) -> &'static str {
        match verdict {
            "likely_audio" => "audio",
            "likely_recognition" => "recognition",
            "coverage_gap" => "coverage gap",
            _ => "uncertain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_hashes_load_and_stale_on_manifest_or_stress() {
        let manifest = b"manifest bytes";
        let stress = b"stress bytes";
        let bytes = serde_json::to_vec(&serde_json::json!({
            "manifest_sha256": sha256_hex(manifest),
            "stress_sha256": sha256_hex(stress),
            "turns": {
                "t1": {
                    "score_whisper": 0.7,
                    "score_phonon": 0.95,
                    "spans": [{
                        "start_token": 1,
                        "end_token": 2,
                        "expected": "green",
                        "whisper": "brown",
                        "phonon": "brown",
                        "verdict": "likely_audio"
                    }],
                    "summary": "likely_audio"
                }
            }
        }))
        .unwrap();
        let loaded = StressEnsemble::from_bytes(&bytes, manifest, Some(stress)).unwrap();
        assert_eq!(loaded.get("t1").unwrap().summary, "likely_audio");
        let stale_m = StressEnsemble::from_bytes(&bytes, b"changed", Some(stress)).unwrap();
        assert!(stale_m.get("t1").is_none());
        assert!(stale_m.notice.as_deref().unwrap().contains("re-rendered"));
        let stale_s = StressEnsemble::from_bytes(&bytes, manifest, Some(b"other")).unwrap();
        assert!(stale_s.get("t1").is_none());
        assert!(
            stale_s
                .notice
                .as_deref()
                .unwrap()
                .contains("verification re-ran")
        );
    }
}
