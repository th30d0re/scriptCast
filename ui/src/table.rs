use crate::{
    editing::{self, EditMode, Editing},
    read_along, view,
};
use eframe::egui::{self, Rect, Sense, Ui, pos2, vec2};
use scriptcast_ui::{
    episode::{Episode, Turn, VerificationState as V},
    not_heard, script, theme as t,
};

/// True when a turn's WAV is newer than `stress.json` (audio re-rendered since verify).
fn turn_stress_stale(episode: &Episode, turn: &Turn) -> bool {
    let Some(stress_time) = episode.stress_mtime else {
        return false;
    };
    turn.wav_paths.iter().any(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
            .is_some_and(|wav_time| wav_time > stress_time)
    })
}

/// Display-text byte ranges for words Whisper did not hear on a Failed turn.
fn not_heard_ranges(episode: &Episode, turn: &Turn, display: &str) -> Vec<(usize, usize)> {
    if turn.verification != V::Failed || turn_stress_stale(episode, turn) {
        return Vec::new();
    }
    let Some(verify) = turn.verify.as_ref() else {
        return Vec::new();
    };
    if verify.expected.trim().is_empty() {
        return Vec::new();
    }
    not_heard::not_heard_in_display(&verify.expected, &verify.heard, display)
}

fn not_heard_style(edit_mode: EditMode) -> read_along::NotHeardStyle {
    match edit_mode {
        EditMode::Row => read_along::NotHeardStyle::Inline,
        EditMode::Panel => read_along::NotHeardStyle::Pill,
    }
}

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
        let verification = [
            "✓  Passed",
            "×  Failed",
            "×  Failed maybe misheard",
            "Not verified",
            "Missing audio",
        ]
        .into_iter()
        .map(|label| t::text_width(ui, label, t::font(t::SECONDARY)))
        .fold(header("Verification"), f32::max);
        Self {
            // The playing row prefixes the number with "▶" (design screen 03).
            turn: t::fit_text_width(
                header("Turn")
                    .max(mono(format!("{max_index:03}")))
                    .max(mono(format!("▶ {max_index:03}"))),
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
#[derive(Default)]
pub struct TextCache {
    rows: std::collections::HashMap<usize, CachedText>,
}
struct CachedText {
    text: String,
    width: f32,
    font: egui::FontId,
    pixels_per_point: f32,
    active: Option<std::ops::Range<usize>>,
    not_heard: Vec<(usize, usize)>,
    not_heard_style: Option<read_along::NotHeardStyle>,
    transcript: read_along::Transcript,
}

struct MeasureArgs<'a> {
    turn: usize,
    text: &'a str,
    width: f32,
    active: Option<std::ops::Range<usize>>,
    not_heard: &'a [(usize, usize)],
    not_heard_style: Option<read_along::NotHeardStyle>,
}

impl TextCache {
    fn measure(&mut self, ui: &Ui, args: MeasureArgs<'_>) -> &read_along::Transcript {
        let font = t::font(t::BODY);
        let pixels_per_point = ui.ctx().pixels_per_point();
        let valid = self.rows.get(&args.turn).is_some_and(|cached| {
            cached.text == args.text
                && cached.width == args.width
                && cached.font == font
                && cached.pixels_per_point == pixels_per_point
                && cached.active == args.active
                && cached.not_heard == args.not_heard
                && cached.not_heard_style == args.not_heard_style
        });
        if !valid {
            self.rows.insert(
                args.turn,
                CachedText {
                    text: args.text.into(),
                    width: args.width,
                    font,
                    pixels_per_point,
                    active: args.active.clone(),
                    not_heard: args.not_heard.to_vec(),
                    not_heard_style: args.not_heard_style,
                    transcript: read_along::Transcript::measure(
                        ui,
                        args.text,
                        args.width,
                        args.active,
                        args.not_heard,
                        args.not_heard_style,
                    ),
                },
            );
        }
        &self.rows[&args.turn].transcript
    }
}

/// Height of the audio-clip badge row (icon + vertical pad).
fn clip_badge_row_height() -> f32 {
    t::AUDIO_CLIP_ICON + t::AUDIO_CLIP_BADGE_PAD * 2.0
}

fn paint_clip_badge(ui: &Ui, rect: Rect, clip_id: &str) -> f32 {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let label = "Audio clip";
    let label_galley =
        painter.layout_no_wrap(label.to_owned(), t::font(t::SECONDARY), t::TEXT_AUDIO_CLIP);
    let badge_w = t::AUDIO_CLIP_BADGE_PAD * 2.0
        + t::AUDIO_CLIP_ICON
        + t::AUDIO_CLIP_BADGE_GAP
        + label_galley.size().x;
    let badge_h = clip_badge_row_height();
    let badge = Rect::from_min_size(rect.min, vec2(badge_w.min(rect.width()), badge_h));
    painter.rect(
        badge,
        t::RADIUS_AUDIO_CLIP_BADGE,
        t::BG_AUDIO_CLIP_BADGE,
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        egui::StrokeKind::Inside,
    );
    let icon = Rect::from_min_size(
        pos2(
            badge.left() + t::AUDIO_CLIP_BADGE_PAD,
            badge.center().y - t::AUDIO_CLIP_ICON / 2.0,
        ),
        vec2(t::AUDIO_CLIP_ICON, t::AUDIO_CLIP_ICON),
    );
    egui::Image::new(t::icon("file-audio-13-text")).paint_at(ui, icon);
    painter.galley(
        pos2(
            icon.right() + t::AUDIO_CLIP_BADGE_GAP,
            badge.center().y - label_galley.size().y / 2.0,
        ),
        label_galley,
        t::TEXT_AUDIO_CLIP,
    );
    let id_x = badge.right() + t::SPACE_8;
    if id_x < rect.right() {
        let id_galley =
            painter.layout_no_wrap(clip_id.to_owned(), t::mono(t::NUMERIC), t::TEXT_MUTED);
        painter.galley(
            pos2(id_x, badge.center().y - id_galley.size().y / 2.0),
            id_galley,
            t::TEXT_MUTED,
        );
    }
    badge_h
}
struct RowOffsets(Vec<f32>);
impl RowOffsets {
    fn new(heights: impl IntoIterator<Item = f32>) -> Self {
        let mut offsets = vec![0.0];
        for height in heights {
            offsets.push(offsets.last().unwrap() + height);
        }
        Self(offsets)
    }
    fn top(&self, row: usize) -> f32 {
        self.0[row]
    }
    fn height(&self, row: usize) -> f32 {
        self.0[row + 1] - self.0[row]
    }
    fn total(&self) -> f32 {
        *self.0.last().unwrap()
    }
    fn scroll_to(&self, row: usize) -> f32 {
        self.top(row)
    }
    fn visible(&self, top: f32, bottom: f32) -> std::ops::Range<usize> {
        let count = self.0.len() - 1;
        let start = self
            .0
            .partition_point(|offset| *offset <= top)
            .saturating_sub(1)
            .min(count);
        let end = self.0.partition_point(|offset| *offset < bottom).min(count);
        start..end.max(start)
    }
}

#[expect(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    episode: &Episode,
    visible: &[usize],
    cache: &mut TextCache,
    selected: Option<usize>,
    playing: Option<usize>,
    read_position: Option<u64>,
    scroll_to: Option<usize>,
    failed_only: bool,
    multi: &std::collections::BTreeSet<usize>,
    edited: &[usize],
    mismatch: bool,
    find: &crate::find_replace::FindReplace,
    query: &str,
    options: &scriptcast_ui::find::Options,
    editing: &mut Editing,
    edit_mode: EditMode,
    enabled: bool,
) -> (Option<(usize, bool)>, Option<editing::Action>) {
    let matcher = scriptcast_ui::find::Matcher::new(query, options).ok();
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
        "Transcript",
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
    let content_width = (body.width() - body_ui.spacing().scroll.allocated_width()).max(0.0);
    let layout_width = columns(
        Rect::from_min_size(body.min, vec2(content_width, body.height())),
        &widths,
    )[3]
    .width();
    // Prefer last painted cell width so wrap matches the clip rect (avoids
    // end-of-line clipping of not-heard labels when scrollbar math differs).
    let width_id = egui::Id::new("turn_table_transcript_w");
    let transcript_width = body_ui.ctx().data(|d| {
        d.get_temp::<f32>(width_id)
            .map(|painted| painted.min(layout_width))
            .unwrap_or(layout_width)
    });
    let editor_turn = (edit_mode == EditMode::Row)
        .then(|| editing.draft.as_ref().map(|draft| draft.turn))
        .flatten();
    let field_height = editing.draft.as_ref().map_or(0.0, |draft| {
        editing::inline_field_height(&body_ui, &draft.text, transcript_width)
    });
    let editor_height = field_height
        + editing::inline_actions_height(&body_ui, transcript_width, editing.draft.as_ref())
        + t::SPACE_4 * 3.0;
    let heights = visible.iter().map(|&idx| {
        let raw = editing.body(idx).unwrap_or(&episode.turns[idx].first_words);
        let clip = script::clip_id(raw);
        let text = if clip.is_some() {
            script::display_body(raw)
        } else {
            raw.to_string()
        };
        let active = read_position
            .filter(|_| playing == Some(idx))
            .and_then(|position| {
                let turn_id = &episode.turns[idx].id;
                let position = scriptcast_ui::word_timings::manifest_position(
                    &episode.manifest.turns[idx],
                    position,
                );
                if clip.is_some() {
                    // Prefer script_words (v2) so digit years still highlight in-row.
                    let words = episode.clip_transcripts.row_words(turn_id)?;
                    read_along::word_mapping(&text, &words)?;
                    read_along::active_range(&text, &words, position)
                } else {
                    let words = episode.read_along.words(turn_id)?;
                    read_along::active_range(&text, words, position)
                }
            });
        let marks = not_heard_ranges(episode, &episode.turns[idx], &text);
        let mark_style = (!marks.is_empty()).then(|| not_heard_style(edit_mode));
        let transcript = cache.measure(
            &body_ui,
            MeasureArgs {
                turn: idx,
                text: &text,
                width: transcript_width,
                active,
                not_heard: &marks,
                not_heard_style: mark_style,
            },
        );
        if editor_turn == Some(idx) {
            editor_height
        } else {
            let chrome = if clip.is_some() {
                clip_badge_row_height() + t::AUDIO_CLIP_BADGE_MARGIN * 2.0
            } else {
                0.0
            };
            chrome + transcript.height()
        }
    });
    let rows = RowOffsets::new(heights);
    if let Some(position) = scroll_to.and_then(|idx| visible.iter().position(|v| *v == idx)) {
        scroll = scroll.vertical_scroll_offset(rows.scroll_to(position));
    }
    let mut clicked = None;
    let mut edit_action = None;
    let mut displayed = 0..0;
    scroll.show_viewport(&mut body_ui, |ui, viewport| {
        ui.set_height(rows.total());
        let origin = ui.max_rect().min;
        let width = ui.available_width();
        // Each turn and its expanded editor share one variable-height item.
        // Only intersecting items create widgets; the editor scrolls with its row.
        displayed = rows.visible(viewport.top(), viewport.bottom());
        for row in displayed.clone() {
            let idx = visible[row];
            let row_rect = Rect::from_min_size(
                origin + vec2(0.0, rows.top(row)),
                vec2(width, rows.height(row)),
            );
            let mut row_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(("row", idx))
                    .max_rect(row_rect),
            );
            let ui = &mut row_ui;
            let turn = &episode.turns[idx];
            let speaker = &episode.speakers[turn.speaker_idx];
            let (rect, response) = ui.allocate_exact_size(row_rect.size(), Sense::click());
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
            ui.ctx()
                .data_mut(|d| d.insert_temp(width_id, cells[3].width()));
            let (row_text, row_active, row_marks, row_style) = {
                let cached = &cache.rows[&idx];
                (
                    cached.text.clone(),
                    cached.active.clone(),
                    cached.not_heard.clone(),
                    cached.not_heard_style,
                )
            };
            let transcript = cache.measure(
                ui,
                MeasureArgs {
                    turn: idx,
                    text: &row_text,
                    width: cells[3].width(),
                    active: row_active,
                    not_heard: &row_marks,
                    not_heard_style: row_style,
                },
            );
            let first_line_height = transcript
                .galley
                .rows
                .first()
                .map_or_else(|| t::line_height(ui, t::font(t::BODY)), |row| row.size.y);
            let top_cells = cells.map(|mut cell| {
                cell.max.y = cell.top() + first_line_height + t::SPACE_4 * 2.0;
                cell
            });
            let playing_row = playing == Some(idx);
            view::line(
                ui,
                top_cells[0],
                &if playing_row {
                    format!("▶ {:03}", turn.index)
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
                top_cells[1],
                &view::time(turn.start_ms),
                t::mono(t::NUMERIC),
                t::TEXT_MUTED,
            );
            view::line(
                ui,
                top_cells[2],
                &speaker.name,
                t::font(t::SPEAKER_LABEL),
                t::speaker_colour(speaker.colour_idx),
            );
            if editor_turn == Some(idx) {
                let mut editor_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("row_editor")
                        .max_rect(cells[3].shrink2(vec2(0.0, t::SPACE_4)))
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                editor_ui.set_clip_rect(cells[3].intersect(ui.clip_rect()));
                edit_action =
                    editing::inline_editor(&mut editor_ui, editing, field_height, enabled);
            } else {
                let raw = editing.body(idx).unwrap_or(&turn.first_words);
                let mut text_rect = cells[3];
                if let Some(id) = script::clip_id(raw) {
                    let mut badge_rect = text_rect;
                    badge_rect.min.y += t::AUDIO_CLIP_BADGE_MARGIN;
                    let used = paint_clip_badge(ui, badge_rect, id);
                    text_rect.min.y += used + t::AUDIO_CLIP_BADGE_MARGIN * 2.0;
                }
                let highlights = matcher.as_ref().map_or_else(Vec::new, |matcher| {
                    scriptcast_ui::find::occurrences(idx, raw, matcher, options)
                });
                let current = find.current.and_then(|i| find.matches.get(i));
                paint_matches(
                    ui,
                    text_rect,
                    transcript,
                    &row_text,
                    &highlights,
                    current,
                );
                transcript.paint(ui, text_rect);
            }
            view::line(
                ui,
                top_cells[4],
                &format!("{:.2} s", (turn.end_ms - turn.start_ms) as f64 / 1000.0),
                t::mono(t::NUMERIC),
                t::TEXT_MUTED,
            );
            let (label, colour) = if edited.contains(&idx) {
                ("Edited", t::accent())
            } else {
                view::verification_label(turn.verification)
            };
            let maybe_misheard = turn.verification == V::Failed
                && episode
                    .stress_ensemble
                    .get(&turn.id)
                    .is_some_and(|e| e.summary == "likely_recognition");
            if maybe_misheard {
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    label,
                    0.0,
                    egui::TextFormat {
                        font_id: t::font(t::SECONDARY),
                        color: colour,
                        ..Default::default()
                    },
                );
                job.append(
                    "  maybe misheard",
                    0.0,
                    egui::TextFormat {
                        font_id: t::font(t::SECONDARY),
                        color: t::TEXT_MUTED,
                        ..Default::default()
                    },
                );
                job.wrap.max_width = top_cells[5].width().max(0.0);
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let galley = ui.painter().layout_job(job);
                ui.painter()
                    .with_clip_rect(top_cells[5].intersect(ui.clip_rect()))
                    .galley(
                        pos2(
                            top_cells[5].left(),
                            top_cells[5].center().y - galley.size().y / 2.0,
                        ),
                        galley,
                        colour,
                    );
            } else {
                view::line(ui, top_cells[5], label, t::font(t::SECONDARY), colour);
            }
            let response = response.on_hover_text(format!(
                "Turn {:03} · {}\n{}\n{}",
                turn.index,
                speaker.name,
                editing.body(idx).unwrap_or(&turn.first_words),
                if maybe_misheard {
                    format!("{label} · maybe misheard")
                } else {
                    label.to_string()
                },
            ));
            if response.clicked() || response.double_clicked() {
                clicked = Some((idx, response.double_clicked()));
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

fn paint_matches(
    ui: &Ui,
    rect: Rect,
    transcript: &read_along::Transcript,
    text: &str,
    matches: &[scriptcast_ui::find::Occurrence],
    current: Option<&scriptcast_ui::find::Occurrence>,
) {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let origin = rect.min + vec2(0.0, t::SPACE_4);
    for found in matches {
        let chars = text[..found.displayed.start].chars().count()
            ..text[..found.displayed.end].chars().count();
        let mut index = 0;
        for row in &transcript.galley.rows {
            let mut highlight = Rect::NOTHING;
            for glyph in &row.glyphs {
                if chars.contains(&index) {
                    highlight = highlight.union(Rect::from_min_size(
                        origin + row.pos.to_vec2() + vec2(glyph.pos.x, 0.0),
                        vec2(glyph.advance_width, row.size.y),
                    ));
                }
                index += 1;
            }
            if row.ends_with_newline {
                index += 1;
            }
            if highlight.is_positive() {
                if current == Some(found) {
                    painter.rect_filled(highlight, t::RADIUS_CONTROL, t::FIND_CURRENT);
                    painter.rect_stroke(
                        highlight,
                        t::RADIUS_CONTROL,
                        egui::Stroke::new(t::BORDER, t::FIND_GOLD),
                        egui::StrokeKind::Inside,
                    );
                }
                painter.line_segment(
                    [highlight.left_bottom(), highlight.right_bottom()],
                    egui::Stroke::new(t::BORDER, t::FIND_GOLD),
                );
            }
        }
    }
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
    fn prefix_sums_mix_one_and_two_line_heights_and_scroll_to() {
        let heights = [
            t::TABLE_ROW_HEIGHT,
            t::TABLE_ROW_HEIGHT_2,
            t::TABLE_ROW_HEIGHT,
            t::TABLE_ROW_HEIGHT_2,
        ];
        let rows = RowOffsets::new(heights);
        assert_eq!(
            rows.total(),
            t::TABLE_ROW_HEIGHT * 2.0 + t::TABLE_ROW_HEIGHT_2 * 2.0
        );
        assert_eq!(rows.height(0), t::TABLE_ROW_HEIGHT);
        assert_eq!(rows.height(1), t::TABLE_ROW_HEIGHT_2);
        assert_eq!(rows.top(0), 0.0);
        assert_eq!(rows.top(1), t::TABLE_ROW_HEIGHT);
        assert_eq!(rows.top(2), t::TABLE_ROW_HEIGHT + t::TABLE_ROW_HEIGHT_2);
        assert_eq!(rows.scroll_to(2), rows.top(2));
        assert_eq!(rows.scroll_to(3), rows.top(3));
        // Viewport covering only the second (tall) row.
        let top = rows.top(1);
        let bottom = top + rows.height(1) - 1.0;
        assert_eq!(rows.visible(top, bottom), 1..2);
        // Viewport spanning a short row into the next tall row.
        let top = rows.top(2) + 1.0;
        let bottom = rows.top(3) + 10.0;
        assert_eq!(rows.visible(top, bottom), 2..4);
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
