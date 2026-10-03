//! Right-hand inspector panel (design screen 03). Phase B ships one mode,
//! Selected turn (376 px); Phase C adds Verification (436) and Job runner
//! (510) — `InspectorMode::width` is where those land.

use crate::{Transport, view};
use eframe::egui::{self, Align, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use scriptcast_ui::{
    episode::{Episode, Turn},
    theme as t,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorMode {
    SelectedTurn,
}
impl InspectorMode {
    pub fn width(self) -> f32 {
        match self {
            Self::SelectedTurn => t::INSPECTOR_WIDTH_SELECTED_TURN,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::SelectedTurn => "Selected turn",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Previous,
    Next,
}

/// The inspector's text field: TRANSCRIPT only when this turn has a
/// `stress.json` entry with real expected text; anything else is FIRST WORDS
/// from `TURN_INDEX.csv` and is never labelled a transcript.
pub fn transcript_field(turn: &Turn) -> (&'static str, &str) {
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
) -> Option<Action> {
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
    egui::ScrollArea::vertical()
        .id_salt("inspector_scroll")
        .auto_shrink([false, false])
        .show(&mut body_ui, |ui| {
            ui.add_space(t::SPACE_16);
            ui.spacing_mut().item_spacing.y = t::SPACE_12;
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
                    let (label, colour) = view::verification_label(turn.verification);
                    ui.label(t::text(label, t::SECONDARY, colour));
                });
            });
            let (label, value) = transcript_field(turn);
            field(ui, label, value, false);
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
                t::ACCENT,
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
            ui.spacing_mut().item_spacing.y = t::SPACE_2;
            ui.label(t::text("Space · Play / pause", t::SECONDARY, t::TEXT_SUBTLE));
            ui.label(t::text(
                "↑ ↓ · Select previous / next turn",
                t::SECONDARY,
                t::TEXT_SUBTLE,
            ));
            ui.label(t::text("Esc · Stop playback", t::SECONDARY, t::TEXT_SUBTLE));
            ui.add_space(t::SPACE_16);
        });
    action
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
    position: Duration,
    auto_advance: bool,
) -> String {
    match transport {
        Transport::Playing(_) => format!(
            "Listening to turn {turn_index:03}. Select any turn to switch audio. Auto-advance is {}.",
            if auto_advance { "on" } else { "off" }
        ),
        Transport::Paused(_) => format!(
            "Paused turn {turn_index:03} at {}. Space resumes; Esc stops.",
            view::time(position.as_millis() as u64)
        ),
        Transport::Idle => format!(
            "Turn {turn_index:03} selected. Space plays it; ↑ ↓ move the selection."
        ),
    }
}
