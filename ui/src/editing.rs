//! Shared transcript draft, worker lifecycle and views for both editing locations.
use crate::{Browser, inspector::InspectorMode};
use eframe::egui::{self, Ui};
use crate::{
    checkpoint::Checkpoint,
    runner::Preset,
    script::{self, Changes, ScriptTurn, ScriptTurns},
    theme as t,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver};

pub const FOOTNOTE: &str =
    "Audio remains unchanged. To replace rendered audio, save this draft and confirm synthesis.";
pub const CLIP_FOOTNOTE: &str =
    "Audio clip: this corrects the transcript only. The original recording is not regenerated.";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditMode {
    #[default]
    Panel,
    Row,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub turn: usize,
    pub text: String,
    pub original: String,
}
impl Draft {
    pub fn changed(&self) -> bool {
        self.text != self.original
    }
}

struct Loaded {
    turns: Result<ScriptTurns, String>,
    changes: Result<Changes, String>,
    registry: Result<crate::structure::Registry, String>,
}

#[derive(Default)]
pub struct Editing {
    pub k: crate::phase_k::PhaseK,
    pub toolbar: crate::tag_editor::Toolbar,
    pub registry: crate::structure::Registry,
    pub manifest_len: Option<usize>,
    pub draft: Option<Draft>,
    pub turns: Option<ScriptTurns>,
    pub changes: Option<Changes>,
    pub loaded_sha256: Option<String>,
    pub error: Option<String>,
    pub changes_error: Option<String>,
    pub notice: Option<String>,
    pub edit_tab: bool,
    pub focus_requested: bool,
    pub pending_selection: Option<(usize, bool)>,
    pub capture_open: bool,
    pub load_requested: bool,
    loading: Option<Receiver<Loaded>>,
    saving: Option<Receiver<Result<(Checkpoint, crate::tags::Report), String>>>,
}
impl Editing {
    pub fn busy(&self) -> bool {
        self.saving.is_some() || self.k.busy()
    }
    pub fn loading(&self) -> bool {
        self.load_requested || self.loading.is_some()
    }
    pub fn open(&mut self, turn: usize) {
        self.edit_tab = true;
        if self.draft.as_ref().is_some_and(|d| d.turn == turn) {
            self.focus_requested = true;
            return;
        }
        self.toolbar.cursor = None;
        if let Some(body) = self.body(turn) {
            self.draft = Some(Draft {
                turn,
                text: body.to_owned(),
                original: body.to_owned(),
            });
            self.focus_requested = true;
        }
    }
    pub fn cancel(&mut self) {
        if !self.busy() {
            self.k.new_turn = None;
            self.draft = None;
            self.edit_tab = false;
            self.focus_requested = false;
        }
    }
    /// Returns false until the operator chooses Discard or Keep editing.
    pub fn allow_selection(&mut self, turn: usize, open: bool) -> bool {
        if self.busy() {
            return false;
        }
        if self
            .draft
            .as_ref()
            .is_some_and(|d| d.turn != turn && d.changed())
        {
            self.pending_selection = Some((turn, open));
            return false;
        }
        if self.draft.as_ref().is_some_and(|d| d.turn != turn) {
            self.cancel();
        }
        true
    }
    pub fn script_turn(&self, row: usize) -> Option<&ScriptTurn> {
        let turns = &self.turns.as_ref()?.turns;
        if !self.k.script_view && self.manifest_len.is_some_and(|n| n != turns.len()) {
            return None;
        }
        turns.get(row)
    }
    pub fn body(&self, row: usize) -> Option<&str> {
        Some(&self.script_turn(row)?.body)
    }
    pub fn unavailable(&self) -> &str {
        if self
            .turns
            .as_ref()
            .is_some_and(|s| self.manifest_len.is_some_and(|n| n != s.turns.len()))
        {
            return "Turns were added or removed since render. Re-render to sync.";
        }
        self.error.as_deref().unwrap_or(if self.loading() {
            "Loading script…"
        } else {
            "No script text for this turn"
        })
    }
    pub fn markers(&self, manifest_len: usize) -> Option<Vec<usize>> {
        match (&self.changes, &self.turns) {
            (Some(changes), Some(turns)) => {
                script::edited_turns(changes, manifest_len, turns.turns.len())
            }
            _ => Some(vec![]),
        }
    }
    pub fn has_edits(&self) -> bool {
        self.changes.as_ref().is_some_and(|c| {
            !c.inserted.is_empty() || !c.modified.is_empty() || !c.deleted.is_empty()
        })
    }
    pub fn invalidate(&mut self) {
        self.loading = None;
        self.turns = None;
        self.changes = None;
        self.loaded_sha256 = None;
        self.error = None;
        self.changes_error = None;
        self.load_requested = true;
    }
}

pub fn editor_id() -> egui::Id {
    egui::Id::new("transcript_editor")
}

/// Text editing owns Space and arrows; dialogs also suppress transport shortcuts.
pub fn suppress_shortcuts(editor_focused: bool, text_focused: bool, dialog: bool) -> bool {
    editor_focused || text_focused || dialog
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Open,
    Save,
    Cancel,
    Render,
    UseEnsemble,
    Regenerate,
}

pub fn field_height(ui: &Ui, text: &str, width: f32, available: f32) -> f32 {
    (t::wrapped_height(
        ui,
        text,
        t::font(t::BODY),
        (width - t::SPACE_16).max(t::SPACE_16),
    ) + t::SPACE_16)
        .clamp(t::EDITOR_MIN_HEIGHT, available.max(t::EDITOR_MIN_HEIGHT))
}

pub fn mode_toggle(ui: &mut Ui, mode: &mut EditMode) -> egui::Response {
    let label = "Inline / Side panel";
    let title_width = t::text_width(ui, label, t::bold(t::BUTTON));
    let pill_width = t::text_width(ui, "Side panel", t::font(t::CAPTION)) + t::SPACE_8 * 2.0;
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(
            title_width + pill_width + t::SPACE_8 * 3.0,
            t::BUTTON_HEIGHT,
        ),
        egui::Sense::click(),
    );
    if response.clicked() {
        *mode = if *mode == EditMode::Row {
            EditMode::Panel
        } else {
            EditMode::Row
        };
        response.mark_changed();
    }
    let active = if *mode == EditMode::Row {
        "Inline"
    } else {
        "Side panel"
    };
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    ui.painter()
        .rect_filled(rect, t::RADIUS_CONTROL, t::accent());
    let title = rect.shrink2(egui::vec2(t::SPACE_8, 0.0));
    crate::view::line(ui, title, label, t::bold(t::BUTTON), t::ON_ACCENT);
    let pill = egui::Rect::from_min_max(
        egui::pos2(
            rect.right() - pill_width - t::SPACE_8,
            rect.top() + t::SPACE_4,
        ),
        rect.right_bottom() - egui::vec2(t::SPACE_8, t::SPACE_4),
    );
    ui.painter()
        .rect_filled(pill, t::READ_ALONG_RADIUS, t::BG_WINDOW);
    crate::view::line(
        ui,
        pill.shrink2(egui::vec2(t::SPACE_8, 0.0)),
        active,
        t::font(t::CAPTION),
        t::accent(),
    );
    response
}

pub fn inline_field_height(ui: &Ui, text: &str, width: f32) -> f32 {
    let line = t::line_height(ui, t::font(t::BODY));
    t::wrapped_height(ui, text, t::font(t::BODY), width.max(t::SPACE_16))
        .clamp(line * t::EDITOR_ROW_MIN_LINES, t::EDITOR_MAX_HEIGHT)
}

/// Toolbar, wrapped body (at least three lines), raw-script row, and buttons.
pub fn row_editor_block_height(ui: &Ui, text: &str, width: f32, draft: Option<&Draft>) -> f32 {
    t::TAG_TOOLBAR_HEIGHT
        + t::SPACE_4
        + inline_field_height(ui, text, width)
        + t::line_height(ui, t::font(t::CAPTION))
        + t::SPACE_8
        + inline_actions_height(ui, width, draft)
        + t::SPACE_4
}

fn inline_actions_width(ui: &Ui) -> f32 {
    t::text_width(ui, "Save draftCancel", t::bold(t::SECONDARY))
        + t::BUTTON_PAD_X * 4.0
        + t::text_width(ui, "Unsaved draft", t::font(t::SECONDARY))
        + t::SPACE_14
        + t::SPACE_8 * 4.0
}

/// Rendered height of a Save draft / Cancel button: the bold label's row
/// height plus button padding, never less than the token. OpenDyslexic's rows
/// are taller than the token assumes, and an underestimate let the buttons
/// spill into the next table row, which then took their clicks.
fn inline_button_height(ui: &Ui) -> f32 {
    let row = ui.fonts_mut(|f| f.row_height(&t::bold(t::SECONDARY)));
    (row + ui.spacing().button_padding.y * 2.0).max(t::INLINE_BUTTON_HEIGHT)
}

pub fn inline_actions_height(ui: &Ui, width: f32, draft: Option<&Draft>) -> f32 {
    let button = inline_button_height(ui);
    let rows = if draft.is_some_and(Draft::changed) && width < inline_actions_width(ui) {
        button * 2.0 + t::SPACE_4
    } else {
        button
    };
    rows + t::INLINE_EDITOR_BOTTOM_MARGIN
}

fn draft_warning(ui: &mut Ui) {
    ui.add(
        egui::Image::new(t::icon("circle-alert-14-danger"))
            .fit_to_exact_size(egui::Vec2::splat(t::SPACE_14)),
    );
    ui.label(t::text("Unsaved draft", t::SECONDARY, t::STATUS_FAILED));
}

pub fn inline_editor(
    ui: &mut Ui,
    editing: &mut Editing,
    height: f32,
    enabled: bool,
) -> Option<Action> {
    let busy = editing.busy();
    let draft = editing.draft.as_mut()?;
    let mut action = None;
    ui.spacing_mut().item_spacing = egui::vec2(t::SPACE_8, t::SPACE_4);
    ui.set_width(ui.available_width());
    ui.add_enabled_ui(!busy, |ui| {
        let response = crate::tag_editor::body(
            ui,
            &mut draft.text,
            &mut editing.toolbar,
            &editing.registry,
            height,
            "Write this turn…",
            t::EDITOR_ROW_MIN_LINES,
        );
        if editing.focus_requested {
            response.request_focus();
            editing.focus_requested = false;
        }
    });
    let wrap_warning = ui.available_width() < inline_actions_width(ui);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        for (label, value, primary, allowed) in [
            ("Cancel", Action::Cancel, false, !busy),
            (
                if busy { "Saving…" } else { "Save draft" },
                Action::Save,
                true,
                enabled
                    && !busy
                    && draft.changed()
                    && crate::tags::can_save(&draft.text, &editing.registry),
            ),
        ] {
            let button = egui::Button::new(t::heading(label, t::SECONDARY).color(if primary {
                t::ON_ACCENT
            } else {
                t::TEXT_PRIMARY
            }))
            .fill(if primary { t::accent() } else { t::BG_CONTROL })
            .corner_radius(t::RADIUS_CONTROL as u8)
            .min_size(egui::vec2(0.0, t::INLINE_BUTTON_HEIGHT));
            if ui.add_enabled(allowed, button).clicked() {
                action = Some(value);
            }
        }
        if draft.changed() && !wrap_warning {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(t::text("Unsaved draft", t::SECONDARY, t::STATUS_FAILED));
                ui.add(
                    egui::Image::new(t::icon("circle-alert-14-danger"))
                        .fit_to_exact_size(egui::Vec2::splat(t::SPACE_14)),
                );
            });
        }
    });
    if draft.changed() && wrap_warning {
        ui.horizontal(draft_warning);
    }
    if script::clip_id(&draft.original).is_some() {
        ui.label(t::text(CLIP_FOOTNOTE, t::CAPTION, t::TEXT_SUBTLE));
    }
    action
}

pub fn editor(
    ui: &mut Ui,
    editing: &mut Editing,
    height: f32,
    row: bool,
    enabled: bool,
    ensemble: Option<&str>,
) -> Option<Action> {
    let busy = editing.busy();
    let draft = editing.draft.as_mut()?;
    let mut action = None;
    ui.spacing_mut().item_spacing.y = t::SPACE_12;
    t::field_label(ui, "Transcript");
    ui.add_enabled_ui(!busy, |ui| {
        let response = egui::ScrollArea::vertical()
            .id_salt("transcript_field_scroll")
            .max_height(height)
            .min_scrolled_height(height)
            .show(ui, |ui| {
                crate::tag_editor::body(
                    ui,
                    &mut draft.text,
                    &mut editing.toolbar,
                    &editing.registry,
                    height,
                    "Write this turn…",
                    t::EDITOR_BODY_LINES,
                )
            })
            .inner;
        if editing.focus_requested {
            response.request_focus();
            editing.focus_requested = false;
        }
    });
    if draft.changed() {
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(t::icon("circle-alert-14-danger"))
                    .fit_to_exact_size(egui::Vec2::splat(t::SPACE_14)),
            );
            ui.label(t::text("Unsaved draft", t::SECONDARY, t::STATUS_FAILED));
        });
    }
    let clip = script::clip_id(&draft.original).is_some();
    let footnote = if clip { CLIP_FOOTNOTE } else { FOOTNOTE };
    if row {
        ui.label(t::text(footnote, t::CAPTION, t::TEXT_SUBTLE));
    }
    if clip {
        let can_use = enabled && !busy && ensemble.is_some_and(|t| !t.trim().is_empty());
        let response = t::secondary_button(ui, "Use ensemble transcript", None, can_use);
        let response = if ensemble.is_none() {
            response
                .on_disabled_hover_text("No ensemble transcript yet. Run Jobs → Clip transcripts.")
        } else {
            response
        };
        if response.clicked() {
            action = Some(Action::UseEnsemble);
        }
    }
    let save_enabled = enabled
        && !busy
        && draft.changed()
        && crate::tags::can_save(&draft.text, &editing.registry);
    let buttons = |ui: &mut Ui| {
        if t::primary_button(
            ui,
            if busy { "Saving…" } else { "Save draft" },
            None,
            save_enabled,
        )
        .clicked()
        {
            action = Some(Action::Save);
        }
        if t::secondary_button(ui, "Cancel", None, !busy).clicked() {
            action = Some(Action::Cancel);
        }
    };
    if row {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), buttons);
    } else {
        ui.horizontal(buttons);
        ui.label(t::text(footnote, t::CAPTION, t::TEXT_SUBTLE));
    }
    action
}

impl Browser {
    pub(crate) fn open_editor(&mut self, row: usize) {
        if !self.editing.allow_selection(row, true) {
            return;
        }
        self.select(row);
        self.jobs.mode = Some(InspectorMode::SelectedTurn);
        self.editing.open(row);
        self.scroll_to = Some(row);
    }
    pub(crate) fn edit_action(&mut self, action: Option<Action>, ctx: &egui::Context) {
        match action {
            Some(Action::Open) => {
                if let Some(row) = self.selected {
                    self.open_editor(row);
                }
            }
            Some(Action::Cancel) => self.editing.cancel(),
            Some(Action::Save) => self.save_draft(ctx),
            Some(Action::UseEnsemble) => self.apply_ensemble_transcript(),
            Some(Action::Regenerate) => {
                let rows = self
                    .editing
                    .markers(self.episode.as_ref().map_or(0, |e| e.turns.len()))
                    .unwrap_or_default();
                self.request_regenerate_edited(rows, ctx);
            }
            Some(Action::Render) => {
                if let (Some(c), Some(turns)) = (&self.editing.changes, &self.editing.turns) {
                    let scope = script::RenderScope::from_changes(c, &turns.turns);
                    if scope.synthesis_enabled() {
                        self.request_job(Preset::PrecisionInsert, Some(scope.synthesis), ctx);
                    }
                }
            }
            None => {}
        }
    }
    pub(crate) fn apply_ensemble_transcript(&mut self) {
        let Some(draft) = self.editing.draft.as_mut() else {
            return;
        };
        let Some(episode) = self.episode.as_ref() else {
            return;
        };
        let Some(turn) = episode.turns.get(draft.turn) else {
            return;
        };
        let Some(clip) = episode.clip_transcripts.get(&turn.id) else {
            self.editing.notice =
                Some("No ensemble transcript yet. Run Jobs → Clip transcripts.".into());
            return;
        };
        let source = if script::clip_tag(&draft.text).is_some() {
            draft.text.clone()
        } else {
            draft.original.clone()
        };
        match crate::clip_transcripts::use_ensemble_transcript(
            &source,
            &clip.ensemble_text(),
        ) {
            Ok(text) => {
                draft.text = text;
                self.editing.notice = Some("Draft updated from ensemble transcript.".into());
            }
            Err(error) => self.editing.notice = Some(error),
        }
    }
    pub(crate) fn save_draft(&mut self, ctx: &egui::Context) {
        if self.editing.k.new_turn.is_some() {
            self.save_new_turn(ctx);
            return;
        }
        if self.editing.busy()
            || self.jobs.busy()
            || self.pending.is_some()
            || self.editing.loading()
        {
            return;
        }
        let Some(draft) = self.editing.draft.as_ref().filter(|d| d.changed()) else {
            return;
        };
        let (Some(mut episode), Some(turn), Some(hash)) = (
            self.episode.clone(),
            self.editing.script_turn(draft.turn).cloned(),
            self.editing.loaded_sha256.clone(),
        ) else {
            return;
        };
        episode.script_path = self.editing.turns.as_ref().map(|s| s.script_path.clone());
        let text = draft.text.clone();
        if !crate::tags::can_save(&text, &self.editing.registry) {
            return;
        }
        let Some(Ok(export)) = self.export_command.clone() else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        self.editing.saving = Some(rx);
        self.editing.notice = Some("Saving draft…".into());
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = script::save(&episode, &turn, &text, &hash).and_then(|cp| {
                let report = script::load_tags(&export, &episode)?;
                Ok((cp, report))
            });
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(crate) fn poll_editing(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.editing.saving {
            let result = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Script save worker stopped".into()))
                }
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                self.editing.saving = None;
                match result {
                    Ok((cp, report)) => {
                        self.jobs.regenerated.clear();
                        self.editing.k.authority = Some(report);
                        let number = self
                            .selected
                            .and_then(|i| self.episode.as_ref()?.turns.get(i))
                            .map(|t| t.index);
                        let name = self
                            .editing
                            .turns
                            .as_ref()
                            .and_then(|s| s.script_path.file_name());
                        self.editing.notice = Some(format!(
                            "Saved turn {:03} to {} · checkpoint v{:02}",
                            number.unwrap_or(0),
                            name.unwrap_or_default().to_string_lossy(),
                            cp.version,
                        ));
                        if let Some((count, turns)) = self.find.saved_counts.take() {
                            self.editing.notice = Some(format!(
                                "Replaced {count} in {turns} turns · checkpoint v{:02}",
                                cp.version,
                            ));
                        }
                        self.editing.cancel();

                        self.refresh_checkpoints();
                        self.reload(ctx);
                    }
                    Err(error) => {
                        self.find.saved_counts = None;
                        self.editing.notice = Some(error);
                    }
                }
            }
        }
        if self.editing.load_requested
            && self.pending.is_none()
            && let (Some(export), Some(episode)) = (&self.export_command, &self.episode)
        {
            self.editing.load_requested = false;
            match export {
                Ok(export) => {
                    let export = export.clone();
                    let episode = episode.clone();
                    let (tx, rx) = mpsc::channel();
                    self.editing.loading = Some(rx);
                    let ctx = ctx.clone();
                    std::thread::spawn(move || {
                        let turns = script::load_turns(&export, &episode);
                        let changes = script::load_changes(&export, &episode);
                        let registry = script::load_registry(&export, &episode);
                        let _ = tx.send(Loaded {
                            turns,
                            changes,
                            registry,
                        });
                        ctx.request_repaint();
                    });
                }
                Err(error) => self.editing.error = Some(error.clone()),
            }
        }
        if let Some(rx) = &self.editing.loading {
            let result = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Loaded {
                    turns: Err("Script loader stopped".into()),
                    changes: Err("Script loader stopped".into()),
                    registry: Err("Speaker loader stopped".into()),
                }),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                self.editing.loading = None;
                match result.turns {
                    Ok(turns) => {
                        self.editing.loaded_sha256 = Some(turns.script_sha256.clone());
                        self.editing.turns = Some(turns);
                        // Full transcript heights replace first_words; re-anchor.
                        if let Some(row) = self.selected {
                            self.scroll_to = Some(row);
                        }
                    }
                    Err(error) => self.editing.error = Some(error),
                }
                match result.registry {
                    Ok(registry) => self.editing.registry = registry,
                    Err(error) => self.editing.notice = Some(error),
                }
                match result.changes {
                    Ok(changes) => self.editing.changes = Some(changes),
                    Err(error) => self.editing.changes_error = Some(error),
                }
            }
        }
        if self.editing.capture_open && !self.editing.loading() && self.pending.is_none() {
            self.editing.capture_open = false;
            if let Some(row) = self.selected {
                self.open_editor(row);
            }
            if self.editing.draft.is_none() && self.capture.is_some() {
                self.select_error = Some(format!(
                    "Cannot open editor: {}",
                    self.editing.unavailable()
                ));
            }
        }
    }
    pub(crate) fn draft_guard(&mut self, ctx: &egui::Context) {
        let Some((row, open)) = self.editing.pending_selection else {
            return;
        };
        let mut discard = false;
        let mut keep = false;
        let response = egui::Modal::new(egui::Id::new("draft_guard")).show(ctx, |ui| {
            ui.set_max_width(t::DIALOG_WIDTH);
            ui.label(t::heading("Discard unsaved draft?", t::SECTION_TITLE));
            ui.label(t::text(
                "This turn has an unsaved draft.",
                t::BODY,
                t::TEXT_PRIMARY,
            ));
            ui.horizontal(|ui| {
                keep = t::primary_button(ui, "Keep editing", None, true).clicked();
                discard = t::secondary_button(ui, "Discard", None, true).clicked();
            });
        });
        if discard {
            self.editing.pending_selection = None;
            self.editing.cancel();
            self.select(row);
            if open {
                self.open_editor(row);
            }
        } else if keep || response.should_close() {
            self.editing.pending_selection = None;
            self.editing.focus_requested = true;
        }
    }
}

impl Browser {
    pub(crate) fn save_replacements(
        &mut self,
        plan: crate::find::Plan,
        ctx: &egui::Context,
    ) {
        if self.editing.draft.is_some() {
            self.editing.notice = Some("Save or cancel the open edit first".into());
            return;
        }
        if self.editing.busy()
            || self.jobs.busy()
            || self.pending.is_some()
            || self.editing.loading()
        {
            return;
        }
        let (Some(mut episode), Some(turns), Some(hash)) = (
            self.episode.clone(),
            self.editing.turns.as_ref(),
            self.editing.loaded_sha256.clone(),
        ) else {
            return;
        };
        if self.editing.manifest_len != Some(turns.turns.len()) {
            self.editing.notice = Some(self.editing.unavailable().into());
            return;
        }
        episode.script_path = Some(turns.script_path.clone());
        self.find.saved_counts = Some((plan.occurrences, plan.edits.len()));
        let export = self.export_command.clone();
        let (tx, rx) = mpsc::channel();
        self.editing.saving = Some(rx);
        self.editing.notice = Some("Saving replacements…".into());
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = script::save_many(&episode, &plan.edits, &hash).and_then(|cp| {
                let report = match export {
                    Some(Ok(export)) => script::load_tags(&export, &episode)?,
                    _ => crate::tags::Report {
                        script_sha256: hash,
                        problems: Vec::new(),
                    },
                };
                Ok((cp, report))
            });
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Preferences, Transport};
    use crate::episode::Episode;
    use std::path::PathBuf;

    fn loaded() -> Editing {
        Editing {
            turns: Some(ScriptTurns {
                script_path: "script.md".into(),
                script_sha256: "test-hash".into(),
                line_ending: "lf".into(),
                turns: (0..3)
                    .map(|index| ScriptTurn {
                        index,
                        turn_id: format!("turn-{index}"),
                        speaker_id: "alice".into(),
                        display_name: "Alice".into(),
                        header_line: index * 3 + 1,
                        body_start: index * 3 + 2,
                        body_end: index * 3 + 2,
                        body: format!("Body {index}"),
                    })
                    .collect(),
            }),
            changes: Some(Changes {
                inserted: vec![1],
                modified: vec![1, 2],
                deleted: vec!["old".into()],
                unchanged: 1,
            }),
            ..Default::default()
        }
    }
    fn app() -> Browser {
        Browser {
            episode: Some(
                Episode::load(
                    &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small"),
                )
                .unwrap(),
            ),
            editing: loaded(),
            selected: Some(0),
            ..Default::default()
        }
    }
    #[test]
    fn open_change_cancel_and_switch_modes_keep_one_draft() {
        let mut app = app();
        app.open_editor(0);
        assert_eq!(app.editing.draft.as_ref().unwrap().text, "Body 0");
        assert!(!app.editing.draft.as_ref().unwrap().changed());
        app.editing
            .draft
            .as_mut()
            .unwrap()
            .text
            .push_str(" changed");
        let expected = app.editing.draft.clone();
        app.preferences.edit_mode = EditMode::Row;
        assert_eq!(app.editing.draft, expected);
        app.preferences.edit_mode = EditMode::Panel;
        app.open_editor(0);
        assert_eq!(app.editing.draft, expected);
        app.editing.cancel();
        assert!(app.editing.draft.is_none());
        assert_eq!(app.editing.body(0), Some("Body 0"));
    }
    #[test]
    fn selection_and_double_click_guard_preserve_selection_and_draft() {
        let mut app = app();
        app.open_editor(0);
        app.editing.draft.as_mut().unwrap().text.push('!');
        app.select(1);
        assert_eq!(app.selected, Some(0));
        assert_eq!(app.editing.pending_selection, Some((1, false)));
        app.editing.pending_selection = None; // Keep editing.
        assert!(app.editing.draft.as_ref().unwrap().changed());
        app.open_editor(2);
        assert_eq!(app.selected, Some(0));
        assert_eq!(app.editing.pending_selection, Some((2, true)));
        app.editing.cancel(); // Discard, then select/open the pending target.
        app.editing.pending_selection = None;
        app.open_editor(2);
        assert_eq!(app.selected, Some(2));
        assert_eq!(app.editing.draft.as_ref().unwrap().text, "Body 2");
    }
    #[test]
    fn marker_mapping_and_count_mismatch_banner() {
        let mut editing = loaded();
        assert_eq!(editing.markers(3), Some(vec![1, 2]));
        assert_eq!(editing.markers(4), None);
        editing.changes.as_mut().unwrap().inserted.push(5);
        assert_eq!(editing.markers(3), None);
        editing.changes = None;
        assert_eq!(editing.markers(3), Some(vec![]));
    }
    #[test]
    fn shortcut_suppression_and_focus_scoped_escape() {
        assert!(suppress_shortcuts(true, false, false));
        assert!(suppress_shortcuts(false, true, false));
        assert!(suppress_shortcuts(false, false, true));
        assert!(!suppress_shortcuts(false, false, false));
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut app = app();
        app.open_editor(0);
        app.editing.draft.as_mut().unwrap().text.push('!');
        ctx.memory_mut(|m| m.request_focus(editor_id()));
        for key in [egui::Key::Space, egui::Key::ArrowDown, egui::Key::ArrowUp] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |_| {
                    ctx.memory_mut(|m| m.request_focus(editor_id()));
                    app.shortcuts(&ctx);
                },
            );
            output.textures_delta.clear();
            assert_eq!(app.selected, Some(0));
            assert_eq!(app.transport, Transport::Idle);
            assert!(app.editing.draft.is_some());
        }
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |_| {
                ctx.memory_mut(|m| m.request_focus(editor_id()));
                app.shortcuts(&ctx);
            },
        );
        output.textures_delta.clear();
        assert!(app.editing.draft.is_none());
    }
    #[test]
    fn both_layouts_render_and_preferences_round_trip() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut app = app();
        app.open_editor(0);
        app.editing
            .draft
            .as_mut()
            .unwrap()
            .text
            .push_str("\nA second line.");
        for mode in [EditMode::Panel, EditMode::Row] {
            app.preferences.edit_mode = mode;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        t::WINDOW_SIZE.into(),
                    )),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            output.textures_delta.clear();
            assert_eq!(
                app.editing.draft.as_ref().unwrap().text,
                "Body 0\nA second line."
            );
        }
        let old: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(old.edit_mode, EditMode::Panel);
        let restored: Preferences =
            serde_json::from_str(&serde_json::to_string(&app.preferences).unwrap()).unwrap();
        assert_eq!(restored.edit_mode, EditMode::Row);
    }
    #[test]
    fn mode_toggle_writes_the_persisted_edit_mode() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut preferences = Preferences::default();
        assert_eq!(preferences.edit_mode, EditMode::Panel);
        let mut rect = egui::Rect::NOTHING;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            rect = mode_toggle(ui, &mut preferences.edit_mode).rect;
        });
        output.textures_delta.clear();
        let pos = rect.center();
        for pressed in [true, false] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ui| {
                    mode_toggle(ui, &mut preferences.edit_mode);
                },
            );
            output.textures_delta.clear();
        }
        assert_eq!(preferences.edit_mode, EditMode::Row);
        let restored: Preferences =
            serde_json::from_str(&serde_json::to_string(&preferences).unwrap()).unwrap();
        assert_eq!(restored.edit_mode, EditMode::Row);
    }
    #[test]
    fn stale_load_receiver_is_invalidated_and_worker_loss_is_reported() {
        let mut app = app();
        let (tx, rx) = mpsc::channel();
        app.editing.loading = Some(rx);
        app.editing.invalidate();
        assert!(
            tx.send(Loaded {
                turns: Err("old".into()),
                changes: Err("old".into()),
                registry: Err("old".into())
            })
            .is_err()
        );
        let (tx, rx) = mpsc::channel();
        app.editing.saving = Some(rx);
        drop(tx);
        app.poll_editing(&egui::Context::default());
        assert_eq!(
            app.editing.notice.as_deref(),
            Some("Script save worker stopped")
        );
    }
    #[test]
    fn row_editor_height_grows_for_long_text() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let short = row_editor_block_height(ui, "Hi", 160.0, None);
            let long = row_editor_block_height(ui, &"word ".repeat(80), 160.0, None);
            let line = t::line_height(ui, t::font(t::BODY));
            assert!(short >= line * t::EDITOR_ROW_MIN_LINES);
            assert!(
                long > short,
                "long text should be taller than a short draft"
            );
        });
        output.textures_delta.clear();
    }
}
