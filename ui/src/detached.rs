//! Durable job records and a Rust supervisor that outlives its UI parent.
use crate::{
    pipeline::{Pipeline, State},
    progress::{Event, Progress, Tail},
    runner::JobSpec,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPlan {
    pub program: PathBuf,
    pub args: Vec<std::ffi::OsString>,
    pub cwd: PathBuf,
    pub path: std::ffi::OsString,
    pub pythonpath: Option<PathBuf>,
    pub checkpoint: Vec<PathBuf>,
}
impl CommandPlan {
    pub fn new(spec: &JobSpec) -> Self {
        Self {
            program: spec.program.clone(),
            args: spec.args.clone(),
            cwd: spec.cwd.clone(),
            path: spec.resolved.path.clone(),
            pythonpath: spec.resolved.pythonpath.clone(),
            checkpoint: vec![],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub pid: u32,
    pub pgid: u32,
    pub preset: String,
    pub argv: Vec<String>,
    pub log: PathBuf,
    #[serde(default)]
    pub log_offset: u64,
    pub progress: PathBuf,
    pub started: String,
    #[serde(default)]
    pub episode: PathBuf,
    #[serde(default, rename = "steps", alias = "commands")]
    pub commands: Vec<CommandPlan>,
    pub pipeline: Option<Pipeline>,
    pub current_step: Option<usize>,
    pub exit: Option<i32>,
    #[serde(default)]
    pub interrupted: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Classification {
    Alive,
    Finished,
    Interrupted,
}
pub fn classify(record: &Record, terminal: bool, alive: impl Fn(u32) -> bool) -> Classification {
    if alive(record.pid) {
        Classification::Alive
    } else if terminal {
        Classification::Finished
    } else {
        Classification::Interrupted
    }
}
pub fn alive(pid: u32) -> bool {
    if pid == 0 || pid > i32::MAX as u32 {
        return false;
    }
    // EPERM still means the process exists, though it belongs to another user.
    unsafe { libc::kill(pid as i32, 0) == 0 || *libc::__error() == libc::EPERM }
}
pub fn stop(record: &Record) {
    if record.pgid > 1 && record.pgid <= i32::MAX as u32 {
        unsafe {
            libc::kill(-(record.pgid as i32), libc::SIGTERM);
        }
    }
}
pub fn read(path: &Path) -> Result<Record, String> {
    let mut record: Record = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if record.episode.as_os_str().is_empty() {
        record.episode = path
            .parent()
            .and_then(Path::parent)
            .ok_or("Job record has no episode parent")?
            .to_owned();
    }
    Ok(record)
}
pub fn write(path: &Path, record: &Record) -> Result<(), String> {
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(
        &temp,
        serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(temp, path).map_err(|e| e.to_string())
}
pub fn discover(dir: &Path) -> Vec<(PathBuf, Record)> {
    let mut records = vec![];
    if let Ok(entries) = fs::read_dir(dir.join("_jobs")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(record) = read(&path)
            {
                records.push((path, record));
            }
        }
    }
    records.sort_by(|a, b| b.1.started.cmp(&a.1.started));
    records
}
pub fn launch(
    dir: &Path,
    preset: &str,
    commands: Vec<CommandPlan>,
    pipeline: Option<Pipeline>,
) -> Result<(PathBuf, Record), String> {
    launch_with_executable(
        &std::env::current_exe().map_err(|e| e.to_string())?,
        dir,
        preset,
        commands,
        pipeline,
    )
}
pub fn launch_with_executable(
    executable: &Path,
    dir: &Path,
    preset: &str,
    commands: Vec<CommandPlan>,
    pipeline: Option<Pipeline>,
) -> Result<(PathBuf, Record), String> {
    use std::os::unix::process::CommandExt;

    fs::create_dir_all(dir.join("_jobs")).map_err(|e| e.to_string())?;
    let dir = fs::canonicalize(dir).map_err(|e| e.to_string())?;
    let id = format!(
        "{}-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        std::process::id()
    );
    let base = dir.join("_jobs").join(id);
    let path = base.with_extension("json");
    let log = base.with_extension("log");
    let stdout = File::create(&log).map_err(|e| e.to_string())?;
    let mut record = Record {
        pid: 0,
        pgid: 0,
        preset: preset.into(),
        argv: commands
            .first()
            .map(|c| {
                std::iter::once(c.program.to_string_lossy().into_owned())
                    .chain(c.args.iter().map(|a| a.to_string_lossy().into_owned()))
                    .collect()
            })
            .unwrap_or_default(),
        log,
        log_offset: 0,
        progress: base.with_extension("progress.jsonl"),
        started: chrono::Utc::now().to_rfc3339(),
        episode: dir,
        commands,
        pipeline,
        current_step: None,
        exit: None,
        interrupted: false,
    };
    write(&path, &record)?;
    let mut child = Command::new(executable)
        .arg("--job-worker")
        .arg(&path)
        .stdin(Stdio::null())
        .stderr(stdout.try_clone().map_err(|e| e.to_string())?)
        .stdout(stdout)
        .process_group(0)
        .spawn()
        .map_err(|e| e.to_string())?;
    record.pid = child.id();
    record.pgid = child.id();
    // The worker publishes its own pid before launching a command, avoiding parent/worker write races.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok((path, record))
}
fn append(path: &Path, event: &Event) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(event).map_err(|e| e.to_string())?
    )
    .map_err(|e| e.to_string())
}
static CANCELLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
extern "C" fn terminate(_signal: libc::c_int) {
    CANCELLED.store(true, std::sync::atomic::Ordering::Relaxed);
}
fn cancelled() -> bool {
    CANCELLED.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn worker(path: &Path) -> Result<(), String> {
    // Keep the supervisor alive long enough to publish the child's terminal outcome.
    unsafe {
        libc::signal(libc::SIGTERM, terminate as *const () as libc::sighandler_t);
    }

    let mut record = read(path)?;
    record.pid = std::process::id();
    record.pgid = std::process::id();
    write(path, &record)?;
    if let Err(message) = run(path, &mut record) {
        if let Some(pipeline) = &mut record.pipeline {
            pipeline.state = State::Failed(message.clone());
        }
        append(&record.progress, &Event::Error { message })?;
        record.exit = Some(1);
        write(path, &record)?;
    }
    Ok(())
}
fn run(path: &Path, record: &mut Record) -> Result<(), String> {
    if let Some(pipeline) = &mut record.pipeline
        && pipeline.state == State::Ready
    {
        pipeline.next();
    }
    loop {
        let index = if let Some(pipeline) = &record.pipeline {
            match &pipeline.state {
                State::Complete => break,
                State::Failed(message) => return Err(message.clone()),
                State::RepairStopped => {
                    write(path, record)?;
                    let decision = path.with_extension("continue");
                    while !decision.exists() {
                        if cancelled() {
                            return Err("cancelled".into());
                        }
                        std::thread::sleep(Duration::from_millis(250));
                    }
                    fs::remove_file(decision).map_err(|e| e.to_string())?;
                    record.pipeline.as_mut().unwrap().continue_anyway();
                    continue;
                }
                _ => pipeline.current.ok_or("Missing pipeline step")?,
            }
        } else {
            0
        };
        if cancelled() {
            return Err("cancelled".into());
        }
        let plan = record.commands.get(index).ok_or("Missing command")?.clone();
        if !plan.checkpoint.is_empty() {
            let files = plan
                .checkpoint
                .iter()
                .map(|p| {
                    let stored_as = p
                        .strip_prefix(&record.episode)
                        .map_err(|e| e.to_string())?
                        .to_owned();
                    Ok(crate::checkpoint::PlannedFile {
                        path: p.clone(),
                        stored_as,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            crate::checkpoint::create(
                &record.episode,
                &record.preset,
                record.argv.clone(),
                vec![],
                &files,
            )?;
        }
        record.current_step = Some(index);
        record.log_offset = fs::metadata(&record.log).map_err(|e| e.to_string())?.len();
        write(path, record)?;
        let step_progress = if record.pipeline.is_some() {
            path.with_extension(format!("step{index}.jsonl"))
        } else {
            record.progress.clone()
        };
        let output = OpenOptions::new()
            .append(true)
            .open(&record.log)
            .map_err(|e| e.to_string())?;
        let mut command = Command::new(&plan.program);
        command
            .args(&plan.args)
            .current_dir(&plan.cwd)
            .env("PATH", &plan.path)
            .env("PYTHONUNBUFFERED", "1")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::null())
            .stderr(output.try_clone().map_err(|e| e.to_string())?)
            .stdout(output);
        if let Some(p) = plan.pythonpath {
            command.env("PYTHONPATH", p);
        }
        if supports_progress(&record.preset, record.pipeline.is_some()) {
            command.arg("--progress-jsonl").arg(&step_progress);
        }
        record.argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        write(path, record)?;
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let mut tail = Tail::default();
        let mut progress = Progress::default();
        let status = loop {
            for event in tail.read(&step_progress)? {
                progress.apply(&event);
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            std::thread::sleep(Duration::from_millis(250));
        };
        for event in tail.read(&step_progress)? {
            progress.apply(&event);
        }
        if cancelled() {
            return Err("cancelled".into());
        }
        if let Some(pipeline) = &mut record.pipeline {
            let error = match progress.terminal {
                Some(result) => result.err(),
                None => Some("Step ended without terminal progress".into()),
            };
            pipeline.finish(status.code(), error, progress.failing);
            write(path, record)?;
        } else {
            record.exit = status.code().or(Some(1));
            if progress.terminal.is_none() && !supports_progress(&record.preset, false) {
                let event = if status.success() {
                    Event::Done {
                        ok: true,
                        outputs: Default::default(),
                    }
                } else {
                    Event::Error {
                        message: format!("Job exited with {:?}", status.code()),
                    }
                };
                append(&record.progress, &event)?;
            }
            write(path, record)?;
            return Ok(());
        }
    }
    record.exit = Some(0);
    append(
        &record.progress,
        &Event::Done {
            ok: true,
            outputs: Default::default(),
        },
    )?;
    write(path, record)
}
fn supports_progress(preset: &str, pipeline: bool) -> bool {
    pipeline
        || matches!(
            preset,
            "render"
                | "verify"
                | "repair"
                | "relayout"
                | "stitch"
                | "turn-index"
                | "re-export"
                | "precision-insert"
                | "regenerate-turns"
                | "detect-changes"
        )
}
