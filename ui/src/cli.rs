//! Per-command discovery and the once-per-launch login-shell environment.
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock, mpsc},
    time::{Duration, Instant, SystemTime},
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
#[derive(Clone, Debug)]
pub struct Skip {
    pub program: PathBuf,
    pub reason: String,
}

/// Flags the app passes to `command`. Empty means "take the first executable",
/// which the resolution tests rely on.
pub fn required_flags(command: &str) -> &'static [&'static str] {
    match command {
        "scriptcast" => &["--progress-jsonl", "--resume", "--export-only"],
        "scriptcast-export" => &["--edit-json", "--fcpxml"],
        "scriptcast-verify"
        | "scriptcast-repair"
        | "scriptcast-relayout"
        | "scriptcast-stitch"
        | "scriptcast-turn-index" => &["--progress-jsonl"],
        _ => &[],
    }
}

pub fn resolve_with_flags(
    command: &str,
    project: Option<&Path>,
    cli_dir: Option<&Path>,
    path: &OsStr,
    exe: Option<&Path>,
    flags: &[&str],
) -> Result<(Resolved, Vec<Skip>), String> {
    let mut skipped = Vec::new();
    let accept = |program: PathBuf,
                  source: ExportSource,
                  prefix: Vec<OsString>,
                  pythonpath: Option<PathBuf>,
                  skipped: &mut Vec<Skip>| {
        capable(program, source, prefix, pythonpath, path, flags, skipped)
    };
    // Clip-transcribe and verify-ensemble need fermion + mlx_whisper.
    let needs_clip_engines = matches!(
        command,
        "scriptcast-clip-transcribe" | "scriptcast-verify-ensemble"
    );
    if let Some(p) = cli_dir
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.join(command))
        .filter(|p| is_executable(p))
        && let Some(resolved) = accept(p, ExportSource::Folder, vec![], None, &mut skipped)
    {
        return Ok((resolved, skipped));
    }
    let venvs = project.map(venvs).unwrap_or_default();
    for venv in &venvs {
        let p = venv.join("bin").join(command);
        if !is_executable(&p) {
            continue;
        }
        // These console scripts are only usable when the venv has the Phonon
        // stack and mlx-whisper; entry-module import is not enough.
        if needs_clip_engines && !probe_clip_engines(&venv.join("bin/python"), None, path) {
            continue;
        }
        if let Some(resolved) = accept(p, ExportSource::ProjectVenv, vec![], None, &mut skipped) {
            return Ok((resolved, skipped));
        }
    }
    for p in env::split_paths(path)
        .map(|p| p.join(command))
        .filter(|p| is_executable(p))
    {
        if let Some(resolved) = accept(p, ExportSource::Path, vec![], None, &mut skipped) {
            return Ok((resolved, skipped));
        }
    }
    if let Some(dir) = exe.and_then(find_dev_checkout) {
        let module = module_for(&dir, command)?;
        let mut ordered: Vec<&PathBuf> = venvs.iter().collect();
        // Prefer interpreters that import fermion + mlx_whisper + engines.
        // Other commands still prefer an installed scriptcast first.
        if needs_clip_engines {
            ordered.sort_by_key(|v| !probe_clip_engines(&v.join("bin/python"), Some(&dir), path));
        } else {
            // Import the entry module (not just scriptcast) to check its
            // dependencies. Venvs with an installed scriptcast come first:
            // their interpreter has the full dependency set (e.g. misaki).
            ordered.sort_by_key(|v| !is_executable(&v.join("bin/scriptcast")));
        }
        let program = ordered
            .into_iter()
            .map(|p| p.join("bin/python"))
            .find(|p| {
                is_executable(p)
                    && if needs_clip_engines {
                        probe_clip_engines(p, Some(&dir), path)
                    } else {
                        probe_module(p, &module, &dir, path)
                    }
            })
            .unwrap_or_else(|| {
                env::split_paths(path)
                    .map(|p| p.join("python3"))
                    .find(|p| is_executable(p))
                    .unwrap_or_else(|| "python3".into())
            });
        let prefix = vec!["-m".into(), module.into()];
        if let Some(resolved) = accept(
            program,
            ExportSource::DevCheckout(dir.clone()),
            prefix,
            Some(dir),
            &mut skipped,
        ) {
            return Ok((resolved, skipped));
        }
    }
    let why = skipped
        .iter()
        .map(|s| s.reason.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let mut message = format!(
        "{command} unavailable. Checked CLI folder {}, project .venv*/bin, login PATH, \
         and dev-checkout pyproject.toml ancestors of {}",
        cli_dir
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unset".into()),
        exe.map(|p| p.display().to_string()).unwrap_or_default()
    );
    if !why.is_empty() {
        message.push_str(". ");
        message.push_str(&why);
    }
    Err(message)
}

pub fn resolve_with(
    command: &str,
    project: Option<&Path>,
    cli_dir: Option<&Path>,
    path: &OsStr,
    exe: Option<&Path>,
) -> Result<Resolved, String> {
    resolve_with_flags(command, project, cli_dir, path, exe, &[]).map(|(r, _)| r)
}

fn capable(
    program: PathBuf,
    source: ExportSource,
    prefix: Vec<OsString>,
    pythonpath: Option<PathBuf>,
    path: &OsStr,
    flags: &[&str],
    skipped: &mut Vec<Skip>,
) -> Option<Resolved> {
    let program = if program.components().nth(1).is_some() {
        absolute(&program)
    } else {
        program
    };
    if !flags.is_empty() {
        let help = command_help(&program, &prefix, pythonpath.as_deref(), path).unwrap_or_default();
        let missing: Vec<String> = flags
            .iter()
            .copied()
            .filter(|flag| !help_has_flag(&help, flag))
            .map(str::to_owned)
            .collect();
        if !missing.is_empty() {
            skipped.push(Skip {
                reason: stale_reason(&program, &missing),
                program: program.clone(),
            });
            return None;
        }
    }
    Some(Resolved {
        source,
        program,
        prefix,
        path: path.into(),
        pythonpath,
    })
}

fn help_has_flag(help: &str, flag: &str) -> bool {
    help.match_indices(flag).any(|(index, _)| {
        help[index + flag.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-')
    })
}

struct HelpHit {
    mtime: Option<SystemTime>,
    text: String,
}
fn help_cache() -> &'static Mutex<HashMap<String, HelpHit>> {
    static CACHE: OnceLock<Mutex<HashMap<String, HelpHit>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
fn command_help(
    program: &Path,
    prefix: &[OsString],
    pythonpath: Option<&Path>,
    path: &OsStr,
) -> Result<String, String> {
    let key = format!(
        "{} {} {}",
        program.display(),
        prefix
            .iter()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        pythonpath
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
    let mtime = fs::metadata(program).ok().and_then(|m| m.modified().ok());
    let cache = help_cache();
    let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(hit) = guard.get(&key)
        && hit.mtime == mtime
    {
        return Ok(hit.text.clone());
    }
    drop(guard);
    let mut command = Command::new(program);
    command
        .args(prefix)
        .arg("--help")
        .env("PATH", path)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = pythonpath {
        command.env("PYTHONPATH", dir);
    }
    let text = help_output(&mut command, Duration::from_secs(4))?;
    help_cache()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(
            key,
            HelpHit {
                mtime,
                text: text.clone(),
            },
        );
    Ok(text)
}
fn help_output(command: &mut Command, timeout: Duration) -> Result<String, String> {
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let _ = stdout.read_to_end(&mut out);
        let _ = stderr.read_to_end(&mut err);
        out.extend(err);
        let _ = tx.send(out);
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Err(e) => return Err(e.to_string()),
            Ok(None) if start.elapsed() > timeout => {
                kill_group(&mut child);
                let _ = child.wait();
                return Err("help timed out".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    let bytes = rx.recv_timeout(Duration::from_secs(1)).unwrap_or_default();
    if !status.success() && bytes.is_empty() {
        return Err(format!("help exited {status}"));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
fn stale_reason(program: &Path, missing: &[String]) -> String {
    let flags = missing.join(", ");
    match install_origin(program) {
        Some(origin) => format!(
            "{} is an older scriptCast (from {origin}) without {flags}",
            short_install(program)
        ),
        None => format!(
            "{} is an older scriptCast without {flags}",
            short_install(program)
        ),
    }
}
fn short_install(program: &Path) -> String {
    let parts: Vec<_> = program.components().collect();
    parts
        .iter()
        .position(|c| c.as_os_str().to_string_lossy().starts_with(".venv"))
        .map(|index| {
            parts[index..]
                .iter()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| program.display().to_string())
}
fn install_origin(program: &Path) -> Option<String> {
    let text = fs::read_to_string(program).ok()?;
    text.split_whitespace().find_map(|token| {
        let token = token.trim_matches(|c: char| c == '\'' || c == '"' || c == '#');
        (token.starts_with('/')
            && (token.contains("scriptCast") || token.contains("scriptcast"))
            && !token.contains(".venv"))
        .then(|| token.to_owned())
    })
}
pub fn stale_install_warning(skipped: &[Skip], using_dev_checkout: bool) -> Option<String> {
    if !using_dev_checkout {
        return None;
    }
    let name = skipped.iter().find_map(|skip| {
        skip.program.components().find_map(|c| {
            let name = c.as_os_str().to_string_lossy();
            name.starts_with(".venv").then(|| name.into_owned())
        })
    })?;
    Some(format!(
        "Skipped an older scriptCast install in {name}. Using the dev checkout."
    ))
}
fn probe_module(p: &Path, module: &str, dir: &Path, path: &OsStr) -> bool {
    probe_python(
        p,
        &format!("import importlib; importlib.import_module({module:?})"),
        Some(dir),
        path,
        Duration::from_secs(2),
    )
}

/// Clip-transcribe needs these imports at resolve time; engines load lazily.
/// Measured cold import of fermion+mlx_whisper is about 2.8 s on this machine.
fn probe_clip_engines(python: &Path, pythonpath: Option<&Path>, path: &OsStr) -> bool {
    if !is_executable(python) {
        return false;
    }
    probe_python(
        python,
        "import fermion, mlx_whisper, scriptcast.asr.engines",
        pythonpath,
        path,
        Duration::from_secs(6),
    )
}

fn probe_python(
    p: &Path,
    code: &str,
    pythonpath: Option<&Path>,
    path: &OsStr,
    timeout: Duration,
) -> bool {
    let mut c = Command::new(p);
    c.args(["-c", code])
        .env("PATH", path)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(dir) = pythonpath {
        c.env("PYTHONPATH", dir);
    }
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
        if start.elapsed() > timeout {
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
