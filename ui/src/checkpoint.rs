//! Explicit copy plans, complete-only checkpoint publication, reversible restore.
use crate::{episode::Episode, runner::Preset};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub version: u32,
    pub created_at: String,
    pub preset: String,
    pub argv: Vec<String>,
    pub turns: Vec<usize>,
    pub files: Vec<PathBuf>,
    pub bytes: u64,
    #[serde(skip)]
    pub dir: PathBuf,
}
/// Separates source/destination from storage, allowing Phase D external plans.
#[derive(Clone, Debug)]
pub struct PlannedFile {
    pub path: PathBuf,
    pub stored_as: PathBuf,
}
pub fn plan_files(p: &Preset, e: &Episode) -> Result<Vec<PlannedFile>, String> {
    let mut paths = vec![];
    if matches!(p, Preset::Verify) {
        paths.push(e.dir.join("stress.json"));
    } else if !matches!(p, Preset::DetectChanges) {
        for name in [
            "episode_manifest.json",
            "render_state.json",
            "TURN_INDEX.csv",
        ] {
            paths.push(e.dir.join(name));
        }
        for entry in fs::read_dir(&e.dir).map_err(|x| x.to_string())? {
            let path = entry.map_err(|x| x.to_string())?.path();
            if path.extension().is_some_and(|x| x == "als") {
                paths.push(path);
            }
        }
        match p {
            Preset::PrecisionInsert | Preset::Repair(None) => {
                walk(&e.dir.join("Samples"), &mut paths)?
            }
            _ => {
                let turns = p.turns();
                for t in &e.turns {
                    if turns.contains(&t.index) {
                        paths.extend(t.wav_paths.clone());
                    }
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter(|p| p.exists())
        .map(|path| {
            let stored_as = path
                .strip_prefix(&e.dir)
                .map_err(|_| format!("File outside episode: {}", path.display()))?
                .to_owned();
            validate_relative(&stored_as)?;
            safe_existing(&e.dir, &path)?;
            Ok(PlannedFile { path, stored_as })
        })
        .collect()
}
fn walk(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let ty = e.file_type().map_err(|e| e.to_string())?;
        if ty.is_symlink() {
            return Err(format!("Refusing symlink {}", e.path().display()));
        }
        if ty.is_dir() {
            walk(&e.path(), paths)?;
        } else if e.path().extension().is_none_or(|x| x != "asd") {
            paths.push(e.path());
        }
    }
    Ok(())
}
fn validate_relative(p: &Path) -> Result<(), String> {
    if p.as_os_str().is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        Err(format!("Unsafe checkpoint path {}", p.display()))
    } else {
        Ok(())
    }
}
fn safe_existing(root: &Path, p: &Path) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let actual = fs::canonicalize(p).map_err(|e| format!("{}: {e}", p.display()))?;
    if !actual.starts_with(root) {
        return Err(format!("Path escapes episode: {}", p.display()));
    }
    Ok(())
}
pub fn create(
    episode: &Path,
    preset: &str,
    argv: Vec<String>,
    turns: Vec<usize>,
    files: &[PlannedFile],
) -> Result<Checkpoint, String> {
    if preset.is_empty()
        || !preset
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid checkpoint preset".into());
    }
    let root = episode.join("_backups/ui");
    fs::create_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
    safe_existing(episode, &root)?;
    let version = fs::read_dir(&root)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter_map(|e| {
            e.file_name()
                .to_str()?
                .strip_prefix('v')?
                .split('-')
                .next()?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        + 1;
    let time = chrono::Local::now();
    let dir = root.join(format!(
        "v{version:02}-{}-{preset}",
        time.format("%Y%m%d_%H%M%S")
    ));
    fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut cp = Checkpoint {
        version,
        created_at: time.to_rfc3339(),
        preset: preset.into(),
        argv,
        turns,
        files: vec![],
        bytes: 0,
        dir,
    };
    for f in files {
        validate_relative(&f.stored_as)?;
        if !f.path.is_file() {
            return Err(format!("Checkpoint source missing: {}", f.path.display()));
        }
        let dest = cp.dir.join(&f.stored_as);
        fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        cp.bytes += fs::copy(&f.path, &dest)
            .map_err(|e| format!("{} → {}: {e}", f.path.display(), dest.display()))?;
        cp.files.push(f.stored_as.clone());
    }
    let manifest = cp.dir.join("checkpoint.json");
    fs::write(
        &manifest,
        serde_json::to_vec_pretty(&cp).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", manifest.display()))?;
    Ok(cp)
}
pub fn list(episode: &Path) -> Vec<Checkpoint> {
    let mut out: Vec<_> = fs::read_dir(episode.join("_backups/ui"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let mut cp: Checkpoint =
                serde_json::from_slice(&fs::read(entry.path().join("checkpoint.json")).ok()?)
                    .ok()?;
            cp.dir = entry.path();
            if cp
                .files
                .iter()
                .any(|p| validate_relative(p).is_err() || !cp.dir.join(p).is_file())
            {
                return None;
            }
            Some(cp)
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.version));
    out
}
pub fn restore(episode: &Path, cp: &Checkpoint) -> Result<Checkpoint, String> {
    safe_existing(&episode.join("_backups/ui"), &cp.dir)?;
    let mut files = vec![];
    for p in &cp.files {
        validate_relative(p)?;
        safe_existing(&cp.dir, &cp.dir.join(p))?;
        let dest = episode.join(p);
        if dest.exists() {
            safe_existing(episode, &dest)?;
            files.push(PlannedFile {
                path: dest,
                stored_as: p.clone(),
            });
        }
    }
    let before = create(episode, "pre-restore", vec![], cp.turns.clone(), &files)?;
    for (i, p) in cp.files.iter().enumerate() {
        let result = (|| {
            let dest = episode.join(p);
            let parent = dest.parent().unwrap();
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            safe_existing(episode, parent)?;
            let tmp = parent.join(format!(
                ".scriptcast-restore-{}-{}-{i}",
                std::process::id(),
                before.version
            ));
            let _file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)
                .map_err(|e| e.to_string())?;
            fs::copy(cp.dir.join(p), &tmp).map_err(|e| e.to_string())?;
            fs::rename(tmp, dest).map_err(|e| e.to_string())
        })();
        if let Err(e) = result {
            return Err(format!(
                "Restore {} failed: {e}. Prior state: {}",
                p.display(),
                before.dir.display()
            ));
        }
    }
    Ok(before)
}
pub fn total_bytes(episode: &Path) -> u64 {
    fn size(p: &Path) -> u64 {
        fs::read_dir(p)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| match e.file_type() {
                Ok(t) if t.is_dir() => size(&e.path()),
                Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
                _ => 0,
            })
            .sum()
    }
    size(&episode.join("_backups/ui"))
}
