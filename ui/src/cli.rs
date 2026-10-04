//! Per-command discovery and the once-per-launch login-shell environment.
use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{OnceLock, mpsc},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct LoginPath {
    pub path: OsString,
    pub note: String,
}
static LOGIN: OnceLock<LoginPath> = OnceLock::new();
pub fn login_path() -> OsString {
    LOGIN
        .get()
        .map(|p| p.path.clone())
        .unwrap_or_else(|| env::var_os("PATH").unwrap_or_default())
}
pub fn start_login_path() -> mpsc::Receiver<LoginPath> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let value = read_login_path(
            env::var_os("SHELL").as_deref(),
            &env::var_os("PATH").unwrap_or_default(),
            Duration::from_secs(2),
        );
        let _ = LOGIN.set(value.clone());
        let _ = tx.send(value);
    });
    rx
}
pub fn read_login_path(shell: Option<&OsStr>, fallback: &OsStr, timeout: Duration) -> LoginPath {
    let fail = |note: String| LoginPath {
        path: fallback.into(),
        note,
    };
    let Some(shell) = shell else {
        return fail("SHELL unset; process PATH".into());
    };
    let mut command = Command::new(shell);
    command
        .args(["-lc", "printf %s \"$PATH\""])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let Ok(mut child) = command.spawn() else {
        return fail("Login shell unavailable; process PATH".into());
    };
    let mut stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        let _ = tx.send(bytes);
    });
    let start = Instant::now();
    loop {
        if start.elapsed() >= timeout {
            kill_group(&mut child);
            let _ = child.wait();
            return fail("Login shell timed out; process PATH".into());
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => return fail("Login shell failed; process PATH".into()),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    let Ok(bytes) = rx.recv_timeout(timeout.saturating_sub(start.elapsed())) else {
        return fail("Login shell output timed out; process PATH".into());
    };
    let output = String::from_utf8_lossy(&bytes);
    let last = output.lines().last().unwrap_or("").trim();
    if last.is_empty() {
        return fail("Empty login PATH; process PATH".into());
    }
    LoginPath {
        path: last.into(),
        note: if output.lines().count() > 1 {
            "Login shell printed noise; used last line"
        } else {
            "Login shell PATH loaded"
        }
        .into(),
    }
}
pub(crate) fn kill_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportSource {
    Folder,
    ProjectVenv,
    Path,
    DevCheckout(PathBuf),
}
#[derive(Clone, Debug)]
pub struct Resolved {
    pub source: ExportSource,
    pub program: PathBuf,
    pub prefix: Vec<OsString>,
    pub path: OsString,
    pub pythonpath: Option<PathBuf>,
}
impl Resolved {
    pub fn description(&self) -> String {
        format!(
            "{}: {}{}",
            match &self.source {
                ExportSource::Folder => "CLI folder",
                ExportSource::ProjectVenv => "Project virtualenv",
                ExportSource::Path => "PATH",
                ExportSource::DevCheckout(_) => "Dev checkout",
            },
            self.program.display(),
            self.pythonpath
                .as_ref()
                .map(|p| format!(
                    " ({}; {})",
                    p.display(),
                    self.prefix
                        .iter()
                        .map(|a| a.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(" ")
                ))
                .unwrap_or_default()
        )
    }
    pub fn base_command(&self) -> Command {
        let mut c = Command::new(&self.program);
        c.args(&self.prefix)
            .env("PATH", &self.path)
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONUNBUFFERED", "1");
        if let Some(p) = &self.pythonpath {
            c.env("PYTHONPATH", p);
        }
        c.stdin(Stdio::null());
        c
    }
}
/// B2 compatibility wrapper. Discovery and all actual app commands use Resolved.
#[derive(Clone, Debug)]
pub struct ExportCommand {
    pub source: ExportSource,
    pub program: PathBuf,
}
impl ExportCommand {
    pub fn description(&self) -> String {
        format!(
            "{}: {}",
            match self.source {
                ExportSource::Folder => "CLI folder",
                ExportSource::ProjectVenv => "Project virtualenv",
                ExportSource::Path => "PATH",
                ExportSource::DevCheckout(_) => "Dev checkout",
            },
            self.program.display()
        )
    }
    pub fn command(&self, episode_dir: &Path) -> Command {
        let module = if let ExportSource::DevCheckout(dir) = &self.source {
            module_for(dir, "scriptcast-export").ok()
        } else {
            None
        };
        let resolved = Resolved {
            source: self.source.clone(),
            program: self.program.clone(),
            prefix: module
                .map(|m| vec!["-m".into(), m.into()])
                .unwrap_or_default(),
            path: login_path(),
            pythonpath: if let ExportSource::DevCheckout(p) = &self.source {
                Some(p.clone())
            } else {
                None
            },
        };
        let mut c = resolved.base_command();
        c.arg(episode_dir).arg("--fcpxml").current_dir(episode_dir);
        c
    }
}
pub fn resolve_export(cli_dir: Option<&Path>) -> Result<ExportCommand, String> {
    resolve_export_with(
        cli_dir,
        Some(&login_path()),
        env::current_exe().ok().as_deref(),
    )
}
pub fn resolve_export_with(
    cli_dir: Option<&Path>,
    path: Option<&OsStr>,
    current_exe: Option<&Path>,
) -> Result<ExportCommand, String> {
    resolve_with(
        "scriptcast-export",
        None,
        cli_dir,
        path.unwrap_or(OsStr::new("")),
        current_exe,
    )
    .map(|r| ExportCommand {
        source: r.source,
        program: r.program,
    })
}
pub fn resolve(
    command: &str,
    project: Option<&Path>,
    cli_dir: Option<&Path>,
) -> Result<Resolved, String> {
    resolve_with(
        command,
        project,
        cli_dir,
        &login_path(),
        env::current_exe().ok().as_deref(),
    )
}
pub fn resolve_with(
    command: &str,
    project: Option<&Path>,
    cli_dir: Option<&Path>,
    path: &OsStr,
    exe: Option<&Path>,
) -> Result<Resolved, String> {
    let direct = |program: PathBuf, source| Resolved {
        source,
        program: absolute(&program),
        prefix: vec![],
        path: path.into(),
        pythonpath: None,
    };
    if let Some(p) = cli_dir
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.join(command))
        .filter(|p| is_executable(p))
    {
        return Ok(direct(p, ExportSource::Folder));
    }
    let venvs = project.map(venvs).unwrap_or_default();
    for venv in &venvs {
        let p = venv.join("bin").join(command);
        if is_executable(&p) {
            return Ok(direct(p, ExportSource::ProjectVenv));
        }
    }
    if let Some(p) = env::split_paths(path)
        .map(|p| p.join(command))
        .find(|p| is_executable(p))
    {
        return Ok(direct(p, ExportSource::Path));
    }
    if let Some(dir) = exe.and_then(find_dev_checkout) {
        let module = module_for(&dir, command)?;
        // Import the entry module (not just scriptcast) to check its dependencies.
        // Venvs with an installed scriptcast come first: their interpreter has the
        // full dependency set (e.g. misaki), which later lazy imports need.
        let mut ordered: Vec<&PathBuf> = venvs.iter().collect();
        ordered.sort_by_key(|v| !is_executable(&v.join("bin/scriptcast")));
        let program = ordered
            .into_iter()
            .map(|p| p.join("bin/python"))
            .find(|p| is_executable(p) && probe_module(p, &module, &dir, path))
            .unwrap_or_else(|| {
                env::split_paths(path)
                    .map(|p| p.join("python3"))
                    .find(|p| is_executable(p))
                    .unwrap_or_else(|| "python3".into())
            });
        return Ok(Resolved {
            source: ExportSource::DevCheckout(dir.clone()),
            program,
            prefix: vec!["-m".into(), module.into()],
            path: path.into(),
            pythonpath: Some(dir),
        });
    }
    Err(format!(
        "{command} unavailable. Checked CLI folder {}, project .venv*/bin, login PATH, \
         and dev-checkout pyproject.toml ancestors of {}",
        cli_dir
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unset".into()),
        exe.map(|p| p.display().to_string()).unwrap_or_default()
    ))
}
fn probe_module(p: &Path, module: &str, dir: &Path, path: &OsStr) -> bool {
    let mut c = Command::new(p);
    c.args([
        "-c",
        &format!("import importlib; importlib.import_module({module:?})"),
    ])
    .env("PYTHONPATH", dir)
    .env("PATH", path)
    .env("PYTHONDONTWRITEBYTECODE", "1")
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
    let Ok(mut child) = c.spawn() else {
        return false;
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(s)) => return s.success(),
            Err(_) => return false,
            _ => {}
        }
        if start.elapsed() > Duration::from_secs(2) {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn venvs(project: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(project)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(".venv") && e.path().is_dir())
        .map(|e| e.path())
        .collect();
    paths.sort();
    paths
}
fn module_for(dir: &Path, command: &str) -> Result<String, String> {
    let file = dir.join("pyproject.toml");
    let parsed: toml::Value =
        toml::from_str(&fs::read_to_string(&file).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    parsed
        .get("project")
        .and_then(|p| p.get("scripts"))
        .and_then(|s| s.get(command))
        .and_then(|v| v.as_str())
        .and_then(|s| s.split(':').next())
        .map(str::to_owned)
        .ok_or_else(|| format!("{} missing [project.scripts].{command}", file.display()))
}
pub fn find_dev_checkout(executable: &Path) -> Option<PathBuf> {
    executable.parent()?.ancestors().find_map(|dir| {
        let p: toml::Value =
            toml::from_str(&fs::read_to_string(dir.join("pyproject.toml")).ok()?).ok()?;
        (p.get("project")?.get("name")?.as_str()? == "scriptcast").then(|| absolute(dir))
    })
}
fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.into())
}
fn is_executable(p: &Path) -> bool {
    let Ok(m) = fs::metadata(p) else {
        return false;
    };
    if !m.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        m.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    true
}
