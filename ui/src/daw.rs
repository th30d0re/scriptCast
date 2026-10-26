//! DAW targets and background work. Only the export CLI writes episode files.
use crate::{cli::ExportCommand, episode::Episode};
use eframe::egui;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
};

pub const NO_EPISODE: &str = "Open an episode first";
pub const NO_LIVE_SET: &str = "No Ableton set (.als) in this episode";

pub fn live_target(episode: &Episode) -> Option<PathBuf> {
    episode
        .als_path
        .as_ref()
        .map(|path| episode.dir.join(path))
        .filter(|path| path.is_file())
        .or_else(|| {
            let path = episode
                .dir
                .join(format!("{}.als", episode.manifest.episode_id));
            path.is_file().then_some(path)
        })
}

pub fn live_command(path: &Path) -> Command {
    open_command("com.ableton.live", path)
}

pub fn logic_command(path: &Path) -> Command {
    open_command("com.apple.logic10", path)
}

fn open_command(bundle: &str, path: &Path) -> Command {
    let mut command = Command::new("/usr/bin/open");
    command
        .args(["-b", bundle])
        .arg(path)
        .env("PATH", crate::cli::login_path())
        .stdin(Stdio::null());
    command
}

pub fn fcpxml_path(episode: &Episode) -> PathBuf {
    episode
        .dir
        .join(format!("{}.fcpxml", episode.manifest.episode_id))
}

pub fn fcpxml_is_fresh(episode: &Episode) -> bool {
    is_fresh(&fcpxml_path(episode), &episode.dir)
}

fn is_fresh(path: &Path, dir: &Path) -> bool {
    let modified = |path: &Path| {
        let metadata = fs::metadata(path).ok()?;
        metadata
            .is_file()
            .then(|| metadata.modified().ok())
            .flatten()
    };
    match (modified(path), modified(&dir.join("episode_manifest.json"))) {
        (Some(xml), Some(manifest)) => xml >= manifest,
        _ => false,
    }
}

pub fn live_disabled_reason(episode: Option<&Episode>) -> Option<String> {
    match episode {
        None => Some(NO_EPISODE.into()),
        Some(episode) if live_target(episode).is_none() => Some(NO_LIVE_SET.into()),
        Some(_) => None,
    }
}

pub fn logic_disabled_reason(
    episode: Option<&Episode>,
    export: &Result<ExportCommand, String>,
) -> Option<String> {
    match episode {
        None => Some(NO_EPISODE.into()),
        Some(episode) if fcpxml_is_fresh(episode) => None,
        Some(_) => export.as_ref().err().cloned(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DawEvent {
    Progress(String),
    Finished(Result<String, String>),
}

pub struct DawTask {
    pub receiver: Receiver<DawEvent>,
}

impl DawTask {
    pub fn live(path: PathBuf, ctx: egui::Context) -> Self {
        Self::spawn(ctx, move |_| {
            run(&mut live_command(&path), "Open in Live")?;
            Ok(format!("Opening {} in Live", filename(&path)))
        })
    }

    /// The injected opener constructs a process; tests supply a no-op command.
    /// All freshness checks, exporting and process waits happen on the worker.
    pub fn logic(
        episode: &Episode,
        export: Result<ExportCommand, String>,
        opener: impl FnOnce(&Path) -> Command + Send + 'static,
        ctx: egui::Context,
    ) -> Self {
        let dir = episode.dir.clone();
        let path = fcpxml_path(episode);
        Self::spawn(ctx, move |progress| {
            if !is_fresh(&path, &dir) {
                let export = export?;
                progress(format!("Writing {}…", filename(&path)));
                run(&mut export.command(&dir), "FCPXML export")?;
                if !is_fresh(&path, &dir) {
                    return Err(format!(
                        "FCPXML export did not create a fresh {}",
                        path.display()
                    ));
                }
            }
            run(&mut opener(&path), "Open in Logic")?;
            Ok("Opened in Logic".into())
        })
    }

    fn spawn(
        ctx: egui::Context,
        work: impl FnOnce(&dyn Fn(String)) -> Result<String, String> + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let progress = |message| {
                let _ = sender.send(DawEvent::Progress(message));
                ctx.request_repaint();
            };
            let result = work(&progress);
            let _ = sender.send(DawEvent::Finished(result));
            ctx.request_repaint();
        });
        Self { receiver }
    }
}

pub fn filename(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn run(command: &mut Command, action: &str) -> Result<(), String> {
    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("{action}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<_> = stderr.lines().collect();
    let tail = lines[lines.len().saturating_sub(6)..].join("\n");
    Err(format!(
        "{action} failed ({}): {}",
        output.status,
        tail.trim()
    ))
}
