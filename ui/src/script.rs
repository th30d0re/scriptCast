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
