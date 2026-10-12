//! Preview on a private script copy; Apply commits exactly the reviewed bytes.
use crate::{
    checkpoint::{self, Checkpoint},
    cli::{self, ExportCommand, ExportSource, Resolved},
    episode::Episode,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Diff {
    pub turn_index: usize,
    pub display_name: String,
    pub old: String,
    pub new: String,
}
#[derive(Clone, Debug)]
pub struct Preview {
    pub path: PathBuf,
    pub hash: String,
    pub bytes: Vec<u8>,
    pub rows: Vec<Diff>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoRetime {
    Off,
    WaitForJob,
    WaitForEdits,
    Run,
    Silent,
    Apply(usize),
}
/// `preview` is `None` before the temp-copy diff, or `Some(changed rows)` after.
pub fn auto_retime_decision(
    enabled: bool,
    job_running: bool,
    unsaved_edits: bool,
    preview: Option<usize>,
) -> AutoRetime {
    if !enabled {
        AutoRetime::Off
    } else if job_running {
        AutoRetime::WaitForJob
    } else if unsaved_edits {
        AutoRetime::WaitForEdits
    } else {
        match preview {
            None => AutoRetime::Run,
            Some(0) => AutoRetime::Silent,
            Some(n) => AutoRetime::Apply(n),
        }
    }
}

pub fn parse_diff(json: &[u8]) -> Result<Vec<Diff>, String> {
    serde_json::from_slice::<Vec<Diff>>(json)
        .map(|rows| rows.into_iter().filter(|r| r.old != r.new).collect())
        .map_err(|e| format!("Invalid retime mapping: {e}"))
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn preview(export: &ExportCommand, episode: &Episode) -> Result<Preview, String> {
    let path = episode
        .script_path
        .as_ref()
        .ok_or("Script path unavailable")?
        .clone();
    let original = fs::read(&path).map_err(|e| e.to_string())?;
    let dir = std::env::temp_dir().join("scriptcast-k").join(format!(
        "retime-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result = (|| {
        let copy = dir.join("script.md");
        let map = dir.join("mapping.json");
        fs::write(&copy, &original).map_err(|e| e.to_string())?;
        let checkout = match &export.source {
            ExportSource::DevCheckout(p) => Some(p.clone()),
            _ => None,
        };
        let command = Resolved {
            source: export.source.clone(),
            program: if checkout.is_some() {
                export.program.clone()
            } else {
                export.program.with_file_name("scriptcast-retime")
            },
            prefix: if checkout.is_some() {
                vec!["-m".into(), "scriptcast.tools.retime_script".into()]
            } else {
                vec![]
            },
            path: cli::login_path(),
            pythonpath: checkout,
        };
        let help = command
            .base_command()
            .arg("--help")
            .output()
            .map_err(|e| e.to_string())?;
        if !help.status.success() || !String::from_utf8_lossy(&help.stdout).contains("--map-out") {
            return Err(
                "Retime tool must support --map-out and --apply; check CLI version.".into(),
            );
        }
        // --apply is confined to the temporary copy; the real script stays untouched until Apply.
        let output = command
            .base_command()
            .arg(&copy)
            .arg("--manifest")
            .arg(episode.dir.join("episode_manifest.json"))
            .arg("--map-out")
            .arg(&map)
            .arg("--apply")
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().into());
        }
        Ok(Preview {
            path,
            hash: hash(&original),
            bytes: fs::read(copy).map_err(|e| e.to_string())?,
            rows: parse_diff(&fs::read(map).map_err(|e| e.to_string())?)?,
        })
    })();
    let _ = fs::remove_dir_all(dir);
    result
}
pub fn apply(episode: &Episode, preview: &Preview) -> Result<Checkpoint, String> {
    let check = || -> Result<(), String> {
        if hash(&fs::read(&preview.path).map_err(|e| e.to_string())?) != preview.hash {
            return Err("The script changed on disk. Preview retime again.".into());
        }
        Ok(())
    };
    check()?;
    let cp = checkpoint::create_external(
        &episode.dir,
        "script-retime",
        std::slice::from_ref(&preview.path),
        vec![],
    )?;
    let temp =
        preview
            .path
            .with_file_name(format!(".retime-{}-{}", std::process::id(), cp.version));
    let result = (|| {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        file.write_all(&preview.bytes).map_err(|e| e.to_string())?;
        file.set_permissions(
            fs::metadata(&preview.path)
                .map_err(|e| e.to_string())?
                .permissions(),
        )
        .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        check()?;
        fs::rename(&temp, &preview.path).map_err(|e| e.to_string())?;
        Ok(cp)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{AutoRetime, auto_retime_decision};

    #[test]
    fn auto_retime_decision_table() {
        assert_eq!(
            auto_retime_decision(true, false, false, Some(0)),
            AutoRetime::Silent
        );
        assert_eq!(
            auto_retime_decision(true, false, false, Some(4)),
            AutoRetime::Apply(4)
        );
        assert_eq!(
            auto_retime_decision(true, true, false, Some(4)),
            AutoRetime::WaitForJob
        );
        assert_eq!(
            auto_retime_decision(true, false, true, Some(4)),
            AutoRetime::WaitForEdits
        );
        assert_eq!(
            auto_retime_decision(false, false, false, Some(4)),
            AutoRetime::Off
        );
    }
}
