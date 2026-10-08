//! Right-hand inspector panel (design screen 03). Phase B ships one mode,
//! Selected turn (376 px); Phase C adds Verification (436) and Job runner
//! (510) — `InspectorMode::width` is where those land.

use crate::{
    Transport,
    editing::{self, EditMode, Editing},
    read_along, view,
};
use eframe::egui::{self, Align, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use scriptcast_ui::{clip_transcripts::ClipTurn, episode::Episode, script, theme as t};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorMode {
    SelectedTurn,
    Verification,
    Jobs,
}
impl InspectorMode {
    pub fn width(self) -> f32 {
        match self {
            Self::SelectedTurn => t::INSPECTOR_WIDTH_SELECTED_TURN,
            Self::Verification => t::INSPECTOR_WIDTH_VERIFICATION,
            Self::Jobs => t::INSPECTOR_WIDTH_JOB_RUNNER,
        }
    }
    /// Width for the review row: a persisted override, else this mode's default,
    /// clamped between `INSPECTOR_MIN` and `INSPECTOR_MAX_FRACTION` of the row.
    pub fn panel_width(self, custom: Option<f32>, row_width: f32) -> f32 {
        let desired = custom.unwrap_or_else(|| self.width());
        let max = (row_width * t::INSPECTOR_MAX_FRACTION).max(t::INSPECTOR_MIN);
        desired.clamp(t::INSPECTOR_MIN, max).min(row_width)
    }
    fn title(self) -> &'static str {
        match self {
            Self::SelectedTurn => "Selected turn",
            Self::Verification => "Verification",
            Self::Jobs => "Job runner",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Previous,
    Next,
    Review,
    RunReadAlong,
    RunClipTranscripts,
}

/// The inspector's text field: TRANSCRIPT only when this turn has a
/// `stress.json` entry with real expected text; anything else is FIRST WORDS
/// from `TURN_INDEX.csv` and is never labelled a transcript.
#[cfg(test)]
pub fn transcript_field(turn: &scriptcast_ui::episode::Turn) -> (&'static str, &str) {
    let expected = turn
        .verify
        .as_ref()
        .map(|v| v.expected.trim())
        .unwrap_or("");
    if expected.is_empty() {
        ("First words", turn.first_words.trim())
    } else {
        ("Transcript", expected)
    }
}

fn read_along_prompt(episode: &Episode, turn_id: &str) -> Option<&'static str> {
    if episode.read_along.words(turn_id).is_some() {
        return None;
    }
    let stale = episode
        .read_along
        .notice
        .as_deref()
        .is_some_and(|n| n.contains("out of date"));
    Some(if stale {
        "Read-along timings are out of date (episode re-rendered). Run Jobs → Read-along timings."
    } else {
        "No read-along timings yet. Run Jobs → Read-along timings."
    })
}

fn clip_transcript_prompt(episode: &Episode, turn_id: &str) -> Option<&'static str> {
    if episode.clip_transcripts.get(turn_id).is_some() {
        return None;
    }
    let stale = episode
        .clip_transcripts
        .notice
        .as_deref()
        .is_some_and(|n| n.contains("out of date"));
    Some(if stale {
        "Clip transcripts are out of date (episode re-rendered). Run Jobs → Clip transcripts."
    } else {
        "No ensemble transcript yet. Run Jobs → Clip transcripts."
    })
}

#[cfg(test)]
pub fn strip_clip_tag_for_display(body: &str) -> String {
    script::display_body(body)
}

#[expect(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    mode: InspectorMode,
    episode: &Episode,
    turn_idx: usize,
    transport: Transport,
    position: Duration,
    auto_advance: bool,
    can_previous: bool,
    can_next: bool,
    file_field: &str,
    editing: &mut Editing,
    edit_mode: EditMode,
    edited: bool,
    enabled: bool,
    read_along: bool,
) -> (Option<Action>, Option<editing::Action>) {
    let turn = &episode.turns[turn_idx];
    let speaker = &episode.speakers[turn.speaker_idx];
    let rect = ui.max_rect();
    ui.painter().line_segment(
        [rect.left_top(), rect.left_bottom()],
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
    );
    let head = Rect::from_min_size(rect.min, vec2(rect.width(), t::INSPECTOR_HEADING_HEIGHT));
    ui.painter().rect_filled(head, 0, t::BG_RAISED);
    view::rule(ui, head);
    let title_rect = view::inset(head, t::SPACE_16, 0.0);
    view::line(
        ui,
        title_rect,
        mode.title(),
        t::bold(t::SECTION_TITLE),
        t::TEXT_PRIMARY,
    );
    let counter = format!("{:03} / {}", turn.index, episode.turns.len());
    let counter_width = t::text_width(ui, &counter, t::font(t::CAPTION));
    view::line(
        ui,
        Rect::from_min_max(
            pos2(title_rect.right() - counter_width, title_rect.top()),
            title_rect.max,
        ),
        &counter,
        t::font(t::CAPTION),
        t::TEXT_MUTED,
    );
    let body = Rect::from_min_max(head.left_bottom(), rect.max);
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .id_salt("inspector_body")
            .max_rect(view::inset(body, t::SPACE_16, 0.0))
            .layout(Layout::top_down(Align::Min)),
    );
    body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
    let mut action = None;
    let mut edit_action = None;
    let scope = match (&editing.changes, &editing.turns) {
        (Some(changes), Some(turns)) => {
            Some(script::RenderScope::from_changes(changes, &turns.turns))
        }
        _ => None,
    };
    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .auto_shrink([false, false])
        .show(&mut body_ui, |ui| {
            ui.add_space(t::SPACE_16);
            ui.spacing_mut().item_spacing.y = t::SPACE_12;
            if edit_mode == EditMode::Panel && editing.k.new_turn.is_some() {
                crate::phase_k::new_editor(ui, editing, true);
                return;
            }
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        !editing.edit_tab,
                        t::text("Turn view", t::BODY, t::TEXT_PRIMARY),
                    )
                    .clicked()
                {
                    editing.edit_tab = false;
                }
                if ui
                    .selectable_label(editing.edit_tab, t::text("Edit", t::BODY, t::TEXT_PRIMARY))
                    .clicked()
                {
                    edit_action = Some(editing::Action::Open);
                }
            });
            // Speaker + verification row.

            ui.horizontal(|ui| {
                let colour = t::speaker_colour(speaker.colour_idx);
                let (bar, _) =
                    ui.allocate_exact_size(vec2(t::SPACE_2, t::SPACE_12), Sense::hover());
                ui.painter().rect_filled(bar, 0, colour);
                ui.label(
                    egui::RichText::new(&speaker.name)
                        .font(t::font(t::SPEAKER_LABEL))
                        .color(colour),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (label, colour) = if edited {
                        ("Edited", t::accent())
                    } else {
                        view::verification_label(turn.verification)
                    };
                    ui.label(t::text(label, t::SECONDARY, colour));
                });
            });
            if edited && transport.turn() == Some(turn_idx) {
                ui.label(t::text(
                    "Audio is from before this edit",
                    t::SECONDARY,
                    t::accent(),
                ));
            }
            let raw_body = editing.body(turn_idx).map(str::to_owned);
            let is_clip = raw_body
                .as_deref()
                .is_some_and(|b| script::clip_id(b).is_some());
            let clip_turn = is_clip
                .then(|| episode.clip_transcripts.get(&turn.id))
                .flatten();
            let ensemble_text = clip_turn.map(ClipTurn::ensemble_text);
            if editing.edit_tab && edit_mode == EditMode::Panel && editing.draft.is_some() {
                let height = editing::field_height(
                    ui,
                    &editing.draft.as_ref().unwrap().text,
                    ui.available_width(),
                    body.height() - t::EDITOR_CHROME_HEIGHT,
                );
                edit_action = editing::editor(
                    ui,
                    editing,
                    height,
                    false,
                    enabled,
                    ensemble_text.as_deref(),
                )
                .or(edit_action);
            } else if let Some(body_text) = &raw_body {
                // Badge already shows the clip id; strip the raw tag here.
                let display = if is_clip {
                    script::display_body(body_text)
                } else {
                    body_text.clone()
                };
                field(ui, "Transcript", &display, false);
            } else {
                // The legacy verification-text helper remains available to verification views.
                field(ui, "First words", &turn.first_words, false);
                ui.label(t::text(
                    format!("Full text unavailable: {}", editing.unavailable()),
                    t::SECONDARY,
                    t::TEXT_MUTED,
                ));
            }
            if let Some(error) = &editing.changes_error {
                ui.label(t::text(
                    format!("Edit markers unavailable: {error}"),
                    t::CAPTION,
                    t::STATUS_FAILED,
                ));
            }
            if is_clip && !editing.edit_tab {
                if let Some(clip) = clip_turn {
                    let script_text = raw_body
                        .as_deref()
                        .map(script::display_body)
                        .unwrap_or_default();
                    let ensemble = ensemble_text.as_deref().unwrap_or("");
                    let words = clip.as_words();
                    let row_maps = read_along::word_mapping(&script_text, &words).is_some();
                    let play_ms = matches!(transport, Transport::Playing(_)).then(|| {
                        scriptcast_ui::word_timings::manifest_position(
                            &episode.manifest.turns[turn_idx],
                            position.as_millis() as u64,
                        )
                    });
                    let active = play_ms.and_then(|ms| {
                        if row_maps {
                            None
                        } else {
                            read_along::active_range(ensemble, &words, ms)
                        }
                    });
                    ensemble_block(
                        ui,
                        clip,
                        ensemble,
                        active,
                        !row_maps,
                        clip.has_script_words(),
                    );
                } else if read_along && let Some(prompt) = clip_transcript_prompt(episode, &turn.id)
                {
                    ui.label(t::text(prompt, t::SECONDARY, t::TEXT_MUTED));
                    if t::secondary_button(ui, "Run…", None, enabled).clicked() {
                        action = Some(Action::RunClipTranscripts);
                    }
                } else if let Some(notice) = &episode.clip_transcripts.notice {
                    ui.label(t::text(notice, t::SECONDARY, t::TEXT_MUTED));
                }
            } else if !is_clip
                && read_along
                && !editing.edit_tab
                && let Some(prompt) = read_along_prompt(episode, &turn.id)
            {
                ui.label(t::text(prompt, t::SECONDARY, t::TEXT_MUTED));
                if t::secondary_button(ui, "Run…", None, enabled).clicked() {
                    action = Some(Action::RunReadAlong);
                }
            } else if !is_clip
                && let Some(notice) = &episode.read_along.notice
                && !read_along
            {
                ui.label(t::text(notice, t::SECONDARY, t::TEXT_MUTED));
            }
            if let Some(notice) = &editing.notice {
                ui.label(t::text(notice, t::SECONDARY, t::TEXT_MUTED));
            }
            if let Some(report) = &editing.k.authority {
                for problem in report.problems.iter().filter(|p| p.index == turn_idx) {
                    ui.label(t::text(
                        format!("{} {}", problem.tag, problem.message),
                        t::CAPTION,
                        t::STATUS_FAILED,
                    ));
                }
            }
            if editing.edit_tab && editing.has_edits() {
                let can_render = scope.as_ref().is_some_and(|s| s.synthesis_enabled());
                let response = t::secondary_button(
                    ui,
                    "Re-render edited turns",
                    None,
                    enabled && !editing.busy() && can_render,
                );
                let response = if !can_render {
                    response.on_hover_text(
                        "Only audio-clip transcript corrections changed; nothing to synthesize.",
                    )
                } else {
                    response
                };
                if response.clicked() {
                    edit_action = Some(editing::Action::Render);
                }
            }
            if editing.edit_tab && edit_mode == EditMode::Panel {
                return;
            }

            field(
                ui,
                "Audio",
                &format!(
                    "{} → {} · {:.3} seconds",
                    view::time(turn.start_ms),
                    view::time(turn.end_ms),
                    (turn.end_ms - turn.start_ms) as f64 / 1000.0
                ),
                true,
            );
            field(ui, "File", file_field, true);
            ui.label(t::text(
                status_sentence(turn.index, transport, position, auto_advance),
                t::BODY,
                t::accent(),
            ));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = t::SPACE_8;
                if t::secondary_button(ui, "↑ Previous", None, can_previous).clicked() {
                    action = Some(Action::Previous);
                }
                if t::secondary_button(ui, "↓ Next", None, can_next).clicked() {
                    action = Some(Action::Next);
                }
            });
            if t::secondary_button(ui, "Review verification", None, true).clicked() {
                action = Some(Action::Review);
            }
            ui.spacing_mut().item_spacing.y = t::SPACE_2;
            ui.label(t::text(
                "Space · Play / pause",
                t::SECONDARY,
                t::TEXT_SUBTLE,
            ));
            ui.label(t::text(
                "↑ ↓ · Select previous / next turn",
                t::SECONDARY,
                t::TEXT_SUBTLE,
            ));
            ui.label(t::text("Esc · Stop playback", t::SECONDARY, t::TEXT_SUBTLE));
            ui.add_space(t::SPACE_16);
        });
    (action, edit_action)
}

fn ensemble_block(
    ui: &mut Ui,
    clip: &ClipTurn,
    ensemble: &str,
    active: Option<std::ops::Range<usize>>,
    follow_recording: bool,
    has_script_words: bool,
) {
    // Status lines first so disagreement / follow-recording stay above the fold
    // when the merged text is long.
    ui.spacing_mut().item_spacing.y = t::SPACE_5;
    t::field_label(ui, "Ensemble transcript");
    ui.spacing_mut().item_spacing.y = t::SPACE_8;
    if has_script_words {
        ui.label(t::text(
            "Row highlighting is aligned to your transcript; unmatched words are approximate.",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
    } else if follow_recording {
        ui.label(t::text(
            "Highlighting follows the recording's transcript",
            t::CAPTION,
            t::TEXT_MUTED,
        ));
    }
    if clip.listen_manually {
        ui.label(t::text(
            format!(
                "Listen manually: engines disagree by {:.1}%",
                clip.whisper_disagreement_pct
            ),
            t::SECONDARY,
            t::STATUS_FAILED,
        ));
    } else {
        ui.label(t::text(
            format!("Whisper disagreement {:.1}%", clip.whisper_disagreement_pct),
            t::CAPTION,
            t::TEXT_MUTED,
        ));
    }
    if !clip.all_phonon2() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = t::SPACE_8;
            for engine in clip.engine_labels() {
                ui.label(t::text(engine, t::CAPTION, t::TEXT_MUTED));
            }
        });
    }
    for warning in &clip.coverage_warnings {
        let excerpt = warning.excerpt.trim();
        if !excerpt.is_empty() {
            ui.label(t::text(
                format!("Coverage warning: {excerpt}"),
                t::CAPTION,
                t::STATUS_FAILED,
            ));
        }
    }
    let width = ui.available_width();
    let measured = read_along::Transcript::measure_simple(ui, ensemble, width, active);
    let height = measured.galley.size().y + t::SPACE_4 * 2.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    measured.paint(ui, rect);
    ui.spacing_mut().item_spacing.y = t::SPACE_12;
}

/// One labelled detail field: 10/600 uppercase muted label over the value.
/// The gap after the label is SPACE_5 (egui applies the spacing in effect when
/// the widget is added); between fields it is SPACE_12.
fn field(ui: &mut Ui, label: &str, value: &str, mono: bool) {
    ui.spacing_mut().item_spacing.y = t::SPACE_5;
    t::field_label(ui, label);
    ui.spacing_mut().item_spacing.y = t::SPACE_12;
    let rich = if mono {
        egui::RichText::new(value)
            .font(t::mono(t::BODY))
            .color(t::TEXT_PRIMARY)
    } else {
        t::text(value, t::BODY, t::TEXT_PRIMARY)
    };
    ui.label(rich);
}

fn status_sentence(
    turn_index: usize,
    transport: Transport,
    _position: Duration,
    _auto_advance: bool,
) -> String {
    let local = "Transcript edits are local until saved; rendered audio remains unchanged.";
    match transport {
        Transport::Playing(_) => format!("Listening to turn {turn_index:03}. {local}"),
        Transport::Paused(_) => format!("Paused on turn {turn_index:03}. {local}"),
        Transport::Idle => {
            format!("Turn {turn_index:03} selected. Space plays it; ↑ ↓ move the selection.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_display_strips_clip_tag() {
        assert_eq!(
            strip_clip_tag_for_display("[clip:atwater_1981] Spoken words."),
            "Spoken words."
        );
        assert_eq!(strip_clip_tag_for_display("plain text"), "plain text");
    }
}
