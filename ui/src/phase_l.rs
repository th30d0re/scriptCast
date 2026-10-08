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
    pub media_pending: Option<Receiver<Result<scriptcast_ui::waveform::Envelope, String>>>,
    pub media: Option<Result<scriptcast_ui::waveform::Envelope, String>>,
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
            self.project.media = Some(result);
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
        let ctx = ui.ctx().clone();
        egui::Frame::new().inner_margin(t::SPACE_16).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    self.project
                        .root
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Projects".into()),
                );
                ui.menu_button("File", |ui| {
                    if ui.button("Open Project…     ⌘⇧O").clicked() {
                        self.choose_project(&ctx);
                        ui.close();
                    }
                    if ui.button("Project Data…     ⌘;").clicked() {
                        self.open_project_data(&ctx);
                        ui.close();
                    }
                    if ui.button("Settings…     ⌘,").clicked() {
                        self.settings_open = true;
                        ui.close();
                    }
                });
                if ui.button("Project settings").clicked() {
                    self.open_project_data(&ctx);
                }
                if ui.button("New episode…").clicked() {
                    self.show_new_episode(None);
                    if let Some(root) = self.project.root.clone() {
                        self.apply_project_defaults(root);
                    }
                }
            });
            if let Some(root) = &self.project.root {
                ui.label(t::text(
                    root.display().to_string(),
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            }
            ui.separator();
            two_columns(ui, t::PROJECT_PICKER_FRACTION, |columns| {
                let left = &mut columns[0];
                left.heading("Recent projects");
                for recent in &self.preferences.recent_projects {
                    egui::Frame::group(left.style()).show(left, |ui| {
                        if ui
                            .selectable_label(
                                self.project.root.as_ref() == Some(&recent.root),
                                &recent.name,
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
                }
                if left.button("Open project folder…").clicked() {
                    self.choose_project(&ctx);
                }
                if self
                    .project
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.report.toml.missing)
                {
                    egui::Frame::group(left.style()).show(left, |ui| {
                        ui.label("Not a scriptCast project");
                        ui.label("This folder has no scriptcast.toml.");
                        ui.add_enabled(false, egui::Button::new("Create scriptcast.toml here"));
                        ui.label(
                            "The project tool needs a create operation for a comment-only file.",
                        );
                    });
                }
                let right = &mut columns[1];
                right.heading("Episodes");
                if self.project.pending.is_some() {
                    right.spinner();
                    right.label("Loading project…");
                }
                for error in &self.project.error {
                    right.colored_label(t::STATUS_FAILED, error);
                }
                egui::ScrollArea::both()
                    .id_salt("project_episodes")
                    .show(right, |ui| {
                        egui::Grid::new("project_episode_table")
                            .striped(true)
                            .show(ui, |ui| {
                                for label in [
                                    "Episode id",
                                    "Script file",
                                    "Turns",
                                    "Duration",
                                    "Last rendered",
                                    "Verification",
                                    "Status",
                                ] {
                                    ui.label(label);
                                }
                                ui.end_row();
                                for (index, row) in self.project.rows.iter().enumerate() {
                                    let response = ui.selectable_label(
                                        self.project.selected == Some(index),
                                        &row.id,
                                    );
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
                                    ui.label(
                                        row.script
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy(),
                                    );
                                    ui.label(
                                        row.turns
                                            .map(|n| n.to_string())
                                            .unwrap_or_else(|| "—".into()),
                                    );
                                    ui.label(
                                        row.duration_ms
                                            .map(|n| format!("{}:{:02}", n / 60000, n / 1000 % 60))
                                            .unwrap_or_else(|| "—".into()),
                                    );
                                    ui.label(&row.rendered);
                                    ui.label(&row.verification);
                                    ui.label(row.status.label());
                                    ui.end_row();
                                }
                            });
                    });
                if let Some(row) = self.project.selected.and_then(|i| self.project.rows.get(i)) {
                    egui::Frame::group(right.style()).show(right, |ui| {
                        ui.heading(&row.id);
                        ui.label(row.script.display().to_string());
                        ui.label(&row.verification);
                        ui.label(row.status.label());
                    });
                }
                right.label(t::text(
                    "Double-click an episode row to open it.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            });
        });
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
    }
}

fn text_field(ui: &mut Ui, fields: &mut serde_json::Map<String, serde_json::Value>, key: &str) {
    ui.label(key.replace('_', " "));
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
        ui.label(key.replace('_', " "));
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
    pub(crate) fn project_data_window(&mut self, ctx: &egui::Context) {
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
                self.save_project_tab(&file, ctx);
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
        let mut open = true;
        let mut save = false;
        let mut reload = false;
        let mut tab = self.project.tab.clone();
        let mut selected = self.project.select_id.clone();
        let mut draft = self.project.draft.take();
        egui::Window::new("Project data")
            .open(&mut open)
            .default_width(t::PROJECT_DATA_WIDTH)
            .show(ctx, |ui| {
                if let Some(root) = &self.project.root {
                    ui.label(root.display().to_string());
                }
                let dirty = draft
                    .as_ref()
                    .zip(self.project.snapshot.as_ref())
                    .is_some_and(|(d, s)| d != &s.report);
                if dirty {
                    ui.colored_label(t::accent(), "Unsaved changes");
                }
                ui.horizontal(|ui| {
                    for (key, label) in [
                        ("voices", "Voices"),
                        ("clips", "Clips"),
                        ("pronunciations", "Pronunciations"),
                        ("accepted_hearings", "Accepted hearings"),
                        ("speaker_rates", "Speaker rates"),
                    ] {
                        ui.selectable_value(&mut tab, key.into(), label);
                    }
                    ui.selectable_value(&mut tab, "toml".into(), "Project settings");
                });
                ui.separator();
                for message in &self.project.error {
                    ui.colored_label(t::STATUS_FAILED, message);
                }
                if self.project.stale && ui.button("Reload from disk (discards drafts)").clicked() {
                    reload = true;
                }
                if tab != self.project.tab {
                    selected.clear();
                }
                let Some(report) = &mut draft else {
                    ui.label(if self.project.pending.is_some() {
                        "Loading project…"
                    } else {
                        "Open a project first."
                    });
                    return;
                };
                for message in &report.section(&tab).problems {
                    ui.colored_label(t::STATUS_FAILED, message);
                }
                if self.project.saving.is_some() {
                    ui.disable();
                }
                egui::ScrollArea::vertical()
                    .max_height(t::PROJECT_DATA_HEIGHT)
                    .show(ui, |ui| match tab.as_str() {
                        "voices" => self.voice_editor(ui, report, &mut selected),
                        "clips" => self.clip_editor(ui, report, &mut selected),
                        "accepted_hearings" => {
                            ui.label(t::text(
                                "These hearings count as a match in every turn of the project.",
                                t::CAPTION,
                                t::TEXT_MUTED,
                            ));
                            for entry in &mut report.accepted_hearings.entries {
                                let word = entry["word"].as_str().unwrap_or("").to_owned();
                                let date = entry["review_ts"].as_str().unwrap_or("—").to_owned();
                                let risky = entry["risky"].as_bool().unwrap_or(false);
                                if let Some(heard) = entry["heard"].as_array_mut() {
                                    heard.retain(|hearing| {
                                        let mut delete = false;
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "{word} → {}",
                                                hearing.as_str().unwrap_or("")
                                            ));
                                            ui.label(&date);
                                            if risky {
                                                ui.colored_label(t::STATUS_FAILED, "Common word");
                                            }
                                            delete = ui.button("Delete").clicked();
                                        });
                                        !delete
                                    });
                                }
                            }
                        }
                        "pronunciations" => {
                            report.pronunciations.entries.retain_mut(|entry| {
                                let mut delete = false;
                                ui.group(|ui| {
                                    text_field(ui, entry.as_object_mut().unwrap(), "word");
                                    if let Some(readings) = entry["readings"].as_object_mut() {
                                        for (key, value) in readings.clone() {
                                            ui.horizontal(|ui| {
                                                let mut next_key = key.clone();
                                                let mut text =
                                                    value.as_str().unwrap_or("").to_owned();
                                                let renamed = ui
                                                    .text_edit_singleline(&mut next_key)
                                                    .changed();
                                                let edited =
                                                    ui.text_edit_singleline(&mut text).changed();
                                                if ui.button("Delete reading").clicked() {
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
                                            });
                                        }
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
                ui.separator();
                ui.label(t::text(
                    "Comments in the file are kept.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                ui.horizontal(|ui| {
                    if ui.button("Revert").clicked()
                        && let Some(snapshot) = &self.project.snapshot
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
                    let reason = self
                        .project
                        .snapshot
                        .as_ref()
                        .and_then(|snapshot| save_reason(&tab, &snapshot.report, report));
                    let dirty_tab = self
                        .project
                        .snapshot
                        .as_ref()
                        .is_some_and(|s| s.report.section(&tab) != report.section(&tab));
                    save = ui
                        .add_enabled(
                            reason.is_none() && dirty_tab && self.project.saving.is_none(),
                            egui::Button::new(format!("Save {tab}")),
                        )
                        .clicked();
                    if let Some(reason) = reason {
                        ui.colored_label(t::STATUS_FAILED, reason);
                    }
                });
            });
        self.project.draft = draft;
        self.project.tab = tab;
        self.project.select_id = selected;
        if save {
            self.save_project_tab(&self.project.tab.clone(), ctx);
        }
        if reload && let Some(root) = self.project.root.clone() {
            self.project.stale = false;
            self.project.draft = self.project.snapshot.as_ref().map(|s| s.report.clone());
            self.load_project(root, ctx);
        }
        if !open {
            if self.dirty_project_files().is_empty() {
                self.project.data_open = false;
            } else {
                self.project.close_prompt = true;
            }
        }
        self.project_close_dialog(ctx);
        self.project_voice_dialogs(ctx);
    }
    fn voice_editor(&mut self, ui: &mut Ui, report: &mut Report, selected: &mut String) {
        two_columns(ui, t::PROJECT_VOICES_FRACTION, |columns| {
            columns[0].heading("Speakers");
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
                columns[0].horizontal(|ui| {
                    ui.colored_label(t::speaker_colour(index), "●");
                    let name = fields
                        .and_then(|f| f.get("name"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(id);
                    if ui.selectable_label(selected == id, name).clicked() {
                        *selected = id.into();
                    }
                    ui.label(
                        fields
                            .and_then(|f| f.get("engine"))
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("kokoro"),
                    );
                    ui.label(count.to_string());
                    if invalid {
                        ui.colored_label(t::STATUS_FAILED, "●");
                    }
                });
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
                report
                    .voices
                    .speakers
                    .push(serde_json::json!({"id":id, "fields":{"name":"", "engine":"omnivoice"}}));
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
            ui.horizontal(|ui| {
                ui.label(format!("Id · {selected}"));
                if ui.button("Rename…").clicked() {
                    self.project.rename = Some((selected.clone(), selected.clone()));
                }
            });
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
                match *key {
                    "speed" => number_field(ui, fields, key, 1.0),
                    "exaggeration" => number_field(ui, fields, key, 0.0),
                    "temperature" => number_field(ui, fields, key, 0.6),
                    "cfg_weight" => number_field(ui, fields, key, 0.7),
                    _ => text_field(ui, fields, key),
                }
                if *key == "reference_audio"
                    && ui.button("Choose reference audio…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .set_directory(&report.root)
                        .pick_file()
                {
                    fields.insert((*key).into(), path.display().to_string().into());
                }
                if let Some(error) = project::required_errors(fields).get(*key) {
                    ui.colored_label(t::STATUS_FAILED, error);
                }
            }
            if let Some(path) = fields
                .get("reference_audio")
                .and_then(serde_json::Value::as_str)
            {
                let path = report.root.join(path);
                self.load_project_media(path.clone(), ui.ctx());
                self.media_play_button(ui, &path, 0.0, None);
                if let Some(Err(error)) = &self.project.media {
                    ui.label(format!("Cannot decode reference audio: {error}"));
                }
            }
            if engine == "omnivoice" {
                ui.collapsing("Expressive reference audio + text · optional pair", |ui| {
                    text_field(ui, fields, "reference_audio_expressive");
                    text_field(ui, fields, "reference_text_expressive");
                });
            }
        });
    }
    fn clip_editor(&mut self, ui: &mut Ui, report: &mut Report, selected: &mut String) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.project.search)
                    .hint_text("Search id or source file…"),
            );
            ui.add_enabled(false, egui::Button::new("+ Register clip…"))
                .on_disabled_hover_text("Coming in Phase M (clip maker)");
        });
        ui.label(t::text(
            "Coming in Phase M (clip maker)",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        let query = self.project.search.to_lowercase();
        if selected.is_empty()
            && let Some(clip) = report.clips.clips.first()
        {
            *selected = clip["id"].as_str().unwrap_or("").into();
        }
        two_columns(ui, t::PROJECT_CLIPS_FRACTION, |columns| {
            egui::ScrollArea::both()
                .id_salt("clips_table")
                .show(&mut columns[0], |ui| {
                    egui::Grid::new("clips_grid").striped(true).show(ui, |ui| {
                        for label in [
                            "Id",
                            "Source file",
                            "Start",
                            "End",
                            "Duration",
                            "Citation",
                            "Play",
                        ] {
                            ui.label(label);
                        }
                        ui.end_row();
                        for clip in &report.clips.clips {
                            let id = clip["id"].as_str().unwrap_or("");
                            let source = clip["source"].as_str().unwrap_or("");
                            if !format!("{id} {source}").to_lowercase().contains(&query) {
                                continue;
                            }
                            if ui.selectable_label(selected == id, id).clicked() {
                                *selected = id.into();
                            }
                            ui.vertical(|ui| {
                                ui.label(source);
                                if clip["source_exists"] == false {
                                    ui.colored_label(t::STATUS_FAILED, "Missing source");
                                }
                            });
                            let start = clip["start"].as_f64().unwrap_or(0.0);
                            let end = clip["end"].as_f64().unwrap_or(start);
                            ui.label(format!("{start:.3}"));
                            ui.label(format!("{end:.3}"));
                            ui.label(format!("{:.1} s", end - start));
                            ui.label(clip["citation"].as_str().unwrap_or("—"));
                            if ui
                                .add_enabled(
                                    clip["source_exists"] == true,
                                    egui::Button::new("Play"),
                                )
                                .clicked()
                            {
                                self.audition_project_media(
                                    &report.root.join(source),
                                    start,
                                    Some(end),
                                );
                            }
                            ui.end_row();
                        }
                    });
                });
            let ui = &mut columns[1];
            if let Some(clip) = report
                .clips
                .clips
                .iter_mut()
                .find(|c| c["id"].as_str() == Some(selected))
            {
                ui.heading(selected.as_str());
                let path = report.root.join(clip["source"].as_str().unwrap_or(""));
                self.load_project_media(path.clone(), ui.ctx());
                let mut start = clip["start"].as_f64().unwrap_or(0.0);
                let mut end = clip["end"].as_f64().unwrap_or(0.0);
                if self.clip_waveform(ui, &mut start, &mut end) {
                    clip["start"] = start.into();
                    clip["end"] = end.into();
                }
                self.media_play_button(ui, &path, start, Some(end));
                if let Some(fields) = clip.as_object_mut() {
                    number_field(ui, fields, "start", 0.0);
                    number_field(ui, fields, "end", 0.0);
                    for key in ["citation", "origin_url", "content_note"] {
                        text_field(ui, fields, key);
                    }
                }
                if start < 0.0 || end <= start {
                    ui.colored_label(
                        t::STATUS_FAILED,
                        "End must be after start; start must be nonnegative.",
                    );
                }
                ui.label("Used in:");
                if let Some(turns) = &self.editing.turns {
                    for turn in &turns.turns {
                        if scriptcast_ui::script::clip_id(&turn.body) == Some(selected) {
                            ui.label(format!("{} · {}", turn.turn_id, turn.display_name));
                        }
                    }
                } else {
                    ui.label("Open an episode to see its clip turns.");
                }
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

fn two_columns(ui: &mut Ui, fraction: f32, body: impl FnOnce(&mut [Ui; 2])) {
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
    fn load_project_media(&mut self, path: PathBuf, ctx: &egui::Context) {
        if self.project.media_path.as_ref() == Some(&path) {
            return;
        }
        self.project.media_path = Some(path.clone());
        self.project.media = None;
        let (tx, rx) = mpsc::channel();
        self.project.media_pending = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(scriptcast_ui::waveform::envelope(&[path]));
            ctx.request_repaint();
        });
    }
    fn audition_project_media(&mut self, path: &std::path::Path, start: f64, end: Option<f64>) {
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
                Err(error) => self.project.error = vec![error],
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
                ui.label(format!(
                    "{:.1} s",
                    envelope.peaks.len() as f32 * scriptcast_ui::waveform::BUCKET_SECS
                ));
            }
        });
    }
    fn clip_waveform(&mut self, ui: &mut Ui, start: &mut f64, end: &mut f64) -> bool {
        let envelope = match &self.project.media {
            Some(Ok(envelope)) if !envelope.peaks.is_empty() => envelope,
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
        let duration =
            envelope.peaks.len() as f64 * f64::from(scriptcast_ui::waveform::BUCKET_SECS);
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
        let x0 = rect.left() + (*start / duration).clamp(0.0, 1.0) as f32 * rect.width();
        let x1 = rect.left() + (*end / duration).clamp(0.0, 1.0) as f32 * rect.width();
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
                let seconds =
                    f64::from(((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0)) * duration;
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
            text_field(ui, &mut report.toml.values, key);
            if ui.button(format!("Choose {key}…")).clicked() {
                let dialog = rfd::FileDialog::new().set_directory(&report.root);
                let path = if ["clip_sources", "references", "scripts", "outputs"].contains(&key) {
                    dialog.pick_folder()
                } else {
                    dialog.pick_file()
                };
                if let Some(path) = path {
                    report
                        .toml
                        .values
                        .insert(key.into(), path.display().to_string().into());
                }
            }
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
                |ui| browser.project_data_window(ui.ctx()),
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
