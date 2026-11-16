//! First-run installation UI. The shell implementation is embedded so the app
//! remains a single distributable and does not rely on its checkout at runtime.
use crate::cli;
use eframe::egui::{self, RichText};
use std::{
    fs,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

const SCRIPT: &str = include_str!("../../install/install.sh");
const GUIDE: &str = "https://github.com/th30d0re/scriptCast/blob/main/docs/setup.md";

#[derive(Default)]
pub struct Installer {
    pub open: bool,
    pub dismissed: bool,
    omnivoice: bool,
    state: State,
    detect: Option<Receiver<bool>>,
    events: Option<Receiver<Event>>,
    child: Arc<Mutex<Option<Child>>>,
    warnings: Vec<String>,
    step: Option<(usize, usize, String)>,
    log: Vec<String>,
    error: Option<(String, String)>,
    reset_confirm: bool,
    toast: Option<String>,
}
#[derive(Default)]
enum State {
    #[default]
    Idle,
    Running,
    Failed,
}
enum Event {
    Line(String),
    Done,
    Exit,
}

impl Installer {
    pub fn start_detection(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.detect = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(
                cli::resolve("scriptcast-project", None, None).is_ok()
                    || cli::resolve("scriptcast", None, None).is_ok(),
            );
        });
    }
    pub fn force(&mut self, state: &str) {
        self.open = true;
        self.state = match state {
            "running" => State::Running,
            "failed" => {
                self.error = Some(("E_PYTHON".into(), "no python3 >= 3.11 found".into()));
                State::Failed
            }
            _ => State::Idle,
        };
        if state == "running" {
            self.step = Some((2, 3, "Creating virtual environment".into()));
            self.log
                .push("STEP 2/3 Creating virtual environment".into());
        }
    }
    pub fn poll(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.detect
            && let Ok(found) = rx.try_recv()
        {
            if !found && !self.dismissed {
                self.open = true;
            }
            self.detect = None;
        }
        let mut finished = false;
        let events = self
            .events
            .as_ref()
            .map(|rx| {
                let mut result = Vec::new();
                while let Ok(event) = rx.try_recv() {
                    result.push(event);
                }
                result
            })
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Line(line) => self.line(line),
                Event::Done => {
                    self.open = false;
                    self.state = State::Idle;
                    finished = true;
                }
                Event::Exit => {
                    if !finished && !matches!(self.state, State::Failed) {
                        self.error =
                            Some(("E_PIP".into(), "installer stopped before completing".into()));
                        self.state = State::Failed;
                    }
                }
            }
        }
        if finished {
            self.toast = Some(
                std::iter::once("scriptCast tools installed".to_owned())
                    .chain(self.warnings.iter().cloned())
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            self.events = None;
            self.start_detection();
        }
        if self.open || self.events.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    fn line(&mut self, line: String) {
        if let Some(rest) = line.strip_prefix("WARN ") {
            let (code, message) = rest.split_once(' ').unwrap_or((rest, ""));
            self.warnings
                .push(format!("{code}: {message}\n{}", fix(code)));
        }
        if let Some(rest) = line.strip_prefix("STEP ") {
            let mut p = rest.splitn(2, ' ');
            if let (Some(n), Some(text)) = (p.next(), p.next())
                && let Some((a, b)) = n.split_once('/')
                && let (Ok(a), Ok(b)) = (a.parse(), b.parse())
            {
                self.step = Some((a, b, text.into()));
            }
        }
        if let Some(rest) = line.strip_prefix("ERROR ") {
            let mut p = rest.splitn(2, ' ');
            self.error = Some((
                p.next().unwrap_or("E_PIP").into(),
                p.next().unwrap_or("installation failed").into(),
            ));
            self.state = State::Failed;
        }
        self.log.push(line);
        if self.log.len() > 200 {
            self.log.remove(0);
        }
    }
    fn run(&mut self) {
        self.log.clear();
        self.warnings.clear();
        self.step = None;
        self.error = None;
        self.state = State::Running;
        let script = std::env::temp_dir().join(format!(
            "scriptcast-install-{}-{}.sh",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        if let Err(e) = fs::write(&script, SCRIPT) {
            self.error = Some(("E_VENV".into(), e.to_string()));
            self.state = State::Failed;
            return;
        }
        let with_omnivoice = self.omnivoice;
        let path = cli::login_path();
        let (tx, rx) = mpsc::channel();
        self.events = Some(rx);
        let child_handle = Arc::new(Mutex::new(None));
        self.child = Arc::clone(&child_handle);
        // Hold this lock through spawn so Cancel cannot race child registration.
        let mut child_slot = child_handle.lock().unwrap();
        let mut command = Command::new("/bin/bash");
        command
            .arg(&script)
            .arg("--app-version")
            .arg(env!("CARGO_PKG_VERSION"));
        if with_omnivoice {
            command.arg("--with-omnivoice");
        }
        command
            .env("PATH", path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.send(Event::Line(format!("ERROR E_VENV {e}")));
                let _ = tx.send(Event::Exit);
                let _ = fs::remove_file(script);
                return;
            }
        };
        let out = child.stdout.take().unwrap();
        let err = child.stderr.take().unwrap();
        *child_slot = Some(child);
        drop(child_slot);
        thread::spawn(move || {
            let stderr_tx = tx.clone();
            let stderr_reader = thread::spawn(move || {
                for line in BufReader::new(err).lines().map_while(Result::ok) {
                    let _ = stderr_tx.send(Event::Line(line));
                }
            });
            let mut done = false;
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                done |= line.starts_with("DONE ");
                let _ = tx.send(Event::Line(line));
            }
            let _ = stderr_reader.join();
            loop {
                let exited = {
                    let mut slot = child_handle.lock().unwrap();
                    match slot.as_mut() {
                        Some(child) => match child.try_wait() {
                            Ok(None) => false,
                            _ => {
                                slot.take();
                                true
                            }
                        },
                        None => true,
                    }
                };
                if exited {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(20));
            }
            let _ = tx.send(if done { Event::Done } else { Event::Exit });
            let _ = fs::remove_file(script);
        });
    }
    fn cancel(&mut self) {
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            cli::kill_group(child);
        }
        self.events = None;
        self.state = State::Idle;
        self.step = None;
        self.error = None;
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        if let Some(toast) = self.toast.take() {
            egui::Area::new(egui::Id::new("installer_toast"))
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new(toast).strong());
                });
        }
        if !self.open {
            return;
        }
        let mut install = false;
        let mut close = false;
        let mut retry = false;
        let mut reset = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("scriptcast_installer")).show(ctx, |ui| {
            ui.set_min_width(520.0);
            ui.heading("scriptCast tools aren't installed");
            ui.label("Installs scriptCast tools in ~/Library/Application Support/scriptCast/ (about 2 GB; models download later). No admin password is needed.");
            match self.state {
                State::Idle => {
                    ui.checkbox(&mut self.omnivoice, "Also install OmniVoice (large)");
                    ui.horizontal(|ui| {
                        install = ui.button("Install").clicked();
                        close = ui.button("Not now").clicked();
                    });
                }
                State::Running => {
                    if let Some((n, total, text)) = &self.step {
                        ui.add(egui::ProgressBar::new(*n as f32 / *total as f32)
                            .text(format!("{text} ({n}/{total})")));
                    } else {
                        ui.spinner();
                    }
                    for warning in &self.warnings {
                        ui.label(warning);
                    }
                    ui.collapsing("Live log", |ui| {
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for line in &self.log {
                                ui.monospace(line);
                            }
                        });
                    });
                    cancel = ui.button("Cancel").clicked();
                }
                State::Failed => {
                    let (code, message) = self.error.clone()
                        .unwrap_or(("E_PIP".into(), "installation failed".into()));
                    ui.colored_label(egui::Color32::from_rgb(210, 80, 80), format!("{code}: {message}"));
                    ui.label(fix(&code));
                    ui.horizontal_wrapped(|ui| {
                        retry = ui.button("Retry").clicked();
                        if ui.button("Open log").clicked() {
                            let _ = Command::new("open").arg(cli::install_root().join("install.log")).spawn();
                        }
                        if ui.button("Copy diagnostics").clicked() {
                            ctx.copy_text(self.diagnostics());
                        }
                        if ui.button("Setup guide").clicked() {
                            let _ = Command::new("open").arg(GUIDE).spawn();
                        }
                        if matches!(code.as_str(), "E_VENV" | "E_PIP") {
                            reset = ui.button("Reset install folder").clicked();
                        }
                    });
                    if self.reset_confirm {
                        ui.horizontal(|ui| {
                            ui.label("Delete the app install folder?");
                            if ui.button("Confirm reset").clicked() {
                                let _ = fs::remove_dir_all(cli::install_root());
                                self.reset_confirm = false;
                                retry = true;
                            }
                            if ui.button("Keep files").clicked() {
                                self.reset_confirm = false;
                            }
                        });
                    }
                }
            }
        });
        if close {
            self.open = false;
            self.dismissed = true;
        }
        if cancel {
            self.cancel();
        }
        if install || retry {
            self.run();
        }
        if reset {
            self.reset_confirm = true;
        }
    }
    fn diagnostics(&self) -> String {
        format!(
            "scriptCast UI {}\nmacOS {}\narch {}\ninstall root {}\n\n{}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            cli::install_root().display(),
            self.log
                .iter()
                .rev()
                .take(40)
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
fn fix(code: &str) -> &'static str {
    match code {
        "E_ARCH" => "scriptCast needs an Apple Silicon Mac.",
        "E_PYTHON" => {
            "Install Python 3.11+ from python.org or run `brew install python@3.12`, then Retry."
        }
        "E_NETWORK" => "Check your connection; installation downloads from PyPI and GitHub.",
        "E_DISK" => "Free about 5 GB of disk space, then Retry.",
        "W_FFMPEG" => "Run `brew install ffmpeg`, or install Homebrew first.",
        _ => "Retry. If it fails again, reset the install folder and retry.",
    }
}
