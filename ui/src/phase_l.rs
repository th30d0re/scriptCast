//! Project selection and project-data drafts. Project files are owned by the Python tool.
use crate::Browser;
use eframe::egui::{self, Ui};
use scriptcast_ui::{
    cli,
    project::{self, ProjectEpisode, RecentProject, Report, Snapshot, ToolError},
    theme as t,
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{SystemTime, UNIX_EPOCH},
};

type ProjectLoad = Result<(Snapshot, Vec<ProjectEpisode>), ToolError>;

#[derive(Default)]
pub struct PhaseL {
    pub picker: bool,
    pub search: String,
    pub engine_change: Option<(String, String, Vec<String>)>,
    pub rename: Option<(String, String)>,
    pub media_path: Option<PathBuf>,
    /// Millisecond start/end of the decode in flight or cached. `None` end means the whole file.
    pub media_key: Option<(PathBuf, i64, i64)>,
    pub decode_requests: u32,
    pub media_pending: Option<Receiver<Result<(scriptcast_ui::waveform::Envelope, f64), String>>>,
    pub media: Option<Result<scriptcast_ui::waveform::Envelope, String>>,
    pub media_origin: f64,
    pub media_span: Option<(f64, f64)>,
    pub audition: Option<scriptcast_ui::audio::AudioOut>,
    pub audition_end: f64,
    pub root: Option<PathBuf>,
    pub snapshot: Option<Snapshot>,
    pub rows: Vec<ProjectEpisode>,
    pub selected: Option<usize>,
    pub error: Vec<String>,
    pub pending: Option<Receiver<ProjectLoad>>,
    pub data_open: bool,
    pub saving: Option<Receiver<Result<Snapshot, ToolError>>>,
    pub saving_file: String,
    pub close_prompt: bool,
    pub close_after_save: bool,
    pub quit_after_close: bool,
    pub stale: bool,
    pub tab: String,
    pub select_id: String,
    pub draft: Option<Report>,
    /// Name rect and engine-chip rect for each speaker row painted this frame.
    pub speaker_rows: Vec<(egui::Rect, egui::Rect)>,
}
impl Browser {
    pub(crate) fn load_project(&mut self, root: PathBuf, ctx: &egui::Context) {
        if self.project.saving.is_some() || !self.dirty_project_files().is_empty() {
            self.project.data_open = true;
            self.project.close_prompt = true;
            self.project.error =
                vec!["Save or discard project changes before opening another project.".into()];
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.project.root = Some(root.clone());
        self.project.pending = Some(rx);
        self.project.snapshot = None;
        self.project.draft = None;
        self.project.rows.clear();
        self.project.selected = None;
        self.project.error.clear();
        let folder = self.preferences.cli_dir.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = (|| {
                let resolved = cli::resolve("scriptcast-project", Some(&root), folder.as_deref())
                    .map_err(ToolError::Failed)?;
                let snapshot = project::load(&resolved, &root)?;
                let rows =
                    project::discover_episodes(&snapshot.report).map_err(ToolError::Failed)?;
                Ok((snapshot, rows))
            })();
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(crate) fn poll_project(&mut self, ctx: &egui::Context) {
        if (self.project.data_open || self.settings_open)
            && self.project.snapshot.is_none()
            && self.project.pending.is_none()
            && self.project.error.is_empty()
        {
            let root = self
                .episode
                .as_ref()
                .and_then(|e| e.project_root.clone())
                .or_else(|| self.project.root.clone());
            if let Some(root) = root {
                self.load_project(root, ctx);
            }
        }
        if let Some(result) = self
            .project
            .media_pending
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.project.media_pending = None;
            match result {
                Ok((envelope, origin)) => {
                    let len = envelope.duration_secs();
                    self.project.media_origin = origin;
                    self.project.media_span = Some((origin, len));
                    self.project.media = Some(Ok(envelope));
                }
                Err(error) => self.project.media = Some(Err(error)),
            }
        }
        if let Some(audio) = &mut self.project.audition
            && audio.position().as_secs_f64() >= self.project.audition_end
        {
            audio.stop();
        }
        if let Some(result) = self
            .project
            .saving
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.project.saving = None;
            match result {
                Ok(saved) => {
                    if self.project.saving_file == "toml" {
                        self.project.draft = Some(saved.report.clone());
                        self.project.snapshot = Some(saved.clone());
                    }
                    let file = &self.project.saving_file;
                    if let Some(draft) = &mut self.project.draft {
                        *draft.section_mut(file) = saved.report.section(file).clone();
                    }
                    if let Some(snapshot) = &mut self.project.snapshot {
                        *snapshot.report.section_mut(file) = saved.report.section(file).clone();
                        snapshot
                            .hashes
                            .insert(file.clone(), saved.hashes[file].clone());
                    }
                    self.project.error.clear();
                }
                Err(error) => {
                    self.project.stale = error == ToolError::Stale;
                    self.project.error = error.messages();
                    self.project.close_after_save = false;
                }
            }
        }
        let Some(result) = self
            .project
            .pending
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        else {
            return;
        };
        self.project.pending = None;
        match result {
            Ok((snapshot, rows)) => {
                let root = snapshot.report.root.clone();
                self.project.root = Some(root.clone());
                if !snapshot.report.toml.missing {
                    self.preferences.recent_projects.retain(|p| p.root != root);
                    self.preferences.recent_projects.insert(
                        0,
                        RecentProject {
                            name: root
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned(),
                            root,
                            opened: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        },
                    );
                    self.preferences.recent_projects.truncate(5);
                }
                self.project.draft = Some(snapshot.report.clone());
                self.project.snapshot = Some(snapshot);
                self.project.rows = rows;
            }
            Err(error) => self.project.error = error.messages(),
        }
    }
    pub(crate) fn create_scriptcast_toml(&mut self, ctx: &egui::Context) {
        let Some(root) = self.project.root.clone() else {
            return;
        };
        if self.project.pending.is_some() || self.project.saving.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.project.pending = Some(rx);
        self.project.error.clear();
        let folder = self.preferences.cli_dir.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = (|| {
                let resolved = cli::resolve("scriptcast-project", Some(&root), folder.as_deref())
                    .map_err(ToolError::Failed)?;
                let snapshot = project::init_project(&resolved, &root)?;
                let rows =
                    project::discover_episodes(&snapshot.report).map_err(ToolError::Failed)?;
                Ok((snapshot, rows))
            })();
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(crate) fn choose_project(&mut self, ctx: &egui::Context) {
        if let Some(root) = rfd::FileDialog::new()
            .set_title("Open project folder")
            .pick_folder()
        {
            self.project.picker = true;
            self.load_project(root, ctx);
        }
    }
    pub(crate) fn open_project_data(&mut self, ctx: &egui::Context) {
        self.project.data_open = true;
        if self.project.tab.is_empty() {
            self.project.tab = "voices".into();
        }
        let episode_root = self.episode.as_ref().and_then(|e| e.project_root.as_ref());
        let different = episode_root.is_some_and(|root| self.project.root.as_ref() != Some(root));
        if (self.project.snapshot.is_none() || different) && self.project.pending.is_none() {
            let root = self
                .episode
                .as_ref()
                .and_then(|e| e.project_root.clone())
                .or_else(|| self.project.root.clone())
                .or_else(|| std::env::var_os("SCRIPTCAST_PROJECT").map(PathBuf::from));
            if let Some(root) = root {
                self.load_project(root, ctx);
            }
        }
    }
    pub(crate) fn project_picker(&mut self, ui: &mut Ui) {
        let mut load = None;
        let mut open = None;
        let mut new_script = None;
        let mut create_toml = false;
        let ctx = ui.ctx().clone();
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, t::BG_WINDOW);
        let header =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), t::EPISODE_HEADER_HEIGHT));
        self.project_header(
            &mut crate::view::region(ui, "project_header", header, t::BG_SURFACE),
            true,
        );
        let body = crate::view::inset(
            egui::Rect::from_min_max(header.left_bottom(), rect.max),
            t::SPACE_16,
            t::SPACE_16,
        );
        let split = body.left() + body.width() * t::PROJECT_PICKER_FRACTION;
        let left_rect =
            egui::Rect::from_min_max(body.min, egui::pos2(split - t::SPACE_8, body.bottom()));
        let right_rect =
            egui::Rect::from_min_max(egui::pos2(split + t::SPACE_8, body.top()), body.max);
        let mut left = bordered_panel(ui, "recent_panel", left_rect);
        left.label(t::heading("Recent projects", t::SECTION_TITLE));
        egui::ScrollArea::vertical()
            .id_salt("recent_projects")
            .show(&mut left, |ui| {
                for recent in &self.preferences.recent_projects {
                    let selected = self.project.root.as_ref() == Some(&recent.root);
                    let fill = if selected {
                        t::BG_SELECTED
                    } else {
                        t::BG_RAISED
                    };
                    egui::Frame::new()
                        .fill(fill)
                        .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                        .corner_radius(t::RADIUS_CARD)
                        .inner_margin(t::SPACE_8)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            if ui
                                .selectable_label(
                                    selected,
                                    t::text(&recent.name, t::BODY, t::TEXT_PRIMARY),
                                )
                                .clicked()
                            {
                                load = Some(recent.root.clone());
                            }
                            ui.label(t::text(
                                recent.root.display().to_string(),
                                t::CAPTION,
                                t::TEXT_MUTED,
                            ));
                            let date = chrono::DateTime::from_timestamp(recent.opened as i64, 0)
                                .map(|d| {
                                    d.with_timezone(&chrono::Local)
                                        .format("%d %b %Y, %H:%M")
                                        .to_string()
                                })
                                .unwrap_or_default();
                            ui.label(t::text(
                                format!("Last opened · {date}"),
                                t::CAPTION,
                                t::TEXT_MUTED,
                            ));
                        });
                    ui.add_space(t::SPACE_8);
                }
            });
        if t::secondary_button(&mut left, "Open project folder…", None, true).clicked() {
            self.choose_project(&ctx);
        }
        if self
            .project
            .snapshot
            .as_ref()
            .is_some_and(|s| s.report.toml.missing)
        {
            egui::Frame::new()
                .fill(t::BG_RAISED)
                .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                .corner_radius(t::RADIUS_CARD)
                .inner_margin(t::SPACE_12)
                .show(&mut left, |ui| {
                    ui.label(t::text(
                        "Not a scriptCast project",
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                    ui.label(t::text(
                        "This folder has no scriptcast.toml.",
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    if t::primary_button(ui, "Create scriptcast.toml here", None, true).clicked() {
                        create_toml = true;
                    }
                });
        }
        let mut right = bordered_panel(ui, "episode_panel", right_rect);
        right.label(t::heading("Episodes", t::SECTION_TITLE));
        if self.project.pending.is_some() {
            right.spinner();
            right.label(t::text("Loading project…", t::BODY, t::TEXT_MUTED));
        }
        for error in &self.project.error {
            right.label(t::text(error, t::CAPTION, t::STATUS_FAILED));
        }
        let summary_h = t::PROJECT_ROW_HEIGHT * 3.0;
        let table_h = (right.available_height() - summary_h).max(t::PROJECT_ROW_HEIGHT);
        egui::ScrollArea::vertical()
            .id_salt("project_episodes")
            .max_height(table_h)
            .show(&mut right, |ui| {
                episode_header_row(ui);
                for (index, row) in self.project.rows.iter().enumerate() {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), t::PROJECT_ROW_HEIGHT),
                        egui::Sense::click(),
                    );
                    if self.project.selected == Some(index) {
                        ui.painter()
                            .rect_filled(rect, t::RADIUS_CONTROL, t::BG_SELECTED);
                    }
                    if response.clicked() {
                        self.project.selected = Some(index);
                    }
                    if response.double_clicked() {
                        if let Some(output) = &row.output {
                            open = Some(output.clone());
                        } else {
                            new_script = Some(row.script.clone());
                        }
                    }
                    paint_episode_row(ui, rect, row);
                }
            });
        right.label(t::text(
            "Double-click an episode row to open it.",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        if let Some(row) = self.project.selected.and_then(|i| self.project.rows.get(i)) {
            egui::Frame::new()
                .fill(t::BG_RAISED)
                .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                .corner_radius(t::RADIUS_CARD)
                .inner_margin(t::SPACE_12)
                .show(&mut right, |ui| {
                    ui.label(t::heading(&row.id, t::SECTION_TITLE));
                    ui.label(t::text(
                        row.script.display().to_string(),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    ui.label(t::text(&row.verification, t::BODY, t::TEXT_PRIMARY));
                    status_chip(ui, row.status);
                });
        }
        if let Some(root) = load {
            self.load_project(root, &ctx);
        }
        if let Some(path) = open {
            self.project.picker = false;
            self.open(path, &ctx);
        }
        if let Some(script) = new_script {
            self.show_new_episode(Some(script));
        }
        if create_toml {
            self.create_scriptcast_toml(&ctx);
        }
    }
    fn project_header(&mut self, ui: &mut Ui, new_episode: bool) {
        let rect = ui.max_rect();
        crate::view::rule(ui, rect);
        let mut row = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(crate::view::inset(rect, t::SPACE_20, 0.0))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        row.spacing_mut().item_spacing.x = t::SPACE_12;
        row.add(
            egui::Image::new(t::icon("audio-lines-22-accent"))
                .fit_to_exact_size(egui::Vec2::splat(t::SPACE_22)),
        );
        let name = self
            .project
            .root
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Projects".into());
        row.label(t::heading(name, t::BRAND));
        if let Some(root) = &self.project.root {
            row.label(t::text(
                root.display().to_string(),
                t::CAPTION,
                t::TEXT_MUTED,
            ));
        }
        let rest = row.available_rect_before_wrap();
        let mut actions = row.new_child(
            egui::UiBuilder::new()
                .id_salt("project_header_actions")
                .max_rect(rest)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        actions.spacing_mut().item_spacing.x = t::SPACE_8;
        if new_episode && t::primary_button(&mut actions, "New episode…", None, true).clicked() {
            self.show_new_episode(None);
            if let Some(root) = self.project.root.clone() {
                self.apply_project_defaults(root);
            }
        }
        if !new_episode && t::secondary_button(&mut actions, "Close", None, true).clicked() {
            self.request_close_project_data();
        }
        if t::secondary_button(&mut actions, "Project settings", None, true).clicked() {
            self.settings_open = true;
            self.project.tab = "toml".into();
        }
    }
    pub(crate) fn request_close_project_data(&mut self) {
        if self.dirty_project_files().is_empty() {
            self.project.data_open = false;
        } else {
            self.project.close_prompt = true;
        }
    }
}

fn bordered_panel(ui: &mut Ui, id: &str, rect: egui::Rect) -> Ui {
    ui.painter()
        .rect_filled(rect, t::RADIUS_CARD, t::BG_SURFACE);
    ui.painter().rect_stroke(
        rect,
        t::RADIUS_CARD,
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        egui::StrokeKind::Inside,
    );
    let mut panel = crate::view::region(
        ui,
        id,
        crate::view::inset(rect, t::SPACE_12, t::SPACE_12),
        t::BG_SURFACE,
    );
    panel.set_width(panel.available_width());
    panel
}
fn episode_header_row(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), t::TABLE_HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    let labels = [
        "Episode id",
        "Script file",
        "Turns",
        "Duration",
        "Last rendered",
        "Verification",
        "Status",
    ];
    for (index, label) in labels.iter().enumerate() {
        let cell = column_rect(rect, index, &t::PROJECT_EPISODE_SPLIT);
        crate::view::line(ui, cell, label, t::font(t::CAPTION), t::TEXT_MUTED);
    }
}
fn paint_episode_row(ui: &Ui, rect: egui::Rect, row: &scriptcast_ui::project::ProjectEpisode) {
    let script = row.script.file_name().unwrap_or_default().to_string_lossy();
    let turns = row
        .turns
        .map(|n| n.to_string())
        .unwrap_or_else(|| "—".into());
    let duration = row
        .duration_ms
        .map(crate::view::duration)
        .unwrap_or_else(|| "—".into());
    let rendered = if row.rendered == "—" {
        "—".into()
    } else {
        project::format_rendered(&row.rendered)
    };
    let values = [
        row.id.as_str(),
        script.as_ref(),
        turns.as_str(),
        duration.as_str(),
        rendered.as_str(),
        row.verification.as_str(),
    ];
    for (index, value) in values.iter().enumerate() {
        let cell = column_rect(rect, index, &t::PROJECT_EPISODE_SPLIT);
        let first = value.lines().next().unwrap_or("");
        crate::view::line(ui, cell, first, t::font(t::BODY), t::TEXT_PRIMARY);
        if let Some(second) = value.lines().nth(1) {
            let below =
                egui::Rect::from_min_max(egui::pos2(cell.left(), cell.center().y), cell.max);
            crate::view::line(ui, below, second, t::font(t::CAPTION), t::TEXT_MUTED);
        }
    }
    let chip = column_rect(rect, 6, &t::PROJECT_EPISODE_SPLIT);
    paint_status_chip(ui, chip, row.status);
}
fn clip_column(rect: egui::Rect, index: usize) -> egui::Rect {
    column_rect(rect, index, &t::PROJECT_CLIP_SPLIT)
}
fn column_rect(rect: egui::Rect, index: usize, fractions: &[f32]) -> egui::Rect {
    let mut x = rect.left();
    for (i, fraction) in fractions.iter().enumerate() {
        let width = rect.width() * fraction;
        if i == index {
            return egui::Rect::from_min_size(
                egui::pos2(x, rect.top()),
                egui::vec2(width, rect.height()),
            );
        }
        x += width;
    }
    rect
}
fn paint_status_chip(ui: &Ui, rect: egui::Rect, status: scriptcast_ui::project::EpisodeStatus) {
    let (fill, colour) = match status {
        scriptcast_ui::project::EpisodeStatus::Rendered => (t::BG_CONTROL, t::STATUS_PASSED),
        scriptcast_ui::project::EpisodeStatus::Interrupted => (t::BG_DANGER, t::STATUS_FAILED),
        scriptcast_ui::project::EpisodeStatus::NotRendered => (t::BG_CONTROL, t::TEXT_MUTED),
    };
    let galley =
        ui.painter()
            .layout_no_wrap(status.label().to_owned(), t::font(t::CAPTION), colour);
    let chip = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - t::INLINE_BUTTON_HEIGHT / 2.0),
        egui::vec2(galley.size().x + t::SPACE_8 * 2.0, t::INLINE_BUTTON_HEIGHT),
    );
    ui.painter().rect_filled(chip, t::RADIUS_CONTROL, fill);
    ui.painter().galley(
        egui::pos2(
            chip.left() + t::SPACE_8,
            chip.center().y - galley.size().y / 2.0,
        ),
        galley,
        colour,
    );
}
fn status_chip(ui: &mut Ui, status: scriptcast_ui::project::EpisodeStatus) {
    let galley = ui.painter().layout_no_wrap(
        status.label().to_owned(),
        t::font(t::CAPTION),
        t::TEXT_PRIMARY,
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.size().x + t::SPACE_8 * 2.0, t::INLINE_BUTTON_HEIGHT),
        egui::Sense::hover(),
    );
    paint_status_chip(ui, rect, status);
}
pub(crate) fn stored_path(root: &std::path::Path, path: PathBuf) -> String {
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    path.strip_prefix(&root)
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
fn field_label(key: &str) -> &'static str {
    match key {
        "reference_audio" => "Reference audio",
        "reference_text" => "Reference text · required",
        "character_profile" => "Character profile · optional",
        "speed" => "Speed",
        "kokoro_voice" => "Kokoro voice",
        "lang_code" => "Language code",
        "exaggeration" => "Exaggeration",
        "temperature" => "Temperature",
        "cfg_weight" => "CFG weight",
        "elevenlabs_voice_id" => "ElevenLabs voice id",
        "name" => "Name",
        "word" => "Word",
        "citation" => "Citation",
        "origin_url" => "Origin URL",
        "content_note" => "Content note",
        "start" => "Start",
        "end" => "End",
        "voices" => "Voices",
        "pronunciations" => "Pronunciations",
        "speaker_rates" => "Speaker rates",
        "clips" => "Clips",
        "clip_sources" => "Clip sources",
        "references" => "References",
        "scripts" => "Scripts",
        "outputs" => "Outputs",
        "omnivoice_python" => "OmniVoice Python",
        "tail_ms" => "Tail (ms)",
        _ => "Field",
    }
}
fn engine_chip_style(engine: &str) -> (&'static str, egui::Color32, egui::Color32) {
    match engine {
        "omnivoice" => ("OmniVoice", t::accent(), t::ON_ACCENT),
        "archive" => ("archive", t::BG_CONTROL, t::TEXT_MUTED),
        "kokoro" | "mlx_kokoro" => ("Kokoro", t::BG_CONTROL, t::TEXT_PRIMARY),
        "mlx_chatterbox" => ("Chatterbox", t::BG_CONTROL, t::TEXT_PRIMARY),
        "elevenlabs" => ("ElevenLabs", t::BG_CONTROL, t::TEXT_PRIMARY),
        _ => ("engine", t::BG_CONTROL, t::TEXT_MUTED),
    }
}
/// Dot, name, chip, and turn count painted inside `row`. Returns the name and chip rects.
fn paint_speaker_row(
    ui: &Ui,
    row: egui::Rect,
    index: usize,
    name: &str,
    engine: &str,
    count: usize,
    invalid: bool,
) -> (egui::Rect, egui::Rect) {
    let dot = egui::pos2(row.left() + t::SPACE_12, row.center().y);
    ui.painter()
        .circle_filled(dot, t::SPACE_4, t::speaker_colour(index));
    let count_galley =
        ui.painter()
            .layout_no_wrap(count.to_string(), t::font(t::CAPTION), t::TEXT_MUTED);
    let count_rect = egui::Rect::from_min_size(
        egui::pos2(
            row.right() - t::SPACE_8 - count_galley.size().x,
            row.center().y - count_galley.size().y / 2.0,
        ),
        count_galley.size(),
    );
    ui.painter()
        .galley(count_rect.min, count_galley, t::TEXT_MUTED);
    let (label, fill, colour) = engine_chip_style(engine);
    let chip_galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), t::font(t::CAPTION), colour);
    let chip_size = egui::vec2(
        chip_galley.size().x + t::SPACE_8 * 2.0,
        t::INLINE_BUTTON_HEIGHT,
    );
    let mut chip_left = count_rect.left() - t::SPACE_8 - chip_size.x;
    if invalid {
        chip_left -= t::SPACE_8 + t::SPACE_8;
    }
    let chip_rect = egui::Rect::from_min_size(
        egui::pos2(chip_left, row.center().y - chip_size.y / 2.0),
        chip_size,
    );
    ui.painter().rect_filled(chip_rect, t::RADIUS_CONTROL, fill);
    ui.painter().galley(
        egui::pos2(
            chip_rect.left() + t::SPACE_8,
            chip_rect.center().y - chip_galley.size().y / 2.0,
        ),
        chip_galley,
        colour,
    );
    if invalid {
        ui.painter().circle_filled(
            egui::pos2(chip_rect.right() + t::SPACE_8, row.center().y),
            t::SPACE_4,
            t::STATUS_FAILED,
        );
    }
    let name_left = dot.x + t::SPACE_8;
    let name_rect = egui::Rect::from_min_max(
        egui::pos2(name_left, row.top()),
        egui::pos2((chip_rect.left() - t::SPACE_8).max(name_left), row.bottom()),
    );
    crate::view::line(ui, name_rect, name, t::font(t::BODY), t::TEXT_PRIMARY);
    (name_rect, chip_rect)
}
fn speed_slider(ui: &mut Ui, fields: &mut serde_json::Map<String, serde_json::Value>) {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, t::BUTTON_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(t::text(field_label("speed"), t::BODY, t::TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(t::text("1.0 · default", t::CAPTION, t::TEXT_MUTED));
            });
        },
    );
    let value_box = ui.spacing().interact_size.x;
    let track = (width - value_box - ui.spacing().item_spacing.x).max(t::SPACE_20);
    let previous = ui.spacing().slider_width;
    ui.spacing_mut().slider_width = track;
    let mut number = fields
        .get("speed")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(1.0);
    let response = ui.add(egui::Slider::new(&mut number, 0.5..=1.5).text(""));
    ui.spacing_mut().slider_width = previous;
    if response.changed() {
        fields.insert("speed".into(), serde_json::json!(number));
    }
    ui.allocate_ui_with_layout(
        egui::vec2(track, t::BUTTON_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(t::text("0.5", t::CAPTION, t::TEXT_MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(t::text("1.5", t::CAPTION, t::TEXT_MUTED));
            });
        },
    );
}
fn text_field(ui: &mut Ui, fields: &mut serde_json::Map<String, serde_json::Value>, key: &str) {
    ui.label(t::text(field_label(key), t::BODY, t::TEXT_PRIMARY));
    let mut value = fields
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    let response = if key.contains("text") || key == "content_note" {
        ui.add(egui::TextEdit::multiline(&mut value).desired_width(f32::INFINITY))
    } else {
        ui.add(egui::TextEdit::singleline(&mut value).desired_width(f32::INFINITY))
    };
    if response.changed() {
        fields.insert(key.into(), serde_json::json!(value));
    }
}
fn number_field(
    ui: &mut Ui,
    fields: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    default: f64,
) {
    ui.horizontal(|ui| {
        ui.label(t::text(field_label(key), t::BODY, t::TEXT_PRIMARY));
        let mut number = fields
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(default);
        let response = if key == "speed" {
            ui.add(egui::Slider::new(&mut number, 0.5..=1.5))
        } else {
            ui.add(egui::DragValue::new(&mut number).speed(0.01))
        };
        if response.changed() {
            fields.insert(key.into(), serde_json::json!(number));
        }
    });
}
impl Browser {
    pub(crate) fn project_data_screen(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !self.project.data_open {
            return;
        }
        if self
            .project
            .audition
            .as_ref()
            .is_some_and(|a| a.playing().is_some())
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
        if self.project.close_after_save && self.project.saving.is_none() {
            if let Some(file) = self.dirty_project_files().first().cloned() {
                self.save_project_tab(&file, &ctx);
            } else {
                self.project.close_after_save = false;
                self.project.close_prompt = false;
                self.project.data_open = false;
                if self.project.quit_after_close {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                return;
            }
        }
        let mut save = false;
        let mut reload = false;
        let mut tab = self.project.tab.clone();
        if tab == "toml" {
            tab = "voices".into();
        }
        let mut selected = self.project.select_id.clone();
        let mut draft = self.project.draft.take();
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, t::BG_WINDOW);
        let header =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), t::EPISODE_HEADER_HEIGHT));
        self.project_header(
            &mut crate::view::region(ui, "data_header", header, t::BG_SURFACE),
            false,
        );
        let tab_bar = egui::Rect::from_min_size(
            header.left_bottom(),
            egui::vec2(rect.width(), t::FILTER_BAR_HEIGHT),
        );
        let mut tabs = crate::view::region(ui, "data_tabs", tab_bar, t::BG_WINDOW);
        tabs.horizontal(|ui| {
            ui.add_space(t::SPACE_16);
            for (key, label) in [
                ("voices", "Voices"),
                ("clips", "Clips"),
                ("pronunciations", "Pronunciations"),
                ("accepted_hearings", "Accepted hearings"),
                ("speaker_rates", "Speaker rates"),
            ] {
                let active = tab == key;
                let text = t::text(
                    label,
                    t::BUTTON,
                    if active {
                        t::ON_ACCENT
                    } else {
                        t::TEXT_PRIMARY
                    },
                );
                if ui
                    .add(
                        egui::Button::new(text)
                            .fill(if active { t::accent() } else { t::BG_CONTROL })
                            .corner_radius(t::RADIUS_CONTROL),
                    )
                    .clicked()
                {
                    tab = key.into();
                }
            }
            if draft
                .as_ref()
                .zip(self.project.snapshot.as_ref())
                .is_some_and(|(d, s)| d != &s.report)
            {
                ui.label(t::text("Unsaved changes", t::CAPTION, t::accent()));
            }
        });
        let footer = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - t::PROJECT_FOOTER_HEIGHT),
            rect.max,
        );
        let content = egui::Rect::from_min_max(tab_bar.left_bottom(), footer.right_top());
        let mut content_ui = crate::view::region(
            ui,
            "data_body",
            crate::view::inset(content, t::SPACE_16, t::SPACE_12),
            t::BG_WINDOW,
        );
        {
            let ui = &mut content_ui;
            for message in &self.project.error {
                ui.colored_label(t::STATUS_FAILED, message);
            }
            if self.project.stale && ui.button("Reload from disk (discards drafts)").clicked() {
                reload = true;
            }
            if tab != self.project.tab {
                selected.clear();
            }
            if draft.is_none() {
                ui.label(if self.project.pending.is_some() {
                    "Loading project…"
                } else {
                    "Open a project first."
                });
            }
            let report = draft.as_mut();
            if let Some(report) = report {
                for message in &report.section(&tab).problems {
                    ui.colored_label(t::STATUS_FAILED, message);
                }
                if self.project.saving.is_some() {
                    ui.disable();
                }
                if self.phase_m_replaces(&tab) {
                    self.phase_m_fill(ui, report);
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| match tab.as_str() {
                            "voices" => self.voice_editor(ui, report, &mut selected),
                            "clips" => self.clip_editor(ui, report, &mut selected),
                            "accepted_hearings" => {
                                ui.label(t::text(
                                    "These hearings count as a match in every turn of the project.",
                                    t::CAPTION,
                                    t::TEXT_MUTED,
                                ));
                                ui.horizontal(|ui| {
                                    for label in
                                        ["Script word", "Heard as", "Review date", "Review risk"]
                                    {
                                        ui.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
                                    }
                                });
                                for entry in &mut report.accepted_hearings.entries {
                                    let word = entry["word"].as_str().unwrap_or("").to_owned();
                                    let date = entry["review_ts"]
                                        .as_str()
                                        .map(project::format_review_date)
                                        .unwrap_or_else(|| "—".into());
                                    let risky = entry["risky"].as_bool().unwrap_or(false);
                                    if let Some(heard) = entry["heard"].as_array_mut() {
                                        heard.retain(|hearing| {
                                            let mut delete = false;
                                            ui.horizontal(|ui| {
                                                ui.label(t::text(&word, t::BODY, t::TEXT_PRIMARY));
                                                ui.label(t::text(
                                                    hearing.as_str().unwrap_or(""),
                                                    t::BODY,
                                                    t::TEXT_PRIMARY,
                                                ));
                                                ui.label(t::text(&date, t::BODY, t::TEXT_MUTED));
                                                if risky {
                                                    ui.label(t::text(
                                                        "Common word",
                                                        t::CAPTION,
                                                        t::STATUS_FAILED,
                                                    ));
                                                } else {
                                                    ui.label(t::text(
                                                        "Reviewed",
                                                        t::CAPTION,
                                                        t::TEXT_MUTED,
                                                    ));
                                                }
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        delete = ui
                                                            .add(
                                                                egui::Button::new(t::text(
                                                                    "Delete",
                                                                    t::BUTTON,
                                                                    t::STATUS_FAILED,
                                                                ))
                                                                .fill(t::BG_WINDOW)
                                                                .stroke(egui::Stroke::new(
                                                                    t::BORDER,
                                                                    t::STATUS_FAILED,
                                                                )),
                                                            )
                                                            .clicked();
                                                    },
                                                );
                                            });
                                            !delete
                                        });
                                    }
                                }
                                report.accepted_hearings.entries.retain(|entry| {
                                    entry["heard"]
                                        .as_array()
                                        .is_some_and(|heard| !heard.is_empty())
                                });
                            }
                            "pronunciations" => {
                                if t::secondary_button(ui, "Calibrate…", None, true).clicked() {
                                    self.makers.calibrate = true;
                                }
                                report.pronunciations.entries.retain_mut(|entry| {
                                    let mut delete = false;
                                    ui.group(|ui| {
                                        text_field(ui, entry.as_object_mut().unwrap(), "word");
                                        let word = entry["word"].as_str().unwrap_or("").to_owned();
                                        if let Some(readings) = entry["readings"].as_object_mut() {
                                            egui::Grid::new(("pron_readings", word))
                                                .striped(true)
                                                .show(ui, |ui| {
                                                    ui.label(t::text(
                                                        "Reading key",
                                                        t::CAPTION,
                                                        t::TEXT_MUTED,
                                                    ));
                                                    ui.label(t::text(
                                                        "Spoken as",
                                                        t::CAPTION,
                                                        t::TEXT_MUTED,
                                                    ));
                                                    ui.label(t::text(
                                                        "",
                                                        t::CAPTION,
                                                        t::TEXT_MUTED,
                                                    ));
                                                    ui.end_row();
                                                    for (key, value) in readings.clone() {
                                                        let mut next_key = key.clone();
                                                        let mut text =
                                                            value.as_str().unwrap_or("").to_owned();
                                                        let renamed = ui
                                                            .text_edit_singleline(&mut next_key)
                                                            .changed();
                                                        let edited = ui
                                                            .text_edit_singleline(&mut text)
                                                            .changed();
                                                        let remove = ui.button("Delete").clicked();
                                                        ui.end_row();
                                                        if remove {
                                                            readings.remove(&key);
                                                        } else if renamed
                                                            && next_key != key
                                                            && readings.contains_key(&next_key)
                                                        {
                                                            ui.colored_label(
                                                                t::STATUS_FAILED,
                                                                "Reading already exists.",
                                                            );
                                                        } else if renamed || edited {
                                                            readings.remove(&key);
                                                            readings.insert(next_key, text.into());
                                                        }
                                                    }
                                                });
                                            if ui.button("+ Add reading").clicked() {
                                                readings.insert(
                                                    format!("reading_{}", readings.len() + 1),
                                                    "".into(),
                                                );
                                            }
                                        }
                                        delete = ui.button("Delete entry").clicked();
                                    });
                                    !delete
                                });
                                if ui.button("+ Add word").clicked() {
                                    report.pronunciations.entries.push(
                                        serde_json::json!({"word":"", "readings":{"default":""}}),
                                    );
                                }
                            }
                            "speaker_rates" => {
                                ui.label("Read-only");
                                egui::Grid::new("rates").striped(true).show(ui, |ui| {
                                    for (speaker, rate) in &report.speaker_rates.rates {
                                        ui.label(speaker);
                                        ui.label(rate.to_string());
                                        ui.end_row();
                                    }
                                });
                            }
                            "toml" => toml_fields(ui, report),
                            _ => {}
                        });
                }
            }
        }
        let save_name = draft
            .as_ref()
            .map(|report| {
                report
                    .section(&tab)
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| tab.clone())
            })
            .unwrap_or_else(|| tab.clone());
        let reason = draft.as_ref().and_then(|report| {
            self.project
                .snapshot
                .as_ref()
                .and_then(|snapshot| save_reason(&tab, &snapshot.report, report))
        });
        let dirty_tab = draft.as_ref().is_some_and(|report| {
            self.project
                .snapshot
                .as_ref()
                .is_some_and(|s| s.report.section(&tab) != report.section(&tab))
        });
        let mut footer_ui = crate::view::region(ui, "data_footer", footer, t::BG_SURFACE);
        if self.phase_m_replaces(&tab) {
            self.project.draft = draft;
            self.phase_m_footer(&mut footer_ui);
            draft = self.project.draft.take();
        } else {
            footer_ui.horizontal(|ui| {
                ui.add_space(t::SPACE_16);
                ui.label(t::text(
                    "Comments in the file are kept.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                if let Some(reason) = &reason {
                    ui.label(t::text(reason, t::CAPTION, t::STATUS_FAILED));
                }
                if self.project.stale
                    && t::secondary_button(ui, "Reload from disk", None, true).clicked()
                {
                    reload = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    save = t::primary_button(
                        ui,
                        &format!("Save {save_name}"),
                        None,
                        reason.is_none() && dirty_tab && self.project.saving.is_none(),
                    )
                    .clicked();
                    if t::secondary_button(ui, "Revert", None, true).clicked()
                        && let Some(snapshot) = &self.project.snapshot
                        && let Some(report) = draft.as_mut()
                    {
                        match tab.as_str() {
                            "voices" => report.voices = snapshot.report.voices.clone(),
                            "clips" => report.clips = snapshot.report.clips.clone(),
                            "pronunciations" => {
                                report.pronunciations = snapshot.report.pronunciations.clone()
                            }
                            "accepted_hearings" => {
                                report.accepted_hearings = snapshot.report.accepted_hearings.clone()
                            }
                            "toml" => report.toml = snapshot.report.toml.clone(),
                            _ => {}
                        }
                    }
                });
            });
        }
        self.project.draft = draft;
        self.project.tab = tab;
        self.project.select_id = selected;
        if save {
            self.save_project_tab(&self.project.tab.clone(), &ctx);
        }
        if reload && let Some(root) = self.project.root.clone() {
            self.project.stale = false;
            self.project.draft = self.project.snapshot.as_ref().map(|s| s.report.clone());
            self.load_project(root, &ctx);
        }
        self.project_close_dialog(&ctx);
        self.project_voice_dialogs(&ctx);
    }
    fn voice_editor(&mut self, ui: &mut Ui, report: &mut Report, selected: &mut String) {
        bordered_columns(
            ui,
            t::PROJECT_VOICES_FRACTION,
            ["speakers_list", "speaker_form"],
            |columns| {
                let invalid_count = report
                    .voices
                    .speakers
                    .iter()
                    .filter(|speaker| {
                        speaker["fields"]
                            .as_object()
                            .is_some_and(|fields| !project::required_errors(fields).is_empty())
                            || speaker["problems"]
                                .as_array()
                                .is_some_and(|problems| !problems.is_empty())
                    })
                    .count();
                columns[0].label(t::text(
                    format!(
                        "Speakers  {} · {invalid_count} invalid",
                        report.voices.speakers.len()
                    ),
                    t::BODY,
                    t::TEXT_PRIMARY,
                ));
                self.project.speaker_rows.clear();
                for (index, speaker) in report.voices.speakers.iter().enumerate() {
                    let id = speaker["id"].as_str().unwrap_or("");
                    let fields = speaker["fields"].as_object();
                    let invalid = fields.is_some_and(|f| !project::required_errors(f).is_empty())
                        || speaker["problems"]
                            .as_array()
                            .is_some_and(|p| !p.is_empty());
                    let count = self
                        .episode
                        .as_ref()
                        .map(|episode| {
                            episode
                                .turns
                                .iter()
                                .filter(|turn| episode.speakers[turn.speaker_idx].id == id)
                                .count()
                        })
                        .unwrap_or(0);
                    let width = columns[0].available_width();
                    let (rect, response) = columns[0].allocate_exact_size(
                        egui::vec2(width, t::PROJECT_ROW_HEIGHT),
                        egui::Sense::click(),
                    );
                    if selected == id {
                        columns[0]
                            .painter()
                            .rect_filled(rect, t::RADIUS_CONTROL, t::BG_SELECTED);
                    }
                    if response.clicked() {
                        *selected = id.into();
                    }
                    let name = fields
                        .and_then(|f| f.get("name"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(id);
                    let engine = fields
                        .and_then(|f| f.get("engine"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("kokoro");
                    let hits =
                        paint_speaker_row(&columns[0], rect, index, name, engine, count, invalid);
                    self.project.speaker_rows.push(hits);
                }
                if columns[0].button("+ Add speaker").clicked() {
                    let mut n = report.voices.speakers.len() + 1;
                    while report
                        .voices
                        .speakers
                        .iter()
                        .any(|s| s["id"] == format!("speaker_{n}"))
                    {
                        n += 1;
                    }
                    let id = format!("speaker_{n}");
                    report.voices.speakers.push(
                        serde_json::json!({"id":id, "fields":{"name":"", "engine":"omnivoice"}}),
                    );
                    *selected = id;
                }
                if selected.is_empty()
                    && let Some(first) = report.voices.speakers.first()
                {
                    *selected = first["id"].as_str().unwrap_or("").into();
                }
                let Some(speaker) = report
                    .voices
                    .speakers
                    .iter_mut()
                    .find(|s| s["id"].as_str() == Some(selected))
                else {
                    return;
                };
                let Some(fields) = speaker["fields"].as_object_mut() else {
                    return;
                };
                let ui = &mut columns[1];
                let id_width = ui.available_width();
                ui.allocate_ui_with_layout(
                    egui::vec2(id_width, t::BUTTON_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(t::text(
                            format!("Id · {selected}"),
                            t::BODY,
                            t::TEXT_PRIMARY,
                        ));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if t::secondary_button(ui, "Rename…", None, true).clicked() {
                                self.project.rename = Some((selected.clone(), selected.clone()));
                            }
                        });
                    },
                );
                text_field(ui, fields, "name");
                let engine = fields
                    .get("engine")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("kokoro")
                    .to_owned();
                let mut next_engine = engine.clone();
                egui::ComboBox::from_id_salt("speaker_engine")
                    .selected_text(&engine)
                    .show_ui(ui, |ui| {
                        for (key, label) in [
                            ("omnivoice", "OmniVoice"),
                            ("kokoro", "Kokoro"),
                            ("mlx_chatterbox", "Chatterbox"),
                            ("elevenlabs", "ElevenLabs"),
                            ("archive", "Archive"),
                        ] {
                            ui.selectable_value(&mut next_engine, key.into(), label);
                        }
                    });
                if next_engine != engine {
                    self.project.engine_change = Some((
                        selected.clone(),
                        next_engine.clone(),
                        project::removed_engine_fields(fields, &next_engine),
                    ));
                }
                if engine == "archive" {
                    ui.label("Archive speakers play registered clips; no voice fields.");
                }
                for key in project::engine_fields(&engine) {
                    if [
                        "name",
                        "engine",
                        "reference_audio_expressive",
                        "reference_text_expressive",
                    ]
                    .contains(key)
                    {
                        continue;
                    }
                    if *key == "reference_audio" {
                        ui.label(t::text(field_label(key), t::BODY, t::TEXT_PRIMARY));
                        let mut value = fields
                            .get(*key)
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("")
                            .to_owned();
                        let path = report.root.join(&value);
                        let duration = self.reference_duration(&path);
                        let row_width = ui.available_width();
                        ui.allocate_ui_with_layout(
                            egui::vec2(row_width, t::BUTTON_HEIGHT),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if let Some(duration) = duration {
                                    ui.label(t::text(duration, t::CAPTION, t::TEXT_MUTED));
                                }
                                if ui
                                    .add_enabled(path.is_file(), egui::Button::new("Play"))
                                    .clicked()
                                {
                                    self.audition_project_media(&path, 0.0, None);
                                }
                                if t::secondary_button(ui, "Choose…", None, true).clicked()
                                    && let Some(picked) = rfd::FileDialog::new()
                                        .set_directory(&report.root)
                                        .pick_file()
                                {
                                    let stored = stored_path(&report.root, picked);
                                    fields.insert((*key).into(), stored.clone().into());
                                    value = stored;
                                }
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut value)
                                            .desired_width(ui.available_width()),
                                    )
                                    .changed()
                                {
                                    fields.insert((*key).into(), value.clone().into());
                                }
                            },
                        );
                        ui.horizontal(|ui| {
                            if t::secondary_button(ui, "Make reference…", None, true).clicked() {
                                self.makers.reference = true;
                                self.makers.ref_source = path.display().to_string();
                            }
                            if scriptcast_ui::project::engine_fields(&engine)
                                .contains(&"reference_audio")
                                && t::secondary_button(ui, "Audition…", None, true).clicked()
                            {
                                self.makers.audition = true;
                                self.makers.audition_speaker = selected.clone();
                            }
                        });
                        self.load_project_media(path.clone(), None, ui.ctx());
                        if let Some(Err(error)) = &self.project.media {
                            ui.label(t::text(
                                format!("Cannot decode reference audio: {error}"),
                                t::CAPTION,
                                t::STATUS_FAILED,
                            ));
                        }
                    } else if *key == "speed" {
                        speed_slider(ui, fields);
                    } else {
                        match *key {
                            "exaggeration" => number_field(ui, fields, key, 0.0),
                            "temperature" => number_field(ui, fields, key, 0.6),
                            "cfg_weight" => number_field(ui, fields, key, 0.7),
                            _ => text_field(ui, fields, key),
                        }
                    }
                    if let Some(error) = project::required_errors(fields).get(*key) {
                        ui.colored_label(t::STATUS_FAILED, error);
                    }
                }
                if engine == "omnivoice" {
                    ui.collapsing("Expressive reference audio + text · optional pair", |ui| {
                        text_field(ui, fields, "reference_audio_expressive");
                        text_field(ui, fields, "reference_text_expressive");
                    });
                }
            },
        );
    }
    fn clip_editor(&mut self, ui: &mut Ui, report: &mut Report, selected: &mut String) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.project.search)
                    .hint_text("Search id or source file…"),
            );
            if ui.button("+ Register clip…").clicked() {
                self.makers.clip = true;
            }
        });
        let query = self.project.search.to_lowercase();
        if selected.is_empty()
            && let Some(clip) = report.clips.clips.first()
        {
            *selected = clip["id"].as_str().unwrap_or("").into();
        }
        two_columns(ui, t::PROJECT_CLIPS_FRACTION, |columns| {
            egui::ScrollArea::vertical()
                .id_salt("clips_table")
                .show(&mut columns[0], |ui| {
                    let (header, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), t::TABLE_HEADER_HEIGHT),
                        egui::Sense::hover(),
                    );
                    for (index, label) in [
                        "Id",
                        "Source file",
                        "Start",
                        "End",
                        "Duration",
                        "Citation",
                        "Play",
                    ]
                    .iter()
                    .enumerate()
                    {
                        crate::view::line(
                            ui,
                            clip_column(header, index),
                            label,
                            t::font(t::CAPTION),
                            t::TEXT_MUTED,
                        );
                    }
                    for clip in &report.clips.clips {
                        let id = clip["id"].as_str().unwrap_or("");
                        let source = clip["source"].as_str().unwrap_or("");
                        if !format!("{id} {source}").to_lowercase().contains(&query) {
                            continue;
                        }
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), t::PROJECT_ROW_HEIGHT),
                            egui::Sense::click(),
                        );
                        if selected == id {
                            ui.painter()
                                .rect_filled(rect, t::RADIUS_CONTROL, t::BG_SELECTED);
                        }
                        if response.clicked() {
                            *selected = id.into();
                        }
                        let start = clip["start"].as_f64().unwrap_or(0.0);
                        let end = clip["end"].as_f64().unwrap_or(start);
                        let missing = clip["source_exists"] == false;
                        let cells = [
                            id.to_owned(),
                            source.to_owned(),
                            format!("{start:.3}"),
                            format!("{end:.3}"),
                            format!("{:.1} s", end - start),
                            clip["citation"].as_str().unwrap_or("—").to_owned(),
                        ];
                        for (index, value) in cells.iter().enumerate() {
                            let colour = if index == 1 && missing {
                                t::STATUS_FAILED
                            } else {
                                t::TEXT_PRIMARY
                            };
                            crate::view::line(
                                ui,
                                clip_column(rect, index),
                                value,
                                t::font(t::BODY),
                                colour,
                            );
                        }
                        if missing {
                            let note = clip_column(rect, 1);
                            let below = egui::Rect::from_min_max(
                                egui::pos2(note.left(), note.center().y),
                                note.max,
                            );
                            crate::view::line(
                                ui,
                                below,
                                "Missing source",
                                t::font(t::CAPTION),
                                t::STATUS_FAILED,
                            );
                        }
                        let play = clip_column(rect, 6);
                        let mut play_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .id_salt(("clip_play", id))
                                .max_rect(play),
                        );
                        if play_ui
                            .add_enabled(clip["source_exists"] == true, egui::Button::new("Play"))
                            .on_disabled_hover_text("Missing source")
                            .clicked()
                        {
                            self.audition_project_media(
                                &report.root.join(source),
                                start,
                                Some(end),
                            );
                        }
                    }
                });
            let ui = &mut columns[1];
            if let Some(clip) = report
                .clips
                .clips
                .iter_mut()
                .find(|c| c["id"].as_str() == Some(selected))
            {
                ui.heading(selected.as_str());
                ui.label(t::text(
                    clip["source"].as_str().unwrap_or(""),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                let source = clip["fields"]
                    .get("source")
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| clip["source"].as_str())
                    .unwrap_or("")
                    .to_owned();
                let path = report.root.join(&source);
                let mut start = clip["fields"]["start"]
                    .as_f64()
                    .or_else(|| clip["start"].as_f64())
                    .unwrap_or(0.0);
                let mut end = clip["fields"]["end"]
                    .as_f64()
                    .or_else(|| clip["end"].as_f64())
                    .unwrap_or(0.0);
                self.load_project_media(path.clone(), Some((start, end)), ui.ctx());
                let dragged = self.clip_waveform(ui, &mut start, &mut end);
                let before_fields = clip.get("fields").cloned();
                if let Some(fields) = clip.get_mut("fields").and_then(|v| v.as_object_mut()) {
                    if dragged {
                        fields.insert("start".into(), start.into());
                        fields.insert("end".into(), end.into());
                    }
                    number_field(ui, fields, "start", 0.0);
                    number_field(ui, fields, "end", 0.0);
                    for key in ["citation", "origin_url", "content_note"] {
                        text_field(ui, fields, key);
                    }
                    start = fields
                        .get("start")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(start);
                    end = fields
                        .get("end")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(end);
                }
                if clip.get("fields") != before_fields.as_ref()
                    && let Some(fields) = clip.get("fields").cloned()
                    && let Some(object) = clip.as_object_mut()
                {
                    for key in [
                        "source",
                        "start",
                        "end",
                        "citation",
                        "origin_url",
                        "content_note",
                    ] {
                        if let Some(value) = fields.get(key) {
                            object.insert((*key).into(), value.clone());
                        }
                    }
                }
                self.media_play_button(ui, &path, start, Some(end));
                if start < 0.0 || end <= start {
                    ui.colored_label(
                        t::STATUS_FAILED,
                        "End must be after start; start must be nonnegative.",
                    );
                }
                ui.label(t::text("Used in:", t::BODY, t::TEXT_PRIMARY));
                let uses = self.clip_usages(selected);
                egui::Frame::new()
                    .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                    .corner_radius(t::RADIUS_CONTROL)
                    .inner_margin(t::SPACE_8)
                    .show(ui, |ui| {
                        if uses.is_empty() {
                            ui.label(t::text(
                                "Open an episode to see its clip turns.",
                                t::CAPTION,
                                t::TEXT_MUTED,
                            ));
                        }
                        for line in uses {
                            ui.label(t::text(line, t::BODY, t::TEXT_PRIMARY));
                        }
                    });
            }
        });
    }
}

fn save_reason(tab: &str, before: &Report, report: &Report) -> Option<String> {
    if tab == "toml"
        && project::FILES
            .iter()
            .any(|file| *file != "toml" && before.section(file) != report.section(file))
    {
        return Some("Save or revert other tabs before changing project paths.".into());
    }
    if tab == "clips" {
        for clip in &report.clips.clips {
            let start = clip["fields"]["start"]
                .as_f64()
                .or_else(|| clip["start"].as_f64())
                .unwrap_or(0.0);
            let end = clip["fields"]["end"]
                .as_f64()
                .or_else(|| clip["end"].as_f64())
                .unwrap_or(0.0);
            if start < 0.0 || end <= start {
                return Some("End must be after start; start must be nonnegative.".into());
            }
        }
    }
    if tab == "voices" {
        for speaker in &report.voices.speakers {
            if let Some(fields) = speaker["fields"].as_object()
                && let Some(error) = project::required_errors(fields).values().next()
            {
                return Some(error.clone());
            }
        }
    }
    project::small_list_ops(tab, before.section(tab), report.section(tab)).err()
}
impl Browser {
    pub(crate) fn dirty_project_files(&self) -> Vec<String> {
        self.project
            .draft
            .as_ref()
            .zip(self.project.snapshot.as_ref())
            .map(|(draft, snapshot)| {
                project::FILES
                    .iter()
                    .filter(|file| draft.section(file) != snapshot.report.section(file))
                    .map(|file| (*file).into())
                    .collect()
            })
            .map(|mut files: Vec<String>| {
                files.sort_by_key(|file| file == "toml");
                files
            })
            .unwrap_or_default()
    }
    fn save_project_tab(&mut self, file: &str, ctx: &egui::Context) {
        let Some(snapshot) = self.project.snapshot.clone() else {
            return;
        };
        let Some(draft) = &self.project.draft else {
            return;
        };
        if let Some(reason) = save_reason(file, &snapshot.report, draft) {
            self.project.error = vec![reason];
            self.project.close_after_save = false;
            return;
        }
        let ops =
            match project::small_list_ops(file, snapshot.report.section(file), draft.section(file))
            {
                Ok(ops) => ops,
                Err(error) => {
                    self.project.error = vec![error];
                    self.project.close_after_save = false;
                    return;
                }
            };
        let (tx, rx) = mpsc::channel();
        self.project.saving = Some(rx);
        self.project.saving_file = file.into();
        let file = file.to_owned();
        let folder = self.preferences.cli_dir.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = cli::resolve(
                "scriptcast-project",
                Some(&snapshot.report.root),
                folder.as_deref(),
            )
            .map_err(ToolError::Failed)
            .and_then(|resolved| project::edit(&resolved, &snapshot, &file, &ops));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    fn project_close_dialog(&mut self, ctx: &egui::Context) {
        if !self.project.close_prompt {
            return;
        }
        egui::Modal::new(egui::Id::new("close_project_data")).show(ctx, |ui| {
            ui.heading("Save project changes?");
            ui.label("Unsaved changes will be lost if you discard them.");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.project.saving.is_none(), egui::Button::new("Save"))
                    .clicked()
                {
                    self.project.close_after_save = true;
                }
                if ui
                    .add_enabled(self.project.saving.is_none(), egui::Button::new("Discard"))
                    .clicked()
                {
                    self.project.draft = self.project.snapshot.as_ref().map(|s| s.report.clone());
                    self.project.data_open = false;
                    self.project.close_prompt = false;
                    if self.project.quit_after_close {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.project.close_prompt = false;
                    self.project.quit_after_close = false;
                }
            });
        });
    }
}

pub(crate) fn bordered_columns(
    ui: &mut Ui,
    fraction: f32,
    ids: [&str; 2],
    body: impl FnOnce(&mut [Ui; 2]),
) {
    let rect = ui.available_rect_before_wrap();
    let split = rect.left() + rect.width() * fraction;
    let left = egui::Rect::from_min_max(rect.min, egui::pos2(split - t::SPACE_8, rect.bottom()));
    let right = egui::Rect::from_min_max(egui::pos2(split + t::SPACE_8, rect.top()), rect.max);
    let mut columns = [
        bordered_panel(ui, ids[0], left),
        bordered_panel(ui, ids[1], right),
    ];
    body(&mut columns);
    ui.advance_cursor_after_rect(rect);
}

pub(crate) fn tool_footer(ui: &mut Ui, status: &str, primary: &str, enabled: bool) -> (bool, bool) {
    let mut cancel = false;
    let mut go = false;
    ui.horizontal(|ui| {
        ui.add_space(t::SPACE_16);
        ui.label(t::text(status, t::CAPTION, t::TEXT_MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            go = t::primary_button(ui, primary, None, enabled).clicked();
            cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
        });
    });
    (cancel, go)
}

fn media_request_key(path: &std::path::Path, window: Option<(f64, f64)>) -> (PathBuf, i64, i64) {
    match window {
        Some((start, end)) => (
            path.to_path_buf(),
            (start * 1000.0).round() as i64,
            (end * 1000.0).round() as i64,
        ),
        None => (path.to_path_buf(), 0, -1),
    }
}
pub(crate) fn two_columns(ui: &mut Ui, fraction: f32, body: impl FnOnce(&mut [Ui; 2])) {
    let rect = ui.available_rect_before_wrap();
    let split = rect.left() + rect.width() * fraction;
    let left = egui::Rect::from_min_max(rect.min, egui::pos2(split - t::SPACE_8, rect.bottom()));
    let right = egui::Rect::from_min_max(egui::pos2(split + t::SPACE_8, rect.top()), rect.max);
    let mut columns = [
        ui.new_child(egui::UiBuilder::new().id_salt("left").max_rect(left)),
        ui.new_child(egui::UiBuilder::new().id_salt("right").max_rect(right)),
    ];
    body(&mut columns);
    let used = columns[0].min_rect().union(columns[1].min_rect());
    ui.advance_cursor_after_rect(used);
}
impl Browser {
    fn clip_usages(&self, clip_id: &str) -> Vec<String> {
        let Some(episode) = &self.episode else {
            return vec![];
        };
        episode
            .turns
            .iter()
            .filter(|turn| scriptcast_ui::script::clip_id(&turn.first_words) == Some(clip_id))
            .map(|turn| {
                let name = episode
                    .speakers
                    .get(turn.speaker_idx)
                    .map(|speaker| speaker.name.as_str())
                    .unwrap_or("");
                format!(
                    "{:03}  {} · {name}",
                    turn.index, episode.manifest.episode_id
                )
            })
            .collect()
    }
    fn reference_duration(&self, path: &std::path::Path) -> Option<String> {
        if self.project.media_key.as_ref() != Some(&media_request_key(path, None)) {
            return None;
        }
        let envelope = self.project.media.as_ref()?.as_ref().ok()?;
        Some(format!("{:.1} s", envelope.duration_secs()))
    }
    fn load_project_media(
        &mut self,
        path: PathBuf,
        window: Option<(f64, f64)>,
        ctx: &egui::Context,
    ) {
        let key = media_request_key(&path, window);
        if self.project.media_key.as_ref() == Some(&key)
            && (self.project.media_pending.is_some() || self.project.media.is_some())
        {
            return;
        }
        self.project.media_path = Some(path.clone());
        self.project.media_key = Some(key);
        self.project.decode_requests += 1;
        self.project.media = None;
        self.project.media_span = None;
        let (tx, rx) = mpsc::channel();
        self.project.media_pending = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = if let Some((start, end)) = window {
                scriptcast_ui::waveform::clip_envelope(&path, start, end)
            } else {
                scriptcast_ui::waveform::envelope(&[path]).map(|envelope| (envelope, 0.0))
            };
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(crate) fn audition_project_media(
        &mut self,
        path: &std::path::Path,
        start: f64,
        end: Option<f64>,
    ) {
        if !start.is_finite() || start < 0.0 {
            return;
        }
        if self.project.audition.is_none() {
            match scriptcast_ui::audio::AudioOut::open(self.preferences.output_device.as_deref()) {
                Ok(mut audio) => {
                    audio.set_muted(self.muted);
                    self.project.audition = Some(audio);
                }
                Err(error) => {
                    self.project.error = vec![error];
                    return;
                }
            }
        }
        if let Some(audio) = &mut self.project.audition {
            match audio.play(0, &[path.to_path_buf()]) {
                Ok(()) => {
                    audio.seek(std::time::Duration::from_secs_f64(start));
                    self.project.audition_end = end.unwrap_or(f64::INFINITY);
                }
                Err(error) => {
                    let Some(stop) = end else {
                        self.project.error = vec![error];
                        return;
                    };
                    match scriptcast_ui::waveform::ffmpeg_wav(path, start, stop - start) {
                        Ok(bytes) => match audio.play_wav_bytes(bytes) {
                            Ok(()) => self.project.audition_end = f64::INFINITY,
                            Err(play) => self.project.error = vec![play],
                        },
                        Err(ffmpeg) => self.project.error = vec![format!("{error}; {ffmpeg}")],
                    }
                }
            }
        }
    }
    fn media_play_button(
        &mut self,
        ui: &mut Ui,
        path: &std::path::Path,
        start: f64,
        end: Option<f64>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    path.is_file() && end.is_none_or(|end| end > start),
                    egui::Button::new("Play"),
                )
                .clicked()
            {
                self.audition_project_media(path, start, end);
            }
            if ui.button("Stop").clicked()
                && let Some(audio) = &mut self.project.audition
            {
                audio.stop();
            }
            if let Some(Ok(envelope)) = &self.project.media {
                ui.label(format!("{:.1} s", envelope.duration_secs()));
            }
        });
    }
    fn clip_waveform(&mut self, ui: &mut Ui, start: &mut f64, end: &mut f64) -> bool {
        let envelope = match &self.project.media {
            Some(Ok(envelope)) if !envelope.is_empty() => envelope,
            Some(Err(error)) => {
                ui.label(format!("Cannot decode source audio: {error}"));
                return false;
            }
            Some(Ok(_)) => {
                ui.label("Source audio contains no samples.");
                return false;
            }
            None => {
                ui.label("Decoding source waveform…");
                return false;
            }
        };
        let origin = self.project.media_origin;
        let duration = envelope
            .duration_secs()
            .max(f64::from(scriptcast_ui::waveform::BUCKET_SECS));
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), t::PROJECT_WAVE_HEIGHT),
            egui::Sense::hover(),
        );
        ui.painter()
            .rect_filled(rect, t::RADIUS_CONTROL, t::BG_CONTROL);
        let bars = scriptcast_ui::waveform::bars(
            envelope,
            (rect.width() / (t::WAVE_BAR + t::WAVE_GAP)) as usize,
        );
        for (i, peak) in bars.iter().enumerate() {
            let x = rect.left() + i as f32 * (t::WAVE_BAR + t::WAVE_GAP);
            let half = peak * rect.height() * t::PROJECT_WAVE_AMPLITUDE;
            ui.painter().line_segment(
                [
                    egui::pos2(x, rect.center().y - half),
                    egui::pos2(x, rect.center().y + half),
                ],
                egui::Stroke::new(t::WAVE_BAR, t::TEXT_MUTED),
            );
        }
        let x_at = |time: f64| {
            rect.left() + ((time - origin) / duration).clamp(0.0, 1.0) as f32 * rect.width()
        };
        let x0 = x_at(*start);
        let x1 = x_at(*end);
        ui.painter().rect_stroke(
            egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom())),
            t::RADIUS_CONTROL,
            egui::Stroke::new(t::BORDER, t::accent()),
            egui::StrokeKind::Inside,
        );
        let mut changed = false;
        for (key, x) in [("start", x0), ("end", x1)] {
            let handle = egui::Rect::from_center_size(
                egui::pos2(x, rect.center().y),
                egui::vec2(t::SPLITTER_HIT, rect.height()),
            );
            ui.painter()
                .rect_filled(handle, t::RADIUS_CONTROL, t::accent());
            let response = ui.interact(handle, ui.id().with(key), egui::Sense::drag());
            if response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let seconds = origin
                    + f64::from(((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0)) * duration;
                if key == "start" {
                    *start = seconds.min(*end);
                } else {
                    *end = seconds.max(*start);
                }
                changed = true;
            }
        }
        changed
    }
}

impl Browser {
    fn project_voice_dialogs(&mut self, ctx: &egui::Context) {
        if let Some((id, engine, removed)) = self.project.engine_change.clone() {
            egui::Modal::new(egui::Id::new("engine_change")).show(ctx, |ui| {
                ui.heading(format!("Change engine to {engine}?"));
                ui.label("Fields to remove:");
                if removed.is_empty() {
                    ui.label("None");
                }
                for field in &removed {
                    ui.label(field);
                }
                ui.horizontal(|ui| {
                    if ui.button("Change engine").clicked() {
                        if let Some(speaker) = self.project.draft.as_mut().and_then(|r| {
                            r.voices
                                .speakers
                                .iter_mut()
                                .find(|s| s["id"].as_str() == Some(&id))
                        }) && let Some(fields) = speaker["fields"].as_object_mut()
                        {
                            fields.retain(|key, _| {
                                project::engine_fields(&engine).contains(&key.as_str())
                            });
                            fields.insert("engine".into(), engine.clone().into());
                        }
                        self.project.engine_change = None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.project.engine_change = None;
                    }
                });
            });
        }
        if let Some((id, mut target)) = self.project.rename.clone() {
            egui::Modal::new(egui::Id::new("rename_speaker")).show(ctx, |ui| {
                ui.heading("Rename speaker id");
                ui.label("Scripts use the speaker's display name, not the id. This does not rename script text.");
                ui.text_edit_singleline(&mut target);
                let duplicate = self.project.draft.as_ref().is_some_and(|r| r.voices.speakers.iter()
                    .any(|s| s["id"].as_str() == Some(&target)));
                let valid = !target.trim().is_empty() && !duplicate;
                if duplicate { ui.colored_label(t::STATUS_FAILED, "This id is already in use."); }
                self.project.rename = Some((id.clone(), target.clone()));
                ui.horizontal(|ui| {
                    if ui.add_enabled(valid, egui::Button::new("Rename")).clicked() {
                        if let Some(speaker) = self.project.draft.as_mut().and_then(|r| r.voices.speakers.iter_mut()
                            .find(|s| s["id"].as_str() == Some(&id))) {
                            if speaker.get("original_id").is_none() { speaker["original_id"] = id.clone().into(); }
                            speaker["id"] = target.clone().into();
                            self.project.select_id = target.clone();
                        }
                        self.project.rename = None;
                    }
                    if ui.button("Cancel").clicked() { self.project.rename = None; }
                });
            });
        }
    }
}

fn toml_fields(ui: &mut Ui, report: &mut Report) {
    for key in project::TOML_KEYS {
        if key == "tail_ms" {
            let mut value = report
                .toml
                .values
                .get(key)
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label("Tail (ms)");
                if ui.add(egui::DragValue::new(&mut value)).changed() {
                    report.toml.values.insert(key.into(), value.into());
                }
            });
        } else {
            ui.label(t::text(field_label(key), t::BODY, t::TEXT_PRIMARY));
            let mut value = report
                .toml
                .values
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned();
            ui.horizontal(|ui| {
                if ui
                    .add(egui::TextEdit::singleline(&mut value).desired_width(ui.available_width()))
                    .changed()
                {
                    report.toml.values.insert(key.into(), value.clone().into());
                }
                if t::secondary_button(ui, "Choose…", None, true).clicked() {
                    let dialog = rfd::FileDialog::new().set_directory(&report.root);
                    let path =
                        if ["clip_sources", "references", "scripts", "outputs"].contains(&key) {
                            dialog.pick_folder()
                        } else {
                            dialog.pick_file()
                        };
                    if let Some(path) = path {
                        let stored = stored_path(&report.root, path);
                        report.toml.values.insert(key.into(), stored.clone().into());
                        value = stored;
                    }
                }
            });
        }
    }
}

impl Browser {
    pub(crate) fn project_settings_section(&mut self, ui: &mut Ui) {
        let Some(mut report) = self.project.draft.take() else {
            ui.label(if self.project.pending.is_some() {
                "Loading project…"
            } else {
                "Open a project to edit its settings."
            });
            return;
        };
        let ctx = ui.ctx().clone();
        let mut save = false;
        ui.add_enabled_ui(self.project.saving.is_none(), |ui| {
            ui.label(report.root.display().to_string());
            toml_fields(ui, &mut report);
            for error in &self.project.error {
                ui.colored_label(t::STATUS_FAILED, error);
            }
            if let Some(snapshot) = &self.project.snapshot {
                let changed = snapshot.report.toml != report.toml;
                let reason = save_reason("toml", &snapshot.report, &report);
                if changed {
                    ui.label("Unsaved changes");
                }
                ui.horizontal(|ui| {
                    save = ui
                        .add_enabled(
                            changed && reason.is_none(),
                            egui::Button::new("Save project settings"),
                        )
                        .clicked();
                    if ui.button("Revert").clicked() {
                        report.toml = snapshot.report.toml.clone();
                    }
                });
                if let Some(reason) = reason {
                    ui.colored_label(t::STATUS_FAILED, reason);
                }
            }
        });
        self.project.draft = Some(report);
        if save {
            self.save_project_tab("toml", &ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draft_browser() -> Browser {
        let report = Report::parse(include_bytes!("../tests/fixtures/phase_l/show.json")).unwrap();
        let mut browser = Browser::default();
        browser.project.root = Some(report.root.clone());
        browser.project.snapshot = Some(Snapshot {
            report: report.clone(),
            hashes: Default::default(),
        });
        browser.project.draft = Some(report);
        browser.project.data_open = true;
        browser
    }
    #[test]
    fn all_project_tabs_paint_without_a_gui_or_audio_device() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        for tab in project::FILES {
            let mut browser = draft_browser();
            browser.project.tab = tab.into();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        t::WINDOW_SIZE.into(),
                    )),
                    ..Default::default()
                },
                |ui| browser.project_data_screen(ui),
            );
            assert!(!output.shapes.is_empty());
            assert!(
                browser.dirty_project_files().is_empty(),
                "painting {tab} changed the draft"
            );
            assert!(browser.project.audition.is_none());
            output.textures_delta.clear();
        }
    }
    #[test]
    fn clip_decode_is_requested_once_across_frames() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut browser = draft_browser();
        browser.project.tab = "clips".into();
        browser.project.select_id = "jalachantelle_2026a".into();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                t::WINDOW_SIZE.into(),
            )),
            ..Default::default()
        };
        for _ in 0..4 {
            let mut output = ctx.run_ui(input.clone(), |ui| browser.project_data_screen(ui));
            output.textures_delta.clear();
            browser.poll_project(&ctx);
        }
        assert_eq!(browser.project.decode_requests, 1);
        assert!(browser.dirty_project_files().is_empty());
    }
    #[test]
    fn speaker_row_chip_overlaps_name() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut browser = draft_browser();
        browser.project.tab = "voices".into();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    t::WINDOW_SIZE.into(),
                )),
                ..Default::default()
            },
            |ui| browser.project_data_screen(ui),
        );
        output.textures_delta.clear();
        assert!(
            browser.project.speaker_rows.len() > 1,
            "expected every speaker row"
        );
        for (name, chip) in &browser.project.speaker_rows {
            assert!(
                name.top() < chip.bottom() && chip.top() < name.bottom(),
                "name {name:?} and chip {chip:?} do not share a row"
            );
        }
    }
    #[test]
    fn draft_paths_are_saved_after_other_dirty_files() {
        let mut browser = draft_browser();
        let draft = browser.project.draft.as_mut().unwrap();
        draft.toml.values.insert("tail_ms".into(), 900.into());
        draft.pronunciations.entries.clear();
        assert_eq!(
            browser.dirty_project_files(),
            vec!["pronunciations", "toml"]
        );
        let snapshot = browser.project.snapshot.as_ref().unwrap();
        assert!(
            save_reason(
                "toml",
                &snapshot.report,
                browser.project.draft.as_ref().unwrap()
            )
            .is_some()
        );
    }
}
