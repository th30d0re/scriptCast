//! Find/replace controls and their UI lifecycle, kept separate from the browser shell.
use crate::{Browser, editing, inspector::InspectorMode, view};
use eframe::egui::{self, Ui};
use scriptcast_ui::{
    find::{self, Matcher, Mode, Occurrence, Plan},
    theme as t,
};

#[derive(Default)]
pub struct FindReplace {
    pub matches: Vec<Occurrence>,
    pub current: Option<usize>,
    pub invalid: bool,
    pub replace_open: bool,
    pub replacement: String,
    pub confirmation: Option<(Plan, String)>,
    pub saved_counts: Option<(usize, usize)>,
    pub capture_query: Option<String>,
}

pub fn field_id() -> egui::Id {
    egui::Id::new("turn_search")
}

impl Browser {
    pub(crate) fn find_visibility(&self) -> Vec<bool> {
        let Some(episode) = &self.episode else {
            return vec![];
        };
        let options = &self.preferences.find;
        let matcher = Matcher::new(&self.filters.search, options);
        episode
            .turns
            .iter()
            .enumerate()
            .map(|(row, turn)| {
                let body = self.editing.body(row).unwrap_or(&turn.first_words);
                self.filters.speaker.is_none_or(|s| s == turn.speaker_idx)
                    && (!self.filters.failed_only || turn.verification == crate::V::Failed)
                    && (options.mode == Mode::Words
                        || ((options.include_clips
                            || scriptcast_ui::script::clip_id(body).is_none())
                            && matcher.as_ref().is_ok_and(|m| {
                                m.matches_turn(
                                    &scriptcast_ui::script::display_body(body),
                                    &episode.speakers[turn.speaker_idx].name,
                                    turn.index,
                                )
                            })))
            })
            .collect()
    }

    pub(crate) fn refresh_find(&mut self) {
        if self.pending.is_none()
            && let Some(query) = self.find.capture_query.take()
        {
            self.filters.search = query;
        }
        let options = &self.preferences.find;
        let matcher = Matcher::new(&self.filters.search, options);
        self.find.invalid = matcher.is_err();
        let mut matches = Vec::new();
        if let (Ok(matcher), Some(episode)) = (matcher, &self.episode)
            && !self.filters.search.is_empty()
        {
            let visible = self.find_visibility();
            for (row, turn) in episode.turns.iter().enumerate() {
                if !visible[row] {
                    continue;
                }
                let body = self.editing.body(row).unwrap_or(&turn.first_words);
                if !options.include_clips && scriptcast_ui::script::clip_id(body).is_some() {
                    continue;
                }
                let mut found = find::occurrences(row, body, &matcher, options);
                if options.mode == Mode::Turns {
                    found.truncate(1);
                    if found.is_empty() {
                        found.push(Occurrence {
                            row,
                            displayed: 0..0,
                            body: None,
                        });
                    }
                }
                matches.extend(found);
            }
        }
        if matches != self.find.matches {
            let old = self.find.current.and_then(|i| self.find.matches.get(i));
            let current = old
                .and_then(|m| matches.iter().position(|n| m == n))
                .or_else(|| (!matches.is_empty()).then_some(0));
            self.find.matches = matches;
            self.find.current = current;
            self.select_find_match();
        }
    }

    fn select_find_match(&mut self) {
        if let Some(row) = self
            .find
            .current
            .and_then(|i| self.find.matches.get(i))
            .map(|m| m.row)
        {
            self.select(row);
            if self.selected == Some(row) {
                self.scroll_to = Some(row);
            }
        }
    }

    pub(crate) fn step_find(&mut self, previous: bool) {
        self.refresh_find();
        self.find.current = find::navigate(self.find.current, self.find.matches.len(), previous);
        self.select_find_match();
    }

    fn replace_plan(&self, current: bool) -> Plan {
        let Some(turns) = &self.editing.turns else {
            return Plan::default();
        };
        let Ok(matcher) = Matcher::new(&self.filters.search, &self.preferences.find) else {
            return Plan::default();
        };
        let selected = if current {
            let Some(found) = self.find.current.and_then(|i| self.find.matches.get(i)) else {
                return Plan::default();
            };
            Some(found)
        } else {
            None
        };
        let visible = self.find_visibility();
        let turns: Vec<_> = turns
            .turns
            .iter()
            .filter(|turn| visible.get(turn.index).copied().unwrap_or(false))
            .cloned()
            .collect();
        find::plan(
            &turns,
            &matcher,
            &self.preferences.find,
            &self.find.replacement,
            selected,
        )
    }

    pub(crate) fn filter_bar(&mut self, ui: &mut Ui) {
        if self.episode.is_none() {
            return;
        }
        self.refresh_find();
        let rect = view::inset(ui.max_rect(), t::SPACE_16, t::SPACE_4);
        let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        ui.spacing_mut().item_spacing = egui::vec2(t::SPACE_8, t::SPACE_4);
        let mut direction = None;
        ui.horizontal(|ui| {
            t::find_shortcut(ui, false);
            // The field grows so prompt and hint always fit; the token is the floor.
            let search_width = t::FIND_FIELD_WIDTH;
            // A child ui, not `ui.put`: `put` advances the parent cursor past the
            // edit rect, which pushed the following ComboBox into the search field.
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.filters.search)
                    .id(field_id())
                    .font(t::font(t::BODY))
                    .hint_text("Find words")
                    .desired_width(search_width),
            );
            if self.focus_search {
                response.request_focus();
                self.focus_search = false;
            }
            if response.changed() {
                self.refresh_find();
            }
            let count = if self.find.invalid {
                "Invalid pattern".into()
            } else if self.find.matches.is_empty() {
                "No matches".into()
            } else {
                format!(
                    "{} of {}",
                    self.find.current.unwrap_or(0) + 1,
                    self.find.matches.len()
                )
            };
            ui.label(t::text(
                count,
                t::SECONDARY,
                if self.find.invalid {
                    t::STATUS_FAILED
                } else {
                    t::TEXT_MUTED
                },
            ));
            for (label, previous) in [("Previous", true), ("Next", false)] {
                if t::secondary_button(ui, label, None, !self.find.matches.is_empty()).clicked() {
                    direction = Some(previous);
                }
            }
            let e = self.episode.as_ref().unwrap();
            egui::ComboBox::from_id_salt("speaker_filter")
                .selected_text(
                    self.filters
                        .speaker
                        .map(|s| e.speakers[s].name.as_str())
                        .unwrap_or("All speakers"),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filters.speaker, None, "All speakers");
                    for (i, speaker) in e.speakers.iter().enumerate() {
                        ui.selectable_value(&mut self.filters.speaker, Some(i), &speaker.name);
                    }
                });
            let failures = e.count(crate::V::Failed);
            ui.toggle_value(
                &mut self.filters.failed_only,
                format!("Failed only · {failures}"),
            );
            editing::mode_toggle(ui, &mut self.preferences.edit_mode);
            let stale = self.editing.markers(e.turns.len()).unwrap_or_default();
            if !stale.is_empty() {
                let label = format!("Regenerate {} edited turns", stale.len());
                if t::secondary_button(ui, &label, None, !self.jobs.busy() && !self.editing.busy())
                    .clicked()
                {
                    self.request_regenerate_edited(stale, ui.ctx());
                }
            }
            ui.label(t::text("Episode text", t::CAPTION, t::TEXT_SUBTLE));
            ui.menu_button("Options ▸", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.selectable_value(&mut self.preferences.find.mode, Mode::Words, "Words");
                    ui.selectable_value(&mut self.preferences.find.mode, Mode::Turns, "Turns");
                });
                ui.checkbox(&mut self.preferences.find.match_case, "Match case");
                ui.checkbox(&mut self.preferences.find.whole_word, "Whole word");
                ui.checkbox(
                    &mut self.preferences.find.regular_expression,
                    "Regular expression",
                );
                ui.checkbox(
                    &mut self.preferences.find.include_clips,
                    "Include audio clips",
                );
            });
            if t::find_shortcut(ui, true).clicked() {
                self.find.replace_open = !self.find.replace_open;
            }
            ui.menu_button("View ▸", |ui| {
                let tip = if self.preferences.inspector_hidden {
                    "Show side panel"
                } else {
                    "Hide side panel"
                };
                if ui.button(tip).clicked() {
                    self.preferences.inspector_hidden = !self.preferences.inspector_hidden;
                    ui.close();
                }
                for (label, mode) in [
                    ("Jobs", InspectorMode::Jobs),
                    ("Verification", InspectorMode::Verification),
                    ("Selected", InspectorMode::SelectedTurn),
                ] {
                    if t::secondary_button(
                        ui,
                        label,
                        None,
                        self.selected.is_some() || label == "Jobs",
                    )
                    .clicked()
                    {
                        self.preferences.inspector_hidden = false;
                        self.jobs.mode = Some(mode);
                        ui.close();
                    }
                }
            });
        });
        self.refresh_find();
        if let Some(previous) = direction {
            self.step_find(previous);
        }
        if self.find.replace_open {
            ui.horizontal(|ui| {
                ui.label(t::text("Replace", t::SECONDARY, t::TEXT_MUTED));
                ui.add(
                    egui::TextEdit::singleline(&mut self.find.replacement)
                        .desired_width(t::FIND_FIELD_WIDTH)
                        .font(t::font(t::BODY)),
                );
                let ready = !self.find.invalid
                    && !self.editing.loading()
                    && !self.editing.busy()
                    && !self.jobs.busy()
                    && self.pending.is_none();
                let current = self.replace_plan(true);
                let all = self.replace_plan(false);
                if t::secondary_button(ui, "Replace", None, ready && current.occurrences > 0)
                    .clicked()
                {
                    self.save_replacements(current, ui.ctx());
                }
                if t::secondary_button(ui, "Replace all", None, ready && all.occurrences > 0)
                    .clicked()
                {
                    if self.editing.draft.is_some() {
                        self.editing.notice = Some("Save or cancel the open edit first".into());
                    } else if let Some(hash) = self.editing.loaded_sha256.clone() {
                        self.find.confirmation = Some((all.clone(), hash));
                    }
                }
                ui.label(t::text(all.preview(), t::SECONDARY, t::TEXT_MUTED));
            });
            if self.editing.draft.is_some() {
                ui.label(t::text(
                    "Save or cancel the open edit first",
                    t::CAPTION,
                    t::STATUS_FAILED,
                ));
            }
        }
    }

    pub(crate) fn replace_dialog(&mut self, ctx: &egui::Context) {
        let Some((plan, hash)) = self.find.confirmation.clone() else {
            return;
        };
        let name = self
            .editing
            .turns
            .as_ref()
            .and_then(|s| s.script_path.file_name())
            .unwrap_or_default()
            .to_string_lossy();
        let mut confirm = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("replace_confirmation"))
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
                ui.spacing_mut().item_spacing.y = t::SPACE_12;
                ui.label(t::heading(
                    format!(
                        "Replace {} occurrences in {} turns of {name}?",
                        plan.occurrences,
                        plan.edits.len()
                    ),
                    t::DIALOG_TITLE,
                ));
                ui.label("A checkpoint of the script is saved first; rendered audio is unchanged.");
                egui::ScrollArea::vertical()
                    .max_height(t::EDITOR_MAX_HEIGHT)
                    .show(ui, |ui| {
                        for (turn, after) in plan.edits.iter().take(10) {
                            ui.label(format!(
                                "Turn {:03}: {} → {}",
                                turn.index,
                                snippet(&turn.body, after).0,
                                snippet(&turn.body, after).1
                            ));
                        }
                        if plan.edits.len() > 10 {
                            ui.label(format!("+{} more", plan.edits.len() - 10));
                        }
                    });
                ui.horizontal(|ui| {
                    confirm = t::primary_button(ui, "Replace all", None, true).clicked();
                    cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
                });
            });
        if cancel || response.should_close() {
            self.find.confirmation = None;
        }
        if confirm {
            self.find.confirmation = None;
            if self.editing.loaded_sha256.as_deref() != Some(&hash) {
                self.editing.notice = Some("The script changed. Review replacements again.".into());
            } else {
                self.save_replacements(plan, ctx);
            }
        }
    }
}

fn snippet(before: &str, after: &str) -> (String, String) {
    let prefix = before
        .chars()
        .zip(after.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let start = prefix.saturating_sub(30);
    let compact = |text: &str| {
        let piece: String = text.chars().skip(start).take(100).collect();
        format!(
            "{}{}{}",
            if start > 0 { "…" } else { "" },
            piece.replace(['\r', '\n'], " "),
            if text.chars().count() > start + 100 {
                "…"
            } else {
                ""
            }
        )
    };
    (compact(before), compact(after))
}

impl Browser {
    pub(crate) fn find_shortcuts(&mut self, ctx: &egui::Context) {
        let focused = ctx.memory(|m| m.has_focus(field_id()));
        let (replace, find, previous, next, escape) = ctx.input_mut(|i| {
            let replace = i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::ALT,
                egui::Key::F,
            );
            let find = i.consume_key(egui::Modifiers::COMMAND, egui::Key::F);
            let previous = i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::G,
            ) || (focused
                && i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter));
            let next = i.consume_key(egui::Modifiers::COMMAND, egui::Key::G)
                || (focused && i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let escape = focused && i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            (replace, find, previous, next, escape)
        });
        if replace {
            self.find.replace_open = true;
        }
        if find || replace {
            self.focus_search = true;
        }
        if previous || next {
            self.step_find(previous);
        }
        if escape {
            if self.filters.search.is_empty() {
                ctx.memory_mut(|m| m.surrender_focus(field_id()));
            } else {
                self.filters.search.clear();
                self.refresh_find();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcast_ui::{
        episode::Episode,
        script::{ScriptTurn, ScriptTurns},
    };
    use std::path::PathBuf;

    fn app() -> Browser {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small");
        let mut app = Browser {
            episode: Some(Episode::load(&path).unwrap()),
            ..Default::default()
        };
        app.editing.manifest_len = Some(3);
        app.editing.turns = Some(ScriptTurns {
            script_path: path.join("script.md"),
            script_sha256: "test-hash".into(),
            line_ending: "lf".into(),
            turns: [
                "A full transcript with café café",
                "Nothing",
                "[clip:café] café",
            ]
            .iter()
            .enumerate()
            .map(|(index, body)| ScriptTurn {
                index,
                turn_id: format!("{index}"),
                speaker_id: "s".into(),
                display_name: "S".into(),
                header_line: 1,
                body_start: 2,
                body_end: 2,
                body: (*body).into(),
            })
            .collect(),
        });
        app
    }

    #[test]
    fn words_search_full_bodies_keeps_rows_and_turns_mode_filters() {
        let mut app = app();
        app.filters.search = "café".into();
        app.refresh_find();
        assert_eq!(app.find.matches.len(), 3);
        assert_eq!(app.visible_indices(), vec![0, 1, 2]);
        assert_eq!(app.selected, Some(0));
        app.step_find(true);
        assert_eq!(app.selected, Some(2));
        assert_eq!(app.scroll_to, Some(2));
        app.step_find(false);
        assert_eq!(app.find.current, Some(0));
        app.preferences.find.mode = Mode::Turns;
        app.refresh_find();
        assert_eq!(app.visible_indices(), vec![0, 2]);
        assert_eq!(app.find.matches.len(), 2);
        app.step_find(true);
        assert_eq!(app.selected, Some(2));
        app.preferences.find.include_clips = false;
        app.refresh_find();
        assert_eq!(app.visible_indices(), vec![0]);
    }

    #[test]
    fn turns_mode_preserves_legacy_filters_with_fallback_text() {
        let mut app = app();
        app.editing.turns = None;
        app.preferences.find.mode = Mode::Turns;
        for query in ["", "WORLD", "alice", "000", "002", "nothing"] {
            for speaker in [None, Some(0), Some(1)] {
                for failed_only in [false, true] {
                    app.filters.search = query.into();
                    app.filters.speaker = speaker;
                    app.filters.failed_only = failed_only;
                    assert_eq!(
                        app.find_visibility(),
                        app.filters.matches(app.episode.as_ref().unwrap())
                    );
                }
            }
        }
    }

    #[test]
    fn replace_preview_scopes_turns_and_any_draft_blocks_save() {
        let mut app = app();
        app.filters.search = "café".into();
        app.find.replacement = "tea".into();
        app.refresh_find();
        let all = app.replace_plan(false);
        assert_eq!((all.occurrences, all.edits.len(), all.skipped), (3, 2, 1));
        assert_eq!(app.replace_plan(true).occurrences, 1);
        app.editing.open(0);
        app.save_replacements(all, &egui::Context::default());
        assert_eq!(
            app.editing.notice.as_deref(),
            Some("Save or cancel the open edit first")
        );
        assert!(!app.editing.busy());
    }

    fn key(
        app: &mut Browser,
        ctx: &egui::Context,
        key: egui::Key,
        modifiers: egui::Modifiers,
        id: egui::Id,
    ) {
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            },
            |_| {
                ctx.memory_mut(|m| m.request_focus(id));
                app.shortcuts(ctx);
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn find_keyboard_navigation_escape_and_inline_suppression() {
        let mut app = app();
        let ctx = egui::Context::default();
        t::apply(&ctx);
        app.filters.search = "café".into();
        app.refresh_find();
        key(
            &mut app,
            &ctx,
            egui::Key::Enter,
            egui::Modifiers::NONE,
            field_id(),
        );
        assert_eq!(app.find.current, Some(1));
        key(
            &mut app,
            &ctx,
            egui::Key::Enter,
            egui::Modifiers::SHIFT,
            field_id(),
        );
        assert_eq!(app.find.current, Some(0));
        key(
            &mut app,
            &ctx,
            egui::Key::G,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            field_id(),
        );
        assert_eq!(app.find.current, Some(2));
        key(
            &mut app,
            &ctx,
            egui::Key::F,
            egui::Modifiers::COMMAND | egui::Modifiers::ALT,
            field_id(),
        );
        assert!(app.find.replace_open);
        key(
            &mut app,
            &ctx,
            egui::Key::Escape,
            egui::Modifiers::NONE,
            field_id(),
        );
        assert!(app.filters.search.is_empty());
        assert!(ctx.memory(|m| m.has_focus(field_id())));
        key(
            &mut app,
            &ctx,
            egui::Key::Escape,
            egui::Modifiers::NONE,
            field_id(),
        );
        assert!(!ctx.memory(|m| m.has_focus(field_id())));
        app.preferences.edit_mode = editing::EditMode::Row;
        app.editing.open(0);
        app.find.replace_open = false;
        app.focus_search = false;
        key(
            &mut app,
            &ctx,
            egui::Key::F,
            egui::Modifiers::COMMAND | egui::Modifiers::ALT,
            editing::editor_id(),
        );
        assert!(!app.find.replace_open && !app.focus_search);
        key(
            &mut app,
            &ctx,
            egui::Key::Escape,
            egui::Modifiers::NONE,
            editing::editor_id(),
        );
        assert!(app.editing.draft.is_none());
    }
}
