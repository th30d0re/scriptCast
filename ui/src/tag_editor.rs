//! Shared body editor and caret-preserving toolbar for inline, panel and new-turn drafts.
use crate::editing::editor_id;
use eframe::egui::{self, Ui, text::CCursorRange};
use crate::{structure::Registry, tags, theme as t};

pub struct Toolbar {
    pub pause: String,
    pub tone: String,
    pub raw: bool,
    pub picker: bool,
    pub search: String,
    pub selected: Option<String>,
    pub preview: Option<String>,
    pub preview_error: Option<String>,
    pub cursor: Option<CCursorRange>,
    pub select_first: bool,
}
impl Default for Toolbar {
    fn default() -> Self {
        Self {
            pause: "800".into(),
            tone: String::new(),
            raw: false,
            picker: false,
            search: String::new(),
            selected: None,
            preview: None,
            preview_error: None,
            cursor: None,
            select_first: false,
        }
    }
}
pub fn insert(text: &mut String, cursor: Option<CCursorRange>, tag: &str) -> CCursorRange {
    let range = cursor.map(|c| c.as_sorted_char_range());
    let start = range
        .as_ref()
        .map(|r| r.start.0)
        .unwrap_or(text.chars().count());
    let end = range.map(|r| r.end.0).unwrap_or(start);
    let byte = |n| {
        text.char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(text.len())
    };
    text.replace_range(byte(start)..byte(end), tag);
    CCursorRange::one(egui::text::CCursor::new(start + tag.chars().count()))
}

pub fn body(
    ui: &mut Ui,
    text: &mut String,
    state: &mut Toolbar,
    registry: &Registry,
    height: f32,
    placeholder: &str,
    min_lines: f32,
    actions: impl FnOnce(&mut Ui),
) -> egui::Response {
    let mut insertion = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = t::SPACE_4;
        ui.spacing_mut().button_padding = egui::vec2(t::SPACE_4, t::SPACE_4);
        if ui.button("Pause").clicked() {
            insertion = Some(format!("[pause:{}ms]", state.pause));
        }
        ui.add(egui::TextEdit::singleline(&mut state.pause).desired_width(t::TAG_FIELD_WIDTH));
        if ui.button("Beat").clicked() {
            insertion = Some("[beat]".into());
        }
        if ui.button("Emphasis").clicked() {
            insertion = Some("[emphasis]".into());
        }
        if ui.button("Tone").clicked() {
            insertion = Some(format!("[tone:{}]", state.tone));
        }
        ui.add(egui::TextEdit::singleline(&mut state.tone).desired_width(t::TAG_FIELD_WIDTH));
        let clip = ui.button("Clip ▾");
        if clip.clicked() {
            state.picker = !state.picker;
        }
        let mut picker = state.picker;
        egui::Popup::from_response(&clip)
            .open_bool(&mut picker)
            .width(t::CLIP_PICKER_WIDTH)
            .show(|ui| {
                ui.set_width(t::CLIP_PICKER_WIDTH);
                if let Some(tag) = clip_picker(ui, state, registry) {
                    insertion = Some(tag);
                }
            });
        state.picker = picker;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), t::TAG_TOOLBAR_HEIGHT),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = t::SPACE_8;
                actions(ui);
            },
        );
    });
    if let Some(tag) = insertion {
        let cursor = insert(text, state.cursor, &tag);
        state.cursor = Some(cursor);
        let mut edit_state = egui::TextEdit::load_state(ui.ctx(), editor_id()).unwrap_or_default();
        edit_state.cursor.set_char_range(Some(cursor));
        edit_state.store(ui.ctx(), editor_id());
        ui.memory_mut(|m| m.request_focus(editor_id()));
    }
    let raw = state.raw;
    let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, width: f32| {
        ui.fonts_mut(|f| f.layout_job(tags::layout(buffer.as_str(), registry, raw, width)))
    };
    let line = t::line_height(ui, t::font(t::BODY));
    let width = ui.available_width();
    let output = egui::TextEdit::multiline(text)
        .id(editor_id())
        .font(if raw {
            t::mono(t::BODY)
        } else {
            t::font(t::BODY)
        })
        .hint_text(placeholder)
        .desired_rows(min_lines.max(1.0) as usize)
        .desired_width(width)
        .min_size(egui::vec2(width, height.max(line * min_lines)))
        .layouter(&mut layouter)
        .show(ui);
    ui.checkbox(&mut state.raw, "Raw script");
    if let Some(cursor) = output.cursor_range {
        state.cursor = Some(cursor);
    }
    for span in tags::spans(text, registry) {
        for message in span.messages {
            let tag = &text[span.range.clone()];
            let message = if span.blocks_save
                && tag.starts_with("[pause:")
                && message.starts_with("unrecognised")
            {
                "is not a number of milliseconds"
            } else {
                &message
            };
            ui.label(t::text(
                format!("{tag} {message}"),
                t::CAPTION,
                t::STATUS_FAILED,
            ));
        }
    }
    if !text.trim().is_empty() && !tags::can_save(text, registry) {
        ui.label(t::text(
            "Fix invalid pause or clip to save.",
            t::CAPTION,
            t::STATUS_FAILED,
        ));
    }
    output.response.response
}

pub fn clip_picker(ui: &mut Ui, state: &mut Toolbar, registry: &Registry) -> Option<String> {
    ui.label(t::heading("Registered clips", t::SECTION_TITLE));
    ui.add(egui::TextEdit::singleline(&mut state.search).hint_text("Search clips…"));
    let query = state.search.to_lowercase();
    if state.select_first && state.selected.is_none() {
        state.selected = registry
            .clips
            .iter()
            .find(|clip| {
                format!("{} {} {}", clip.id, clip.source.display(), clip.citation)
                    .to_lowercase()
                    .contains(&query)
            })
            .map(|clip| clip.id.clone());
    }
    egui::ScrollArea::vertical()
        .max_height(t::CLIP_PICKER_HEIGHT)
        .show(ui, |ui| {
            for clip in &registry.clips {
                if !format!("{} {} {}", clip.id, clip.source.display(), clip.citation)
                    .to_lowercase()
                    .contains(&query)
                {
                    continue;
                }
                let chosen = state.selected.as_ref() == Some(&clip.id);
                let frame = egui::Frame::new()
                    .fill(if chosen {
                        t::BG_SELECTED
                    } else {
                        t::BG_SURFACE
                    })
                    .inner_margin(t::SPACE_4);
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(chosen, &clip.id).clicked() {
                            state.selected = Some(clip.id.clone());
                        }
                        ui.label(t::text(
                            format!("{:.1} s", clip.end - clip.start),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                        if ui.button("▶").clicked() {
                            state.preview = Some(clip.id.clone());
                        }
                    });
                    ui.label(t::text(
                        clip.source.to_string_lossy(),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    ui.label(t::text(&clip.citation, t::CAPTION, t::TEXT_SUBTLE));
                });
            }
            if registry.clips.is_empty() {
                ui.label("No registered clips");
            }
        });
    if let Some(error) = &state.preview_error {
        ui.label(t::text(error, t::CAPTION, t::STATUS_FAILED));
    }
    let id = state.selected.clone();
    if t::primary_button(
        ui,
        &format!("Insert selected {}", id.as_deref().unwrap_or("clip")),
        None,
        id.is_some(),
    )
    .clicked()
    {
        state.picker = false;
        return id.map(|id| format!("[clip:{id}]"));
    }
    None
}
