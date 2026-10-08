//! Phase J sheets and durable-job presentation.
use crate::{Browser, capture};
use eframe::egui::{self, Ui};
use scriptcast_ui::{
    detached::{self, Classification, CommandPlan, Record},
    episode::Episode,
    new_episode::{Options, Preview},
    pipeline::{Pipeline, State, Step, relayout_length},
    progress::{self, Event, Progress, Tail},
    runner::{Job, JobSpec, JobState, Preset},
    theme as t,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(serde::Deserialize)]
struct PipelineFixture {
    #[serde(flatten)]
    pipeline: Pipeline,
    #[serde(default)]
    log: Vec<String>,
    #[serde(default)]
    events: Vec<Event>,
}
pub struct Active {
    pub path: PathBuf,
    pub record: Record,
    tail: Tail,
    pub progress: Progress,
    step: Option<usize>,
    last_poll: Instant,
    prefix_log: Vec<String>,
}
impl Active {
    fn new(path: PathBuf, record: Record) -> Self {
        Self {
            path,
            record,
            tail: Tail::default(),
            progress: Progress::default(),
            step: None,
            last_poll: Instant::now() - progress::TICK,
            prefix_log: vec![],
        }
    }
}
#[derive(Default)]
pub struct PhaseJ {
    pub new_open: bool,
    pub pipeline_open: bool,
    pub quit_alert: bool,
    pub project: Option<PathBuf>,
    pub options: Options,
    preview: Option<Preview>,
    validated: Option<Options>,
    dry_job: Option<Job>,
    dry_path: PathBuf,
    pub pipeline: Pipeline,
    fixture: Option<Progress>,
    step_fixture: Option<Progress>,
    pub export_fcpxml: bool,
    pub stitch_output: Option<PathBuf>,
    pub export_logic: bool,
    existing: bool,
    error: Option<String>,
    interrupted: Option<Record>,
    allow_close: bool,
    stopping: Option<Instant>,
    refreshed: usize,
    live_load:
        Option<std::sync::mpsc::Receiver<Result<Episode, scriptcast_ui::episode::LoadError>>>,
}
pub fn reveal(path: &Path) {
    let _ = std::process::Command::new("/usr/bin/open")
        .arg("-R")
        .arg(path)
        .spawn();
}
fn modal(id: &str) -> egui::Modal {
    egui::Modal::new(egui::Id::new(id))
        .backdrop_color(t::BG_SCRIM)
        .frame(
            egui::Frame::new()
                .fill(t::BG_RAISED)
                .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                .corner_radius(t::RADIUS_DIALOG as u8)
                .inner_margin(t::DIALOG_PADDING),
        )
}
fn path_field(ui: &mut Ui, label: &str, value: &mut String, folder: bool) -> bool {
    ui.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
    ui.horizontal(|ui| {
        let mut changed = ui
            .add(
                egui::TextEdit::singleline(value)
                    .desired_width(ui.available_width() - t::PATH_CHOOSE_WIDTH),
            )
            .changed();
        if ui.button("Choose…").clicked() {
            let dialog = rfd::FileDialog::new();
            let selected = if folder {
                dialog.pick_folder()
            } else {
                dialog.pick_file()
            };
            if let Some(path) = selected {
                *value = path.display().to_string();
                changed = true;
            }
        }
        changed
    })
    .inner
}
impl Browser {
    pub(crate) fn init_phase_j(&mut self, cli: &capture::Cli) {
        if let Some(script) = &cli.new_episode {
            self.show_new_episode(Some(script.clone()));
        }
        if let Some(path) = &cli.dry_run_json {
            match std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| Preview::parse(&b))
            {
                Ok(preview) => {
                    self.phase.options.voices = preview.voices.display().to_string();
                    self.phase.preview = Some(preview);
                    self.phase.validated = Some(self.phase.options.clone());
                }
                Err(e) => capture::fail(&e),
            }
        }
        if let Some(path) = &cli.progress_fixture {
            let mut progress = Progress::default();
            match Tail::default().read(path) {
                Ok(events) => {
                    for event in events {
                        progress.apply(&event);
                    }
                }
                Err(e) => capture::fail(&e),
            }
            self.phase.fixture = Some(progress);
        }
        if let Some(path) = &cli.pipeline_fixture {
            let fixture: PipelineFixture = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
                .unwrap_or_else(|e| capture::fail(&e));
            self.phase.pipeline = fixture.pipeline;
            self.jobs.log = fixture.log;
            let mut progress = Progress::default();
            for event in fixture.events {
                progress.apply(&event);
            }
            self.phase.step_fixture = Some(progress);
            self.phase.pipeline_open = true;
        }
        self.phase.quit_alert = cli.quit_alert;
    }
    pub(crate) fn show_new_episode(&mut self, script: Option<PathBuf>) {
        if self.jobs.busy() {
            return;
        }
        self.phase.new_open = true;
        if script.is_none()
            && let Some(project) = self.episode.as_ref().and_then(|e| e.project_root.clone())
        {
            self.apply_project_defaults(project);
        }
        if let Some(script) = script {
            let script = std::fs::canonicalize(&script).unwrap_or(script);
            self.phase.options.script = script.display().to_string();
            self.phase.options.episode_id = script
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            self.phase.project = scriptcast_ui::episode::find_project_root(
                script.parent().unwrap_or(Path::new(".")),
            );
            if let Some(project) = self.phase.project.clone() {
                self.apply_project_defaults(project);
            }
            self.phase.preview = None;
            self.phase.validated = None;
        }
    }
    fn apply_project_defaults(&mut self, project: PathBuf) {
        let config = std::fs::read_to_string(project.join("scriptcast.toml")).unwrap_or_default();
        let config: toml::Value =
            toml::from_str(&config).unwrap_or(toml::Value::Table(Default::default()));
        let voices = config
            .get("voices")
            .and_then(toml::Value::as_str)
            .unwrap_or("voices.yaml");
        let output = config
            .get("outputs")
            .and_then(toml::Value::as_str)
            .unwrap_or("outputs");
        self.phase.options.voices = project.join(voices).display().to_string();
        self.phase.options.output = project.join(output).display().to_string();
        if let Some(tail) = config
            .get("tail_ms")
            .and_then(toml::Value::as_integer)
            .and_then(|n| u32::try_from(n).ok())
        {
            self.phase.options.tail_ms = tail;
        }
        if let Some(threshold) = config
            .get("speech_threshold")
            .and_then(toml::Value::as_float)
        {
            self.phase.options.speech_threshold = threshold;
        }
        self.phase.project = Some(project);
    }
    fn output_dir(&self) -> PathBuf {
        self.phase
            .project
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join(self.phase.options.dir())
    }
    fn new_spec(&self, dry: Option<&Path>, resume: bool) -> Result<JobSpec, String> {
        let project = self
            .phase
            .project
            .clone()
            .or_else(|| self.episode.as_ref()?.project_root.clone())
            .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
        let mut options = self.phase.options.clone();
        options.script = project.join(&options.script).display().to_string();
        options.voices = project.join(&options.voices).display().to_string();
        options.output = project.join(&options.output).display().to_string();
        options.validate()?;
        let resolved = crate::cli::resolve_with(
            "scriptcast",
            Some(&project),
            self.preferences.cli_dir.as_deref(),
            self.jobs
                .login_path
                .as_deref()
                .unwrap_or(std::ffi::OsStr::new("/usr/bin:/bin")),
            std::env::current_exe().ok().as_deref(),
        )?;
        Ok(options.spec(resolved, project, dry, resume))
    }
    fn dry_run(&mut self, ctx: &egui::Context) {
        let path = std::env::temp_dir().join(format!(
            "scriptcast-preview-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        match self
            .new_spec(Some(&path), false)
            .and_then(|spec| Job::start(&spec, ctx.clone()))
        {
            Ok(job) => {
                self.phase.dry_job = Some(job);
                self.phase.dry_path = path;
                self.phase.error = None;
            }
            Err(error) => self.phase.error = Some(error),
        }
    }
    pub(crate) fn launch_durable(
        &mut self,
        dir: &Path,
        preset: &str,
        commands: Vec<CommandPlan>,
        pipeline: Option<Pipeline>,
    ) {
        #[cfg(test)]
        let result = detached::launch_with_executable(
            &std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("scriptcast-ui"),
            dir,
            preset,
            commands,
            pipeline,
        );
        #[cfg(not(test))]
        let result = detached::launch(dir, preset, commands, pipeline);
        match result {
            Ok((path, record)) => {
                self.phase.refreshed = 0;
                let mut active = Active::new(path, record);
                active.prefix_log = self.jobs.log.clone();
                self.jobs.durable = Some(active);
                self.jobs.state = JobState::Running;
                self.jobs.started = Some(Instant::now());
                self.jobs.notice = None;
            }
            Err(error) => self.job_error(error),
        }
    }
    fn render_new(&mut self, resume: bool, ctx: &egui::Context) {
        self.jobs.title = "Render".into();
        self.jobs.log.clear();
        let result = self.new_spec(None, resume).and_then(|spec| {
            let dir = self.output_dir();
            let mut plan = CommandPlan::new(&spec);
            if self.phase.options.overwrite && !resume {
                plan.checkpoint = ["episode_manifest.json", "render_state.json"]
                    .iter()
                    .map(|n| dir.join(n))
                    .filter(|p| p.exists())
                    .map(|p| std::fs::canonicalize(p).map_err(|e| e.to_string()))
                    .collect::<Result<_, _>>()?;
            }
            Ok((dir, plan))
        });
        match result {
            Ok((dir, plan)) => {
                self.launch_durable(&dir, "render", vec![plan], None);
                if self.jobs.durable.is_some() {
                    if let (Some(active), Some(preview)) = (&self.jobs.durable, &self.phase.preview)
                    {
                        let path = active.path.with_extension("preview.json");
                        if let Err(e) =
                            std::fs::write(path, serde_json::to_vec(preview).unwrap_or_default())
                        {
                            self.phase.error = Some(e.to_string());
                        }
                        let episode = preview.episode(&active.record.episode, &Default::default());
                        self.accept_episode(episode, ctx);
                    }
                    self.phase.new_open = false;
                    self.phase.existing = false;
                }
            }
            Err(e) => self.phase.error = Some(e),
        }
    }
    pub(crate) fn reattach_phase_j(&mut self) {
        if self.jobs.durable.is_some() || self.phase.fixture.is_some() {
            return;
        }
        let Some(episode) = &self.episode else { return };
        let mut seen = std::collections::HashSet::new();
        for (path, record) in detached::discover(&episode.dir) {
            if !seen.insert(record.preset.clone()) {
                continue;
            }
            let mut progress = Progress::default();
            if let Ok(events) = Tail::default().read(&record.progress) {
                for event in events {
                    progress.apply(&event);
                }
            }
            match detached::classify(&record, progress.terminal.is_some(), detached::alive) {
                Classification::Alive => {
                    if let Some(pipeline) = &record.pipeline {
                        self.phase.pipeline = pipeline.clone();
                    }
                    self.jobs.title = Preset::parse(&record.preset, vec![])
                        .map(|p| p.label().to_owned())
                        .unwrap_or(record.preset.clone());
                    self.jobs.durable = Some(Active::new(path, record));
                    self.jobs.state = JobState::Running;
                    break;
                }
                Classification::Interrupted if self.phase.interrupted.is_none() => {
                    self.phase.interrupted = Some(record)
                }
                _ => {}
            }
        }
    }
    pub(crate) fn phase_status(&self) -> Option<String> {
        let progress = self
            .jobs
            .durable
            .as_ref()
            .map(|a| &a.progress)
            .or(self.phase.fixture.as_ref())?;
        if let Some(active) = &self.jobs.durable
            && active.record.pipeline.is_some()
        {
            return Some(format!(
                "Finish episode · {} · {:?}",
                progress.stage, self.phase.pipeline.state
            ));
        }
        Some(
            progress.status(self.jobs.started.map(|s| s.elapsed()).unwrap_or_else(|| {
                self.jobs
                    .durable
                    .as_ref()
                    .and_then(|a| chrono::DateTime::parse_from_rfc3339(&a.record.started).ok())
                    .and_then(|s| {
                        (chrono::Utc::now() - s.with_timezone(&chrono::Utc))
                            .to_std()
                            .ok()
                    })
                    .unwrap_or(Duration::from_secs(372))
            })),
        )
    }
    pub(crate) fn render_progress(&self) -> Option<&Progress> {
        self.jobs
            .durable
            .as_ref()
            .filter(|a| a.record.preset == "render")
            .map(|a| &a.progress)
            .or(self.phase.fixture.as_ref())
    }
    pub(crate) fn poll_phase_j(&mut self, ctx: &egui::Context) {
        let quit = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Q));
        if !self.phase.allow_close && (quit || ctx.input(|i| i.viewport().close_requested())) {
            if self.jobs.busy() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.phase.quit_alert = true;
            } else if quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if let Some(started) = self.phase.stopping {
            if started.elapsed() >= Duration::from_secs(3) || !self.jobs.busy() {
                self.phase.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ctx.request_repaint_after(progress::TICK);
            }
        }
        let dry_exit = self.phase.dry_job.as_ref().and_then(|job| {
            job.receiver.try_iter().find_map(|e| {
                if let scriptcast_ui::runner::JobEvent::Exited(code) = e {
                    Some(code)
                } else {
                    None
                }
            })
        });
        if let Some(code) = dry_exit {
            self.phase.dry_job = None;
            let result = std::fs::read(&self.phase.dry_path)
                .map_err(|e| format!("Dry run exited {code:?}: {e}"))
                .and_then(|b| Preview::parse(&b));
            match result {
                Ok(preview) => {
                    self.phase.preview = Some(preview);
                    self.phase.validated = Some(self.phase.options.clone());
                }
                Err(e) => self.phase.error = Some(e),
            }
            let _ = std::fs::remove_file(&self.phase.dry_path);
        }
        let mut finish = false;
        let mut reload = false;
        if let Some(active) = &mut self.jobs.durable {
            ctx.request_repaint_after(progress::TICK);
            if active.last_poll.elapsed() < progress::TICK {
                return;
            }
            active.last_poll = Instant::now();
            if let Ok(record) = detached::read(&active.path)
                && record.pid != 0
            {
                active.record = record;
            }
            let path = if let Some(pipeline) = &active.record.pipeline {
                self.phase.pipeline = pipeline.clone();
                if active.step != active.record.current_step {
                    active.step = active.record.current_step;
                    active.tail = Tail::default();
                    active.progress = Progress::default();
                }
                active
                    .path
                    .with_extension(format!("step{}.jsonl", active.step.unwrap_or(0)))
            } else {
                active.record.progress.clone()
            };
            match active.tail.read(&path) {
                Ok(events) => {
                    for event in events {
                        reload |= matches!(event, Event::TurnDone { .. } | Event::Done { .. });
                        active.progress.apply(&event);
                    }
                }
                Err(e) => self.phase.error = Some(e),
            }
            if let Ok(log) = std::fs::read_to_string(&active.record.log) {
                let log = if active.record.pipeline.is_some() {
                    log.get(active.record.log_offset as usize..)
                        .unwrap_or_default()
                } else {
                    &log
                };
                self.jobs.log = log.lines().rev().take(5000).map(str::to_owned).collect();
                self.jobs.log.reverse();
                self.jobs.log.splice(0..0, active.prefix_log.clone());
            }
            if !detached::alive(active.record.pid) {
                finish = true;
                reload = true;
                let terminal = if active.record.pipeline.is_some() {
                    Tail::default()
                        .read(&active.record.progress)
                        .unwrap_or_default()
                        .iter()
                        .any(|e| matches!(e, Event::Done { .. } | Event::Error { .. }))
                } else {
                    active.progress.terminal.is_some()
                };
                if let Some(output) = active.progress.outputs.get("mp3") {
                    self.phase.stitch_output = Some(active.record.episode.join(output));
                }
                let interrupted = !terminal;
                if interrupted {
                    self.phase.interrupted = Some(active.record.clone());
                }
                self.jobs.state = if self.jobs.state == JobState::Canceled {
                    JobState::Canceled
                } else if !interrupted
                    && active.record.exit == Some(0)
                    && !matches!(active.progress.terminal, Some(Err(_)))
                {
                    JobState::Complete
                } else {
                    JobState::Failed(active.record.exit)
                };
                self.jobs.notice = Some(if interrupted {
                    "Interrupted — resume available".into()
                } else {
                    format!("{} · {:?}", active.record.preset, self.jobs.state)
                });
            }
            if reload && !finish && self.phase.live_load.is_none() {
                let dir = active.record.episode.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                self.phase.live_load = Some(rx);
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(scriptcast_ui::new_episode::load_live(&dir));
                    ctx.request_repaint();
                });
            }
        }
        if finish && let Some(active) = self.jobs.durable.take() {
            self.phase.live_load = None;
            self.open(active.record.episode, ctx);
        }
        if let Some(rx) = &self.phase.live_load
            && let Ok(result) = rx.try_recv()
        {
            self.phase.live_load = None;
            if let Ok(episode) = result {
                if self.jobs.durable.is_some()
                    && self.episode.as_ref().is_some_and(|e| e.dir == episode.dir)
                {
                    // Updating metadata must leave the audio transport available during rendering.
                    self.waveforms = Some(scriptcast_ui::waveform::Waveforms::start(
                        &episode,
                        ctx.clone(),
                    ));
                    self.overview = episode.overview_sections(
                        scriptcast_ui::episode::overview_section_count(episode.total_ms),
                    );
                    self.timeline.zoom(1.0, episode.total_ms);
                    self.episode = Some(episode);
                } else {
                    self.accept_episode(episode, ctx);
                }
            }
        }
    }
    pub(crate) fn phase_j_sheets(&mut self, ctx: &egui::Context) {
        if self.phase.new_open {
            self.new_episode_sheet(ctx);
        }
        if self.phase.pipeline_open {
            self.pipeline_sheet(ctx);
        }
        if self.phase.quit_alert {
            self.quit_sheet(ctx);
        }
        if self.phase.interrupted.is_some() && !self.phase.quit_alert {
            let mut resume = false;
            let mut dismiss = false;
            egui::Window::new("Interrupted job")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("The previous job ended without a done or error event.");
                    ui.horizontal(|ui| {
                        resume = ui.button("Resume").clicked();
                        dismiss = ui.button("Dismiss").clicked();
                    });
                });
            if resume && let Some(record) = self.phase.interrupted.take() {
                let mut commands = record.commands;
                if record.preset == "render"
                    && let Some(command) = commands.first_mut()
                {
                    command
                        .args
                        .retain(|a| a != "--overwrite" && a != "--resume");
                    command.args.push("--resume".into());
                }
                self.launch_durable(&record.episode, &record.preset, commands, record.pipeline);
            }
            if dismiss {
                self.phase.interrupted = None;
            }
        }
    }
    fn new_episode_sheet(&mut self, ctx: &egui::Context) {
        let mut script_changed = false;
        let mut dry = false;
        let mut render = false;
        let mut close = false;
        modal("new_episode").show(ctx, |ui| {
            ui.set_width(t::NEW_EPISODE_WIDTH);
            ui.label(t::heading("New episode", t::STATE_TITLE));
            ui.add_enabled_ui(self.phase.dry_job.is_none(), |ui| {
                ui.columns(2, |cols| {
                    if path_field(
                        &mut cols[0],
                        "Script file",
                        &mut self.phase.options.script,
                        false,
                    ) {
                        script_changed = true;
                        self.phase.options.episode_id = Path::new(&self.phase.options.script)
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                    }
                    cols[1].label(t::text("Episode id", t::CAPTION, t::TEXT_MUTED));
                    cols[1].text_edit_singleline(&mut self.phase.options.episode_id);
                });
                ui.columns(2, |cols| {
                    path_field(
                        &mut cols[0],
                        "Voices file",
                        &mut self.phase.options.voices,
                        false,
                    );
                    path_field(
                        &mut cols[1],
                        "Output folder",
                        &mut self.phase.options.output,
                        true,
                    );
                });
                ui.horizontal(|ui| {
                    dry = ui.button("Dry run").clicked();
                    ui.label(t::text(
                        "Preview · planned timing, not measured audio",
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                });
            });
            if let Some(preview) = &self.phase.preview {
                for warning in &preview.warnings {
                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(
                            if warning.blocking {
                                t::STATUS_FAILED
                            } else {
                                t::TEXT_MUTED
                            },
                            &warning.message,
                        );
                        if warning.kind == "unknown_speaker"
                            && ui.button("Open voices file").clicked()
                        {
                            reveal(Path::new(&self.phase.options.voices));
                        }
                    });
                }
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(t::PREVIEW_TURNS_WIDTH);

                        ui.label(t::heading("Parsed turns", t::SECTION_TITLE));
                        egui::ScrollArea::vertical()
                            .id_salt("preview_turns")
                            .max_height(t::PREVIEW_HEIGHT)
                            .show(ui, |ui| {
                                egui::Grid::new("planned_turns")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for heading in
                                            ["Turn", "Time · planned", "Speaker", "First words"]
                                        {
                                            ui.label(heading);
                                        }
                                        ui.end_row();
                                        for turn in &preview.turns {
                                            ui.label(format!("{:03}", turn.index));
                                            ui.label(progress::clock(Duration::from_millis(
                                                turn.timestamp_ms,
                                            )));
                                            ui.label(&turn.speaker_name);
                                            ui.label(&turn.first_words);
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                    ui.vertical(|ui| {
                        ui.set_width(t::PREVIEW_SPEAKERS_WIDTH);
                        ui.label(t::heading("Speaker summary", t::SECTION_TITLE));
                        egui::ScrollArea::vertical()
                            .id_salt("preview_speakers")
                            .max_height(t::PREVIEW_HEIGHT)
                            .show(ui, |ui| {
                                egui::Grid::new("planned_speakers")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for heading in ["Speaker", "Engine", "Turns"] {
                                            ui.label(heading);
                                        }
                                        ui.end_row();
                                        for speaker in &preview.speakers {
                                            let color = if speaker.engine.is_none() {
                                                t::STATUS_FAILED
                                            } else {
                                                t::TEXT_PRIMARY
                                            };
                                            for text in [
                                                &speaker.name,
                                                speaker.engine.as_deref().unwrap_or("Unassigned"),
                                                &speaker.turns.to_string(),
                                            ] {
                                                ui.colored_label(color, text);
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                });
            }
            ui.add_enabled_ui(self.phase.dry_job.is_none(), |ui| {
                ui.collapsing("Render options", |ui| {
                    let options = &mut self.phase.options;
                    ui.horizontal(|ui| {
                        ui.label("Gap ms");
                        ui.add(egui::DragValue::new(&mut options.gap_ms));
                        ui.label("Tail ms");
                        ui.add(egui::DragValue::new(&mut options.tail_ms));
                        ui.label("Speech threshold");
                        ui.add(egui::DragValue::new(&mut options.speech_threshold).speed(0.001));
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut options.overwrite, "Overwrite");
                        ui.checkbox(&mut options.no_trim, "No trim");
                        ui.checkbox(&mut options.skip_als, "Skip ALS");
                        ui.checkbox(&mut options.fcpxml, "FCPXML");
                        ui.label("Max turns (0 = all)");
                        ui.add(egui::DragValue::new(&mut options.max_turns));
                    });
                });
            });
            let valid = self.phase.validated.as_ref() == Some(&self.phase.options);
            let ready = valid
                && self.phase.preview.as_ref().is_some_and(Preview::can_render)
                && self.phase.dry_job.is_none();
            if !ready {
                ui.colored_label(
                    t::STATUS_FAILED,
                    if valid {
                        "Resolve blocking warnings to enable rendering."
                    } else {
                        "Run a dry run to validate these settings before rendering."
                    },
                );
            }
            if let Some(error) = &self.phase.error {
                ui.colored_label(t::STATUS_FAILED, error);
            }
            ui.horizontal(|ui| {
                close = ui.button("Cancel").clicked();
                let count = self.phase.preview.as_ref().map_or(0, |p| p.turns.len());
                render =
                    t::primary_button(ui, &format!("Render {count} turns"), None, ready).clicked();
            });
        });
        if script_changed && Path::new(&self.phase.options.script).is_file() {
            self.show_new_episode(Some(PathBuf::from(&self.phase.options.script)));
        }
        if close {
            self.phase.new_open = false;
        }
        if dry {
            self.dry_run(ctx);
        }
        if render {
            if self.output_dir().exists() {
                self.phase.existing = true;
            } else {
                self.render_new(false, ctx);
            }
        }
        if self.phase.existing {
            let mut choice = None;
            modal("existing_episode").show(ctx, |ui| {
                ui.label(t::heading("Episode output already exists", t::STATE_TITLE));
                ui.label("Resume existing audio, or checkpoint the manifest and render state before overwriting.");
                ui.horizontal(|ui| {
                    if ui.button("Resume").clicked() { choice = Some(true); }
                    if ui.button("Overwrite").clicked() { choice = Some(false); }
                    if ui.button("Cancel").clicked() { self.phase.existing = false; }
                });
            });
            if let Some(resume) = choice {
                self.phase.options.overwrite = !resume;
                self.render_new(resume, ctx);
            }
        }
    }
    fn start_pipeline(&mut self) {
        self.jobs.title = "Finish episode".into();
        self.jobs.log.clear();
        let Some(episode) = &self.episode else { return };
        let dir = episode.dir.clone();
        let presets = [
            Preset::Render,
            Preset::Verify,
            Preset::Repair(None),
            Preset::Relayout(self.phase.pipeline.gap_ms),
            Preset::Stitch,
            Preset::TurnIndex,
        ];
        let result = presets
            .iter()
            .enumerate()
            .map(|(i, preset)| {
                if !self.phase.pipeline.enabled[i] {
                    return Ok(CommandPlan {
                        program: PathBuf::new(),
                        args: vec![],
                        cwd: dir.clone(),
                        path: Default::default(),
                        pythonpath: None,
                        checkpoint: vec![],
                    });
                }
                let spec = JobSpec::from_preset(preset, episode, &self.jobs.commands)?;
                let mut plan = CommandPlan::new(&spec);
                if i == 2 {
                    plan.args.extend([
                        "--max-passes".into(),
                        self.phase.pipeline.max_passes.to_string().into(),
                    ]);
                }
                if spec.synthesizes || !spec.overwrites.is_empty() {
                    plan.checkpoint = scriptcast_ui::checkpoint::plan_files(preset, episode)?
                        .into_iter()
                        .map(|f| f.path)
                        .collect();
                }
                Ok(plan)
            })
            .collect::<Result<Vec<_>, String>>();
        match result {
            Ok(plans) => {
                self.phase.pipeline.current = None;
                self.phase.pipeline.state = State::Ready;
                self.launch_durable(&dir, "pipeline", plans, Some(self.phase.pipeline.clone()));
            }
            Err(e) => self.phase.error = Some(e),
        }
    }
    fn pipeline_sheet(&mut self, ctx: &egui::Context) {
        let mut run = false;
        let mut close = false;
        let mut review = false;
        let mut proceed = false;
        let locked = self.jobs.busy() || self.phase.pipeline.state != State::Ready;
        modal("pipeline").show(ctx, |ui| {
            ui.set_width(t::PIPELINE_WIDTH);
            ui.label(t::heading("Finish episode", t::STATE_TITLE));
            if let Some(e) = &self.episode {
                ui.label(t::text(
                    format!(
                        "{} · {} turns · {} speakers",
                        e.manifest.episode_id,
                        e.turns.len(),
                        e.speakers.len()
                    ),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            }
            ui.label(t::text(
                if locked {
                    "Settings are locked while the run is active."
                } else {
                    "Choose the steps to run in sequence."
                },
                t::BODY,
                t::TEXT_MUTED,
            ));
            for (i, step) in Step::ALL.iter().enumerate() {
                let active = self.phase.pipeline.current == Some(i);
                egui::Frame::new()
                    .fill(if active {
                        t::BG_SELECTED
                    } else {
                        t::BG_SURFACE
                    })
                    .inner_margin(t::SPACE_12)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_enabled(
                                !locked,
                                egui::Checkbox::new(
                                    &mut self.phase.pipeline.enabled[i],
                                    step.label(),
                                ),
                            );
                            let status = if !self.phase.pipeline.enabled[i] {
                                "Skipped"
                            } else if active && self.phase.pipeline.state == State::RepairStopped {
                                "Stopped"
                            } else if active {
                                "Running"
                            } else if self.phase.pipeline.current.is_some_and(|c| i < c)
                                || self.phase.pipeline.state == State::Complete
                            {
                                "Complete"
                            } else {
                                "Queued"
                            };
                            ui.label(t::text(status, t::CAPTION, t::TEXT_MUTED));
                        });
                        ui.label(t::text(
                            [
                                "Synthesize one WAV per turn.",
                                "Check speech against the script and flag mismatches.",
                                "Retry failed turns; verify each replacement before accepting it.",
                                "Recalculate turn positions using the selected gap between turns.",
                                "Join the laid-out turns and write the final episode MP3.",
                                "Export turn times, speakers and text as a searchable index.",
                            ][i],
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                        if i == 2
                            && active
                            && self.phase.pipeline.state == State::Running
                            && let Some(progress) = self
                                .jobs
                                .durable
                                .as_ref()
                                .map(|a| &a.progress)
                                .or(self.phase.step_fixture.as_ref())
                        {
                            ui.colored_label(
                                t::accent(),
                                format!(
                                    "Pass {} of {} · {} still failing",
                                    progress.pass, self.phase.pipeline.max_passes, progress.failing
                                ),
                            );
                        }
                        if i == 2 {
                            ui.add_enabled_ui(!locked, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Max passes");
                                    ui.add(
                                        egui::DragValue::new(&mut self.phase.pipeline.max_passes)
                                            .range(1..=100),
                                    );
                                });
                            });
                        }
                        if i == 3 {
                            ui.add_enabled(
                                !locked,
                                egui::Slider::new(&mut self.phase.pipeline.gap_ms, 0..=1000)
                                    .text("Gap ms"),
                            );
                            if let Some(e) = &self.episode {
                                let length = relayout_length(
                                    e.manifest.turns.iter().map(|turn| {
                                        turn.segments
                                            .iter()
                                            .map(|s| s.speech_duration_ms.unwrap_or(s.duration_ms))
                                            .sum()
                                    }),
                                    self.phase.pipeline.gap_ms,
                                );
                                ui.colored_label(
                                    t::accent(),
                                    format!(
                                        "new length {}",
                                        progress::clock(Duration::from_millis(length))
                                    ),
                                );
                            }
                        }
                    });
            }
            if self.phase.pipeline.state == State::RepairStopped {
                ui.colored_label(
                    t::STATUS_FAILED,
                    format!(
                        "Stopped: {} turns still fail after {} passes",
                        self.phase.pipeline.failing, self.phase.pipeline.max_passes
                    ),
                );
                ui.horizontal(|ui| {
                    review = t::primary_button(ui, "Review failed turns", None, true).clicked();
                    proceed = ui.button("Continue anyway").clicked();
                });
            }
            if let State::Failed(message) = &self.phase.pipeline.state {
                ui.colored_label(t::STATUS_FAILED, message);
            }
            egui::CollapsingHeader::new("Log · last 4 output lines")
                .default_open(true)
                .show(ui, |ui| {
                    for line in self
                        .jobs
                        .log
                        .iter()
                        .skip(self.jobs.log.len().saturating_sub(4))
                    {
                        ui.label(egui::RichText::new(line).font(t::mono(t::CAPTION)));
                    }
                });
            if let Some(e) = &self.phase.error {
                ui.colored_label(t::STATUS_FAILED, e);
            }
            ui.horizontal(|ui| {
                run =
                    t::primary_button(ui, "Run pipeline", None, !locked && self.episode.is_some())
                        .clicked();
                if self.jobs.busy() && ui.button("Cancel run").clicked() {
                    self.cancel_job();
                }
                close = ui.button("Close").clicked();
            });
        });
        if run {
            self.start_pipeline();
        }
        if close || review {
            self.phase.pipeline_open = false;
        }
        if review {
            self.filters.failed_only = true;
        }
        if proceed {
            if let Some(active) = &self.jobs.durable {
                if let Err(e) = std::fs::write(active.path.with_extension("continue"), b"continue")
                {
                    self.phase.error = Some(e.to_string());
                }
            } else {
                self.phase.pipeline.continue_anyway();
            }
        }
    }
    fn quit_sheet(&mut self, ctx: &egui::Context) {
        let mut keep = false;
        let mut stop = false;
        let mut cancel = false;
        modal("quit_render").show(ctx, |ui| {
            ui.set_width(t::QUIT_WIDTH);
            ui.label(t::heading("A render is still running", t::STATE_TITLE));
            if let Some(status) = self.phase_status() { ui.label(t::text(status, t::BODY, t::TEXT_MUTED)); }
            ui.label(t::text("Keep rendering in the background? scriptCast will pick it up again when you reopen it.",
                t::BODY, t::TEXT_MUTED));
            stop = ui.button(egui::RichText::new("Stop render").color(t::STATUS_FAILED)).clicked();
            ui.horizontal(|ui| {
                cancel = ui.button("Cancel").clicked();
                keep = t::primary_button(ui, "Keep rendering", None, self.jobs.preparing.is_none())
                    .on_disabled_hover_text("Wait for the checkpoint to finish and the job to start").clicked();
            });
        });
        if cancel {
            self.phase.quit_alert = false;
        }
        if keep {
            self.phase.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if stop {
            self.cancel_job();
            self.phase.stopping = Some(Instant::now());
            self.phase.quit_alert = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_flags_and_sheets_render_headlessly() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let script = root.join("tests/fixtures/script.md");
        let preview = root.join("tests/fixtures/phase_j/dry-run.json");
        let progress = root.join("tests/fixtures/phase_j/progress.jsonl");
        let pipeline = root.join("tests/fixtures/phase_j/pipeline.json");
        for flags in [
            vec![
                "--new-episode".into(),
                script.into_os_string(),
                "--dry-run-json".into(),
                preview.into_os_string(),
            ],
            vec![
                "--progress-fixture".into(),
                progress.clone().into_os_string(),
            ],
            vec![
                "--progress-fixture".into(),
                progress.into_os_string(),
                "--quit-alert".into(),
            ],
            vec!["--pipeline-fixture".into(), pipeline.into_os_string()],
        ] {
            let cli = capture::Cli::parse(flags).unwrap();
            let mut app = Browser::default();
            app.init_phase_j(&cli);
            app.episode = Some(Episode::load(&root.join("tests/fixtures/episode_small")).unwrap());
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| app.show(ui));
            assert!(!output.shapes.is_empty());
            assert!(!app.jobs.busy(), "fixtures must never launch work");
            output.textures_delta.clear();
        }
    }
    #[test]
    fn close_request_is_cancelled_when_a_job_is_active() {
        let ctx = egui::Context::default();
        let mut app = Browser::default();
        let (_tx, rx) = std::sync::mpsc::channel();
        app.jobs.preparing = Some(rx);
        let mut input = egui::RawInput::default();
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
        let mut output = ctx.run_ui(input, |ui| app.poll_phase_j(ui.ctx()));
        output.textures_delta.clear();
        assert!(app.phase.quit_alert);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::CancelClose)
        );
    }
}
