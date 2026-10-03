use crate::view;
use eframe::egui::{self, Rect, Sense, Ui, pos2, vec2};
use scriptcast_ui::{
    episode::{Episode, VerificationState as V},
    theme as t,
};

/// Fixed column widths sized from measured text, with the design tokens as
/// minimums (OpenDyslexic is much wider than the design's Inter).
#[derive(Clone, Copy, Debug)]
pub struct ColumnWidths {
    turn: f32,
    time: f32,
    speaker: f32,
    duration: f32,
    verification: f32,
}
impl ColumnWidths {
    pub fn measure(ui: &Ui, episode: &Episode) -> Self {
        let header = |title: &str| t::text_width(ui, title, t::bold(t::SECONDARY));
        let mono = |value: String| t::text_width(ui, value, t::mono(t::NUMERIC));
        let max_index = episode.turns.iter().map(|turn| turn.index).max().unwrap_or(0);
        let max_duration = episode
            .turns
            .iter()
            .map(|turn| turn.end_ms - turn.start_ms)
            .max()
            .unwrap_or(0);
        let verification = [
            "✓  Passed",
            "×  Failed",
            "Not verified",
            "Missing audio",
        ]
        .into_iter()
        .map(|label| t::text_width(ui, label, t::font(t::SECONDARY)))
        .fold(header("Verification"), f32::max);
        Self {
            turn: t::fit_text_width(header("Turn").max(mono(format!("{max_index:03}"))), t::TABLE_COL_TURN),
            time: t::fit_text_width(
                header("Time").max(mono(view::time(episode.total_ms))),
                t::TABLE_COL_TIME,
            ),
            speaker: t::fit_text_width(
                view::speaker_label_width(ui, &episode.speakers),
                t::TABLE_COL_SPEAKER,
            ),
            duration: t::fit_text_width(
                header("Duration").max(mono(format!("{:.2} s", max_duration as f64 / 1000.0))),
                t::TABLE_COL_DURATION,
            ),
            verification: t::fit_text_width(verification, t::TABLE_COL_VERIFICATION),
        }
    }
}
fn columns(rect: Rect, widths: &ColumnWidths) -> [Rect; 6] {
    let widths = [
        widths.turn,
        widths.time,
        widths.speaker,
        (rect.width()
            - t::SPACE_16 * 2.0
            - t::TABLE_COL_GAP * 5.0
            - widths.turn
            - widths.time
            - widths.speaker
            - widths.duration
            - widths.verification)
            .max(0.0),
        widths.duration,
        widths.verification,
    ];
    let mut x = rect.left() + t::SPACE_16;
    widths.map(|width| {
        let r = Rect::from_min_size(pos2(x, rect.top()), vec2(width, rect.height()));
        x += width + t::TABLE_COL_GAP;
        r
    })
}
pub fn show(
    ui: &mut Ui,
    episode: &Episode,
    visible: &[usize],
    selected: Option<usize>,
    scroll_to: Option<usize>,
    failed_only: bool,
) -> Option<usize> {
    let area = ui.max_rect();
    let widths = ColumnWidths::measure(ui, episode);
    let head = Rect::from_min_size(area.min, vec2(area.width(), t::TABLE_HEADER_HEIGHT));
    ui.painter().rect_filled(head, 0, t::BG_WINDOW);
    for (rect, title) in columns(head, &widths).into_iter().zip([
        "Turn",
        "Time",
        "Speaker",
        "First words",
        "Duration",
        "Verification",
    ]) {
        view::line(ui, rect, title, t::bold(t::SECONDARY), t::TEXT_MUTED);
    }
    let footer = Rect::from_min_max(
        pos2(area.left(), area.bottom() - t::TABLE_HEADER_HEIGHT),
        area.right_bottom(),
    );
    let body = Rect::from_min_max(head.left_bottom(), footer.right_top());
    let mut body_ui = view::region(ui, "turn_table", body, t::BG_SURFACE);
    body_ui.spacing_mut().item_spacing.y = 0.0;
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt("turn_rows")
        .auto_shrink([false, false]);
    if let Some(position) = scroll_to.and_then(|idx| visible.iter().position(|v| *v == idx)) {
        scroll = scroll.vertical_scroll_offset(
            (position as f32 * t::TABLE_ROW_HEIGHT - body.height() / 2.0).max(0.0),
        );
    }
    let mut clicked = None;
    let result = scroll.show_rows(
        &mut body_ui,
        t::TABLE_ROW_HEIGHT,
        visible.len(),
        |ui, range| {
            let displayed = range.clone();
            for row in range {
                let idx = visible[row];
                let turn = &episode.turns[idx];
                let speaker = &episode.speakers[turn.speaker_idx];
                let (rect, response) = ui.allocate_exact_size(
                    vec2(ui.available_width(), t::TABLE_ROW_HEIGHT),
                    Sense::click(),
                );
                let active = selected == Some(idx);
                ui.painter().rect_filled(
                    rect,
                    0,
                    if active {
                        t::BG_SELECTED
                    } else if turn.verification == V::Failed {
                        t::BG_DANGER_ROW
                    } else {
                        t::BG_SURFACE
                    },
                );
                view::rule(ui, rect);
                let cells = columns(rect, &widths);
                view::line(
                    ui,
                    cells[0],
                    &format!("{:03}", turn.index),
                    t::mono(t::NUMERIC),
                    if active { t::ACCENT } else { t::TEXT_MUTED },
                );
                view::line(
                    ui,
                    cells[1],
                    &view::time(turn.start_ms),
                    t::mono(t::NUMERIC),
                    t::TEXT_MUTED,
                );
                let colour = t::speaker_colour(speaker.colour_idx);
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        pos2(cells[2].left(), cells[2].center().y - t::SPACE_6),
                        vec2(t::SPACE_2, t::SPACE_12),
                    ),
                    0,
                    colour,
                );
                view::line(
                    ui,
                    Rect::from_min_max(cells[2].min + vec2(t::SPACE_6, 0.0), cells[2].max),
                    &speaker.name,
                    t::font(t::SPEAKER_LABEL),
                    colour,
                );
                view::line(
                    ui,
                    cells[3],
                    &turn.first_words,
                    t::font(t::BODY),
                    t::TEXT_PRIMARY,
                );
                view::line(
                    ui,
                    cells[4],
                    &format!("{:.2} s", (turn.end_ms - turn.start_ms) as f64 / 1000.0),
                    t::mono(t::NUMERIC),
                    t::TEXT_MUTED,
                );
                let (label, colour) = match turn.verification {
                    V::Passed => ("✓  Passed", t::STATUS_PASSED),
                    V::Failed => ("×  Failed", t::STATUS_FAILED),
                    V::NotVerified => ("Not verified", t::TEXT_SUBTLE),
                    V::MissingAudio => ("Missing audio", t::STATUS_FAILED),
                };
                view::line(ui, cells[5], label, t::font(t::SECONDARY), colour);
                let response = response.on_hover_text(format!(
                    "Turn {:03} · {}\n{}\n{}",
                    turn.index, speaker.name, turn.first_words, label
                ));
                if response.clicked() {
                    clicked = Some(idx);
                }
            }
            displayed
        },
    );
    if visible.is_empty() {
        view::line(
            ui,
            body.shrink(t::SPACE_16),
            "No turns match these filters.",
            t::font(t::BODY),
            t::TEXT_MUTED,
        );
    }
    let first = (result.state.offset.y / t::TABLE_ROW_HEIGHT).floor() as usize;
    let last = ((result.state.offset.y + result.inner_rect.height()) / t::TABLE_ROW_HEIGHT).ceil()
        as usize;
    let range = first.min(visible.len())..last.min(visible.len());
    let summary = if failed_only {
        format!(
            "{} failures · {} passed · {} missing audio",
            episode.count(V::Failed),
            episode.count(V::Passed),
            episode.count(V::MissingAudio)
        )
    } else {
        format!(
            "Showing {}–{} of {} turns{}",
            if visible.is_empty() {
                0
            } else {
                range.start + 1
            },
            range.end,
            visible.len(),
            if visible.len() != episode.turns.len() {
                format!(" · {} total", episode.turns.len())
            } else {
                String::new()
            }
        )
    };
    let footer = footer.shrink2(vec2(t::SPACE_16, 0.0));
    let mut left = footer;
    left.max.x = footer.center().x;
    view::line(ui, left, &summary, t::font(t::SECONDARY), t::TEXT_SUBTLE);
    let right = Rect::from_min_max(pos2(footer.center().x, footer.top()), footer.max);
    let hint = if failed_only {
        episode
            .stress_mtime
            .and_then(|s| s.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|s| {
                format!(
                    "Last verified {:02}:{:02} UTC",
                    s.as_secs() / 3600 % 24,
                    s.as_secs() / 60 % 60
                )
            })
            .unwrap_or_else(|| "Not verified".into())
    } else {
        "↑ ↓  Select turn".into()
    };
    let galley = ui
        .painter()
        .layout_no_wrap(hint, t::font(t::SECONDARY), t::TEXT_SUBTLE);
    ui.painter().galley(
        pos2(
            (right.right() - galley.size().x).max(right.left()),
            right.center().y - galley.size().y / 2.0,
        ),
        galley,
        t::TEXT_SUBTLE,
    );
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn fixture() -> Episode {
        Episode::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small"),
        )
        .unwrap()
    }
    #[test]
    fn columns_keep_at_least_the_design_token_width() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let episode = fixture();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let widths = ColumnWidths::measure(ui, &episode);
            assert!(widths.turn >= t::TABLE_COL_TURN);
            assert!(widths.time >= t::TABLE_COL_TIME);
            assert!(widths.speaker >= t::TABLE_COL_SPEAKER);
            assert!(widths.duration >= t::TABLE_COL_DURATION);
            assert!(widths.verification >= t::TABLE_COL_VERIFICATION);
        });
        output.textures_delta.clear();
    }
    #[test]
    fn wide_content_grows_columns_beyond_the_tokens() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let episode = fixture();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let widths = ColumnWidths::measure(ui, &episode);
            // "Duration" in bold OpenDyslexic 11 is wider than the 60 px token.
            assert!(widths.duration > t::TABLE_COL_DURATION);
            assert!(widths.verification > t::TABLE_COL_VERIFICATION);
            assert!(widths.duration <= t::TEXT_WIDTH_CAP);
            assert!(widths.speaker <= t::TEXT_WIDTH_CAP);
        });
        output.textures_delta.clear();
    }
}
