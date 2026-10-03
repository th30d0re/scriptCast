use eframe::egui::{self, Color32, FontId, Rect, Ui, UiBuilder, pos2, vec2};
use scriptcast_ui::{episode::Speaker, theme as t};

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
            format!("▎{}", speaker.name),
            t::font(t::SPEAKER_LABEL),
        ))
    })
}
