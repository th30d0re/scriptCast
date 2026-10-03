use eframe::egui::{self, Color32, CursorIcon, FontId, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use scriptcast_ui::{
    episode::{Speaker, VerificationState},
    theme as t,
};

/// Result of a pane-boundary drag handle for one frame.
pub struct SplitterOutcome {
    /// Axis delta this frame (y for a horizontal bar, x for a vertical bar).
    pub drag_delta: f32,
    /// Double-click: caller should clear the persisted size back to default.
    pub reset: bool,
}

/// Horizontal bar (resize up/down) centred on `boundary_y`, spanning `[left, right]`.
pub fn splitter_horizontal(
    ui: &mut Ui,
    id: &'static str,
    boundary_y: f32,
    left: f32,
    right: f32,
) -> SplitterOutcome {
    let half = t::SPLITTER_HIT / 2.0;
    let hit = Rect::from_min_max(
        pos2(left, boundary_y - half),
        pos2(right, boundary_y + half),
    );
    splitter(ui, id, hit, true, CursorIcon::ResizeVertical)
}

/// Vertical bar (resize left/right) centred on `boundary_x`, spanning `[top, bottom]`.
pub fn splitter_vertical(
    ui: &mut Ui,
    id: &'static str,
    boundary_x: f32,
    top: f32,
    bottom: f32,
) -> SplitterOutcome {
    let half = t::SPLITTER_HIT / 2.0;
    let hit = Rect::from_min_max(
        pos2(boundary_x - half, top),
        pos2(boundary_x + half, bottom),
    );
    splitter(ui, id, hit, false, CursorIcon::ResizeHorizontal)
}

fn splitter(
    ui: &mut Ui,
    id: &'static str,
    hit: Rect,
    horizontal_bar: bool,
    cursor: CursorIcon,
) -> SplitterOutcome {
    let response = ui.interact(hit, ui.id().with(id), Sense::click_and_drag());
    let active = response.hovered() || response.dragged();
    if active {
        ui.ctx().set_cursor_icon(cursor);
    }
    let colour = if active { t::ACCENT } else { t::BORDER_DEFAULT };
    let line = if horizontal_bar {
        [
            pos2(hit.left(), hit.center().y),
            pos2(hit.right(), hit.center().y),
        ]
    } else {
        [
            pos2(hit.center().x, hit.top()),
            pos2(hit.center().x, hit.bottom()),
        ]
    };
    ui.painter()
        .line_segment(line, egui::Stroke::new(t::BORDER, colour));
    let drag_delta = if response.dragged() {
        let d = response.drag_delta();
        if horizontal_bar { d.y } else { d.x }
    } else {
        0.0
    };
    if response.dragged() {
        ui.ctx().request_repaint();
    }
    SplitterOutcome {
        drag_delta,
        reset: response.double_clicked(),
    }
}

pub fn region(ui: &mut Ui, id: &str, rect: Rect, fill: Color32) -> Ui {
    ui.painter().rect_filled(rect, 0, fill);
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child
}
pub fn line(ui: &Ui, rect: Rect, value: &str, font: FontId, colour: Color32) {
    let mut job = egui::text::LayoutJob::simple_singleline(value.into(), font, colour);
    job.wrap.max_width = rect.width().max(0.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = ui.painter().layout_job(job);
    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .galley(
            pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            colour,
        );
}
pub fn rule(ui: &Ui, rect: Rect) {
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
    );
}
pub fn inset(rect: Rect, horizontal: f32, vertical: f32) -> Rect {
    rect.shrink2(vec2(horizontal, vertical))
}
pub fn time(ms: u64) -> String {
    format!("{:02}:{:02}.{:03}", ms / 60000, (ms / 1000) % 60, ms % 1000)
}
pub fn duration(ms: u64) -> String {
    format!("{:02}:{:02}", ms / 60000, (ms / 1000) % 60)
}
/// Widest `▎<display name>` across speakers in the speaker-label font. `▎` is
/// not in OpenDyslexic, so it measures as the replacement glyph — roughly the
/// width of the painted colour bar and its gap.
pub fn speaker_label_width(ui: &Ui, speakers: &[Speaker]) -> f32 {
    speakers.iter().fold(0.0, |widest, speaker| {
        widest.max(t::text_width(
            ui,
            speaker.name.clone(),
            t::font(t::SPEAKER_LABEL),
        ))
    })
}
/// Verification column / inspector label and colour for a state. Never
/// manufactures a result: turns without a `stress.json` entry are NotVerified.
pub fn verification_label(state: VerificationState) -> (&'static str, Color32) {
    match state {
        VerificationState::Passed => ("✓  Passed", t::STATUS_PASSED),
        VerificationState::Failed => ("×  Failed", t::STATUS_FAILED),
        VerificationState::NotVerified => ("Not verified", t::TEXT_SUBTLE),
        VerificationState::MissingAudio => ("Missing audio", t::STATUS_FAILED),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn time_formats_mm_ss_mmm() {
        assert_eq!(super::time(0), "00:00.000");
        assert_eq!(super::time(4_620), "00:04.620");
        assert_eq!(super::time(9_060), "00:09.060");
        assert_eq!(super::time(90_060), "01:30.060");
        assert_eq!(super::time(3_861_350), "64:21.350");
        // sub-millisecond precision is truncated, and minutes are not padded
        // beyond two digits
        assert_eq!(super::time(999), "00:00.999");
        assert_eq!(super::time(7_200_000), "120:00.000");
    }
}
