//! Structural editing UI. Script order stays separate from the rendered audio manifest.
use crate::{
    Browser,
    editing::{self, EditMode},
    inspector::InspectorMode,
    tag_editor,
};
use eframe::egui::{self, Ui};
use crate::{
    checkpoint::Checkpoint,
    retime,
    script::{self, ScriptTurns},
    structure::{EditOp, Registry, archive_ready},
    tags, theme as t,
};
use std::sync::mpsc::{self, Receiver};

#[derive(Clone, Debug)]
pub struct NewTurn {
    pub position: usize,
    pub speaker: String,
    pub timestamp: String,
    pub body: String,
    pub after: Option<usize>,
    pub before: Option<usize>,
}
impl NewTurn {
    fn submitted_timestamp(&self) -> Option<String> {
        let trimmed = self.timestamp.trim();
        if trimmed.is_empty() || trimmed.ends_with("est.") {
            None
        } else {
            Some(trimmed.to_owned())
        }
    }
    fn op(&self) -> EditOp {
        EditOp::Insert {
            position: self.position,
            speaker_id: self.speaker.clone(),
            timestamp: self.submitted_timestamp(),
            body: self.body.clone(),
        }
    }
    fn ready(&self, registry: &Registry) -> bool {
        registry
            .speakers
            .iter()
            .find(|s| s.id == self.speaker)
            .is_some_and(|s| {
                archive_ready(&s.engine, &self.body, registry)
                    && tags::can_save(&self.body, registry)
            })
    }
}
#[derive(Clone, Debug)]
pub enum MenuAction {
    Insert(usize),
    Apply(EditOp, usize),
    Delete(usize),
    Stress(usize),
    Regenerate(usize),
}
type Saved = Result<(Checkpoint, ScriptTurns, tags::Report), String>;
#[derive(Default)]
pub struct PhaseK {
    pub new_turn: Option<NewTurn>,
    pub action: Option<MenuAction>,
    pub delete: Option<usize>,
    pub structure_changed: bool,
    pub script_view: bool,
    pub selected: Option<usize>,
    pub context_capture: bool,
    pub capture_insert: bool,
    pub capture_speaker: Option<String>,
    pub capture_done: bool,
    pub authority: Option<tags::Report>,
    pub rendered_rows: Vec<Option<usize>>,
    pending_op: Option<EditOp>,
    pub retime: Option<retime::Preview>,
    saving: Option<Receiver<Saved>>,
    retiming: Option<Receiver<Result<retime::Preview, String>>>,
    applying: Option<Receiver<Result<Checkpoint, String>>>,
    preview_player: Option<rodio::Player>,
}
impl PhaseK {
    pub fn busy(&self) -> bool {
        self.saving.is_some() || self.retiming.is_some() || self.applying.is_some()
    }
}

fn displayed(editing: &editing::Editing, index: usize) -> (usize, String) {
    editing
        .turns
        .as_ref()
        .and_then(|s| s.turns.get(index))
        .map(|turn| (turn.index, turn.display_name.clone()))
        .unwrap_or((index, String::new()))
}

fn menu_row(ui: &mut Ui, label: &str, shortcut: &str, enabled: bool, danger: bool) -> bool {
    let text = if danger {
        t::text(label, t::BODY, t::STATUS_FAILED)
    } else {
        t::text(label, t::BODY, t::TEXT_PRIMARY)
    };
    ui.add_enabled(
        enabled,
        egui::Button::new(text).shortcut_text(t::text(shortcut, t::CAPTION, t::TEXT_MUTED)),
    )
    .clicked()
}

/// Right-click menu, or the capture popup anchored to this row.
pub fn open_row_menu(
    response: &egui::Response,
    editing: &mut editing::Editing,
    index: usize,
    enabled: bool,
    capture: bool,
) {
    let mut popup = egui::Popup::context_menu(response).width(t::CONTEXT_MENU_WIDTH);
    if capture {
        popup = popup.at_position(response.rect.left_bottom()).open(true);
    }
    popup.show(|ui| {
        ui.set_min_width(t::CONTEXT_MENU_WIDTH);
        menu(ui, editing, index, enabled);
    });
}

pub fn menu(ui: &mut Ui, editing: &mut editing::Editing, index: usize, enabled: bool) {
    let count = editing.turns.as_ref().map_or(0, |s| s.turns.len());
    let enabled = enabled
        && index < count
        && !editing.busy()
        && !editing.loading()
        && editing.draft.is_none()
        && editing.k.new_turn.is_none();
    let (number, name) = displayed(editing, index);
    ui.label(t::text(
        format!("TURN {number:03} · {}", name.to_uppercase()),
        t::CAPTION,
        t::TEXT_MUTED,
    ));
    ui.separator();
    let blocked = (!enabled).then_some("Finish the current edit first");
    let rows = [
        (
            "Insert turn above",
            "Opt+Cmd+Up",
            MenuAction::Insert(index),
            None,
        ),
        (
            "Insert turn below",
            "Opt+Cmd+Down",
            MenuAction::Insert(index + 1),
            None,
        ),
    ];
    for (label, shortcut, action, boundary) in rows {
        let reason = blocked.or(boundary);
        let label = reason.map_or_else(|| label.to_owned(), |r| format!("{label} · {r}"));
        if menu_row(ui, &label, shortcut, reason.is_none(), false) {
            editing.k.action = Some(action);
            ui.close();
        }
    }
    ui.separator();
    for (label, shortcut, action, boundary) in [
        (
            "Move up",
            "Cmd+Up",
            MenuAction::Apply(
                EditOp::Move {
                    index,
                    to: index.saturating_sub(1),
                },
                index.saturating_sub(1),
            ),
            (index == 0).then_some("Already first turn"),
        ),
        (
            "Move down",
            "Cmd+Down",
            MenuAction::Apply(
                EditOp::Move {
                    index,
                    to: index + 1,
                },
                index + 1,
            ),
            (index + 1 >= count).then_some("Already last turn"),
        ),
    ] {
        let reason = blocked.or(boundary);
        let label = reason.map_or_else(|| label.to_owned(), |r| format!("{label} · {r}"));
        if menu_row(ui, &label, shortcut, reason.is_none(), false) {
            editing.k.action = Some(action);
            ui.close();
        }
    }
    ui.separator();
    let current = editing
        .turns
        .as_ref()
        .and_then(|s| s.turns.get(index))
        .map(|s| s.speaker_id.clone());
    let speaker_label = blocked.map_or_else(
        || "Change speaker".to_owned(),
        |reason| format!("Change speaker · {reason}"),
    );
    let button = ui.add_enabled(
        blocked.is_none(),
        egui::Button::new(speaker_label).right_text("▸"),
    );
    if blocked.is_none() {
        if editing.k.context_capture {
            let id = egui::containers::menu::SubMenu::id_from_widget_id(button.id);
            egui::containers::menu::MenuState::from_id(ui.ctx(), id, |_| {});
            egui::containers::menu::MenuState::from_ui(ui, |state, _| {
                state.open_item = Some(id);
            });
        }
        egui::containers::menu::SubMenu::new().show(ui, &button, |ui| {
            if editing.registry.speakers.is_empty() {
                ui.label("Speakers unavailable");
            }
            for (i, speaker) in editing.registry.speakers.iter().enumerate() {
                let checked = current.as_ref() == Some(&speaker.id);
                let label = format!("● {} · {}", speaker.name, speaker.engine);
                let button = egui::Button::new(t::text(label, t::BODY, t::speaker_colour(i)))
                    .shortcut_text(if checked { "✓" } else { "" });
                if ui.add(button).clicked() && !checked {
                    editing.k.action = Some(MenuAction::Apply(
                        EditOp::SetSpeaker {
                            index,
                            speaker_id: speaker.id.clone(),
                        },
                        index,
                    ));
                    ui.close();
                }
            }
        });
    }
    ui.separator();
    let delete_label = blocked.map_or_else(
        || "Delete turn".to_owned(),
        |reason| format!("Delete turn · {reason}"),
    );
    if menu_row(ui, &delete_label, "Delete", enabled, true) {
        editing.k.action = Some(MenuAction::Delete(index));
        ui.close();
    }
    ui.separator();
    let stress_label = blocked.map_or_else(
        || "Check stress".to_owned(),
        |reason| format!("Check stress · {reason}"),
    );
    if menu_row(ui, &stress_label, "", blocked.is_none(), false) {
        editing.k.action = Some(MenuAction::Stress(index));
        ui.close();
    }
    let regenerate_label = blocked.map_or_else(
        || "Regenerate turn".to_owned(),
        |reason| format!("Regenerate turn · {reason}"),
    );
    if menu_row(ui, &regenerate_label, "", blocked.is_none(), false) {
        editing.k.action = Some(MenuAction::Regenerate(index));
        ui.close();
    }
}

pub fn new_editor(ui: &mut Ui, editing: &mut editing::Editing, panel: bool) {
    let Some(draft) = editing.k.new_turn.as_mut() else {
        return;
    };
    ui.label(t::heading("New · unsaved", t::SECTION_TITLE));
    let place = match (draft.after, draft.before) {
        (Some(after), Some(before)) => format!("After {after:03} / before {before:03}"),
        (Some(after), None) => format!("After {after:03}"),
        (None, Some(before)) => format!("Before {before:03}"),
        (None, None) => "New turn".into(),
    };
    if panel {
        ui.label(t::text(&place, t::CAPTION, t::TEXT_MUTED));
        ui.label(t::text("SPEAKER", t::CAPTION, t::TEXT_MUTED));
    } else {
        ui.label(t::text(&place, t::CAPTION, t::TEXT_MUTED));
    }
    let speaker = editing
        .registry
        .speakers
        .iter()
        .find(|s| s.id == draft.speaker);
    let title = speaker
        .map(|s| format!("● {} · {}", s.name, s.engine))
        .unwrap_or("Choose speaker".into());
    egui::ComboBox::from_id_salt("new_speaker")
        .selected_text(title)
        .show_ui(ui, |ui| {
            for (i, s) in editing.registry.speakers.iter().enumerate() {
                ui.selectable_value(
                    &mut draft.speaker,
                    s.id.clone(),
                    t::text(
                        format!("● {} · {}", s.name, s.engine),
                        t::BODY,
                        t::speaker_colour(i),
                    ),
                );
            }
        });
    if panel {
        ui.label(t::text("TIME · EDITABLE", t::CAPTION, t::TEXT_MUTED));
    }
    ui.add(
        egui::TextEdit::singleline(&mut draft.timestamp)
            .desired_width(t::TIME_FIELD_WIDTH)
            .hint_text("est."),
    );
    let speaker = editing
        .registry
        .speakers
        .iter()
        .find(|s| s.id == draft.speaker);
    let archive = speaker.is_some_and(|s| s.engine == "archive");
    let ready = speaker.is_some_and(|s| archive_ready(&s.engine, &draft.body, &editing.registry));
    if archive && !ready {
        let response = ui.button("Choose archive clip…");
        if response.clicked() {
            editing.toolbar.picker = true;
        }
        let mut open = editing.toolbar.picker;
        egui::Popup::from_response(&response)
            .open_bool(&mut open)
            .show(|ui| {
                ui.set_width(t::CLIP_PICKER_WIDTH);
                if let Some(tag) =
                    tag_editor::clip_picker(ui, &mut editing.toolbar, &editing.registry)
                {
                    draft.body = format!("{tag} ");
                }
            });
        editing.toolbar.picker = open;
        ui.label(t::text(
            "Clip required · no free text",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
    } else {
        if panel {
            ui.label(t::text("TRANSCRIPT", t::CAPTION, t::TEXT_MUTED));
        }
        let placeholder = format!(
            "Write what {} says…",
            speaker.map_or("Speaker", |s| s.name.as_str())
        );
        tag_editor::body(
            ui,
            &mut draft.body,
            &mut editing.toolbar,
            &editing.registry,
            t::EDITOR_MIN_HEIGHT,
            &placeholder,
            t::EDITOR_BODY_LINES,
            |_| {},
        );
    }
    if let Some(speaker) = speaker {
        let engines: Vec<_> = editing
            .turns
            .as_ref()
            .into_iter()
            .flat_map(|s| &s.turns)
            .filter(|turn| {
                turn.index.checked_add(1) == Some(draft.position) || turn.index == draft.position
            })
            .filter_map(|turn| {
                editing
                    .registry
                    .speakers
                    .iter()
                    .find(|s| s.id == turn.speaker_id)
            })
            .map(|s| &s.engine)
            .collect();
        if !engines.is_empty() && engines.iter().all(|e| **e != speaker.engine) {
            ui.label(t::text(
                format!("{} renders with {}.", speaker.name, speaker.engine),
                t::CAPTION,
                t::TEXT_MUTED,
            ));
        }
    }
    let save = draft.ready(&editing.registry);
    let mut cancel = false;
    ui.horizontal(|ui| {
        cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
        if t::primary_button(ui, "Save Cmd+Return", None, save).clicked() {
            editing.k.action = Some(MenuAction::Apply(draft.op(), draft.position));
        }
    });
    if cancel {
        editing.k.new_turn = None;
    }
}

/// Script-only rows after structure edits; rendered waveforms remain in their original audio timeline.
pub fn script_table(
    ui: &mut Ui,
    editing: &mut editing::Editing,
    mode: EditMode,
    enabled: bool,
) -> Option<editing::Action> {
    let turns = editing
        .turns
        .as_ref()
        .map(|s| s.turns.clone())
        .unwrap_or_default();
    let mut action = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(t::text(
            "Script turns · audio unchanged",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        for index in 0..=turns.len() {
            if mode == EditMode::Row
                && editing
                    .k
                    .new_turn
                    .as_ref()
                    .is_some_and(|d| d.position == index)
            {
                egui::Frame::new()
                    .fill(t::BG_SELECTED)
                    .inner_margin(t::SPACE_8)
                    .show(ui, |ui| new_editor(ui, editing, false));
            }
            let Some(turn) = turns.get(index) else {
                continue;
            };
            let response = ui.selectable_label(
                editing.k.selected == Some(index),
                format!("{:03}   {}   {}", turn.index, turn.display_name, turn.body),
            );
            crate::phase_k::open_row_menu(&response, editing, index, enabled, false);
            if response.clicked() && editing.allow_selection(index, false) {
                editing.k.selected = Some(index);
            }
            if response.double_clicked() && editing.allow_selection(index, true) {
                editing.k.selected = Some(index);
                editing.open(index);
            }
            if editing.k.structure_changed && editing.k.selected == Some(index) {
                ui.label(t::text(
                    "Unsaved · structure changed",
                    t::CAPTION,
                    t::accent(),
                ));
            }
            if mode == EditMode::Row && editing.draft.as_ref().is_some_and(|d| d.turn == index) {
                action = editing::inline_editor(ui, editing, t::EDITOR_MIN_HEIGHT, enabled);
            }
            ui.separator();
        }
    });
    action
}

impl Browser {
    pub(crate) fn begin_insert(&mut self, position: usize) {
        if self.editing.busy() || self.editing.loading() || self.editing.draft.is_some() {
            return;
        }
        let speaker = self
            .editing
            .k
            .capture_speaker
            .take()
            .or_else(|| {
                self.editing
                    .turns
                    .as_ref()
                    .and_then(|s| s.turns.get(position.saturating_sub(1)))
                    .map(|s| s.speaker_id.clone())
            })
            .or_else(|| self.editing.registry.speakers.first().map(|s| s.id.clone()))
            .unwrap_or_default();
        if self.editing.k.rendered_rows.is_empty() && !self.editing.k.script_view {
            self.editing.k.rendered_rows = self
                .editing
                .turns
                .as_ref()
                .map(|s| (0..s.turns.len()).map(Some).collect())
                .unwrap_or_default();
        }
        let turns = self.editing.turns.clone();
        let (after, before, timestamp) = turns.as_ref().map_or((None, None, "est.".into()), |s| {
            let after = position.checked_sub(1).and_then(|i| s.turns.get(i));
            let before = s.turns.get(position);
            let timestamp = estimate_timestamp(
                s.script_path.as_path(),
                after.map(|t| t.header_line),
                before.map(|t| t.header_line),
            );
            (after.map(|t| t.index), before.map(|t| t.index), timestamp)
        });
        self.editing.k.new_turn = Some(NewTurn {
            position,
            speaker,
            timestamp,
            body: String::new(),
            after,
            before,
        });
        self.preferences.inspector_hidden = false;
        self.jobs.mode = Some(InspectorMode::SelectedTurn);
        self.scroll_to = Some(position.saturating_sub(1));
    }
    pub(crate) fn save_new_turn(&mut self, ctx: &egui::Context) {
        if let Some(draft) = &self.editing.k.new_turn
            && draft.ready(&self.editing.registry)
        {
            self.start_structure(draft.op(), draft.position, ctx);
        }
    }
    fn start_structure(&mut self, op: EditOp, selection: usize, ctx: &egui::Context) {
        if self.editing.busy()
            || self.jobs.busy()
            || self.pending.is_some()
            || self.editing.loading()
        {
            return;
        }
        let (Some(Ok(export)), Some(mut episode), Some(turns)) = (
            self.export_command.clone(),
            self.episode.clone(),
            self.editing.turns.clone(),
        ) else {
            return;
        };
        if self.editing.k.rendered_rows.is_empty() && !self.editing.k.script_view {
            self.editing.k.rendered_rows = (0..turns.turns.len())
                .map(|i| episode.turns.get(i).map(|_| i))
                .collect();
        }
        self.editing.k.pending_op = Some(op.clone());
        episode.script_path = Some(turns.script_path);
        let (tx, rx) = mpsc::channel();
        self.editing.k.saving = Some(rx);
        self.editing.k.selected = Some(selection);
        self.editing.notice = Some("Saving structure…".into());
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = script::apply_edits(&export, &episode, &[op], &turns.script_sha256)
                .and_then(|(cp, turns)| {
                    let tags = script::load_tags(&export, &episode)?;
                    Ok((cp, turns, tags))
                });
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(crate) fn structure_shortcuts(&mut self, ctx: &egui::Context) {
        let Some(index) = self.editing.k.selected.or(self.selected) else {
            return;
        };
        if self.editing.draft.is_some() || self.editing.k.new_turn.is_some() || self.editing.busy()
        {
            return;
        }
        let count = self.editing.turns.as_ref().map_or(0, |s| s.turns.len());
        for (key, delta) in [(egui::Key::ArrowUp, -1isize), (egui::Key::ArrowDown, 1)] {
            let insert = egui::Modifiers {
                alt: true,
                ..egui::Modifiers::COMMAND
            };
            if ctx.input_mut(|i| i.consume_key(insert, key)) {
                self.begin_insert(index + usize::from(delta > 0));
                return;
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key)) {
                if let Some(to) = index.checked_add_signed(delta).filter(|n| *n < count) {
                    self.start_structure(EditOp::Move { index, to }, to, ctx);
                }
                return;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace)) {
            self.editing.k.action = Some(MenuAction::Delete(index));
        }
    }
    pub(crate) fn phase_k_poll(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.editing.k.action.take() {
            match action {
                MenuAction::Insert(position) => self.begin_insert(position),
                MenuAction::Apply(op, selected) => self.start_structure(op, selected, ctx),
                MenuAction::Stress(index) => self.start_stress_check(index),
                MenuAction::Regenerate(index) => {
                    let mut rows = self
                        .editing
                        .markers(self.episode.as_ref().map_or(0, |e| e.turns.len()))
                        .unwrap_or_default();
                    if !rows.contains(&index) {
                        rows.push(index);
                    }
                    self.request_regenerate_edited(rows, ctx);
                }
                MenuAction::Delete(index) => {
                    let rendered_index = if self.editing.k.script_view {
                        self.editing.k.rendered_rows.get(index).copied().flatten()
                    } else {
                        Some(index)
                    };
                    let rendered = rendered_index
                        .and_then(|i| self.episode.as_ref()?.turns.get(i))
                        .is_some_and(|t| !t.wav_paths.is_empty());
                    if rendered {
                        self.editing.k.delete = Some(index);
                    } else {
                        self.start_structure(
                            EditOp::Delete { index },
                            index.saturating_sub(1),
                            ctx,
                        );
                    }
                }
            }
        }
        if let Some(result) = receive(&mut self.editing.k.saving) {
            match result {
                Ok((cp, turns, tags)) => {
                    if let Some(op) = self.editing.k.pending_op.take() {
                        update_mapping(&mut self.editing.k.rendered_rows, &op);
                    }
                    self.editing.loaded_sha256 = Some(turns.script_sha256.clone());
                    self.editing.k.selected = self.editing.k.selected.and_then(|i| {
                        (!turns.turns.is_empty()).then(|| i.min(turns.turns.len() - 1))
                    });
                    self.editing.turns = Some(turns);
                    self.editing.k.authority = Some(tags);
                    self.editing.k.new_turn = None;
                    self.editing.draft = None;
                    self.editing.k.script_view = true;
                    self.editing.k.structure_changed = true;
                    self.editing.notice = Some(format!(
                        "Unsaved · structure changed · checkpoint v{:02}",
                        cp.version
                    ));
                    self.refresh_checkpoints();
                }
                Err(e) => self.editing.notice = Some(e),
            }
        }
        if let Some(result) = receive(&mut self.editing.k.retiming) {
            match result {
                Ok(p) => self.editing.k.retime = Some(p),
                Err(e) => self.editing.notice = Some(e),
            }
        }
        if let Some(result) = receive(&mut self.editing.k.applying) {
            match result {
                Ok(_) => {
                    self.editing.k.retime = None;
                    self.refresh_checkpoints();
                    self.reload(ctx);
                }
                Err(e) => self.editing.notice = Some(e),
            }
        }
        if let Some(id) = self.editing.toolbar.preview.take() {
            let result = (|| {
                let clip = self
                    .editing
                    .registry
                    .clips
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("Unknown clip")?;
                let root = self
                    .episode
                    .as_ref()
                    .and_then(|e| e.project_root.as_ref())
                    .ok_or("No project root")?;
                let path = if clip.source.is_absolute() {
                    clip.source.clone()
                } else {
                    root.join(&clip.source)
                };
                self.audio
                    .as_ref()
                    .and_then(|a| a.as_ref().ok())
                    .ok_or("Audio unavailable")?
                    .preview_range(&path, clip.start, clip.end)
            })();
            match result {
                Ok(player) => {
                    self.stop_audio();
                    self.editing.k.preview_player = Some(player);
                    self.editing.toolbar.preview_error = None;
                }
                Err(e) => {
                    self.editing.toolbar.preview_error = Some(format!("{id}: can't preview · {e}"))
                }
            }
        }
        if !self.editing.k.capture_done
            && !self.editing.loading()
            && self.pending.is_none()
            && self.editing.turns.is_some()
        {
            self.editing.k.capture_done = true;
            if self.editing.k.capture_insert
                && let Some(i) = self.selected
            {
                self.begin_insert(i + 1);
            }
        }
    }
    pub(crate) fn phase_k_windows(&mut self, ctx: &egui::Context) {
        if let Some(index) = self.editing.k.delete {
            egui::Modal::new(egui::Id::new("delete_turn")).show(ctx, |ui| {
                ui.label(t::heading("Delete rendered turn?", t::SECTION_TITLE));
                ui.label(
                    "The rendered audio stays unchanged until Detect changes → Precision insert.",
                );
                if ui.button("Cancel").clicked() {
                    self.editing.k.delete = None;
                }
                if ui
                    .button(t::text("Delete turn", t::BODY, t::STATUS_FAILED))
                    .clicked()
                {
                    self.editing.k.delete = None;
                    self.start_structure(EditOp::Delete { index }, index.saturating_sub(1), ctx);
                }
            });
        }
        if let Some(preview) = self.editing.k.retime.clone() {
            egui::Window::new("Retime script · preview").show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(t::CLIP_PICKER_HEIGHT)
                    .show(ui, |ui| {
                        for row in &preview.rows {
                            ui.label(format!(
                                "{:03} {}   {} → {}",
                                row.turn_index, row.display_name, row.old, row.new
                            ));
                        }
                    });
                if preview.rows.is_empty() {
                    ui.label("Timestamps already match.");
                }
                if let Some(note) = &self.editing.notice {
                    ui.label(note);
                }
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.editing.k.retime = None;
                    }
                    if ui
                        .add_enabled(
                            !self.editing.busy() && !preview.rows.is_empty(),
                            egui::Button::new("Apply"),
                        )
                        .clicked()
                        && let Some(episode) = self.episode.clone()
                    {
                        let (tx, rx) = mpsc::channel();
                        self.editing.k.applying = Some(rx);
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(retime::apply(&episode, &preview));
                            ctx.request_repaint();
                        });
                    }
                });
            });
        }
    }
    pub(crate) fn poll_auto_retime(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.auto_retime_apply {
            if let Ok(result) = rx.try_recv() {
                self.auto_retime_apply = None;
                match result {
                    Ok((checkpoint, n)) => {
                        self.retime_banner = Some((
                            format!("Retimed {n} timestamps to match the audio"),
                            checkpoint,
                        ));
                        if let Some(episode) = &self.episode {
                            let dir = episode.dir.clone();
                            self.open(dir, ctx);
                        }
                    }
                    Err(error) => self.editing.notice = Some(error),
                }
            }
            return;
        }
        if let Some(rx) = &self.auto_retime_job {
            if let Ok(result) = rx.try_recv() {
                self.auto_retime_job = None;
                if let Ok(preview) = result {
                    let n = preview.rows.len();
                    if let retime::AutoRetime::Apply(_) =
                        retime::auto_retime_decision(true, false, false, Some(n))
                        && let Some(episode) = self.episode.clone()
                    {
                        let (tx, rx) = mpsc::channel();
                        self.auto_retime_apply = Some(rx);
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(retime::apply(&episode, &preview).map(|cp| (cp, n)));
                            ctx.request_repaint();
                        });
                    }
                } else if let Err(error) = result {
                    self.editing.notice = Some(error);
                }
            }
            return;
        }
        if !self.retime_arm || self.editing.k.retime.is_some() {
            return;
        }
        let decision = retime::auto_retime_decision(
            self.preferences.retime_on_open,
            self.jobs.busy(),
            self.unsaved_script(),
            None,
        );
        match decision {
            retime::AutoRetime::Off => self.retime_arm = false,
            retime::AutoRetime::WaitForJob | retime::AutoRetime::WaitForEdits => {}
            retime::AutoRetime::Run => {
                if self.jobs.resolving.is_some() || self.export_command.is_none() {
                    return;
                }
                let Some(Ok(export)) = self.export_command.clone() else {
                    self.retime_arm = false;
                    return;
                };
                let Some(mut episode) = self.episode.clone() else {
                    self.retime_arm = false;
                    return;
                };
                if let Some(turns) = &self.editing.turns {
                    episode.script_path = Some(turns.script_path.clone());
                }
                if episode.script_path.is_none() {
                    self.retime_arm = false;
                    return;
                }
                self.retime_arm = false;
                let (tx, rx) = mpsc::channel();
                self.auto_retime_job = Some(rx);
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let _ = tx.send(retime::preview(&export, &episode));
                    ctx.request_repaint();
                });
            }
            retime::AutoRetime::Silent | retime::AutoRetime::Apply(_) => {}
        }
    }
    fn unsaved_script(&self) -> bool {
        self.editing.draft.as_ref().is_some_and(|d| d.changed())
            || self.editing.k.new_turn.is_some()
            || self.editing.busy()
    }
    pub(crate) fn retime_button(&mut self, ui: &mut Ui) {
        if t::secondary_button(
            ui,
            "Retime script",
            None,
            !self.editing.busy()
                && !self.jobs.busy()
                && self.editing.draft.is_none()
                && self.editing.k.new_turn.is_none(),
        )
        .clicked()
            && let (Some(Ok(export)), Some(mut episode)) =
                (self.export_command.clone(), self.episode.clone())
        {
            if let Some(turns) = &self.editing.turns {
                episode.script_path = Some(turns.script_path.clone());
            }
            let (tx, rx) = mpsc::channel();
            self.editing.k.retiming = Some(rx);
            let ctx = ui.ctx().clone();
            std::thread::spawn(move || {
                let _ = tx.send(retime::preview(&export, &episode));
                ctx.request_repaint();
            });
        }
    }
    pub(crate) fn structure_panel(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.editing.k.new_turn.is_some() && self.preferences.edit_mode == EditMode::Panel {
                new_editor(ui, &mut self.editing, true);
            } else if self.editing.draft.is_some() && self.preferences.edit_mode == EditMode::Panel
            {
                let action = editing::editor(
                    ui,
                    &mut self.editing,
                    t::EDITOR_MIN_HEIGHT,
                    false,
                    !self.jobs.busy(),
                    None,
                );
                self.edit_action(action, &ctx);
            } else if let Some(turn) = self
                .editing
                .k
                .selected
                .and_then(|i| self.editing.turns.as_ref()?.turns.get(i))
            {
                ui.label(t::heading(&turn.display_name, t::SECTION_TITLE));
                ui.label(&turn.body);
                if ui.button("Edit transcript").clicked() {
                    self.editing.open(turn.index);
                }
            }
            if self.editing.k.structure_changed {
                ui.label(t::text(
                    "Unsaved · structure changed",
                    t::CAPTION,
                    t::accent(),
                ));
                ui.separator();
                ui.label("NEXT STEP");
                ui.label(t::text(
                    "Detect changes → Precision insert",
                    t::BODY,
                    t::accent(),
                ));
            }
            if let Some(note) = &self.editing.notice {
                ui.label(note);
            }
            if let Some(report) = &self.editing.k.authority {
                for p in &report.problems {
                    ui.label(t::text(
                        format!("{}: {} {}", p.index + 1, p.tag, p.message),
                        t::CAPTION,
                        t::STATUS_FAILED,
                    ));
                }
            }
        });
    }
}
fn estimate_timestamp(
    path: &std::path::Path,
    previous: Option<usize>,
    next: Option<usize>,
) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<_> = text.lines().collect();
    let seconds = |line: usize| -> Option<u64> {
        let header = lines.get(line.wrapping_sub(1))?;
        let start = header.rfind('(')?;
        let end = header.rfind(')')?;
        if end <= start {
            return None;
        }
        let mut parts = header[start + 1..end].split(':');
        let first = parts.next()?.parse::<u64>().ok()?;
        let second = parts.next()?.parse::<u64>().ok()?;
        Some(match parts.next() {
            Some(third) => first * 3600 + second * 60 + third.parse::<u64>().ok()?,
            None => first * 60 + second,
        })
    };
    let total = match (previous.and_then(seconds), next.and_then(seconds)) {
        (Some(before), Some(after)) => (before + after) / 2,
        (Some(before), None) => before + 5,
        _ => 0,
    };
    format!("{:02}:{:02} est.", total / 60, total % 60)
}

fn receive<T>(receiver: &mut Option<Receiver<Result<T, String>>>) -> Option<Result<T, String>> {
    let result = match receiver.as_ref()?.try_recv() {
        Ok(r) => r,
        Err(mpsc::TryRecvError::Empty) => return None,
        Err(mpsc::TryRecvError::Disconnected) => {
            Err("Worker stopped; reload before retrying.".into())
        }
    };
    *receiver = None;
    Some(result)
}

fn update_mapping(rows: &mut Vec<Option<usize>>, op: &EditOp) {
    match op {
        EditOp::Insert { position, .. } => rows.insert(*position, None),
        EditOp::Delete { index } => {
            rows.remove(*index);
        }
        EditOp::Move { index, to } => {
            let item = rows.remove(*index);
            rows.insert(*to, item);
        }
        EditOp::SetSpeaker { .. } => {}
    }
}
