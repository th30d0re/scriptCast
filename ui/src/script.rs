//! Python-owned transcript spans, byte-preserving edits and reversible saves.
use crate::{
    checkpoint::{self, Checkpoint},
    cli::{self, ExportCommand, ExportSource, Resolved},
    episode::Episode,
};
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct ScriptTurns {
    pub script_path: PathBuf,
    pub script_sha256: String,
    pub line_ending: String,
    pub turns: Vec<ScriptTurn>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScriptTurn {
    pub index: usize,
    pub turn_id: String,
    pub speaker_id: String,
    pub display_name: String,
    pub header_line: usize,
    pub body_start: usize,
    pub body_end: usize,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Changes {
    pub inserted: Vec<usize>,
    pub modified: Vec<usize>,
    pub deleted: Vec<String>,
    pub unchanged: usize,
}

/// Blocking worker function: call off the UI thread, like DAW export work.
pub fn load_turns(export: &ExportCommand, episode: &Episode) -> Result<ScriptTurns, String> {
    load_json(export, episode, "--turns-json")
}

/// Blocking worker function; subprocesses inherit the resolved login PATH and null stdin.
pub fn load_changes(export: &ExportCommand, episode: &Episode) -> Result<Changes, String> {
    load_json(export, episode, "--changes-json")
}

fn load_json<T: DeserializeOwned>(
    export: &ExportCommand,
    episode: &Episode,
    mode: &str,
) -> Result<T, String> {
    let checkout = match &export.source {
        ExportSource::DevCheckout(path) => Some(path.clone()),
        _ => None,
    };
    let resolved = Resolved {
        source: export.source.clone(),
        program: export.program.clone(),
        prefix: if checkout.is_some() {
            vec!["-m".into(), "scriptcast.tools.export_session".into()]
        } else {
            vec![]
        },
        path: cli::login_path(),
        pythonpath: checkout,
    };
    let mut command = resolved.base_command();
    command
        .arg(&episode.dir)
        .arg(mode)
        .current_dir(&episode.dir);
    if let Some(root) = &episode.project_root {
        command.env("SCRIPTCAST_PROJECT", root);
    }
    let output = command.output().map_err(|e| format!("{mode}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{mode} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("Invalid {mode} JSON: {e}"))
}

/// Replace the exported body span, keeping all surrounding bytes and its final newline.
/// Empty drafts and invalid/stale spans are errors, rather than panics or silent edits.
pub fn splice(original: &[u8], turn: &ScriptTurn, new_body: &str) -> Result<Vec<u8>, String> {
    if new_body.trim().is_empty() {
        return Err("A turn's body cannot be empty.".into());
    }
    let mut starts = vec![0];
    for (i, byte) in original.iter().enumerate() {
        if *byte == b'\n' {
            starts.push(i + 1);
        }
    }
    if starts.last() != Some(&original.len()) {
        starts.push(original.len());
    }
    let line_count = starts.len() - 1;
    if turn.header_line == 0
        || turn.header_line > line_count
        || turn.body_start <= turn.header_line
        || turn.body_start > line_count + 1
        || turn.body_end < turn.body_start - 1
        || turn.body_end > line_count
    {
        return Err("Invalid script body span. Reload to edit.".into());
    }
    let start = starts[turn.body_start - 1];
    let end = starts[turn.body_end];
    let old_body = &original[start..end];
    let content = old_body.strip_suffix(b"\n").unwrap_or(old_body);
    let content = content.strip_suffix(b"\r").unwrap_or(content);
    if content != turn.body.as_bytes() {
        return Err("Script body does not match its loaded span. Reload to edit.".into());
    }
    let ending = if original.windows(2).any(|p| p == b"\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let normalized = new_body.replace("\r\n", "\n");
    if normalized.contains('\r') {
        return Err("Unsupported bare carriage return in draft.".into());
    }
    let body = normalized.replace('\n', ending);
    let mut result = original[..start].to_vec();
    // A header at EOF may have no terminator yet; inserting its first body needs one.
    if start == end && start > 0 && original[start - 1] != b'\n' {
        result.extend_from_slice(ending.as_bytes());
    }
    result.extend_from_slice(body.as_bytes());
    if old_body.ends_with(b"\n") || (start == end && end < original.len()) {
        result.extend_from_slice(ending.as_bytes());
    }
    result.extend_from_slice(&original[end..]);
    Ok(result)
}

/// Refuse stale data before writing anything, checkpoint, then atomically replace the script.
pub fn save(
    episode: &Episode,
    turn: &ScriptTurn,
    new_body: &str,
    loaded_sha256: &str,
) -> Result<Checkpoint, String> {
    preserve_clip_tag(&turn.body, new_body)?;
    let path = episode
        .script_path
        .as_ref()
        .ok_or("Script path unavailable")?;
    let path = fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let original = fs::read(&path).map_err(|e| e.to_string())?;
    check_hash(&original, loaded_sha256)?;
    let edited = splice(&original, turn, new_body)?;
    let mode = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();
    let cp = checkpoint::create_external(
        &episode.dir,
        "script-edit",
        std::slice::from_ref(&path),
        vec![turn.index],
    )?;
    let parent = path.parent().ok_or("Script has no parent directory")?;
    let temporary = parent.join(format!(
        ".scriptcast-edit-{}-{}",
        std::process::id(),
        cp.version
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| format!("{}: {e}", temporary.display()))?;
    let result = (|| {
        file.write_all(&edited).map_err(|e| e.to_string())?;
        file.set_permissions(mode).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        // Catch changes made while the checkpoint or temporary file was being written.
        check_hash(&fs::read(&path).map_err(|e| e.to_string())?, loaded_sha256)?;
        fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
        Ok(cp)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn check_hash(bytes: &[u8], expected: &str) -> Result<(), String> {
    let hash: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if hash != expected {
        return Err("The script changed on disk. Reload to edit.".into());
    }
    Ok(())
}

pub fn edited_turns(
    changes: &Changes,
    manifest_len: usize,
    script_len: usize,
) -> Option<Vec<usize>> {
    if manifest_len != script_len {
        return None;
    }
    let mut indices = changes.inserted.clone();
    indices.extend(&changes.modified);
    if indices.iter().any(|i| *i >= script_len) {
        return None;
    }
    indices.sort_unstable();
    indices.dedup();
    Some(indices)
}

/// First `[clip:<id>]` tag id in a turn body, if any.
pub fn clip_id(body: &str) -> Option<&str> {
    let start = body.find("[clip:")? + "[clip:".len();
    let rest = body.get(start..)?;
    let end = rest.find(']')?;
    let id = rest.get(..end)?.trim();
    if id.is_empty() || id.chars().any(char::is_whitespace) {
        None
    } else {
        Some(id)
    }
}

/// Full `[clip:<id>]` token for the first clip tag in `body`.
pub fn clip_tag(body: &str) -> Option<String> {
    clip_id(body).map(|id| format!("[clip:{id}]"))
}

/// Body text with the first clip tag removed, for table display.
pub fn display_body(body: &str) -> String {
    match clip_tag(body) {
        Some(tag) => {
            let stripped = body.replacen(&tag, "", 1);
            stripped.trim().to_string()
        }
        None => body.to_string(),
    }
}

/// Refuse saves that drop an existing audio-clip tag.
pub fn preserve_clip_tag(original: &str, edited: &str) -> Result<(), String> {
    let Some(tag) = clip_tag(original) else {
        return Ok(());
    };
    if edited.contains(&tag) {
        Ok(())
    } else {
        Err(format!(
            "Keep the audio clip tag {tag}. Transcript-only edits cannot remove it."
        ))
    }
}

/// Edited turns split into synthesis work versus clip transcript corrections.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderScope {
    pub synthesis: Vec<usize>,
    pub clip_corrections: Vec<usize>,
}

impl RenderScope {
    pub fn from_changes(changes: &Changes, turns: &[ScriptTurn]) -> Self {
        let mut synthesis = Vec::new();
        let mut clip_corrections = Vec::new();
        let mut indices = changes.inserted.clone();
        indices.extend(&changes.modified);
        indices.sort_unstable();
        indices.dedup();
        for index in indices {
            match turns.iter().find(|t| t.index == index) {
                Some(turn) if clip_id(&turn.body).is_some() => clip_corrections.push(index),
                Some(_) => synthesis.push(index),
                None => synthesis.push(index),
            }
        }
        Self {
            synthesis,
            clip_corrections,
        }
    }

    pub fn synthesis_enabled(&self) -> bool {
        !self.synthesis.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(index: usize, body: &str) -> ScriptTurn {
        ScriptTurn {
            index,
            turn_id: format!("t{index}"),
            speaker_id: "s".into(),
            display_name: "S".into(),
            header_line: 1,
            body_start: 2,
            body_end: 2,
            body: body.into(),
        }
    }

    #[test]
    fn clip_id_detects_tag_and_rejects_empty_or_spaced() {
        assert_eq!(clip_id("[clip:atwater_1981] spoken"), Some("atwater_1981"));
        assert_eq!(clip_id("pre [clip:x] post"), Some("x"));
        assert_eq!(clip_id("no clip here"), None);
        assert_eq!(clip_id("[clip:]"), None);
        assert_eq!(clip_id("[clip:has space]"), None);
        assert_eq!(
            display_body("[clip:atwater_1981] Spoken words."),
            "Spoken words."
        );
    }

    #[test]
    fn save_refuses_removed_clip_tag() {
        let original = "[clip:atwater_1981] Hello there.";
        assert!(preserve_clip_tag(original, original).is_ok());
        assert!(preserve_clip_tag(original, "[clip:atwater_1981] Hello.").is_ok());
        let err = preserve_clip_tag(original, "Hello there.").unwrap_err();
        assert!(err.contains("[clip:atwater_1981]"));
        assert!(preserve_clip_tag("plain text", "other").is_ok());
    }

    #[test]
    fn render_scope_splits_synthesis_and_clip_only_disables() {
        let turns = vec![
            turn(1, "Synthesized line."),
            turn(2, "[clip:atwater_1981] Archive line."),
            turn(3, "Another synth."),
        ];
        let mixed = Changes {
            inserted: vec![],
            modified: vec![1, 2],
            deleted: vec![],
            unchanged: 1,
        };
        let scope = RenderScope::from_changes(&mixed, &turns);
        assert_eq!(scope.synthesis, vec![1]);
        assert_eq!(scope.clip_corrections, vec![2]);
        assert!(scope.synthesis_enabled());

        let clip_only = Changes {
            inserted: vec![],
            modified: vec![2],
            deleted: vec![],
            unchanged: 2,
        };
        let scope = RenderScope::from_changes(&clip_only, &turns);
        assert!(scope.synthesis.is_empty());
        assert_eq!(scope.clip_corrections, vec![2]);
        assert!(!scope.synthesis_enabled());
    }
}
