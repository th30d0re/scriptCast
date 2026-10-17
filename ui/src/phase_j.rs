//! Phase J sheets and durable-job presentation.
use crate::{Browser, capture};
use eframe::egui::{self, Ui};
use crate::{
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
    pub(crate) interrupted: Option<Record>,
    allow_close: bool,
    stopping: Option<Instant>,
    refreshed: usize,
    live_load:
        Option<std::sync::mpsc::Receiver<Result<Episode, crate::episode::LoadError>>>,
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
fn paint_well(ui: &Ui, rect: egui::Rect) {
    ui.painter().rect(
        rect,
        t::RADIUS_CONTROL,
        t::BG_CONTROL,
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        egui::StrokeKind::Inside,
    );
}
fn path_field(ui: &mut Ui, label: &str, value: &mut String, folder: bool) -> bool {
    ui.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), t::BUTTON_HEIGHT),
        egui::Sense::hover(),
    );
    paint_well(ui, rect);
    let choose_w = t::text_width(ui, "Choose…", t::font(t::BODY)) + t::SPACE_16;
    let choose = egui::Rect::from_min_size(
        egui::pos2(rect.right() - choose_w, rect.top()),
        egui::vec2(choose_w, rect.height()),
    );
    let edit = egui::Rect::from_min_max(rect.min, choose.left_bottom())
        .shrink2(egui::vec2(t::SPACE_8, 0.0));
    let mut changed = ui
        .place(
            edit,
            egui::TextEdit::singleline(value)
                .frame(egui::Frame::NONE)
                .desired_width(edit.width()),
        )
        .changed();
    let button = ui.place(
        choose,
        egui::Button::new(t::text("Choose…", t::BODY, t::accent())).frame(false),
    );
    if button.clicked() {
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
}
fn text_well(ui: &mut Ui, value: &mut String) -> bool {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), t::BUTTON_HEIGHT),
        egui::Sense::hover(),
    );
    paint_well(ui, rect);
    let edit = rect.shrink2(egui::vec2(t::SPACE_8, 0.0));
    ui.place(
        edit,
        egui::TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .desired_width(edit.width()),
    )
    .changed()
}
macro_rules! two_columns {
    ($ui:expr, $left:expr, $right:expr) => {{
        let gap = t::SPACE_16;
        $ui.allocate_ui_with_layout(
            egui::vec2($ui.available_width(), 0.0),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let width = ((ui.available_width() - gap) / 2.0).max(0.0);
                ui.vertical(|ui| {
                    ui.set_width(width);
                    $left(ui);
                });
                ui.vertical(|ui| {
                    ui.set_width(width);
                    $right(ui);
                });
            },
        );
    }};
}
fn link_button(ui: &mut Ui, label: &str) -> bool {
    ui.add(egui::Button::new(t::text(label, t::BODY, t::accent())).frame(false))
        .clicked()
}
fn full_frame(fill: egui::Color32, stroke: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(t::BORDER, stroke))
        .corner_radius(t::RADIUS_CONTROL as u8)
        .inner_margin(t::SPACE_8)
}
fn span(ui: &mut Ui, frame: egui::Frame, body: impl FnOnce(&mut Ui)) {
    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        body(ui);
    });
}
fn right_row(ui: &mut Ui, left: impl FnOnce(&mut Ui), right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            right(ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                left(ui);
            });
        });
    });
}
fn switch_toggle(ui: &mut Ui, on: &mut bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(t::SWITCH_WIDTH, t::SWITCH_HEIGHT),
        egui::Sense::click(),
    );
    if response.clicked() && ui.is_enabled() {
        *on = !*on;
        response.mark_changed();
    }
    let track = if *on { t::accent() } else { t::BG_CONTROL };
    ui.painter()
        .rect_filled(rect, t::SWITCH_HEIGHT / 2.0, track);
    ui.painter().rect_stroke(
        rect,
        t::SWITCH_HEIGHT / 2.0,
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        egui::StrokeKind::Inside,
    );
    let knob = t::SWITCH_HEIGHT - t::SPACE_4;
    let x = if *on {
        rect.right() - t::SPACE_2 - knob / 2.0
    } else {
        rect.left() + t::SPACE_2 + knob / 2.0
    };
    ui.painter()
        .circle_filled(egui::pos2(x, rect.center().y), knob / 2.0, t::TEXT_PRIMARY);
    response
}
fn danger_outline(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .font(t::bold(t::BUTTON))
                .color(t::STATUS_FAILED),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(t::BORDER, t::STATUS_FAILED))
        .corner_radius(t::RADIUS_CONTROL as u8)
        .min_size(egui::vec2(0.0, t::BUTTON_HEIGHT)),
    )
}
fn row_height(ui: &Ui) -> f32 {
    t::line_height(ui, t::font(t::BODY)) + t::SPACE_8
}
fn paint_cell(ui: &Ui, rect: egui::Rect, value: &str, color: egui::Color32, align_right: bool) {
    let font = t::font(t::BODY);
    let used = if align_right {
        let width = t::text_width(ui, value, font.clone());
        egui::Rect::from_min_max(
            egui::pos2((rect.right() - width).max(rect.left()), rect.top()),
            rect.max,
        )
    } else {
        rect
    };
    crate::view::line(ui, used, value, font, color);
}
fn hline(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), t::BORDER),
        egui::Sense::hover(),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
    );
}
fn render_block_reason(preview: Option<&Preview>, valid: bool) -> Option<String> {
    let Some(preview) = preview else {
        return Some("Run a dry run to validate these settings before rendering.".into());
    };
    if !valid {
        return Some("Run a dry run to validate these settings before rendering.".into());
    }
    let blocking: Vec<_> = preview.warnings.iter().filter(|w| w.blocking).collect();
    if blocking.is_empty() {
        return None;
    }
    let mut text = blocking
        .iter()
        .find_map(|w| {
            w.speaker_id
                .as_ref()
                .map(|id| format!("Assign a voice to {id} to enable rendering."))
        })
        .unwrap_or_else(|| blocking[0].message.clone());
    if preview.warnings.iter().any(|w| !w.blocking) {
        text.push_str(" Skipped text does not block.");
    }
    Some(text)
}
fn warning_rows(ui: &mut Ui, preview: &Preview, voices: &str, script: &str) {
    for warning in &preview.warnings {
        let blocking = warning.blocking;
        let fill = if blocking {
            t::BG_DANGER_ROW
        } else {
            t::BG_SURFACE
        };
        let stroke = if blocking {
            t::STATUS_FAILED
        } else {
            t::BORDER_DEFAULT
        };
        let message = warning.message.clone();
        let kind = warning.kind.clone();
        let line = warning.line;
        span(ui, full_frame(fill, stroke), |ui| {
            right_row(
                ui,
                |ui| {
                    ui.add(
                        egui::Label::new(t::text(
                            &message,
                            t::BODY,
                            if blocking {
                                t::STATUS_FAILED
                            } else {
                                t::TEXT_PRIMARY
                            },
                        ))
                        .wrap(),
                    );
                },
                |ui| {
                    if blocking
                        && kind == "unknown_speaker"
                        && link_button(ui, "Open voices editor ↗")
                    {
                        reveal(Path::new(voices));
                    } else if let Some(line) = line
                        && link_button(ui, &format!("Review line {line} ↗"))
                    {
                        reveal(Path::new(script));
                    }
                },
            );
        });
    }
}
fn data_row(
    ui: &mut Ui,
    height: f32,
    fill: Option<egui::Color32>,
    cells: &[(&str, bool, egui::Color32)],
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, 0.0, fill);
    }
    let widths = if cells.len() == 4 {
        let turn = 56.0;
        let time = 110.0;
        let speaker = 110.0;
        [
            turn,
            time,
            speaker,
            (rect.width() - turn - time - speaker).max(0.0),
        ]
    } else {
        let turns = 56.0;
        let engine = 110.0;
        [(rect.width() - turns - engine).max(0.0), engine, turns, 0.0]
    };
    let mut x = rect.left();
    for (i, (text, right, color)) in cells.iter().enumerate() {
        let cell = egui::Rect::from_min_size(
            egui::pos2(x, rect.top()),
            egui::vec2(widths[i], rect.height()),
        );
        paint_cell(
            ui,
            cell.shrink2(egui::vec2(t::SPACE_4, 0.0)),
            text,
            *color,
            *right,
        );
        x += widths[i];
    }
}
fn preview_panels(ui: &mut Ui, preview: &Preview) {
    let row_h = row_height(ui);
    let fit = (t::PREVIEW_HEIGHT / row_h).floor().max(1.0);
    let view_h = fit * row_h;
    let shown = preview.turns.len().min(fit as usize);
    let gap = t::SPACE_12;
    let width = ((ui.available_width() - gap)
        / (t::PREVIEW_TURNS_WIDTH + t::PREVIEW_SPEAKERS_WIDTH))
        .max(0.0);
    let turns_w = width * t::PREVIEW_TURNS_WIDTH;
    let speakers_w = width * t::PREVIEW_SPEAKERS_WIDTH;
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(turns_w);
            span(ui, full_frame(t::BG_SURFACE, t::BORDER_DEFAULT), |ui| {
                right_row(
                    ui,
                    |ui| {
                        ui.label(t::heading("Parsed turns", t::SECTION_TITLE));
                    },
                    |ui| {
                        ui.label(t::text(
                            format!("First {shown} of {}", preview.turns.len()),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    },
                );
                data_row(
                    ui,
                    row_h,
                    None,
                    &[
                        ("Turn", false, t::TEXT_MUTED),
                        ("Time · planned", false, t::TEXT_MUTED),
                        ("Speaker", false, t::TEXT_MUTED),
                        ("First words", false, t::TEXT_MUTED),
                    ],
                );
                egui::ScrollArea::vertical()
                    .id_salt("preview_turns")
                    .max_height(view_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        for (i, turn) in preview.turns.iter().enumerate() {
                            let time = progress::clock(Duration::from_millis(turn.timestamp_ms));
                            let index = format!("{:03}", turn.index);
                            data_row(
                                ui,
                                row_h,
                                (i % 2 == 1).then_some(t::BG_WINDOW),
                                &[
                                    (&index, false, t::TEXT_PRIMARY),
                                    (&time, false, t::TEXT_MUTED),
                                    (&turn.speaker_name, false, t::TEXT_PRIMARY),
                                    (&turn.first_words, false, t::TEXT_PRIMARY),
                                ],
                            );
                        }
                    });
            });
        });
        ui.vertical(|ui| {
            ui.set_width(speakers_w);
            span(ui, full_frame(t::BG_SURFACE, t::BORDER_DEFAULT), |ui| {
                right_row(
                    ui,
                    |ui| {
                        ui.label(t::heading("Speaker summary", t::SECTION_TITLE));
                    },
                    |ui| {
                        ui.label(t::text(
                            format!("{} voices", preview.speakers.len()),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    },
                );
                data_row(
                    ui,
                    row_h,
                    None,
                    &[
                        ("Speaker", false, t::TEXT_MUTED),
                        ("Engine", false, t::TEXT_MUTED),
                        ("Turns", true, t::TEXT_MUTED),
                    ],
                );
                let total: usize = preview.speakers.iter().map(|s| s.turns).sum();
                let total_label = total.to_string();
                let list_h = (view_h - row_h).max(row_h);
                let speaker_rows_h = preview.speakers.len() as f32 * row_h;
                let bars = if speaker_rows_h > list_h {
                    egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible
                } else {
                    egui::containers::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
                };
                egui::ScrollArea::vertical()
                    .id_salt("preview_speakers")
                    .max_height(list_h)
                    .min_scrolled_height(list_h)
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(bars)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(preview.speakers.len() as f32 * row_h);
                        for (i, speaker) in preview.speakers.iter().enumerate() {
                            let unknown = speaker.engine.is_none();
                            let turns = speaker.turns.to_string();
                            let engine = speaker.engine.as_deref().unwrap_or("Unassigned");
                            let color = if unknown {
                                t::STATUS_FAILED
                            } else {
                                t::TEXT_PRIMARY
                            };
                            data_row(
                                ui,
                                row_h,
                                if unknown {
                                    Some(t::BG_DANGER_ROW)
                                } else if i % 2 == 1 {
                                    Some(t::BG_WINDOW)
                                } else {
                                    None
                                },
                                &[
                                    (speaker.name.as_str(), false, color),
                                    (engine, false, color),
                                    (turns.as_str(), true, color),
                                ],
                            );
                        }
                    });
                hline(ui);
                data_row(
                    ui,
                    row_h,
                    None,
                    &[
                        ("Total", false, t::TEXT_PRIMARY),
                        ("", false, t::TEXT_PRIMARY),
                        (total_label.as_str(), true, t::TEXT_PRIMARY),
                    ],
                );
            });
        });
    });
}
enum StepLook {
    Muted(&'static str),
    Passed(String),
    Accent(String),
    Failed(String),
    Verify { failing: usize },
}
struct StepQuery<'a> {
    index: usize,
    enabled: bool,
    current: Option<usize>,
    state: &'a State,
    failing: usize,
    max_passes: usize,
    turns: Option<usize>,
    progress: Option<&'a Progress>,
}
fn step_look(query: StepQuery<'_>) -> StepLook {
    if !query.enabled {
        return StepLook::Muted("Skipped");
    }
    let done = match query.current {
        Some(current) => query.index < current,
        None => *query.state == State::Complete,
    };
    if done {
        if query.index == 0 {
            return StepLook::Passed(match query.turns {
                Some(n) => format!("✓ {n} rendered"),
                None => "✓ Complete".into(),
            });
        }
        if query.index == 1 && query.failing > 0 {
            return StepLook::Verify {
                failing: query.failing,
            };
        }
        return StepLook::Passed("✓ Complete".into());
    }
    if query.current == Some(query.index) {
        if matches!(query.state, State::RepairStopped | State::Failed(_)) {
            return StepLook::Failed(if query.failing > 0 {
                format!("Stopped · {} still failing", query.failing)
            } else {
                "Stopped".into()
            });
        }
        if query.index == 2 {
            let pass = query
                .progress
                .map(|p| p.pass)
                .filter(|p| *p > 0)
                .unwrap_or(1);
            let live = query.progress.map(|p| p.failing).unwrap_or(query.failing);
            return StepLook::Accent(format!(
                "Running · pass {pass} of {} · {live} still failing",
                query.max_passes
            ));
        }
        if let Some(stage) = query
            .progress
            .map(|p| p.stage.as_str())
            .filter(|s| !s.is_empty())
        {
            return StepLook::Accent(format!("Running · {stage}"));
        }
        return StepLook::Accent("Running".into());
    }
    StepLook::Muted("Queued")
}
fn paint_step_status(ui: &mut Ui, look: StepLook) {
    match look {
        StepLook::Muted(text) => {
            ui.label(t::text(text, t::CAPTION, t::TEXT_MUTED));
        }
        StepLook::Passed(text) => {
            ui.label(t::text(text, t::CAPTION, t::STATUS_PASSED));
        }
        StepLook::Accent(text) => {
            ui.label(t::text(text, t::CAPTION, t::accent()));
        }
        StepLook::Failed(text) => {
            ui.label(t::text(text, t::CAPTION, t::STATUS_FAILED));
        }
        StepLook::Verify { failing } => {
            let mut job = egui::text::LayoutJob::default();
            let font = t::font(t::CAPTION);
            job.append(
                "Complete · ",
                0.0,
                egui::TextFormat {
                    font_id: font.clone(),
                    color: t::STATUS_PASSED,
                    ..Default::default()
                },
            );
            job.append(
                &format!("{failing} failed"),
                0.0,
                egui::TextFormat {
                    font_id: font,
                    color: t::STATUS_FAILED,
                    ..Default::default()
                },
            );
            ui.label(job);
        }
    }
}
fn pipeline_banner(state: &State) -> Option<(&'static str, egui::Color32)> {
    match state {
        State::Running => Some(("Running", t::accent())),
        State::RepairStopped | State::Failed(_) => Some(("Stopped", t::STATUS_FAILED)),
        State::Ready | State::Complete => None,
    }
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
            let script = if self.harness {
                script
            } else {
                std::fs::canonicalize(&script).unwrap_or(script)
            };
            self.phase.options.script = script.display().to_string();
            self.phase.options.episode_id = script
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if !self.harness {
                self.phase.project = crate::episode::find_project_root(
                    script.parent().unwrap_or(Path::new(".")),
                );
                if let Some(project) = self.phase.project.clone() {
                    self.apply_project_defaults(project);
                }
            }
            self.phase.preview = None;
            self.phase.validated = None;
        }
    }
    pub(crate) fn apply_project_defaults(&mut self, project: PathBuf) {
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
        let resolved = crate::cli::resolve_with_flags(
            "scriptcast",
            Some(&project),
            self.preferences.cli_dir.as_deref(),
            self.jobs
                .login_path
                .as_deref()
                .unwrap_or(std::ffi::OsStr::new("/usr/bin:/bin")),
            std::env::current_exe().ok().as_deref(),
            crate::cli::required_flags("scriptcast"),
        )
        .map(|(resolved, _)| resolved)?;
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
                Classification::Finished => self.note_failed_record(&record),
                _ => {}
            }
        }
    }
    fn note_failed_record(&mut self, record: &crate::detached::Record) {
        if self.jobs.failure.is_some() || record.exit.is_none_or(|code| code == 0) {
            return;
        }
        let log = std::fs::read_to_string(&record.log).unwrap_or_default();
        let label = crate::runner::preset_label(&record.preset);
        let message = crate::detached::failure_banner(&label, &log);
        self.jobs.notice = Some(message.clone());
        self.jobs.failure = Some(crate::jobs::JobFailure {
            message,
            log,
            show_log: false,
        });
        self.jobs.state = JobState::Failed(record.exit);
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
        let elapsed = self.jobs.started.map(|s| s.elapsed()).unwrap_or_else(|| {
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
        });
        let render = self
            .jobs
            .durable
            .as_ref()
            .is_none_or(|a| a.record.preset == "render")
            || self.phase.fixture.is_some();
        if render
            && self
                .jobs
                .durable
                .as_ref()
                .is_none_or(|a| a.record.preset == "render")
        {
            return Some(progress.status(elapsed));
        }
        let label = self
            .jobs
            .durable
            .as_ref()
            .map(|a| crate::runner::preset_label(&a.record.preset))
            .unwrap_or_else(|| "Job".into());
        let detail = if progress.total > 0 {
            format!(
                "turn {} of {} · {}",
                progress.n,
                progress.total,
                crate::progress::clock(elapsed)
            )
        } else if !progress.stage.is_empty() {
            format!(
                "{} · {}",
                progress.stage,
                crate::progress::clock(elapsed)
            )
        } else {
            crate::progress::clock(elapsed)
        };
        Some(format!("{label} · {detail}"))
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
            if !self.dirty_project_files().is_empty() || self.project.saving.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.project.data_open = true;
                self.project.close_prompt = true;
                self.project.quit_after_close = true;
            } else if self.jobs.busy() {
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
                if let crate::runner::JobEvent::Exited(code) = e {
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
                let exit = active.record.exit;
                let terminal_ok = terminal
                    && exit == Some(0)
                    && !matches!(active.progress.terminal, Some(Err(_)));
                let canceled = self.jobs.state == JobState::Canceled;
                self.phase.interrupted = None;
                self.jobs.state =
                    match crate::detached::watched_outcome(exit, terminal_ok, canceled) {
                        crate::detached::Watched::Canceled => JobState::Canceled,
                        crate::detached::Watched::Complete => {
                            if active.record.preset == "regenerate-edited" {
                                self.jobs.regenerated = self.jobs.regen_rows.clone();
                            }
                            JobState::Complete
                        }
                        crate::detached::Watched::Failed => {
                            let log = self.jobs.log.join("\n");
                            let label = crate::runner::preset_label(&active.record.preset);
                            let message = crate::detached::failure_banner(&label, &log);
                            self.jobs.failure = Some(crate::jobs::JobFailure {
                                message: message.clone(),
                                log,
                                show_log: false,
                            });
                            self.jobs.notice = Some(message);
                            JobState::Failed(exit)
                        }
                    };
                if !matches!(self.jobs.state, JobState::Failed(_)) {
                    self.jobs.notice =
                        Some(format!("{} · {:?}", active.record.preset, self.jobs.state));
                }
            }
            if reload && !finish && self.phase.live_load.is_none() {
                let dir = active.record.episode.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                self.phase.live_load = Some(rx);
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(crate::new_episode::load_live(&dir));
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
                    self.waveforms = Some(crate::waveform::Waveforms::start(
                        &episode,
                        ctx.clone(),
                    ));
                    self.overview = episode.overview_sections(
                        crate::episode::overview_section_count(episode.total_ms),
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
        self.failure_log_window(ctx);
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
            let turns = self.phase.preview.as_ref().map_or(0, |p| p.turns.len());
            let speakers = self.phase.preview.as_ref().map_or(0, |p| p.speakers.len());
            right_row(
                ui,
                |ui| {
                    ui.label(t::heading("New episode", t::STATE_TITLE));
                },
                |ui| {
                    if self.phase.preview.is_some() {
                        ui.label(t::text(
                            format!("{turns} turns · {speakers} speakers · 48 kHz"),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    }
                },
            );
            ui.add_enabled_ui(self.phase.dry_job.is_none(), |ui| {
                two_columns!(
                    ui,
                    |ui: &mut Ui| {
                        script_changed |=
                            path_field(ui, "Script file", &mut self.phase.options.script, false);
                    },
                    |ui: &mut Ui| {
                        ui.label(t::text("Episode id", t::CAPTION, t::TEXT_MUTED));
                        text_well(ui, &mut self.phase.options.episode_id);
                        ui.label(t::text(
                            "From filename · editable",
                            t::CAPTION,
                            t::TEXT_SUBTLE,
                        ));
                    }
                );
                two_columns!(
                    ui,
                    |ui: &mut Ui| {
                        path_field(ui, "Voices file", &mut self.phase.options.voices, false);
                    },
                    |ui: &mut Ui| {
                        path_field(ui, "Output folder", &mut self.phase.options.output, true);
                    }
                );
                ui.horizontal(|ui| {
                    dry = ui.button("Dry run").clicked();
                    ui.label(t::text(
                        "Preview result · planned timing, not measured audio",
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                });
            });
            if let Some(preview) = &self.phase.preview {
                warning_rows(
                    ui,
                    preview,
                    &self.phase.options.voices,
                    &self.phase.options.script,
                );
                preview_panels(ui, preview);
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
            if let Some(error) = &self.phase.error {
                ui.colored_label(t::STATUS_FAILED, error);
            }
            let count = self.phase.preview.as_ref().map_or(0, |p| p.turns.len());
            let reason = if ready {
                None
            } else {
                render_block_reason(self.phase.preview.as_ref(), valid)
            };
            right_row(
                ui,
                |ui| {
                    if let Some(reason) = reason {
                        ui.add(
                            egui::Label::new(t::text(reason, t::CAPTION, t::STATUS_FAILED)).wrap(),
                        );
                    }
                },
                |ui| {
                    render = t::primary_button(ui, &format!("Render {count} turns"), None, ready)
                        .clicked();
                    close = ui.button("Cancel").clicked();
                },
            );
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
                    plan.checkpoint = crate::checkpoint::plan_files(preset, episode)?
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
        let progress = self
            .jobs
            .durable
            .as_ref()
            .map(|active| active.progress.clone())
            .or_else(|| self.phase.step_fixture.clone());
        let turns = self.episode.as_ref().map(|e| e.turns.len());
        let gap_ms = self.phase.pipeline.gap_ms;
        let length_text = self.episode.as_ref().map(|e| {
            let length = relayout_length(
                e.manifest.turns.iter().map(|turn| {
                    turn.segments
                        .iter()
                        .map(|s| s.speech_duration_ms.unwrap_or(s.duration_ms))
                        .sum()
                }),
                gap_ms,
            );
            format!(
                "new length {}",
                progress::clock(Duration::from_millis(length))
            )
        });
        modal("pipeline").show(ctx, |ui| {
            ui.set_width(t::PIPELINE_WIDTH);
            ui.set_min_width(t::PIPELINE_WIDTH);
            right_row(
                ui,
                |ui| {
                    ui.label(t::heading("Finish episode", t::STATE_TITLE));
                },
                |ui| {
                    if let Some((label, color)) = pipeline_banner(&self.phase.pipeline.state) {
                        ui.label(t::text(label, t::CAPTION, color));
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(t::SPACE_8, t::SPACE_8),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(rect.center(), t::SPACE_4, color);
                    }
                },
            );
            if let Some(e) = &self.episode {
                ui.label(t::text(
                    format!(
                        "{} · {} turns · {} speakers · {} kHz",
                        e.manifest.episode_id,
                        e.turns.len(),
                        e.speakers.len(),
                        e.manifest.sample_rate / 1000
                    ),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            }
            ui.label(t::text(
                "All six steps included. Toggles and settings are locked while the run is active.",
                t::BODY,
                t::TEXT_MUTED,
            ));
            for (i, step) in Step::ALL.iter().enumerate() {
                if i > 0 {
                    hline(ui);
                }
                let active = self.phase.pipeline.current == Some(i);
                egui::Frame::new()
                    .fill(if active {
                        t::BG_SELECTED
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .inner_margin(t::SPACE_8)
                    .show(ui, |ui| {
                        let row_w = ui.available_width();
                        ui.set_min_width(row_w);
                        let col_w =
                            (row_w - t::SWITCH_WIDTH - ui.spacing().item_spacing.x).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(!locked, |ui| {
                                switch_toggle(ui, &mut self.phase.pipeline.enabled[i]);
                            });
                            ui.vertical(|ui| {
                                ui.set_width(col_w);
                                let look = step_look(StepQuery {
                                    index: i,
                                    enabled: self.phase.pipeline.enabled[i],
                                    current: self.phase.pipeline.current,
                                    state: &self.phase.pipeline.state,
                                    failing: self.phase.pipeline.failing,
                                    max_passes: self.phase.pipeline.max_passes,
                                    turns,
                                    progress: progress.as_ref(),
                                });
                                right_row(
                                    ui,
                                    |ui| {
                                        ui.label(t::heading(step.label(), t::SECTION_TITLE));
                                    },
                                    |ui| paint_step_status(ui, look),
                                );
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
                                if i == 2 {
                                    ui.add_enabled_ui(!locked, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Max passes");
                                            ui.add(
                                                egui::DragValue::new(
                                                    &mut self.phase.pipeline.max_passes,
                                                )
                                                .range(1..=100),
                                            );
                                            if locked {
                                                ui.label(t::text(
                                                    "Locked for this run",
                                                    t::CAPTION,
                                                    t::TEXT_MUTED,
                                                ));
                                            }
                                        });
                                    });
                                }
                                if i == 3 {
                                    let label_w = length_text.as_ref().map_or(0.0, |text| {
                                        t::text_width(ui, text, t::font(t::CAPTION))
                                    });
                                    let slider_w = (col_w
                                        - label_w
                                        - ui.spacing().item_spacing.x)
                                        .max(t::SPACE_20);
                                    ui.add_enabled_ui(!locked, |ui| {
                                        ui.spacing_mut().slider_width = slider_w;
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().slider_width = slider_w;
                                            ui.add(
                                                egui::Slider::new(
                                                    &mut self.phase.pipeline.gap_ms,
                                                    0..=1000,
                                                )
                                                .show_value(false),
                                            );
                                            if let Some(text) = &length_text {
                                                ui.label(t::text(text, t::CAPTION, t::accent()));
                                            }
                                        });
                                    });
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(slider_w, row_height(ui)),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.label(t::text("0 ms", t::CAPTION, t::TEXT_MUTED));
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(t::text(
                                                        "1000 ms",
                                                        t::CAPTION,
                                                        t::TEXT_MUTED,
                                                    ));
                                                },
                                            );
                                        },
                                    );
                                }
                            });
                        });
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
            span(
                ui,
                full_frame(t::BG_WINDOW, t::BORDER_DEFAULT),
                |ui| {
                    ui.label(t::text("Log · last 4 output lines", t::CAPTION, t::TEXT_MUTED));
                    for line in self
                        .jobs
                        .log
                        .iter()
                        .skip(self.jobs.log.len().saturating_sub(4))
                    {
                        ui.label(egui::RichText::new(line).font(t::mono(t::CAPTION)));
                    }
                },
            );
            if let Some(e) = &self.phase.error {
                ui.colored_label(t::STATUS_FAILED, e);
            }
            let running =
                self.jobs.busy() || matches!(self.phase.pipeline.state, State::Running);
            right_row(
                ui,
                |ui| {
                    ui.add(
                        egui::Label::new(t::text(
                            "Original audio kept in a checkpoint. Output replaced only after a safe write.",
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ))
                        .wrap(),
                    );
                },
                |ui| {
                    if running {
                        if ui.button("Cancel run").clicked() {
                            self.cancel_job();
                        }
                    } else if !locked {
                        run = t::primary_button(ui, "Run pipeline", None, self.episode.is_some())
                            .clicked();
                        close = ui.button("Close").clicked();
                    } else {
                        close = ui.button("Close").clicked();
                    }
                },
            );
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
        let body = if let Some(progress) = self.render_progress() {
            let episode = self
                .episode
                .as_ref()
                .map(|e| e.manifest.episode_id.as_str())
                .unwrap_or("This episode");
            format!(
                "{episode} is on turn {} of {}. Keep rendering in the background? \
                 scriptCast will pick it up again when you reopen it.",
                progress.n, progress.total
            )
        } else {
            "Keep rendering in the background? scriptCast will pick it up again when you reopen it."
                .into()
        };
        modal("quit_render").show(ctx, |ui| {
            ui.set_width(t::QUIT_WIDTH);
            ui.set_min_width(t::QUIT_WIDTH);
            ui.set_max_width(t::QUIT_WIDTH);
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(t::icon("shield-check-26-accent"))
                        .fit_to_exact_size(egui::vec2(t::SPACE_22, t::SPACE_22)),
                );
                ui.label(t::heading("A render is still running", t::STATE_TITLE));
            });
            ui.add(egui::Label::new(t::text(body, t::BODY, t::TEXT_MUTED)).wrap());
            right_row(
                ui,
                |ui| {
                    stop = danger_outline(ui, "Stop render").clicked();
                },
                |ui| {
                    keep = t::primary_button(
                        ui,
                        "Keep rendering",
                        None,
                        self.jobs.preparing.is_none(),
                    )
                    .on_disabled_hover_text(
                        "Wait for the checkpoint to finish and the job to start",
                    )
                    .clicked();
                    cancel = ui.button("Cancel").clicked();
                },
            );
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
