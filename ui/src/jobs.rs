use crate::{Browser, inspector::InspectorMode, view};
use eframe::egui::{self, Align, Layout, Ui};
use scriptcast_ui::{
    checkpoint::Checkpoint,
    runner::{Commands, Job, JobEvent, JobSpec, JobState, Preset},
    theme as t,
};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// One Expected / Whisper / Phonon line; words absent from `reference` are emphasised.
fn paint_opinion_line(ui: &mut Ui, label: &str, value: &str, reference: &str) {
    let reference: std::collections::HashSet<&str> = reference.split_whitespace().collect();
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &format!("{label}: "),
        0.0,
        egui::TextFormat {
            font_id: t::font(t::CAPTION),
            color: t::TEXT_MUTED,
            ..Default::default()
        },
    );
    if value.trim().is_empty() {
        job.append(
            "—",
            0.0,
            egui::TextFormat {
                font_id: t::font(t::BODY),
                color: t::TEXT_SUBTLE,
                ..Default::default()
            },
        );
    } else {
        for (index, word) in value.split_whitespace().enumerate() {
            if index > 0 {
                job.append(
                    " ",
                    0.0,
                    egui::TextFormat {
                        font_id: t::font(t::BODY),
                        color: t::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
            let differs = !reference.contains(word);
            job.append(
                word,
                0.0,
                egui::TextFormat {
                    font_id: t::font(t::BODY),
                    color: if differs {
                        t::STATUS_FAILED
                    } else {
                        t::TEXT_PRIMARY
                    },
                    ..Default::default()
                },
            );
        }
    }
    ui.label(job);
}

#[derive(Clone, Debug)]
pub(crate) struct Confirmation {
    pub preset: Preset,
    pub spec: JobSpec,
    pub affected: Option<Vec<usize>>,
}

fn scope(c: &Confirmation) -> String {
    if c.preset == Preset::PrecisionInsert && c.affected.is_none() {
        "Whole episode: only changed turns are synthesized".into()
    } else {
        let turns = c.affected.clone().unwrap_or_else(|| c.preset.turns());
        if turns.is_empty() {
            "Whole episode".into()
        } else {
            format!("Turns {}", scriptcast_ui::runner::compact_ranges(&turns))
        }
    }
}

pub(crate) struct Jobs {
    pub login: Option<Receiver<crate::cli::LoginPath>>,
    pub login_path: Option<std::ffi::OsString>,
    pub resolving: Option<Receiver<Commands>>,
    pub mode: Option<InspectorMode>,
    pub auto_confirm: Option<String>,
    pub auto_run: Option<String>,
    pub running_screenshot: Option<std::path::PathBuf>,
    pub running_requested: Option<Instant>,

    pub commands: Commands,
    pub configure: bool,
    pub title: String,
    pub subtitle: String,
    pub state: JobState,
    pub started: Option<Instant>,
    pub elapsed: Duration,
    pub job: Option<Job>,
    pub preparing: Option<Receiver<Result<Checkpoint, String>>>,
    pub prepared: Option<Confirmation>,
    pub notice: Option<String>,
    pub checkpoints: Vec<Checkpoint>,
    pub restore: Option<Checkpoint>,
    pub total_bytes: u64,
    pub confirmation: Option<Confirmation>,
    pub selected: std::collections::BTreeSet<usize>,
    pub log: Vec<String>,
}

impl Default for Jobs {
    fn default() -> Self {
        Self {
            login: None,
            login_path: None,
            resolving: None,
            mode: None,
            auto_confirm: None,
            auto_run: None,
            running_screenshot: None,
            running_requested: None,
            commands: Default::default(),
            configure: false,
            title: String::new(),
            subtitle: String::new(),
            state: JobState::Idle,
            started: None,
            elapsed: Duration::ZERO,
            job: None,
            preparing: None,
            prepared: None,
            notice: None,
            checkpoints: vec![],
            restore: None,
            total_bytes: 0,
            confirmation: None,
            selected: std::collections::BTreeSet::new(),
            log: vec![],
        }
    }
}

impl Jobs {
    pub fn busy(&self) -> bool {
        self.job.is_some() || self.preparing.is_some()
    }
}

impl Browser {
    pub(crate) fn refresh_checkpoints(&mut self) {
        if let Some(e) = &self.episode {
            self.jobs.checkpoints = scriptcast_ui::checkpoint::list(&e.dir);
            self.jobs.total_bytes = self.jobs.checkpoints.iter().map(|c| c.bytes).sum();
        }
    }

    pub(crate) fn has_clip_turns(&self) -> bool {
        self.editing.turns.as_ref().is_some_and(|turns| {
            turns
                .turns
                .iter()
                .any(|turn| scriptcast_ui::script::clip_id(&turn.body).is_some())
        })
    }

    pub(crate) fn selected_turns(&self) -> Vec<usize> {
        let Some(e) = &self.episode else {
            return vec![];
        };
        self.jobs
            .selected
            .iter()
            .copied()
            .chain(self.selected)
            .filter_map(|i| e.turns.get(i).map(|t| t.index))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn multi_select(&mut self, i: usize, modifiers: egui::Modifiers) {
        let Some(e) = &self.episode else { return };
        if i >= e.turns.len() {
            return;
        }
        let mut selected = self.jobs.selected.clone();
        if modifiers.shift {
            let anchor = self.selected.unwrap_or(i);
            let matches = self.filters.matches(e);
            selected.extend((anchor.min(i)..=anchor.max(i)).filter(|n| matches[*n]));
        } else if modifiers.command {
            if !selected.remove(&i) {
                selected.insert(i);
            }
        } else {
            selected.clear();
            selected.insert(i);
        }
        self.select(i);
        if self.selected == Some(i) {
            self.jobs.selected = selected;
        }
    }

    fn job_error(&mut self, error: String) {
        self.jobs.state = JobState::Failed(None);
        self.jobs.notice = Some(error.clone());
        self.jobs.log.push(error.clone());
        eprintln!("scriptcast-ui job: {error}");
        if self.capture.is_some() {
            self.select_error = Some(error);
        }
    }

    pub(crate) fn poll_jobs(&mut self, ctx: &egui::Context) {
        self.poll_running_screenshot(ctx);
        if let Some(rx) = &self.jobs.login {
            match rx.try_recv() {
                Ok(login) => {
                    eprintln!("scriptcast-ui: {}", login.note);
                    self.jobs.login_path = Some(login.path);
                    self.jobs.login = None;
                    self.refresh_commands(ctx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.jobs.login_path = Some(crate::cli::login_path());
                    self.jobs.login = None;
                    self.refresh_commands(ctx);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.jobs.resolving {
            match rx.try_recv() {
                Ok(commands) => {
                    for (name, result) in &commands {
                        let source = result
                            .as_ref()
                            .map(|r| r.description())
                            .unwrap_or_else(|e| e.clone());
                        eprintln!("scriptcast-ui command: {name} · {source}");
                    }
                    self.export_command = commands.get("scriptcast-export").cloned().map(|r| {
                        r.map(|r| crate::cli::ExportCommand {
                            source: r.source,
                            program: r.program,
                        })
                    });
                    self.jobs.commands = commands;
                    self.jobs.resolving = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.jobs.resolving = None;
                    self.export_command = Some(Err("CLI discovery stopped unexpectedly".into()));
                    self.job_error("CLI discovery stopped unexpectedly".into());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        // Automation must wait for both the episode and the login PATH / resolver.
        if self.pending.is_none()
            && self.episode.is_some()
            && self.jobs.login.is_none()
            && self.jobs.resolving.is_none()
            && !self.jobs.commands.is_empty()
            && !self.jobs.busy()
        {
            if let Some(name) = self.jobs.auto_run.take() {
                match Preset::safe_run(&name).and_then(|preset| {
                    self.spec(&preset).map(|spec| Confirmation {
                        preset,
                        spec,
                        affected: None,
                    })
                }) {
                    Ok(c) => self.begin_job(c, ctx),
                    Err(error) => self.job_error(error),
                }
            } else if let Some(name) = self.jobs.auto_confirm.take() {
                match Preset::parse(&name, self.selected_turns()).and_then(|preset| {
                    self.spec(&preset).map(|spec| Confirmation {
                        preset,
                        spec,
                        affected: None,
                    })
                }) {
                    Ok(c) => self.jobs.confirmation = Some(c),
                    Err(error) => self.job_error(error),
                }
            }
        }
        if let Some(rx) = &self.jobs.preparing {
            let result = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                    "Checkpoint worker stopped unexpectedly; no job started".into(),
                )),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                self.jobs.preparing = None;
                let prepared = self.jobs.prepared.take();
                match result {
                    Ok(cp) => {
                        self.refresh_checkpoints();
                        self.jobs.log.push(format!(
                            "Checkpoint v{:02} saved: {}",
                            cp.version,
                            cp.dir.display()
                        ));
                        eprintln!("scriptcast-ui checkpoint saved: {}", cp.dir.display());
                        if let Some(c) = prepared {
                            self.jobs.subtitle =
                                format!("{} · checkpoint v{:02} saved", scope(&c), cp.version);
                            self.start_prepared_job(c, ctx);
                        } else {
                            self.jobs.state = JobState::Complete;
                            self.jobs.notice = Some(format!(
                                "Restore complete · pre-restore checkpoint v{:02} saved",
                                cp.version
                            ));
                            self.reload_after_job(ctx);
                        }
                    }
                    Err(error) => self.job_error(error),
                }
            }
        }
        let mut done = None;
        if let Some(job) = &self.jobs.job {
            while let Ok(event) = job.receiver.try_recv() {
                match event {
                    JobEvent::Line(stream, line) => {
                        eprintln!("scriptcast-ui log ({stream:?}): {line}");
                        self.jobs.log.push(line);
                    }
                    JobEvent::Exited(code) => done = Some(code),
                }
            }
        }
        if self.jobs.log.len() > 5000 {
            self.jobs.log.drain(..self.jobs.log.len() - 5000);
        }
        if let Some(code) = done {
            self.jobs.job = None;
            self.jobs.elapsed = self.jobs.started.map(|s| s.elapsed()).unwrap_or_default();
            if self.jobs.state != JobState::Canceled {
                self.jobs.state = if code == Some(0) {
                    JobState::Complete
                } else {
                    JobState::Failed(code)
                };
            }
            let notice = format!(
                "{} · {:?} · exit {code:?}",
                self.jobs.title, self.jobs.state
            );
            eprintln!("scriptcast-ui job: {notice}");
            self.jobs.notice = Some(notice);
            self.reload_after_job(ctx);
        }
        if self.jobs.login.is_some() || self.jobs.resolving.is_some() || self.jobs.busy() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }

    fn reload_after_job(&mut self, ctx: &egui::Context) {
        if let Some(e) = &self.episode {
            self.open(e.dir.clone(), ctx);
        }
    }

    pub(crate) fn refresh_commands(&mut self, ctx: &egui::Context) {
        if self.jobs.login.is_some() {
            return;
        }
        let project = self.episode.as_ref().and_then(|e| e.project_root.clone());
        let folder = self.preferences.cli_dir.clone();
        let path = self
            .jobs
            .login_path
            .clone()
            .unwrap_or_else(crate::cli::login_path);
        let exe = std::env::current_exe().ok();
        let (tx, rx) = mpsc::channel();
        self.jobs.commands.clear();
        self.jobs.resolving = Some(rx);
        self.export_command = None;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let commands = [
                "scriptcast",
                "scriptcast-verify",
                "scriptcast-word-timings",
                "scriptcast-clip-transcribe",
                "scriptcast-verify-ensemble",
                "scriptcast-review",
                "scriptcast-review-stats",
                "scriptcast-repair",
                "scriptcast-export",
            ]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    crate::cli::resolve_with(
                        name,
                        project.as_deref(),
                        folder.as_deref(),
                        &path,
                        exe.as_deref(),
                    ),
                )
            })
            .collect();
            let _ = tx.send(commands);
            ctx.request_repaint();
        });
    }

    pub(crate) fn spec(&self, preset: &Preset) -> Result<JobSpec, String> {
        if self.jobs.busy() || self.editing.busy() {
            return Err("Wait for the current job or restore to finish".into());
        }
        let e = self.episode.as_ref().ok_or("No episode")?;
        JobSpec::from_preset(preset, e, &self.jobs.commands)
    }

    pub(crate) fn begin_job(&mut self, c: Confirmation, ctx: &egui::Context) {
        if self.jobs.busy() || self.editing.busy() {
            return;
        }
        self.jobs.mode = Some(InspectorMode::Jobs);
        self.jobs.title = c.preset.label().into();
        self.jobs.subtitle = scope(&c);
        self.jobs.notice = None;
        self.jobs.log.clear();
        self.jobs.elapsed = Duration::ZERO;
        if c.spec.needs_confirmation() {
            let Some(e) = &self.episode else { return };
            let files = match scriptcast_ui::checkpoint::plan_files(&c.preset, e) {
                Ok(files) => files,
                Err(error) => {
                    self.job_error(error);
                    return;
                }
            };
            let dir = e.dir.clone();
            let preset = c.preset.name().to_owned();
            let turns = c.affected.clone().unwrap_or_else(|| c.preset.turns());
            let argv = c.spec.argv();
            let (tx, rx) = mpsc::channel();
            self.jobs.preparing = Some(rx);
            self.jobs.prepared = Some(c);
            self.jobs.state = JobState::Running;
            self.jobs.notice = Some("Saving checkpoint before launch…".into());
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(scriptcast_ui::checkpoint::create(
                    &dir, &preset, argv, turns, &files,
                ));
                ctx.request_repaint();
            });
        } else {
            self.start_prepared_job(c, ctx);
        }
    }

    fn start_prepared_job(&mut self, c: Confirmation, ctx: &egui::Context) {
        eprintln!("scriptcast-ui starting: {}", c.spec.display());
        self.jobs.log.push(format!("Command: {}", c.spec.display()));
        match Job::start(&c.spec, ctx.clone()) {
            Ok(job) => {
                self.jobs.job = Some(job);
                self.jobs.state = JobState::Running;
                self.jobs.started = Some(Instant::now());
                self.jobs.notice = Some(format!("{} · Running", c.preset.label()));
            }
            Err(error) => self.job_error(error),
        }
    }

    pub(crate) fn request_job(
        &mut self,
        preset: Preset,
        affected: Option<Vec<usize>>,
        ctx: &egui::Context,
    ) {
        let affected = if preset == Preset::PrecisionInsert {
            affected.or_else(|| {
                self.editing.changes.as_ref().map(|c| {
                    let mut turns = c.inserted.clone();
                    turns.extend(&c.modified);
                    turns.sort_unstable();
                    turns.dedup();
                    turns
                })
            })
        } else {
            affected
        };
        match self.spec(&preset) {
            Ok(spec) => {
                let c = Confirmation {
                    preset,
                    spec,
                    affected,
                };
                if c.spec.needs_confirmation() {
                    self.jobs.confirmation = Some(c);
                } else {
                    self.begin_job(c, ctx);
                }
            }
            Err(error) => self.job_error(error),
        }
    }

    fn begin_restore(&mut self, cp: Checkpoint, ctx: &egui::Context) {
        if self.jobs.busy() || self.editing.busy() {
            return;
        }
        let Some(e) = &self.episode else { return };
        let dir = e.dir.clone();
        let (tx, rx) = mpsc::channel();
        self.jobs.preparing = Some(rx);
        self.jobs.prepared = None;
        self.jobs.state = JobState::Running;
        self.jobs.title = format!("Restore v{:02}", cp.version);
        self.jobs.notice = Some("Saving current files and restoring checkpoint…".into());
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(scriptcast_ui::checkpoint::restore(&dir, &cp));
            ctx.request_repaint();
        });
    }

    fn cancel_job(&mut self) {
        if let Some(job) = &self.jobs.job {
            job.cancel();
        }
        // A checkpoint may finish its copy, but discarding its receiver forbids launch.
        // Restore is an atomic-file copy sequence and cannot be canceled mid-restore.
        if self.jobs.prepared.take().is_some() {
            self.jobs.preparing = None;
        }
        self.jobs.state = JobState::Canceled;
        self.jobs.notice = Some("Canceled".into());
    }

    pub(crate) fn running_capture(&mut self, ctx: &egui::Context) {
        if self.jobs.job.is_some()
            && self.jobs.running_screenshot.is_some()
            && self.jobs.running_requested.is_none()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                "running-job",
            )));
            self.jobs.running_requested = Some(Instant::now());
        }
    }

    fn poll_running_screenshot(&mut self, ctx: &egui::Context) {
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot {
                    image, user_data, ..
                } if user_data.data.is_some() => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image
            && let Some(path) = self.jobs.running_screenshot.take()
        {
            match crate::capture::write_png(&path, &image) {
                Ok(()) => eprintln!("running screenshot: {}", path.display()),
                Err(error) => self.job_error(error),
            }
        }
        if self
            .jobs
            .running_requested
            .is_some_and(|at| at.elapsed() >= Duration::from_secs(1))
        {
            self.jobs.running_requested = None;
        }
    }

    pub(crate) fn jobs_panel(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        ui.add_space(t::SPACE_12);
        ui.horizontal(|ui| {
            ui.label(t::heading("Job runner", t::SECTION_TITLE));
            ui.label(t::text(
                if self.jobs.commands.values().any(Result::is_ok) {
                    "Local CLI · available"
                } else {
                    "Local CLI · unavailable"
                },
                t::CAPTION,
                t::TEXT_MUTED,
            ));
        });
        ui.separator();
        let turns = self.selected_turns();
        let has_clips = self.has_clip_turns();
        let presets = [
            Preset::DetectChanges,
            Preset::PrecisionInsert,
            Preset::RegenerateTurns(turns.clone()),
            Preset::Verify,
            Preset::ReadAlong,
            Preset::ClipTranscripts,
            Preset::SecondOpinion,
            Preset::ReviewStats,
            Preset::Repair((!turns.is_empty()).then_some(turns)),
        ];

        egui::ScrollArea::vertical().id_salt("jobs_body").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for p in presets {
                    let no_clips = p == Preset::ClipTranscripts && !has_clips;
                    let spec = if no_clips {
                        Err("Episode has no clip turns".into())
                    } else {
                        self.spec(&p)
                    };
                    let hint = if no_clips {
                        "Episode has no clip turns".into()
                    } else {
                        spec.as_ref()
                            .map(|s| format!("{}\n{}", s.resolved.description(), s.display()))
                            .unwrap_or_else(|e| e.clone())
                    };
                    if t::secondary_button(ui, p.label(), None, spec.is_ok())
                        .on_hover_text(&hint)
                        .on_disabled_hover_text(&hint)
                        .clicked()
                    {
                        self.request_job(p, None, &ctx);
                    }
                }
            });

            if self.jobs.commands.values().any(Result::is_err) {
                ui.label(t::heading("Rendering CLI unavailable", t::STATE_TITLE));
                ui.label(t::text(
                    "Review audio normally; configure the CLI folder before launching unavailable presets.",
                    t::BODY,
                    t::TEXT_MUTED,
                ));
                if t::primary_button(ui, "Configure CLI", None, true).clicked() {
                    self.jobs.configure = true;
                }
            }

            egui::CollapsingHeader::new("Command sources").show(ui, |ui| {
                for (name, c) in &self.jobs.commands {
                    ui.label(t::text(
                        format!(
                            "{} · {}",
                            name,
                            c.as_ref().map(|r| r.description()).unwrap_or_else(|e| e.clone())
                        ),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                }
            });

            ui.add_space(t::SPACE_8);
            ui.label(t::heading(
                if self.jobs.title.is_empty() { "Ready to run" } else { &self.jobs.title },
                t::SECTION_TITLE,
            ));
            ui.label(t::text(&self.jobs.subtitle, t::CAPTION, t::TEXT_MUTED));
            ui.label(t::text(
                format!("{:?}", self.jobs.state),
                t::SECONDARY,
                t::accent(),
            ));

            if self.jobs.busy() || self.editing.busy() {
                ui.add(egui::ProgressBar::new(0.0).animate(true).desired_height(t::PROGRESS_HEIGHT));
            }

            egui::Frame::new()
                .fill(t::BG_WINDOW)
                .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                .corner_radius(t::RADIUS_CONTROL as u8)
                .inner_margin(t::SPACE_8)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    t::field_label(ui, "Output · auto-scroll on");
                    egui::ScrollArea::vertical()
                        .id_salt("job_log")
                        .max_height(t::JOB_LOG_HEIGHT)
                        .min_scrolled_height(t::JOB_LOG_HEIGHT)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for line in &self.jobs.log {
                                ui.label(egui::RichText::new(line).font(t::mono(t::LOG_LINE)).color(t::TEXT_MUTED));
                            }
                        });
                });

            ui.horizontal_wrapped(|ui| {
                let elapsed = if self.jobs.busy() || self.editing.busy() {
                    self.jobs.started.map(|s| s.elapsed()).unwrap_or_default()
                } else {
                    self.jobs.elapsed
                };
                ui.label(t::text(
                    format!("{} elapsed · reload when complete", view::duration(elapsed.as_millis() as u64)),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
let cancelable = self.jobs.job.is_some() || self.jobs.prepared.is_some();
if t::secondary_button(ui, "Cancel", None, cancelable).clicked() {
    self.cancel_job();
}

            });

            ui.separator();
            ui.label(t::heading("Checkpoints", t::SECTION_TITLE));
            for cp in self.jobs.checkpoints.clone() {
                ui.horizontal_wrapped(|ui| {
                    let kind = cp.kind.replace('-', " ");
                    ui.label(t::text(
                        format!(
                            "v{:02} · {} · {} · {}",
                            cp.version, kind, cp.preset, cp.created_at
                        ),
                        t::CAPTION,
                        t::TEXT_PRIMARY,
                    ));
                    let can_restore = !self.jobs.busy() && !self.editing.busy();
                    if t::secondary_button(ui, "Restore", None, can_restore).clicked() {
                        self.jobs.restore = Some(cp.clone());
                    }
                });
                ui.label(t::text(
                    format!(
                        "Turns {:?} · {} files · {:.2} MiB",
                        cp.turns,
                        cp.files.len(),
                        cp.bytes as f64 / 1048576.0
                    ),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            }
            ui.label(t::text(
                format!(
                    "Total backups: {:.2} MiB · manage in Finder",
                    self.jobs.total_bytes as f64 / 1048576.0
                ),
                t::CAPTION,
                t::TEXT_SUBTLE,
            ));
        });
    }

    pub(crate) fn verification_panel(&mut self, ui: &mut Ui, turn: usize) {
        let (v, failed, turn_id, turn_index, ensemble, ensemble_notice, state) = {
            let e = self.episode.as_ref().unwrap();
            let turn_data = &e.turns[turn];
            let turn_id = turn_data.id.clone();
            (
                turn_data.verify.clone(),
                matches!(
                    turn_data.verification,
                    scriptcast_ui::episode::VerificationState::Failed
                ),
                turn_id.clone(),
                turn_data.index,
                e.stress_ensemble.get(&turn_id).cloned(),
                e.stress_ensemble.notice.clone(),
                turn_data.verification,
            )
        };
        let label = format!("Verification · Turn {turn_index:03}");
        ui.add_space(t::SPACE_16);
        ui.label(t::heading(label, t::SECTION_TITLE));
        ui.separator();
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            let (label, color) = view::verification_label(state);
            ui.horizontal_wrapped(|ui| {
                ui.label(t::text(label, t::BODY, color));
                if let Some(score) = v.as_ref().and_then(|v| v.score) {
                    ui.label(
                        egui::RichText::new(format!("Score {:.3} / 0.90", score))
                            .font(t::mono(t::BODY))
                            .color(color),
                    );
                }
            });
            if let Some(v) = v {
                t::field_label(ui, "Expected text");
                ui.label(t::text(v.expected, t::BODY, t::TEXT_PRIMARY));
                t::field_label(ui, "Heard text");
                ui.label(t::text(
                    v.heard,
                    t::BODY,
                    if v.failed {
                        t::STATUS_FAILED
                    } else {
                        t::TEXT_PRIMARY
                    },
                ));
                t::field_label(ui, "Stress checks");
                if v.stress.is_empty() {
                    ui.label(t::text(
                        "No stress checks recorded",
                        t::SECONDARY,
                        t::TEXT_MUTED,
                    ));
                }
                for s in v.stress {
                    ui.label(t::text(
                        format!("{} · {} · {}", s.word, s.reading, s.verdict),
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                }
                if let Some(repetition) = v.repetition {
                    ui.label(t::text(
                        format!("Repetition: {}", repetition),
                        t::BODY,
                        t::STATUS_FAILED,
                    ));
                }
            } else {
                ui.label(t::text(
                    "This turn has no verification result.",
                    t::BODY,
                    t::TEXT_MUTED,
                ));
            }
            if failed {
                ui.add_space(t::SPACE_16);
                ui.label(t::heading("Second opinion", t::SECTION_TITLE));
                if let Some(second) = &ensemble {
                    ui.label(t::text(
                        scriptcast_ui::stress_ensemble::StressEnsemble::summary_line(
                            &second.summary,
                        ),
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Whisper {:.3}", second.score_whisper))
                                .font(t::mono(t::SECONDARY))
                                .color(t::TEXT_MUTED),
                        );
                        ui.label(
                            egui::RichText::new(format!("Phonon {:.3}", second.score_phonon))
                                .font(t::mono(t::SECONDARY))
                                .color(t::TEXT_MUTED),
                        );
                    });
                    for span in &second.spans {
                        ui.add_space(t::SPACE_8);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(t::text(
                                scriptcast_ui::stress_ensemble::StressEnsemble::span_verdict_label(
                                    &span.verdict,
                                ),
                                t::CAPTION,
                                t::TEXT_MUTED,
                            ));
                        });
                        paint_opinion_line(ui, "Expected", &span.expected, &span.whisper);
                        paint_opinion_line(ui, "Whisper", &span.whisper, &span.expected);
                        paint_opinion_line(ui, "Phonon", &span.phonon, &span.expected);
                    }
                } else {
                    let hint = ensemble_notice.unwrap_or_else(|| {
                        "No second opinion yet for failed turns. \
                         Run Jobs → Second opinion (failed turns)."
                            .into()
                    });
                    ui.label(t::text(hint, t::BODY, t::TEXT_MUTED));
                    let s = self.spec(&Preset::SecondOpinion);
                    let run_hint = s
                        .as_ref()
                        .map(|s| s.resolved.description())
                        .unwrap_or_else(|e| e.clone());
                    if t::secondary_button(ui, "Run…", None, s.is_ok())
                        .on_hover_text(&run_hint)
                        .on_disabled_hover_text(&run_hint)
                        .clicked()
                    {
                        action = Some(Preset::SecondOpinion);
                    }
                }
                self.reviewer_decision_block(ui, turn, &turn_id);
            }
            ui.add_space(t::SPACE_16);
            ui.horizontal_wrapped(|ui| {
                for p in [
                    Preset::RegenerateTurns(self.selected_turns()),
                    Preset::Repair(Some(self.selected_turns())),
                    Preset::Verify,
                ] {
                    let s = self.spec(&p);
                    let hint = s
                        .as_ref()
                        .map(|s| s.resolved.description())
                        .unwrap_or_else(|e| e.clone());
                    if t::secondary_button(
                        ui,
                        if p == Preset::Verify {
                            "Verify again"
                        } else {
                            p.label()
                        },
                        None,
                        s.is_ok(),
                    )
                    .on_hover_text(&hint)
                    .on_disabled_hover_text(&hint)
                    .clicked()
                    {
                        action = Some(p);
                    }
                }
            });
            ui.label(t::text(
                "Verify replaces verification output only. \
                 Regenerate / Repair can replace audio; confirm scope before launching.",
                t::CAPTION,
                t::TEXT_SUBTLE,
            ));
        });
        if let Some(p) = action {
            self.request_job(p, None, ui.ctx());
        }
    }

    fn ensure_review_form(&mut self, turn_id: &str, expected: &str, heard: &str) {
        let needs_new = self.review.as_ref().is_none_or(|f| f.turn_id != turn_id);
        if needs_new {
            self.review = Some(scriptcast_ui::review::DecisionForm::for_turn(
                turn_id, expected, heard,
            ));
        }
    }

    fn reviewer_decision_block(&mut self, ui: &mut Ui, turn: usize, turn_id: &str) {
        let (expected, heard, phonon) = {
            let e = self.episode.as_ref().unwrap();
            let turn_data = &e.turns[turn];
            let expected = turn_data
                .verify
                .as_ref()
                .map(|v| v.expected.clone())
                .unwrap_or_default();
            let heard = turn_data
                .verify
                .as_ref()
                .map(|v| v.heard.clone())
                .unwrap_or_default();
            let phonon = e
                .stress_ensemble
                .get(turn_id)
                .and_then(|s| s.spans.first())
                .map(|s| s.phonon.clone());
            (expected, heard, phonon)
        };
        self.ensure_review_form(turn_id, &expected, &heard);
        ui.add_space(t::SPACE_16);
        ui.label(t::heading("Reviewer decision", t::SECTION_TITLE));
        ui.label(t::text(
            scriptcast_ui::review::NO_RETRAIN,
            t::CAPTION,
            t::TEXT_SUBTLE,
        ));
        let (saved, notice, save_clicked) = {
            let Some(form) = self.review.as_mut() else {
                return;
            };
            for outcome in scriptcast_ui::review::Outcome::all() {
                let selected = form.outcome == outcome;
                if ui
                    .radio(selected, t::text(outcome.label(), t::BODY, t::TEXT_PRIMARY))
                    .clicked()
                {
                    form.outcome = outcome;
                    form.saved = false;
                }
                ui.label(t::text(outcome.explanation(), t::CAPTION, t::TEXT_MUTED));
            }
            t::field_label(ui, "Optional correction");
            let response = ui.add(
                egui::TextEdit::multiline(&mut form.correction)
                    .desired_width(ui.available_width())
                    .desired_rows(2),
            );
            if response.changed() {
                form.saved = false;
            }
            let can_save = !form.busy && self.review_save.is_none() && !self.jobs.busy();
            let save_hint = self
                .jobs
                .commands
                .get("scriptcast-review")
                .map(|r| match r {
                    Ok(resolved) => resolved.description(),
                    Err(error) => error.clone(),
                })
                .unwrap_or_else(|| "scriptcast-review unresolved".into());
            ui.add_space(t::SPACE_8);
            let save_clicked = t::primary_button(ui, "Save decision", None, can_save)
                .on_hover_text(&save_hint)
                .on_disabled_hover_text(&save_hint)
                .clicked();
            if form.outcome == scriptcast_ui::review::Outcome::RecognizerMisheard {
                if let Some(pair) = form
                    .single_accept()
                    .map(|p| (p.word.clone(), p.heard.clone()))
                {
                    let checked = &mut form.accepts[0].checked;
                    ui.checkbox(
                        checked,
                        t::text(
                            format!("Accept this hearing for {} in future verifications", pair.0),
                            t::BODY,
                            t::TEXT_PRIMARY,
                        ),
                    );
                } else if !form.accepts.is_empty() {
                    ui.label(t::text(
                        "Accept hearings for future verifications:",
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    for pair in &mut form.accepts {
                        ui.checkbox(
                            &mut pair.checked,
                            t::text(
                                format!("{} ← {}", pair.word, pair.heard),
                                t::BODY,
                                t::TEXT_PRIMARY,
                            ),
                        );
                    }
                }
            }
            if let Some(hint) = form.calibrate_hint() {
                ui.label(t::text(hint, t::CAPTION, t::TEXT_SUBTLE));
            }
            (form.saved, form.notice.clone(), save_clicked)
        };
        if save_clicked {
            self.save_review_decision(turn_id, &expected, &heard, phonon.as_deref(), ui.ctx());
        }
        if saved {
            ui.label(t::text(
                scriptcast_ui::review::SAVED_BANNER,
                t::BODY,
                t::STATUS_PASSED,
            ));
        }
        if let Some(notice) = notice
            && !saved
        {
            ui.label(t::text(notice, t::CAPTION, t::STATUS_FAILED));
        }
    }

    fn save_review_decision(
        &mut self,
        turn_id: &str,
        expected: &str,
        heard: &str,
        phonon: Option<&str>,
        ctx: &egui::Context,
    ) {
        let Some(form) = self.review.clone() else {
            return;
        };
        let resolved = match self.jobs.commands.get("scriptcast-review") {
            Some(Ok(resolved)) => resolved.clone(),
            Some(Err(error)) => {
                if let Some(f) = self.review.as_mut() {
                    f.notice = Some(error.clone());
                }
                return;
            }
            None => {
                if let Some(f) = self.review.as_mut() {
                    f.notice = Some("scriptcast-review unresolved".into());
                }
                return;
            }
        };
        let (project_dir, episode, cwd) = {
            let e = match &self.episode {
                Some(e) => e,
                None => return,
            };
            let voices = match &e.voices_path {
                Some(path) => path,
                None => {
                    if let Some(f) = self.review.as_mut() {
                        f.notice = Some("Missing voices path".into());
                    }
                    return;
                }
            };
            let project_dir = match scriptcast_ui::review::project_dir_from_voices(voices) {
                Some(path) => path,
                None => {
                    if let Some(f) = self.review.as_mut() {
                        f.notice = Some("Could not resolve project_dir".into());
                    }
                    return;
                }
            };
            let cwd = e.project_root.clone().unwrap_or_else(|| e.dir.clone());
            (project_dir, e.manifest.episode_id.clone(), cwd)
        };
        let start_token = 0usize;
        let end_token = expected.split_whitespace().count().saturating_sub(1);
        let args = scriptcast_ui::review::add_args(
            &scriptcast_ui::review::AddRequest {
                project_dir: &project_dir,
                turn_id,
                episode: &episode,
                start_token,
                end_token,
                script_text: expected,
                heard_whisper: heard,
                heard_phonon: phonon,
            },
            &form,
        );
        if let Some(f) = self.review.as_mut() {
            f.busy = true;
            f.notice = None;
            f.saved = false;
        }
        let (tx, rx) = mpsc::channel();
        self.review_save = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut command = std::process::Command::new(&resolved.program);
            command.args(&resolved.prefix);
            command.args(&args);
            command
                .current_dir(&cwd)
                .env("PATH", &resolved.path)
                .env("PYTHONUNBUFFERED", "1")
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            if let Some(p) = &resolved.pythonpath {
                command.env("PYTHONPATH", p);
            }
            let result = match command.output() {
                Ok(output) if output.status.success() => Ok(()),
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let detail = if !stderr.trim().is_empty() {
                        stderr.trim()
                    } else {
                        stdout.trim()
                    };
                    Err(format!("scriptcast-review failed: {detail}"))
                }
                Err(error) => Err(format!("scriptcast-review: {error}")),
            };
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }

    pub(crate) fn poll_review_save(&mut self) {
        let Some(rx) = &self.review_save else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Disconnected) => Some(Err("Review save worker stopped".into())),
            Err(mpsc::TryRecvError::Empty) => None,
        };
        if let Some(result) = result {
            self.review_save = None;
            if let Some(form) = self.review.as_mut() {
                form.busy = false;
                match result {
                    Ok(()) => {
                        form.saved = true;
                        form.notice = Some(scriptcast_ui::review::SAVED_BANNER.into());
                    }
                    Err(error) => {
                        form.saved = false;
                        form.notice = Some(error);
                    }
                }
            }
        }
    }

    pub(crate) fn confirmation_dialog(&mut self, ctx: &egui::Context) {
        let c = self.jobs.confirmation.clone();
        let restore = self.jobs.restore.clone();
        if (c.is_none() && restore.is_none()) || self.jobs.preparing.is_some() {
            return;
        }
        let mut confirm = false;
        let mut cancel = false;

        let response = egui::Modal::new(egui::Id::new("job_confirmation"))
            .backdrop_color(t::BG_SCRIM)
            .frame(
                egui::Frame::new()
                    .fill(t::BG_RAISED)
                    .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                    .corner_radius(t::RADIUS_DIALOG as u8)
                    .inner_margin(t::DIALOG_PADDING),
            )
            .show(ctx, |ui| {
                ui.set_width(t::DIALOG_WIDTH - t::DIALOG_PADDING * 2.0);
                ui.spacing_mut().item_spacing.y = t::SPACE_18;

                let title = c.as_ref()
                    .map(|c| {
                        if c.preset == Preset::Verify {
                            "Replace verification?".into()
                        } else {
                            format!("{}?", c.preset.label())
                        }
                    })
                    .unwrap_or_else(|| format!("Restore checkpoint v{:02}?", restore.as_ref().unwrap().version));

                ui.horizontal_wrapped(|ui| {
                    ui.add(
    egui::Image::new(t::icon("shield-check-26-accent"))
        .fit_to_exact_size(egui::Vec2::splat(t::CONFIRM_ICON)),
);

                    ui.label(t::heading(title, t::DIALOG_TITLE));
                });

                egui::Frame::new()
                    .fill(t::BG_WINDOW)
                    .corner_radius(t::RADIUS_CONTROL as u8)
                    .inner_margin(t::SPACE_14)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        t::field_label(ui, "Episode");
                        if let Some(e) = &self.episode {
                            ui.label(t::text(&e.manifest.episode_id, t::BODY, t::TEXT_PRIMARY));
                        }
                        t::field_label(ui, "Affected turns");
                        if let Some(c) = &c {
                            ui.label(t::text(scope(c), t::BODY, t::TEXT_PRIMARY));
                            let turns = c.affected.clone().unwrap_or_else(|| c.preset.turns());
if c.preset == Preset::PrecisionInsert && c.affected.is_some() {
    let scope = match (&self.editing.changes, &self.editing.turns) {
        (Some(changes), Some(script_turns)) => Some(
            scriptcast_ui::script::RenderScope::from_changes(changes, &script_turns.turns),
        ),
        _ => None,
    };
    if let Some(changes) = &self.editing.changes {
        ui.label(t::text(format!("{} inserted · {} modified · {} deleted",
            changes.inserted.len(), changes.modified.len(), changes.deleted.len()),
            t::SECONDARY, t::TEXT_MUTED));
    }
    egui::ScrollArea::vertical().max_height(t::RESTORE_LIST_HEIGHT).show(ui, |ui| {
        if let Some(script_turns) = &self.editing.turns {
            for turn in script_turns.turns.iter().filter(|t| turns.contains(&t.index)) {
                ui.label(t::text(format!("{:03} · {}", turn.index, turn.display_name),
                    t::SECONDARY, t::TEXT_PRIMARY));
            }
            if let Some(scope) = &scope
                && !scope.clip_corrections.is_empty()
            {
                ui.label(t::text(
                    "Transcript corrections (no synthesis)",
                    t::SECONDARY,
                    t::TEXT_MUTED,
                ));
                for turn in script_turns
                    .turns
                    .iter()
                    .filter(|t| scope.clip_corrections.contains(&t.index))
                {
                    ui.label(t::text(
                        format!("{:03} · {}", turn.index, turn.display_name),
                        t::SECONDARY,
                        t::TEXT_MUTED,
                    ));
                }
            }
        }
    });
    if let Some(e) = &self.episode {
        let engines = e.speakers.iter().map(|s| s.engine.as_str())
            .filter(|s| !s.is_empty()).collect::<std::collections::BTreeSet<_>>()
            .into_iter().collect::<Vec<_>>().join(", ");
        ui.label(t::text(format!("Engines: {}", if engines.is_empty() {
            e.manifest.engine.as_str() } else { &engines }), t::SECONDARY, t::TEXT_MUTED));
    }
    ui.label(t::text("Later turns move: Precision insert recomputes positions.",
        t::BODY, t::STATUS_FAILED));
} else if let Some(e) = &self.episode {

                                for t0 in e.turns.iter().filter(|t| turns.contains(&t.index)) {
                                    ui.label(t::text(
                                        format!(
                                            "{:03} · {} · {}–{}",
                                            t0.index,
                                            e.speakers[t0.speaker_idx].name,
                                            view::time(t0.start_ms),
                                            view::time(t0.end_ms)
                                        ),
                                        t::SECONDARY,
                                        t::TEXT_MUTED,
                                    ));
                                }
                            }
                        }
                        if let Some(cp) = &restore {
                            egui::ScrollArea::vertical().max_height(t::RESTORE_LIST_HEIGHT).show(ui, |ui| {
ui.label(t::text(format!("Kind: {}", cp.kind.replace('-', " ")),
    t::SECONDARY, t::TEXT_MUTED));
for external in &cp.external_files {
    ui.label(t::text(format!("Replace script: {}", external.path.display()),
        t::BODY, t::STATUS_FAILED));
}
for p in &cp.files {

                                    ui.label(t::text(p.display().to_string(), t::SECONDARY, t::TEXT_PRIMARY));
                                }
                            });
                        }
                    });

                let warning = if restore.is_some() {
                    "The listed files will be replaced. A pre-restore checkpoint is saved first."
                } else if c.as_ref().unwrap().preset == Preset::ReadAlong {
                    "word_timings.json will be replaced. A checkpoint is saved first."
                } else if c.as_ref().unwrap().preset == Preset::ClipTranscripts {
                    "clip_transcripts.json will be replaced. A checkpoint is saved first."
                } else if c.as_ref().unwrap().preset == Preset::SecondOpinion {
                    "stress_ensemble.json will be replaced. A checkpoint is saved first. \
                     stress.json is not modified."
                } else if c.as_ref().unwrap().preset == Preset::Verify {
                    "stress.json will be replaced. A checkpoint is saved first."
                } else {
                    "Rendered audio, manifest, render state, turn index and Ableton set may be replaced. \
                     A checkpoint is saved first."
                };
                ui.label(t::text(warning, t::BODY, t::STATUS_FAILED));

                if let Some(c) = &c {
                    egui::Frame::new()
                        .fill(t::BG_SURFACE)
                        .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                        .corner_radius(t::RADIUS_CONTROL as u8)
                        .inner_margin(t::SPACE_12)
                        .show(ui, |ui| {
                            t::field_label(ui, "Exact command");
                            ui.label(egui::RichText::new(c.spec.display()).font(t::mono(t::LOG_LINE)));
                        });
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    confirm = t::primary_button(
                        ui,
                        &format!("Confirm · {}", c.as_ref().map(|c| c.preset.label()).unwrap_or("Restore")),
                        None,
                        true,
                    )
                    .clicked();
                    cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
                });
            });

        if cancel || response.should_close() {
            self.jobs.confirmation = None;
            self.jobs.restore = None;
        }
        if confirm {
            if let Some(c) = self.jobs.confirmation.take() {
                self.begin_job(c, ctx);
            } else if let Some(restore) = self.jobs.restore.take() {
                self.begin_restore(restore, ctx);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcast_ui::{
        checkpoint,
        cli::{ExportSource, Resolved},
    };
    use std::fs;

    fn app() -> (common::Scratch, Browser, egui::Context) {
        let tmp = common::Scratch::new();
        let episode = tmp.fixture();
        let app = Browser {
            episode: Some(episode),
            ..Default::default()
        };
        (tmp, app, egui::Context::default())
    }

    fn verify(app: &Browser, body: &str) -> Confirmation {
        let e = app.episode.as_ref().unwrap();
        let resolved = Resolved {
            source: ExportSource::Folder,
            program: "/bin/sh".into(),
            prefix: vec![],
            path: "/usr/bin:/bin".into(),
            pythonpath: None,
        };
        let mut spec = JobSpec::from_preset(
            &Preset::Verify,
            e,
            &[("scriptcast-verify".into(), Ok(resolved))].into(),
        )
        .unwrap();
        spec.args = vec!["-c".into(), body.into()];
        Confirmation {
            preset: Preset::Verify,
            spec,
            affected: None,
        }
    }

    fn finish(app: &mut Browser, ctx: &egui::Context) {
        let start = Instant::now();
        while app.jobs.busy() {
            assert!(start.elapsed() < Duration::from_secs(3));
            app.poll_jobs(ctx);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn checkpoint_is_published_before_child_output_and_restore_reloads() {
        let (_tmp, mut app, ctx) = app();
        let dir = app.episode.as_ref().unwrap().dir.clone();
        let original = fs::read(dir.join("stress.json")).unwrap();
        let c = verify(
            &app,
            "test -f episode_small/_backups/ui/*/checkpoint.json || exit 9; \
             cmp episode_small/stress.json episode_small/_backups/ui/*/stress.json || exit 10; \
             echo checkpoint-exists; printf '[]' > episode_small/stress.json",
        );
        app.begin_job(c, &ctx);
        finish(&mut app, &ctx);
        assert_eq!(app.jobs.state, JobState::Complete);
        assert!(app.pending.is_some(), "successful job schedules reload");
        let checkpoint_line = app
            .jobs
            .log
            .iter()
            .position(|s| s.starts_with("Checkpoint v"))
            .unwrap();
        let child_line = app
            .jobs
            .log
            .iter()
            .position(|s| s == "checkpoint-exists")
            .unwrap();
        assert!(checkpoint_line < child_line);
        assert_eq!(fs::read(dir.join("stress.json")).unwrap(), b"[]");
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.pending.is_some() {
            assert!(Instant::now() < deadline);
            app.poll_load(&ctx);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            app.episode.as_ref().unwrap().count(crate::V::NotVerified),
            3
        );
        let cp = checkpoint::list(&dir).remove(0);

        app.pending = None;
        app.begin_restore(cp, &ctx);
        finish(&mut app, &ctx);
        assert_eq!(fs::read(dir.join("stress.json")).unwrap(), original);
        assert!(app.pending.is_some(), "restore schedules reload");
        let before = checkpoint::list(&dir).remove(0);
        assert_eq!(before.preset, "pre-restore");
        assert_eq!(fs::read(before.dir.join("stress.json")).unwrap(), b"[]");
    }

    #[test]
    fn checkpoint_failure_and_worker_disconnect_never_spawn() {
        let (_tmp, mut app, ctx) = app();
        let dir = app.episode.as_ref().unwrap().dir.clone();
        fs::write(dir.join("_backups"), b"blocks backup directory").unwrap();
        let original = fs::read(dir.join("stress.json")).unwrap();
        let c = verify(&app, "printf unsafe > episode_small/stress.json");
        app.begin_job(c.clone(), &ctx);
        finish(&mut app, &ctx);
        assert_eq!(app.jobs.state, JobState::Failed(None));
        assert_eq!(fs::read(dir.join("stress.json")).unwrap(), original);
        let (tx, rx) = mpsc::channel();
        drop(tx);
        app.jobs.preparing = Some(rx);
        app.jobs.prepared = Some(c);
        app.poll_jobs(&ctx);
        assert!(app.jobs.job.is_none());
        assert!(app.jobs.prepared.is_none());
        assert!(app.jobs.notice.as_ref().unwrap().contains("worker stopped"));
        assert_eq!(fs::read(dir.join("stress.json")).unwrap(), original);
    }

    #[test]
    fn cancel_preparation_prevents_launch_and_second_job_is_refused() {
        let (_tmp, mut app, ctx) = app();
        let c = verify(&app, "exit 99");
        let (tx, rx) = mpsc::channel();
        app.jobs.preparing = Some(rx);
        app.jobs.prepared = Some(c.clone());
        assert!(app.spec(&Preset::Verify).unwrap_err().contains("Wait"));
        app.begin_job(c, &ctx);
        app.cancel_job();
        drop(tx);
        app.poll_jobs(&ctx);
        assert!(!app.jobs.busy());
        assert!(app.jobs.job.is_none());
        assert_eq!(app.jobs.state, JobState::Canceled);
    }

    #[test]
    fn failed_and_canceled_children_reload_and_preserve_status() {
        for canceled in [false, true] {
            let (_tmp, mut app, ctx) = app();
            let c = verify(
                &app,
                if canceled {
                    "sleep 2"
                } else {
                    "echo failure >&2; exit 7"
                },
            );
            app.start_prepared_job(c, &ctx);
            if canceled {
                app.cancel_job();
            }
            finish(&mut app, &ctx);
            assert_eq!(
                app.jobs.state,
                if canceled {
                    JobState::Canceled
                } else {
                    JobState::Failed(Some(7))
                }
            );
            assert!(app.pending.is_some());
        }
    }

    #[test]
    fn automation_waits_for_discovery_and_confirmation_never_starts() {
        let (_tmp, mut app, ctx) = app();
        let c = verify(&app, "exit 99");
        let (tx, rx) = mpsc::channel();
        app.jobs.resolving = Some(rx);
        app.jobs.auto_confirm = Some("verify".into());
        app.poll_jobs(&ctx);
        assert!(app.jobs.auto_confirm.is_some());
        tx.send([("scriptcast-verify".into(), Ok(c.spec.resolved))].into())
            .unwrap();
        app.poll_jobs(&ctx);
        assert!(app.jobs.auto_confirm.is_none());
        assert!(app.jobs.confirmation.is_some());
        assert!(!app.jobs.busy());
        assert!(checkpoint::list(&app.episode.as_ref().unwrap().dir).is_empty());
    }

    #[test]
    fn multi_selection_maps_rows_to_manifest_numbers_and_filters_ranges() {
        let (_tmp, mut app, _ctx) = app();
        app.select(0);
        app.filters.speaker = Some(0);
        app.multi_select(2, egui::Modifiers::SHIFT);
        assert_eq!(
            app.jobs.selected.iter().copied().collect::<Vec<_>>(),
            [0, 2]
        );
        assert_eq!(app.selected_turns(), [1, 3]);
        app.multi_select(1, egui::Modifiers::COMMAND);
        assert_eq!(app.selected_turns(), [1, 2, 3]);
        app.select(1);
        assert_eq!(app.selected_turns(), [2]);
    }

    #[test]
    fn phase_c_panels_and_confirmation_render_with_real_fixture_data() {
        let (_tmp, mut app, ctx) = app();
        t::apply(&ctx);
        app.select(1);
        let c = verify(&app, "exit 99");
        for mode in [InspectorMode::Jobs, InspectorMode::Verification] {
            app.jobs.mode = Some(mode);
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.show(ui));
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
        app.jobs.confirmation = Some(c);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.show(ui));
        output.textures_delta.clear();
        assert!(!app.jobs.busy());
        assert!(app.jobs.confirmation.is_some());
    }
}
