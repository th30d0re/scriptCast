//! Turn 21 of `APAGroupDefinitions`: definition card, field panel, derived-illustration tray.
//!
//! The three regions sit in auto-layout. Their outer boxes match the scene's
//! `APA_GROUP_LAYOUT` (definition at 110,190 650×400 for the bias card; field at
//! 830,130 960×520; tray at 830,690 960×220). Person marks, the tendency arrow and
//! the resource dots stay drawings in Stage; this document only places frames and text.
//!
//! Type matches the React labels: IBM Plex Sans where `sans` is set (term, body),
//! IBM Plex Mono otherwise. Weight 650 is SemiBold, because Stage loads Sans 400/500/600
//! and CSS maps 650 onto 600. Line height is the label's unitless `lh` (default 1.25).

use figcraft_doc::{
    AutoLayout, CrossAlign, Document, LayoutMode, LetterSpacing, LineHeight, MainAlign, NodeId, NodeKind, Sizing, TextAlignH, TextAutoResize,
    TextData, TextStyle,
};

const SANS: &str = "IBM Plex Sans";
const MONO: &str = "IBM Plex Mono";
const BIAS: &str = "the tendency to favor one\u{2019}s own group and its members when dividing resources, judging conduct, and explaining behavior";

pub fn apa_document_json() -> String {
    serde_json::to_string_pretty(&apa_document()).expect("APA document serializes")
}

pub fn apa_document() -> Document {
    let mut doc = Document::new("APAGroupDefinitions");
    let page = doc.pages[0];
    doc.get_mut(page).expect("page").name = "page".into();

    let stage = frame(&mut doc, page, "stage", 1920.0, 1080.0, Sizing::Fixed, Sizing::Fixed);
    lay(&mut doc, stage, flow(LayoutMode::Horizontal, 70.0, [130.0, 130.0, 0.0, 110.0], MainAlign::Min, CrossAlign::Min));

    let wrap = frame(&mut doc, stage, "definition-wrap", 650.0, 460.0, Sizing::Fixed, Sizing::Hug);
    lay(&mut doc, wrap, flow(LayoutMode::Vertical, 0.0, [60.0, 0.0, 0.0, 0.0], MainAlign::Min, CrossAlign::Min));

    let card = frame(&mut doc, wrap, "definition-card", 650.0, 400.0, Sizing::Fill, Sizing::Fixed);
    lay(&mut doc, card, flow(LayoutMode::Horizontal, 0.0, [0.0; 4], MainAlign::Min, CrossAlign::Min));
    frame(&mut doc, card, "definition-accent", 5.0, 400.0, Sizing::Fixed, Sizing::Fill);
    let copy = frame(&mut doc, card, "definition-copy", 645.0, 400.0, Sizing::Fill, Sizing::Fill);
    lay(&mut doc, copy, flow(LayoutMode::Vertical, 0.0, [34.0, 38.0, 14.0, 33.0], MainAlign::Min, CrossAlign::Min));

    let header = frame(&mut doc, copy, "definition-header", 574.0, 160.0, Sizing::Fill, Sizing::Hug);
    lay(&mut doc, header, flow(LayoutMode::Vertical, 16.0, [0.0; 4], MainAlign::Min, CrossAlign::Min));
    text(&mut doc, header, "definition-term", "INGROUP BIAS", SANS, "SemiBold", 38.0, 1.25, 0.0, TextAlignH::Left, false, 400.0);
    text(&mut doc, header, "definition-body", BIAS, SANS, "Regular", 28.0, 1.34, 0.0, TextAlignH::Left, true, 574.0);

    frame(&mut doc, copy, "definition-spacer", 574.0, 40.0, Sizing::Fill, Sizing::Fill);
    let footer = frame(&mut doc, copy, "definition-footer", 574.0, 62.0, Sizing::Fill, Sizing::Hug);
    lay(&mut doc, footer, flow(LayoutMode::Vertical, 12.0, [0.0; 4], MainAlign::Min, CrossAlign::Min));
    text(&mut doc, footer, "definition-source", "APA Dictionary of Psychology", MONO, "Regular", 20.0, 1.3, 0.0, TextAlignH::Left, false, 400.0);
    text(&mut doc, footer, "definition-paraphrased", "(paraphrased)", MONO, "Regular", 20.0, 1.3, 0.0, TextAlignH::Left, false, 200.0);

    let right = frame(&mut doc, stage, "right-column", 960.0, 780.0, Sizing::Fixed, Sizing::Hug);
    lay(&mut doc, right, flow(LayoutMode::Vertical, 40.0, [0.0; 4], MainAlign::Min, CrossAlign::Min));
    let field = frame(&mut doc, right, "apa-field", 960.0, 520.0, Sizing::Fill, Sizing::Fixed);
    let boundary = doc
        .create(field, NodeKind::Ellipse { start: 0.0, sweep: std::f64::consts::TAU, inner: 0.0 }, "group-boundary", 155.0, 35.0, 430.0, 430.0)
        .expect("boundary");
    size_to(&mut doc, boundary, Sizing::Fixed, Sizing::Fixed);

    let tray = frame(&mut doc, right, "allocation-tray", 960.0, 220.0, Sizing::Fill, Sizing::Fixed);
    // Heading line box is 25px (20 × 1.25). Gap 9 puts the panels at tray+44, and the
    // remaining height is 164, matching `AllocationTray`.
    lay(&mut doc, tray, flow(LayoutMode::Vertical, 9.0, [10.0, 12.0, 12.0, 12.0], MainAlign::Min, CrossAlign::Min));
    text(&mut doc, tray, "tray-heading", "DERIVED ILLUSTRATION", MONO, "Medium", 20.0, 1.25, 1.1, TextAlignH::Left, false, 420.0);

    let row = frame(&mut doc, tray, "allocation-row", 936.0, 164.0, Sizing::Fill, Sizing::Fill);
    lay(&mut doc, row, flow(LayoutMode::Horizontal, 10.0, [0.0; 4], MainAlign::Min, CrossAlign::Min));
    resources(&mut doc, row);
    conduct(&mut doc, row);
    behavior(&mut doc, row);
    doc
}

fn resources(doc: &mut Document, row: NodeId) {
    let panel = panel(doc, row, "allocation-panel-1");
    // Padding bottom 7 and a 25px line leave the numbers at y+132 inside the 164px panel.
    lay(doc, panel, flow(LayoutMode::Vertical, 0.0, [9.0, 8.0, 7.0, 8.0], MainAlign::Min, CrossAlign::Min));
    text(doc, panel, "allocation-label-1", "DIVIDING RESOURCES", MONO, "Medium", 19.0, 1.05, 0.0, TextAlignH::Center, true, 288.0);
    frame(doc, panel, "resource-spacer", 288.0, 20.0, Sizing::Fill, Sizing::Fill);
    let numbers = frame(doc, panel, "resource-row", 288.0, 25.0, Sizing::Fill, Sizing::Hug);
    // 14px inset plus the 12px "8" and a 216px gap lands "4" on the scene's x+250.
    lay(doc, numbers, flow(LayoutMode::Horizontal, 0.0, [0.0, 0.0, 0.0, 14.0], MainAlign::Min, CrossAlign::Min));
    text(doc, numbers, "resource-eight", "8", MONO, "Medium", 20.0, 1.25, 0.0, TextAlignH::Left, false, 30.0);
    frame(doc, numbers, "resource-gap", 216.0, 25.0, Sizing::Fixed, Sizing::Fixed);
    text(doc, numbers, "resource-four", "4", MONO, "Medium", 20.0, 1.25, 0.0, TextAlignH::Left, false, 30.0);
}

fn conduct(doc: &mut Document, row: NodeId) {
    let panel = panel(doc, row, "allocation-panel-2");
    // A fill spacer keeps the 70px frames on the bottom. 164 − 16 − 70 = 78, the scene's y.
    lay(doc, panel, flow(LayoutMode::Vertical, 0.0, [9.0, 8.0, 16.0, 8.0], MainAlign::Min, CrossAlign::Min));
    text(doc, panel, "allocation-label-2", "JUDGING CONDUCT", MONO, "Medium", 19.0, 1.05, 0.0, TextAlignH::Center, true, 288.0);
    frame(doc, panel, "conduct-spacer", 288.0, 20.0, Sizing::Fill, Sizing::Fill);
    let frames = frame(doc, panel, "conduct-row", 288.0, 70.0, Sizing::Fill, Sizing::Fixed);
    lay(doc, frames, flow(LayoutMode::Horizontal, 32.0, [0.0; 4], MainAlign::Center, CrossAlign::Center));
    mark(doc, frames, "conduct-warm", "conduct-warm-mark");
    mark(doc, frames, "conduct-cool", "conduct-cool-mark");
}

fn behavior(doc: &mut Document, row: NodeId) {
    let panel = panel(doc, row, "allocation-panel-3");
    lay(doc, panel, flow(LayoutMode::Vertical, 0.0, [9.0, 8.0, 8.0, 8.0], MainAlign::Min, CrossAlign::Min));
    text(doc, panel, "allocation-label-3", "EXPLAINING BEHAVIOR", MONO, "Medium", 19.0, 1.05, 0.0, TextAlignH::Center, true, 288.0);
    // 9 + 20px label + 51 = 80, then a 25px line and a 13px gap land the second line at 118.
    frame(doc, panel, "behavior-spacer", 288.0, 51.0, Sizing::Fill, Sizing::Fixed);
    let stack = frame(doc, panel, "behavior-stack", 288.0, 63.0, Sizing::Fill, Sizing::Hug);
    lay(doc, stack, flow(LayoutMode::Vertical, 13.0, [0.0, 0.0, 0.0, 14.0], MainAlign::Min, CrossAlign::Min));
    text(doc, stack, "behavior-favorable", "\u{2192} favorable", MONO, "Regular", 20.0, 1.25, 0.0, TextAlignH::Left, false, 250.0);
    text(doc, stack, "behavior-situational", "\u{2192} situational", MONO, "Regular", 20.0, 1.25, 0.0, TextAlignH::Left, false, 250.0);
}

fn panel(doc: &mut Document, row: NodeId, name: &str) -> NodeId {
    frame(doc, row, name, 304.0, 164.0, Sizing::Fixed, Sizing::Fill)
}

fn mark(doc: &mut Document, row: NodeId, frame_name: &str, text_name: &str) {
    let item = frame(doc, row, frame_name, 118.0, 70.0, Sizing::Fixed, Sizing::Fixed);
    lay(doc, item, flow(LayoutMode::Vertical, 0.0, [0.0; 4], MainAlign::Center, CrossAlign::Center));
    text(doc, item, text_name, "\u{2261}", MONO, "Medium", 28.0, 1.25, 0.0, TextAlignH::Center, false, 40.0);
}

fn frame(doc: &mut Document, parent: NodeId, name: &str, w: f64, h: f64, horizontal: Sizing, vertical: Sizing) -> NodeId {
    let id = doc.create(parent, NodeKind::Frame, name, 0.0, 0.0, w, h).expect("frame");
    size_to(doc, id, horizontal, vertical);
    id
}

fn lay(doc: &mut Document, id: NodeId, layout: AutoLayout) {
    doc.get_mut(id).expect("node").layout = Some(layout);
}

fn size_to(doc: &mut Document, id: NodeId, horizontal: Sizing, vertical: Sizing) {
    let node = doc.get_mut(id).expect("node");
    node.child_layout.sizing_h = horizontal;
    node.child_layout.sizing_v = vertical;
}

fn flow(mode: LayoutMode, gap: f64, padding: [f64; 4], main: MainAlign, cross: CrossAlign) -> AutoLayout {
    AutoLayout { mode, gap, padding, main_align: main, cross_align: cross, strokes_included: false, ..AutoLayout::default() }
}

fn text(
    doc: &mut Document,
    parent: NodeId,
    name: &str,
    characters: &str,
    family: &str,
    style: &str,
    size: f64,
    line_height: f64,
    tracking_px: f64,
    align: TextAlignH,
    fill_width: bool,
    w: f64,
) {
    let mut data = TextData::new(characters);
    data.style = TextStyle {
        font_family: family.into(),
        font_style: style.into(),
        font_size: size,
        line_height: LineHeight::Percent(line_height * 100.0),
        letter_spacing: if tracking_px == 0.0 { LetterSpacing::default() } else { LetterSpacing::Pixels(tracking_px) },
        ..TextStyle::default()
    };
    data.align_h = align;
    data.auto_resize = if fill_width { TextAutoResize::Height } else { TextAutoResize::WidthAndHeight };
    let id = doc.create(parent, NodeKind::Text(data), name, 0.0, 0.0, w, size * line_height).expect("text");
    size_to(doc, id, if fill_width { Sizing::Fill } else { Sizing::Hug }, Sizing::Hug);
}
