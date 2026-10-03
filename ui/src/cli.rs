//! Shared CLI resolution. No process environment is changed during discovery.
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportSource {
    Folder,
    Path,
    DevCheckout(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ExportCommand {
    pub source: ExportSource,
    pub program: PathBuf,
}

impl ExportCommand {
    pub fn description(&self) -> String {
        match &self.source {
            ExportSource::Folder => format!("CLI folder: {}", self.program.display()),
            ExportSource::Path => format!("PATH: {}", self.program.display()),
            ExportSource::DevCheckout(dir) => format!("Dev checkout: {} (python3)", dir.display()),
        }
    }

    pub fn command(&self, episode_dir: &Path) -> Command {
        let mut command = Command::new(&self.program);
        if let ExportSource::DevCheckout(dir) = &self.source {
            command
                .args(["-m", "scriptcast.tools.export_session"])
                .env("PYTHONPATH", dir)
                // Source checkouts remain read-only, including __pycache__.
                .env("PYTHONDONTWRITEBYTECODE", "1");
        }
        command
            .arg(episode_dir)
            .arg("--fcpxml")
            .current_dir(episode_dir)
            .stdin(Stdio::null());
        command
    }
}

/// The caller owns persisted preferences; an empty folder means unset.
pub fn resolve_export(cli_dir: Option<&Path>) -> Result<ExportCommand, String> {
    resolve_export_with(
        cli_dir,
        env::var_os("PATH").as_deref(),
        env::current_exe().ok().as_deref(),
    )
}

/// Explicit discovery inputs make resolution testable without mutating PATH.
pub fn resolve_export_with(
    cli_dir: Option<&Path>,
    path: Option<&OsStr>,
    current_exe: Option<&Path>,
) -> Result<ExportCommand, String> {
    let cli_dir = cli_dir.filter(|dir| !dir.as_os_str().is_empty());
    if let Some(program) = cli_dir
        .map(|dir| dir.join("scriptcast-export"))
        .filter(|program| is_executable(program))
    {
        return Ok(ExportCommand {
            source: ExportSource::Folder,
            program: absolute(&program),
        });
    }
    if let Some(program) = path.and_then(|path| {
        env::split_paths(path)
            .map(|dir| dir.join("scriptcast-export"))
            .find(|program| is_executable(program))
    }) {
        return Ok(ExportCommand {
            source: ExportSource::Path,
            program: absolute(&program),
        });
    }
    if let Some(dir) = current_exe.and_then(find_dev_checkout) {
        return Ok(ExportCommand {
            source: ExportSource::DevCheckout(dir),
            program: "python3".into(),
        });
    }
    Err(format!(
        "Export command unavailable. Checked CLI folder ({}), PATH for scriptcast-export, and dev-checkout ancestors of {} for pyproject.toml with [project] name = \"scriptcast\".",
        cli_dir.map_or_else(|| "unset".into(), |p| p.display().to_string()),
        current_exe.map_or_else(
            || "unavailable current_exe".into(),
            |p| p.display().to_string()
        ),
    ))
}

pub fn find_dev_checkout(executable: &Path) -> Option<PathBuf> {
    executable.parent()?.ancestors().find_map(|dir| {
        let text = fs::read_to_string(dir.join("pyproject.toml")).ok()?;
        let project: toml::Value = toml::from_str(&text).ok()?;
        (project.get("project")?.get("name")?.as_str()? == "scriptcast").then(|| absolute(dir))
    })
}

fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_owned())
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    true
}
