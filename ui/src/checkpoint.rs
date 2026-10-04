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
    #[serde(default = "audio_kind")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_files: Vec<PlannedFile>,
    pub bytes: u64,
    #[serde(skip)]
    pub dir: PathBuf,
}
/// Separates source/destination from storage, allowing Phase D external plans.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlannedFile {
    pub path: PathBuf,
    pub stored_as: PathBuf,
}
fn audio_kind() -> String {
    "audio".into()
}

/// Absolute destinations are allowed only through this explicit checkpoint kind.
pub fn create_external(
    episode: &Path,
    preset: &str,
    files: &[PathBuf],
    turns: Vec<usize>,
) -> Result<Checkpoint, String> {
    let files = files
        .iter()
        .map(|path| {
            if !path.is_absolute() {
                return Err(format!(
                    "External path must be absolute: {}",
                    path.display()
                ));
            }
            let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
            let name = path.file_name().ok_or("External file has no name")?;
            let stored_as = Path::new("external").join(name);
            Ok(PlannedFile { path, stored_as })
        })
        .collect::<Result<Vec<_>, String>>()?;
    create_impl(episode, preset, vec![], turns, &files, true)
}

pub fn plan_files(p: &Preset, e: &Episode) -> Result<Vec<PlannedFile>, String> {
    let mut paths = vec![];
    if matches!(p, Preset::Verify) {
        paths.push(e.dir.join("stress.json"));
    } else if matches!(p, Preset::ReadAlong) {
        paths.push(e.dir.join("word_timings.json"));
    } else if matches!(p, Preset::ClipTranscripts) {
        paths.push(e.dir.join("clip_transcripts.json"));
    } else if matches!(p, Preset::SecondOpinion) {
        paths.push(e.dir.join("stress_ensemble.json"));
    } else if !matches!(p, Preset::DetectChanges | Preset::ReviewStats) {
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
    create_impl(episode, preset, argv, turns, files, false)
}

fn create_impl(
    episode: &Path,
    preset: &str,
    argv: Vec<String>,
    turns: Vec<usize>,
    files: &[PlannedFile],
    external: bool,
) -> Result<Checkpoint, String> {
    let mut stored = std::collections::HashSet::new();
    for f in files {
        validate_relative(&f.stored_as)?;
        if f.stored_as == Path::new("checkpoint.json") || !stored.insert(&f.stored_as) {
            return Err("Checkpoint storage paths must be unique and not checkpoint.json".into());
        }
        if external {
            validate_external(f)?;
        } else {
            safe_existing(episode, &f.path)?;
            if fs::canonicalize(&f.path).map_err(|e| e.to_string())?
                != fs::canonicalize(episode.join(&f.stored_as)).map_err(|e| e.to_string())?
            {
                return Err("Checkpoint storage path does not match its episode source".into());
            }
        }
    }
    if preset.is_empty()
        || !preset
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid checkpoint preset".into());
    }
    let root = episode.join("_backups/ui");
    let ancestor = root
        .ancestors()
        .find(|p| p.exists())
        .ok_or("Missing episode directory")?;
    safe_existing(episode, ancestor)?;
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
        kind: if external { "script-edit" } else { "audio" }.into(),
        external_files: if external { files.to_vec() } else { vec![] },
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
            if checkpoint_files(episode, &cp).is_err() {
                return None;
            }
            Some(cp)
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.version));
    out
}
fn validate_external(f: &PlannedFile) -> Result<(), String> {
    if !f.path.is_absolute()
        || f.path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        || f.stored_as != Path::new("external").join(f.path.file_name().ok_or("Missing filename")?)
    {
        return Err("Invalid external checkpoint mapping".into());
    }
    let parent = f.path.parent().ok_or("Missing external parent")?;
    if fs::canonicalize(parent).map_err(|e| e.to_string())? != parent {
        return Err("External checkpoint parent changed or contains a symlink".into());
    }
    if let Ok(meta) = fs::symlink_metadata(&f.path)
        && (!meta.is_file() || meta.file_type().is_symlink())
    {
        return Err("External checkpoint destination must be a regular file".into());
    }
    Ok(())
}

fn checkpoint_files(episode: &Path, cp: &Checkpoint) -> Result<Vec<PlannedFile>, String> {
    safe_existing(&episode.join("_backups/ui"), &cp.dir)?;
    let external = match cp.kind.as_str() {
        "audio" if cp.external_files.is_empty() => false,
        "script-edit" if cp.external_files.len() == cp.files.len() => true,
        _ => return Err("Invalid checkpoint kind or external mappings".into()),
    };
    let mut seen = std::collections::HashSet::new();
    cp.files
        .iter()
        .enumerate()
        .map(|(i, stored_as)| {
            validate_relative(stored_as)?;
            if stored_as == Path::new("checkpoint.json") || !seen.insert(stored_as) {
                return Err("Invalid checkpoint storage paths".into());
            }
            safe_existing(&cp.dir, &cp.dir.join(stored_as))?;
            if !cp.dir.join(stored_as).is_file() {
                return Err("Checkpoint source is not a file".into());
            }
            if external {
                let f = &cp.external_files[i];
                if &f.stored_as != stored_as {
                    return Err("External checkpoint mapping mismatch".into());
                }
                validate_external(f)?;
                Ok(f.clone())
            } else {
                let path = episode.join(stored_as);
                // Check the nearest existing ancestor before creating any directories.
                let ancestor = path
                    .ancestors()
                    .find(|p| p.exists())
                    .ok_or("Missing destination")?;
                safe_existing(episode, ancestor)?;
                Ok(PlannedFile {
                    path,
                    stored_as: stored_as.clone(),
                })
            }
        })
        .collect()
}

pub fn restore(episode: &Path, cp: &Checkpoint) -> Result<Checkpoint, String> {
    let files = checkpoint_files(episode, cp)?;
    let existing: Vec<_> = files.iter().filter(|f| f.path.exists()).cloned().collect();
    let before = create_impl(
        episode,
        "pre-restore",
        vec![],
        cp.turns.clone(),
        &existing,
        cp.kind == "script-edit",
    )?;
    for (i, f) in files.iter().enumerate() {
        let result = (|| {
            let parent = f.path.parent().ok_or("Missing destination parent")?;
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            if cp.kind == "script-edit" {
                validate_external(f)?;
            } else {
                safe_existing(episode, parent)?;
            }
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
            let result = fs::copy(cp.dir.join(&f.stored_as), &tmp)
                .and_then(|_| fs::rename(&tmp, &f.path))
                .map_err(|e| e.to_string());
            if result.is_err() {
                let _ = fs::remove_file(&tmp);
            }
            result
        })();
        if let Err(e) = result {
            return Err(format!(
                "Restore {} failed: {e}. Prior state: {}",
                f.path.display(),
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
