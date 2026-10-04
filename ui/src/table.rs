use crate::{
    editing::{self, EditMode, Editing},
    read_along::{self, Strip},
    view,
};
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
        let max_index = episode
            .turns
            .iter()
            .map(|turn| turn.index)
            .max()
            .unwrap_or(0);
        let max_duration = episode
            .turns
            .iter()
            .map(|turn| turn.end_ms - turn.start_ms)
            .max()
            .unwrap_or(0);
        let verification = ["✓  Passed", "×  Failed", "Not verified", "Missing audio"]
            .into_iter()
            .map(|label| t::text_width(ui, label, t::font(t::SECONDARY)))
            .fold(header("Verification"), f32::max);
        Self {
            // The playing row prefixes the number with "▶" (design screen 03).
            turn: t::fit_text_width(
                header("Turn")
                    .max(mono(format!("{max_index:03}")))
                    .max(mono(format!("▶{max_index:03}"))),
                t::TABLE_COL_TURN,
            ),
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
#[expect(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    episode: &Episode,
    visible: &[usize],
    selected: Option<usize>,
    playing: Option<usize>,
    read_position: Option<u64>,
    scroll_to: Option<usize>,
    failed_only: bool,
    multi: &std::collections::BTreeSet<usize>,
    edited: &[usize],
    mismatch: bool,
    editing: &mut Editing,
    edit_mode: EditMode,
    enabled: bool,
) -> (Option<(usize, bool)>, Option<editing::Action>) {
    let mut area = ui.max_rect();
    if mismatch {
        let banner = Rect::from_min_size(area.min, vec2(area.width(), t::EDITOR_BANNER_HEIGHT));
        let mut banner_ui = view::region(
            ui,
            "script_count_banner",
            banner.shrink(t::SPACE_8),
            t::BG_RAISED,
        );
        banner_ui.label(t::text(
            "Turns were added or removed since render. Re-render to sync.",
            t::SECONDARY,
            t::accent(),
        ));
        area.min.y = banner.bottom();
    }

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
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .auto_shrink([false, false]);
    let editor_row = if edit_mode == EditMode::Row {
        editing
            .draft
            .as_ref()
            .and_then(|d| visible.iter().position(|i| *i == d.turn))
    } else {
        None
    };
    let field_height = editing.draft.as_ref().map_or(t::EDITOR_MIN_HEIGHT, |d| {
        editing::field_height(
            &body_ui,
            &d.text,
            body.width() - t::SPACE_16 * 2.0,
            (body.height() - t::EDITOR_ROW_CHROME).min(t::EDITOR_MAX_HEIGHT),
        )
    });
    let editor_height = field_height + t::EDITOR_ROW_CHROME;
    let read_row = read_position.and_then(|_| {
        playing.and_then(|idx| {
            episode.read_along.words(&episode.turns[idx].id)?;
            visible.iter().position(|row| *row == idx)
        })
    });
    let strip = read_row.map(|row| {
        Strip::measure(
            &body_ui,
            episode
                .read_along
                .words(&episode.turns[visible[row]].id)
                .unwrap(),
            scriptcast_ui::word_timings::manifest_position(
                &episode.manifest.turns[visible[row]],
                read_position.unwrap(),
            ),
            body.width() - body_ui.spacing().scroll.allocated_width(),
        )
    });
    let read_height = strip.as_ref().map_or(0.0, |strip| strip.height);
    let row_top = |row: usize| {
        read_along::row_top(
            row,
            editor_row.map(|row| (row, editor_height)),
            read_row.map(|row| (row, read_height)),
        )
    };
    if let Some(position) = scroll_to.and_then(|idx| visible.iter().position(|v| *v == idx)) {
        scroll = scroll.vertical_scroll_offset((row_top(position) - t::TABLE_ROW_HEIGHT).max(0.0));
    }
    let mut clicked = None;
    let mut edit_action = None;
    let mut displayed = 0..0;
    let result = scroll.show_viewport(&mut body_ui, |ui, viewport| {
        let total = row_top(visible.len());
        ui.set_height(total);
        let origin = ui.max_rect().min;
        let width = ui.available_width();
        // Each turn and the expanded editor are distinct variable-height items.
        // Only intersecting items create widgets; the editor scrolls with its row.
        for (row, &idx) in visible.iter().enumerate() {
            let top = row_top(row);
            let bottom = top + t::TABLE_ROW_HEIGHT;
            if bottom >= viewport.top() && top <= viewport.bottom() {
                if displayed.is_empty() {
                    displayed.start = row;
                }
                displayed.end = row + 1;
                let row_rect =
                    Rect::from_min_size(origin + vec2(0.0, top), vec2(width, t::TABLE_ROW_HEIGHT));
                let mut row_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt(("row", idx))
                        .max_rect(row_rect),
                );
                let ui = &mut row_ui;
                let idx = visible[row];
                let turn = &episode.turns[idx];
                let speaker = &episode.speakers[turn.speaker_idx];
                let (rect, response) = ui.allocate_exact_size(
                    vec2(ui.available_width(), t::TABLE_ROW_HEIGHT),
                    Sense::click(),
                );
                let active = selected == Some(idx) || multi.contains(&idx);
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
                let playing_row = playing == Some(idx);
                view::line(
                    ui,
                    cells[0],
                    &if playing_row {
                        format!("▶{:03}", turn.index)
                    } else {
                        format!("{:03}", turn.index)
                    },
                    t::mono(t::NUMERIC),
                    if active || playing_row {
                        t::accent()
                    } else {
                        t::TEXT_MUTED
                    },
                );
                view::line(
                    ui,
                    cells[1],
                    &view::time(turn.start_ms),
                    t::mono(t::NUMERIC),
                    t::TEXT_MUTED,
                );
                let colour = t::speaker_colour(speaker.colour_idx);
                view::line(
                    ui,
                    cells[2],
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
                let (label, colour) = if edited.contains(&idx) {
                    ("Edited", t::accent())
                } else {
                    view::verification_label(turn.verification)
                };
                view::line(ui, cells[5], label, t::font(t::SECONDARY), colour);
                let response = response.on_hover_text(format!(
                    "Turn {:03} · {}\n{}\n{}",
                    turn.index, speaker.name, turn.first_words, label
                ));
                if response.clicked() || response.double_clicked() {
                    clicked = Some((idx, response.double_clicked()));
                }
            }
            if read_row == Some(row)
                && bottom + read_height >= viewport.top()
                && bottom <= viewport.bottom()
            {
                let rect =
                    Rect::from_min_size(origin + vec2(0.0, bottom), vec2(width, read_height));
                strip.as_ref().unwrap().paint(ui, rect);
                view::rule(ui, rect);
            }
            let bottom = bottom
                + if read_row == Some(row) {
                    read_height
                } else {
                    0.0
                };
            if editor_row == Some(row)
                && bottom + editor_height >= viewport.top()
                && bottom <= viewport.bottom()
            {
                let rect =
                    Rect::from_min_size(origin + vec2(0.0, bottom), vec2(width, editor_height));
                ui.painter().rect_filled(rect, 0, t::BG_SURFACE);
                let mut editor_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("row_editor")
                        .max_rect(rect.shrink(t::SPACE_16))
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                editor_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                let turn = &episode.turns[idx];
                editor_ui.horizontal_wrapped(|ui| {
                    ui.label(t::heading("Inline transcript edit", t::SECONDARY));
                    ui.label(
                        t::text("Draft", t::CAPTION, t::accent()).background_color(t::BG_CONTROL),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(t::text(
                            format!(
                                "Turn {:03} · {}",
                                turn.index, episode.speakers[turn.speaker_idx].name
                            ),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    });
                });
                edit_action = editing::editor(&mut editor_ui, editing, field_height, true, enabled);
                view::rule(ui, rect);
            }
        }
    });

    if visible.is_empty() {
        view::line(
            ui,
            body.shrink(t::SPACE_16),
            "No turns match these filters.",
            t::font(t::BODY),
            t::TEXT_MUTED,
        );
    }
    let _ = result;
    let range = displayed;
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
    (clicked, edit_action)
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
            // Font-agnostic: every header must fit its column, whatever the face.
            let header = |title: &str| t::text_width(ui, title, t::bold(t::SECONDARY));
            assert!(widths.turn >= header("Turn"));
            assert!(widths.time >= header("Time"));
            assert!(widths.speaker >= header("Speaker"));
            assert!(widths.duration >= header("Duration"));
            assert!(widths.verification >= header("Verification"));
            assert!(widths.duration <= t::TEXT_WIDTH_CAP);
            assert!(widths.speaker <= t::TEXT_WIDTH_CAP);
        });
        output.textures_delta.clear();
    }
}
