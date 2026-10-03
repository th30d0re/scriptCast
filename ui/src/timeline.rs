use crate::view;
use eframe::egui::{self, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use scriptcast_ui::{
    episode::{Episode, VerificationState},
    theme as t,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct TimelineView {
    pub start: f64,
    pub end: f64,
}
impl TimelineView {
    pub fn initial(total: u64) -> Self {
        Self {
            start: 0.0,
            end: (total as f64).clamp(1.0, 90_000.0),
        }
    }
    pub fn fit(&mut self, total: u64) {
        self.start = 0.0;
        self.end = (total as f64).max(1.0);
    }
    pub fn zoom(&mut self, factor: f64, total: u64) {
        let total = (total as f64).max(1.0);
        let width = ((self.end - self.start) / factor).clamp(total.min(1000.0), total);
        self.pan((self.start + self.end) / 2.0, width, total);
    }
    fn pan(&mut self, centre: f64, width: f64, total: f64) {
        self.start = (centre - width / 2.0).clamp(0.0, (total - width).max(0.0));
        self.end = self.start + width;
    }
    pub fn reveal(&mut self, start: u64, end: u64, total: u64) {
        if (start as f64) < self.start || (end as f64) > self.end {
            let width = (self.end - self.start)
                .max((end - start) as f64)
                .min((total as f64).max(1.0));
            self.pan(
                (start as f64 + end as f64) / 2.0,
                width,
                (total as f64).max(1.0),
            );
        }
    }
}

/// Frameless clickable text brightening on hover (design screen 02 zoom controls).
fn zoom_button(ui: &mut Ui, label: &str) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.into(), t::font(t::SECONDARY), t::TEXT_MUTED);
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::click());
    if ui.is_rect_visible(rect) {
        ui.painter().galley_with_override_text_color(
            rect.min,
            galley,
            if response.hovered() {
                t::TEXT_PRIMARY
            } else {
                t::TEXT_MUTED
            },
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn show(
    ui: &mut Ui,
    episode: &Episode,
    sections: &[Option<usize>],
    view: &mut TimelineView,
    matches: &[bool],
    selected: Option<usize>,
    playing: Option<(usize, u64)>,
) -> Option<usize> {
    let rect = ui.max_rect();
    let content = view::inset(rect, t::SPACE_20, t::SPACE_10);
    let heading = Rect::from_min_size(content.min, vec2(content.width(), t::SPACE_22));
    let mut top = view::region(ui, "timeline_heading", heading, t::BG_WINDOW);
    top.horizontal_centered(|ui| {
        ui.label(t::heading("Timeline", t::SECTION_TITLE));
        ui.label(
            egui::RichText::new(view::time(episode.total_ms))
                .font(t::mono(t::NUMERIC))
                .color(t::TEXT_MUTED),
        );
        ui.label(t::text(
            format!("· {} turns", episode.turns.len()),
            t::SECONDARY,
            t::TEXT_MUTED,
        ));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            // Design screen 02: plain muted text, no frame or fill.
            if zoom_button(ui, "Fit episode").clicked() {
                view.fit(episode.total_ms);
            }
            ui.add_space(t::SPACE_14);
            if zoom_button(ui, "+").clicked() {
                view.zoom(1.5, episode.total_ms);
            }
            ui.add_space(t::SPACE_8);
            ui.label(t::text("Zoom", t::SECONDARY, t::TEXT_MUTED));
            ui.add_space(t::SPACE_8);
            if zoom_button(ui, "−").clicked() {
                view.zoom(1.0 / 1.5, episode.total_ms);
            }
        });
    });
    let zoom = ui.input(|i| {
        if i.modifiers.ctrl && i.pointer.hover_pos().is_some_and(|p| rect.contains(p)) {
            i.zoom_delta()
        } else {
            1.0
        }
    });
    if zoom != 1.0 {
        view.zoom(zoom as f64, episode.total_ms);
    }
    let overview = Rect::from_min_size(
        pos2(content.left(), heading.bottom() + t::SPACE_6),
        vec2(content.width(), t::TIMELINE_OVERVIEW_HEIGHT),
    );
    let total = (episode.total_ms as f64).max(1.0);
    let response = ui.interact(
        overview.expand(t::SPACE_2),
        ui.id().with("overview"),
        Sense::click_and_drag(),
    );
    if (response.clicked() || response.dragged())
        && let Some(p) = response.interact_pointer_pos()
    {
        view.pan(
            ((p.x - overview.left()) / overview.width()) as f64 * total,
            view.end - view.start,
            total,
        );
    }
    let painter = ui.painter();
    for (i, speaker) in sections.iter().enumerate() {
        let begin = total * i as f64 / sections.len() as f64;
        let end = total * (i + 1) as f64 / sections.len() as f64;
        let colour = speaker
            .map(|s| t::speaker_colour(episode.speakers[s].colour_idx))
            .unwrap_or(t::BG_CONTROL);
        let colour = if end < view.start || begin > view.end {
            colour.gamma_multiply(t::OPACITY_OVERVIEW_SECTION_OUTSIDE_VISIBLE_RANGE)
        } else {
            colour
        };
        let x = overview.left() + overview.width() * i as f32 / sections.len() as f32;
        painter.rect_filled(
            Rect::from_min_max(
                pos2(x, overview.top()),
                pos2(
                    x + overview.width() / sections.len() as f32 - t::SPACE_2,
                    overview.bottom(),
                ),
            ),
            t::RADIUS_OVERVIEW_SECTION,
            colour,
        );
    }
    for turn in &episode.turns {
        if turn.verification == VerificationState::Failed {
            let x = overview.left() + overview.width() * (turn.start_ms as f64 / total) as f32;
            painter.line_segment(
                [pos2(x, overview.top()), pos2(x, overview.bottom())],
                Stroke::new(t::BORDER, t::STATUS_FAILED),
            );
        }
    }
    let visible = Rect::from_min_max(
        pos2(
            overview.left() + overview.width() * (view.start / total) as f32,
            overview.center().y - t::TIMELINE_VISIBLE_RANGE_HEIGHT / 2.0,
        ),
        pos2(
            overview.left() + overview.width() * (view.end / total) as f32,
            overview.center().y + t::TIMELINE_VISIBLE_RANGE_HEIGHT / 2.0,
        ),
    );
    painter.rect_stroke(
        visible,
        t::RADIUS_OVERVIEW_SECTION,
        Stroke::new(t::BORDER, t::ACCENT),
        StrokeKind::Inside,
    );
    // The lane label column and everything right of it (ruler, tracks) follow
    // the widest speaker name, with the design token as the minimum width.
    let label_width = t::fit_text_width(
        view::speaker_label_width(ui, &episode.speakers),
        t::TIMELINE_LANE_LABEL_WIDTH,
    );
    let gutter = label_width + t::TIMELINE_LANES_TO_LABEL_GAP;
    let ruler = Rect::from_min_size(
        pos2(content.left() + gutter, overview.bottom() + t::SPACE_6),
        vec2(
            (content.width() - gutter).max(0.0),
            t::TIMELINE_RULER_HEIGHT,
        ),
    );
    let width = (view.end - view.start).max(1.0);
    let x_at = |ms: f64| ruler.left() + ruler.width() * ((ms - view.start) / width) as f32;
    // Aim for six labelled intervals, rounded to useful time units.
    let step = [
        1000.0, 2000.0, 5000.0, 10000.0, 15000.0, 30000.0, 60000.0, 120000.0, 300000.0, 600000.0,
        1800000.0, 3600000.0,
    ]
    .into_iter()
    .find(|s| width / s <= 6.0)
    .unwrap_or(width / 6.0);
    let mut tick = (view.start / step).ceil() * step;
    while tick <= view.end {
        let label = view::duration(tick as u64);
        let label_width = ui
            .painter()
            .layout_no_wrap(label.clone(), t::mono(t::RULER_TICK), t::TEXT_SUBTLE)
            .size()
            .x;
        let x = x_at(tick).min(ruler.right() - label_width);
        view::line(
            ui,
            Rect::from_min_size(pos2(x, ruler.top()), vec2(label_width, ruler.height())),
            &label,
            t::mono(t::RULER_TICK),
            t::TEXT_SUBTLE,
        );
        tick += step;
    }
    let lanes = Rect::from_min_max(pos2(content.left(), ruler.bottom()), content.right_bottom());
    let mut lane_ui = view::region(ui, "lanes", lanes, t::BG_WINDOW);
    lane_ui.spacing_mut().item_spacing.y = t::TIMELINE_LANE_GAP;
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .id_salt("speaker_lanes")
        .auto_shrink([false, false])
        .show(&mut lane_ui, |ui| {
            for (speaker_idx, speaker) in episode.speakers.iter().enumerate() {
                let (lane, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), t::TIMELINE_LANE_HEIGHT),
                    Sense::hover(),
                );
                let colour = t::speaker_colour(speaker.colour_idx);
                let label = Rect::from_min_size(lane.min, vec2(label_width, lane.height()));
                view::line(
                    ui,
                    label,
                    &speaker.name,
                    t::font(t::SPEAKER_LABEL),
                    colour,
                );
                ui.interact(
                    label,
                    ui.id().with(("speaker", speaker_idx)),
                    Sense::hover(),
                )
                .on_hover_text(&speaker.name);
                let track = Rect::from_min_max(pos2(ruler.left(), lane.top()), lane.right_bottom());
                ui.painter().rect_filled(track, 0, t::BG_SURFACE);
                for (idx, turn) in episode.turns.iter().enumerate().filter(|(_, turn)| {
                    turn.speaker_idx == speaker_idx
                        && turn.end_ms as f64 > view.start
                        && (turn.start_ms as f64) < view.end
                }) {
                    let block = Rect::from_min_max(
                        pos2(x_at(turn.start_ms as f64).max(track.left()), lane.top()),
                        pos2(
                            x_at(turn.end_ms as f64)
                                .min(track.right())
                                .max(track.left()),
                            lane.bottom(),
                        ),
                    );
                    if block.width() <= 0.0 {
                        continue;
                    }
                    // The playing segment wears the selected style (screen 03).
                    let active = selected == Some(idx) || playing.is_some_and(|(t, _)| t == idx);
                    let failed = turn.verification == VerificationState::Failed;
                    let opacity = if matches[idx] {
                        1.0
                    } else {
                        t::OPACITY_DISABLED
                    };
                    let fill = if active && failed {
                        t::BG_DANGER
                    } else if active {
                        t::BG_SELECTED
                    } else {
                        t::BG_SEGMENT
                    };
                    ui.painter().rect_filled(
                        block,
                        t::RADIUS_SEGMENT,
                        fill.gamma_multiply(opacity),
                    );
                    ui.painter().rect_stroke(
                        block,
                        t::RADIUS_SEGMENT,
                        Stroke::new(
                            if active { t::SPACE_2 } else { t::BORDER },
                            if active {
                                t::ACCENT
                            } else {
                                colour.gamma_multiply(opacity)
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    if block.width() > t::SPACE_8 {
                        view::line(
                            ui,
                            block.shrink2(vec2(t::SPACE_4, 0.0)),
                            &format!("{:03}", turn.index),
                            t::mono(t::SEGMENT_LABEL),
                            if active && failed {
                                t::STATUS_FAILED
                            } else {
                                colour.gamma_multiply(opacity)
                            },
                        );
                    }
                    let response = ui
                        .interact(block, ui.id().with(("turn", idx)), Sense::click())
                        .on_hover_text(format!(
                            "Turn {:03} · {}\n{} → {}\n{}",
                            turn.index,
                            speaker.name,
                            view::time(turn.start_ms),
                            view::time(turn.end_ms),
                            turn.first_words
                        ));
                    if response.clicked() {
                        clicked = Some(idx);
                    }
                }
            }
        });
    // Playhead: 1 px TEXT_PRIMARY across the speaker lanes at the playing
    // position (episode time = turn start + playback position).
    if let Some((_, ms)) = playing {
        let ms = ms as f64;
        if ms >= view.start && ms <= view.end {
            let x = x_at(ms).clamp(ruler.left(), ruler.right());
            ui.painter().line_segment(
                [pos2(x, lanes.top()), pos2(x, lanes.bottom())],
                Stroke::new(t::BORDER, t::TEXT_PRIMARY),
            );
        }
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_and_reveal_stay_within_episode() {
        let mut v = TimelineView::initial(100_000);
        v.zoom(2.0, 100_000);
        assert_eq!(v.end - v.start, 45_000.0);
        v.reveal(99_000, 100_000, 100_000);
        assert_eq!(v.end, 100_000.0);
        v.zoom(0.001, 100_000);
        assert_eq!((v.start, v.end), (0.0, 100_000.0));
        v.fit(0);
        v.zoom(2.0, 0);
        assert!(v.end > v.start);
    }
}
