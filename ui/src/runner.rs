//! Fixed argv presets and a single cancellable subprocess with ordered completion.
use crate::{
    cli::{self, Resolved},
    episode::Episode,
};
use eframe::egui;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{ErrorKind, Read},
    path::PathBuf,
    process::{Child, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Preset {
    Render,
    Relayout(u64),
    Stitch,
    TurnIndex,
    ReExport { fcpxml: bool, logic_dry_run: bool },
    DetectChanges,
    PrecisionInsert,
    RegenerateTurns(Vec<usize>),
    Verify,
    ReadAlong,
    ClipTranscripts,
    SecondOpinion,
    ReviewStats,
    Repair(Option<Vec<usize>>),
}
impl Preset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Render => "render",
            Self::Relayout(_) => "relayout",
            Self::Stitch => "stitch",
            Self::TurnIndex => "turn-index",
            Self::ReExport { .. } => "re-export",
            Self::DetectChanges => "detect-changes",
            Self::PrecisionInsert => "precision-insert",
            Self::RegenerateTurns(_) => "regenerate-turns",
            Self::Verify => "verify",
            Self::ReadAlong => "read-along-timings",
            Self::ClipTranscripts => "clip-transcripts",
            Self::SecondOpinion => "second-opinion",
            Self::ReviewStats => "review-stats",
            Self::Repair(_) => "repair",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Render => "Render",
            Self::Relayout(_) => "Relayout",
            Self::Stitch => "Stitch MP3",
            Self::TurnIndex => "Turn index",
            Self::ReExport { .. } => "Re-export",
            Self::DetectChanges => "Detect changes",
            Self::PrecisionInsert => "Precision insert",
            Self::RegenerateTurns(_) => "Regenerate selected",
            Self::Verify => "Verify",
            Self::ReadAlong => "Read-along timings",
            Self::ClipTranscripts => "Clip transcripts",
            Self::SecondOpinion => "Second opinion (failed turns)",
            Self::ReviewStats => "Review stats",
            Self::Repair(_) => "Repair",
        }
    }
    pub fn command(&self) -> &'static str {
        match self {
            Self::Relayout(_) => "scriptcast-relayout",
            Self::Stitch => "scriptcast-stitch",
            Self::TurnIndex => "scriptcast-turn-index",
            Self::Verify => "scriptcast-verify",
            Self::ReadAlong => "scriptcast-word-timings",
            Self::ClipTranscripts => "scriptcast-clip-transcribe",
            Self::SecondOpinion => "scriptcast-verify-ensemble",
            Self::ReviewStats => "scriptcast-review-stats",
            Self::Repair(_) => "scriptcast-repair",
            _ => "scriptcast",
        }
    }
    pub fn turns(&self) -> Vec<usize> {
        match self {
            Self::RegenerateTurns(t) | Self::Repair(Some(t)) => t.clone(),
            _ => vec![],
        }
    }
    pub fn parse(name: &str, turns: Vec<usize>) -> Result<Self, String> {
        match name {
            "render" => Ok(Self::Render),
            "relayout" => Ok(Self::Relayout(250)),
            "stitch" => Ok(Self::Stitch),
            "turn-index" => Ok(Self::TurnIndex),
            "re-export" => Ok(Self::ReExport {
                fcpxml: false,
                logic_dry_run: false,
            }),
            "detect-changes" => Ok(Self::DetectChanges),
            "precision-insert" => Ok(Self::PrecisionInsert),
            "regenerate-turns" | "regenerate-selected" => Ok(Self::RegenerateTurns(turns)),
            "verify" => Ok(Self::Verify),
            "read-along-timings" => Ok(Self::ReadAlong),
            "clip-transcripts" => Ok(Self::ClipTranscripts),
            "second-opinion" => Ok(Self::SecondOpinion),
            "review-stats" => Ok(Self::ReviewStats),
            "repair" => Ok(Self::Repair((!turns.is_empty()).then_some(turns))),
            _ => Err(format!("Unknown preset {name}")),
        }
    }
    pub fn safe_run(name: &str) -> Result<Self, String> {
        match name {
            "detect-changes" => Ok(Self::DetectChanges),
            "verify" => Ok(Self::Verify),
            "read-along-timings" => Ok(Self::ReadAlong),
            "clip-transcripts" => Ok(Self::ClipTranscripts),
            "second-opinion" => Ok(Self::SecondOpinion),
            "review-stats" => Ok(Self::ReviewStats),
            _ => Err(
                "--run refuses synthesizing presets; use detect-changes, verify, \
                 read-along-timings, clip-transcripts, second-opinion or review-stats"
                    .into(),
            ),
        }
    }
}
pub type Commands = BTreeMap<String, Result<Resolved, String>>;
pub struct Discovery {
    pub commands: Commands,
    pub skipped: Vec<crate::cli::Skip>,
}
pub fn preset_label(name: &str) -> String {
    if name == "regenerate-edited" {
        return "Regenerate edited turns".into();
    }
    Preset::parse(name, vec![])
        .map(|preset| preset.label().to_owned())
        .unwrap_or_else(|_| name.to_owned())
}
#[derive(Clone, Debug)]
pub struct JobSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub synthesizes: bool,
    pub overwrites: Vec<PathBuf>,
    pub resolved: Resolved,
}
impl JobSpec {
    pub fn from_preset(p: &Preset, e: &Episode, commands: &Commands) -> Result<Self, String> {
        let cwd = e
            .project_root
            .clone()
            .ok_or("Missing scriptcast.toml project root")?;
        let script = e.script_path.as_ref().filter(|p| p.is_file());
        let utility = matches!(
            p,
            Preset::Relayout(_) | Preset::Stitch | Preset::ReExport { .. }
        );
        if !utility && script.is_none() {
            return Err(
                "Missing transcript: source_file in render_state.json / episode_manifest.json"
                    .into(),
            );
        }
        let script = script.unwrap_or(&cwd);
        let turns = p.turns();
        if matches!(p, Preset::RegenerateTurns(_) | Preset::Repair(Some(_))) {
            if e.script_in_sync != Some(true) {
                return Err(
                    "Script changed since render: run Detect changes or Precision insert first"
                        .into(),
                );
            }
            if turns.is_empty() {
                return Err("Select at least one turn".into());
            }
            if turns.iter().any(|n| !e.turns.iter().any(|t| t.index == *n)) {
                return Err("Selected turn is absent from the manifest".into());
            }
        }
        let resolved = commands
            .get(p.command())
            .ok_or_else(|| format!("{} unresolved", p.command()))?
            .clone()?;
        let mut args = resolved.prefix.clone();
        let out = e.dir.parent().ok_or("Episode has no output parent")?;
        let mut synthesizes = false;
        match p {
            Preset::ReExport {
                fcpxml,
                logic_dry_run,
            } => {
                args.extend([
                    "--export-only".into(),
                    "--episode-id".into(),
                    e.manifest.episode_id.clone().into(),
                    "--out-dir".into(),
                    out.as_os_str().into(),
                ]);
                if *fcpxml {
                    args.push("--fcpxml".into());
                }
                if *logic_dry_run {
                    args.push("--logic-dry-run".into());
                }
            }

            Preset::Relayout(gap) => {
                args.extend([
                    e.dir.as_os_str().into(),
                    "--gap-ms".into(),
                    gap.to_string().into(),
                ]);
            }
            Preset::Stitch => args.push(e.dir.as_os_str().into()),
            Preset::TurnIndex => args.extend([
                e.dir.as_os_str().into(),
                "--transcript".into(),
                script.as_os_str().into(),
            ]),
            Preset::ReadAlong | Preset::ClipTranscripts | Preset::SecondOpinion => {
                args.extend([
                    e.dir.clone().into_os_string(),
                    "--script".into(),
                    script.as_os_str().into(),
                ]);
            }
            Preset::ReviewStats => {
                let voices = e
                    .voices_path
                    .as_ref()
                    .ok_or("Missing voices path for review stats")?;
                let project_dir = crate::review::project_dir_from_voices(voices)
                    .ok_or("Could not resolve project_dir from voices")?;
                args.push(project_dir.into_os_string());
                args.extend(["--episode".into(), e.manifest.episode_id.clone().into()]);
            }
            Preset::Verify => {
                args.extend([
                    e.dir.clone().into_os_string(),
                    "--transcript".into(),
                    script.as_os_str().into(),
                    "--json-out".into(),
                    e.dir.join("stress.json").into_os_string(),
                ]);
            }
            Preset::Repair(_) => {
                let voices = e
                    .voices_path
                    .as_ref()
                    .filter(|p| p.is_file())
                    .ok_or_else(|| {
                        format!(
                            "{}: missing voices key or referenced file",
                            cwd.join("scriptcast.toml").display()
                        )
                    })?;
                args.extend([
                    e.dir.clone().into_os_string(),
                    "--transcript".into(),
                    script.as_os_str().into(),
                    "--episode-id".into(),
                    e.manifest.episode_id.clone().into(),
                    "--voices".into(),
                    voices.as_os_str().into(),
                    "--out-dir".into(),
                    out.as_os_str().into(),
                ]);
                if !turns.is_empty() {
                    args.extend(["--turns".into(), compact_ranges(&turns).into()]);
                }
                synthesizes = true;
            }
            _ => {
                args.extend([
                    "--transcript".into(),
                    script.as_os_str().into(),
                    "--episode-id".into(),
                    e.manifest.episode_id.clone().into(),
                    "--out-dir".into(),
                    out.as_os_str().into(),
                ]);
                match p {
                    Preset::Render => {
                        args.push("--resume".into());
                        if let Some(voices) = &e.voices_path {
                            args.extend(["--voices".into(), voices.as_os_str().into()]);
                        }
                        synthesizes = true;
                    }
                    Preset::DetectChanges => args.push("--detect-changes".into()),
                    Preset::PrecisionInsert => {
                        args.push("--precision-insert".into());
                        synthesizes = true
                    }
                    Preset::RegenerateTurns(t) => {
                        args.extend(["--regenerate-turns".into(), compact_ranges(t).into()]);
                        synthesizes = true
                    }
                    _ => unreachable!(),
                }
            }
        }
        let derived = match p {
            Preset::Verify => Some("stress.json"),
            Preset::ReadAlong => Some("word_timings.json"),
            Preset::ClipTranscripts => Some("clip_transcripts.json"),
            Preset::SecondOpinion => Some("stress_ensemble.json"),
            _ => None,
        };
        let mut overwrites: Vec<PathBuf> = derived
            .map(|name| e.dir.join(name))
            .filter(|path| path.exists())
            .into_iter()
            .collect();
        if matches!(p, Preset::Relayout(_) | Preset::ReExport { .. }) {
            overwrites.push(e.dir.join("episode_manifest.json"));
        }
        Ok(Self {
            program: resolved.program.clone(),
            args,
            cwd,
            synthesizes,
            overwrites,
            resolved,
        })
    }
    pub fn needs_confirmation(&self) -> bool {
        self.synthesizes || !self.overwrites.is_empty()
    }
    pub fn display(&self) -> String {
        std::iter::once(self.program.as_os_str())
            .chain(self.args.iter().map(|s| s.as_os_str()))
            .map(|s| shell_quote(&s.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(" ")
    }
    pub fn argv(&self) -> Vec<String> {
        std::iter::once(self.program.as_os_str())
            .chain(self.args.iter().map(|s| s.as_os_str()))
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }
}
pub fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-=:,".contains(&b))
    {
        s.into()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
pub fn compact_ranges(turns: &[usize]) -> String {
    let mut t = turns.to_vec();
    t.sort_unstable();
    t.dedup();
    let mut out = vec![];
    let mut i = 0;
    while i < t.len() {
        let start = t[i];
        let mut end = start;
        i += 1;
        while i < t.len() && t[i] == end + 1 {
            end = t[i];
            i += 1;
        }
        out.push(if start == end {
            start.to_string()
        } else {
            format!("{start}-{end}")
        });
    }
    out.join(",")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}
#[derive(Debug)]
pub enum JobEvent {
    Line(Stream, String),
    Exited(Option<i32>),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JobState {
    #[default]
    Idle,
    Running,
    Complete,
    Failed(Option<i32>),
    Canceled,
}
pub struct Job {
    pub receiver: Receiver<JobEvent>,
    child: Arc<Mutex<Option<Child>>>,
}
impl Job {
    pub fn start(spec: &JobSpec, ctx: egui::Context) -> Result<Self, String> {
        let mut command = std::process::Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env("PATH", &spec.resolved.path)
            .env("PYTHONUNBUFFERED", "1")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(p) = &spec.resolved.pythonpath {
            command.env("PYTHONPATH", p);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("{}: {e}", spec.program.display()))?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let child = Arc::new(Mutex::new(Some(child)));
        let worker = child.clone();
        let (tx, receiver) = mpsc::channel();
        let exited = Arc::new(AtomicBool::new(false));
        let a = read_pipe(
            stdout,
            Stream::Stdout,
            tx.clone(),
            ctx.clone(),
            exited.clone(),
        );
        let b = read_pipe(
            stderr,
            Stream::Stderr,
            tx.clone(),
            ctx.clone(),
            exited.clone(),
        );
        std::thread::spawn(move || {
            let code = loop {
                let done = {
                    let mut guard = worker.lock().unwrap();
                    match guard.as_mut().unwrap().try_wait() {
                        Ok(Some(s)) => Some(s.code()),
                        Err(_) => Some(None),
                        _ => None,
                    }
                };
                if let Some(code) = done {
                    break code;
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            // Drain output already written before publishing exit. Nonblocking readers
            // stop even if a descendant inherited a pipe and keeps it open.
            exited.store(true, Ordering::Release);
            let _ = a.join();
            let _ = b.join();
            worker.lock().unwrap().take();
            let _ = tx.send(JobEvent::Exited(code));
            ctx.request_repaint();
        });
        Ok(Self { receiver, child })
    }
    pub fn cancel(&self) {
        if let Some(c) = self.child.lock().unwrap().as_mut() {
            cli::kill_group(c);
        }
    }
}

fn read_pipe(
    mut pipe: impl Read + std::os::fd::AsRawFd + Send + 'static,
    stream: Stream,
    tx: mpsc::Sender<JobEvent>,
    ctx: egui::Context,
    exited: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let fd = pipe.as_raw_fd();
        // The child owns the write end; only this reader's descriptor is changed.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            let _ = tx.send(JobEvent::Line(
                stream,
                "Could not configure nonblocking job output".into(),
            ));
            return;
        }
        let mut pending = Vec::new();
        let mut bytes = [0; 8192];
        let mut drain_started = None;
        loop {
            if exited.load(Ordering::Acquire) {
                let started = drain_started.get_or_insert_with(Instant::now);
                if started.elapsed() >= Duration::from_millis(200) {
                    break;
                }
            }
            match pipe.read(&mut bytes) {
                Ok(0) => break,
                Ok(n) => {
                    pending.extend_from_slice(&bytes[..n]);
                    while let Some(end) = pending.iter().position(|b| *b == b'\n') {
                        let line: Vec<_> = pending.drain(..=end).collect();
                        let line = String::from_utf8_lossy(&line)
                            .trim_end_matches(['\r', '\n'])
                            .to_owned();
                        let _ = tx.send(JobEvent::Line(stream, line));
                        ctx.request_repaint();
                    }
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    if drain_started.is_some() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => {
                    let _ = tx.send(JobEvent::Line(
                        stream,
                        format!("Job output read failed: {error}"),
                    ));
                    break;
                }
            }
        }
        if !pending.is_empty() {
            let _ = tx.send(JobEvent::Line(
                stream,
                String::from_utf8_lossy(&pending).into_owned(),
            ));
        }
    })
}
impl Drop for Job {
    fn drop(&mut self) {
        // Dropping a handle must not terminate a detached process group.
    }
}
