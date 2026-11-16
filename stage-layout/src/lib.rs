//! Layout a FigCraft document and return one box per named node.
//!
//! Coordinates are page space: origin at the top-left, y down, in CSS pixels.
//! A text node's `lines` are the breaks FigCraft computed. Each line's `x`/`y`
//! is the top-left of that line's glyphs, in the same page space. Stage paints
//! one `white-space: pre` span per line and does not wrap again.

mod fonts;
mod measure;
mod scene;

#[cfg(feature = "python")]
mod python;

#[cfg(target_arch = "wasm32")]
mod wasm_api;

use std::collections::BTreeMap;

use figcraft_doc::{Document, NodeId, NodeKind};
use figcraft_doc::geom::Point;

pub use fonts::FontBytes;
pub use scene::apa_document_json;

/// Lay out `doc_json` (a FigCraft [`Document`](figcraft_doc::Document)) and return JSON
/// `{ element_id: {x, y, w, h, lines?: [{text, x, y, w}]} }`.
///
/// `element_id` is the node's name. Names should be unique; if two nodes share one,
/// the higher id wins. Bundled IBM Plex Sans (Regular, Medium, SemiBold) and IBM Plex
/// Mono (Regular, Medium) are always registered. `fonts` are extra TTF/OTF files.
pub fn layout(doc_json: &str, fonts: &[FontBytes]) -> String {
    match layout_value(doc_json, fonts) {
        Ok(value) => value.to_string(),
        Err(err) => serde_json::json!({ "error": err }).to_string(),
    }
}

fn layout_value(doc_json: &str, fonts: &[FontBytes]) -> Result<serde_json::Value, String> {
    fonts::register(fonts);
    let mut doc: Document = serde_json::from_str(doc_json).map_err(|err| err.to_string())?;
    let measure = measure::TextMeasure;
    let pages = doc.pages.clone();
    for page in pages {
        figcraft_layout::layout_page(&mut doc, page, &measure);
    }
    Ok(serde_json::Value::Object(boxes(&doc)))
}

fn boxes(doc: &Document) -> serde_json::Map<String, serde_json::Value> {
    let mut named: BTreeMap<String, (NodeId, serde_json::Map<String, serde_json::Value>)> = BTreeMap::new();
    for id in doc.ids() {
        let Some(node) = doc.node(id) else { continue };
        if node.name.is_empty() {
            continue;
        }
        let Some(bounds) = doc.world_bounds(id) else { continue };
        let mut entry = serde_json::Map::new();
        entry.insert("x".into(), num(bounds.x0));
        entry.insert("y".into(), num(bounds.y0));
        entry.insert("w".into(), num(bounds.width()));
        entry.insert("h".into(), num(bounds.height()));
        if let Some(lines) = line_boxes(doc, id) {
            if !lines.is_empty() {
                entry.insert("lines".into(), serde_json::Value::Array(lines));
            }
        }
        named.insert(node.name.clone(), (id, entry));
    }
    named.into_iter().map(|(name, (_id, entry))| (name, serde_json::Value::Object(entry))).collect()
}

/// Visible lines of a text node. `char_start`/`char_end` index Unicode scalars, not bytes.
fn line_boxes(doc: &Document, id: NodeId) -> Option<Vec<serde_json::Value>> {
    let node = doc.node(id)?;
    let NodeKind::Text(text) = &node.kind else { return None };
    let (laid, dy) = figcraft_text::layout_node(node)?;
    let transform = doc.world_transform(id);
    let chars: Vec<char> = text.characters.chars().collect();
    let mut lines = Vec::new();
    for line in &laid.lines {
        let start = line.char_start.min(chars.len());
        let end = line.char_end.min(chars.len()).max(start);
        let raw: String = chars[start..end].iter().collect();
        let shown = raw.trim_end_matches(['\n', '\r', '\u{2028}', ' ', '\t']);
        if shown.is_empty() {
            continue;
        }
        let origin = transform * Point::new(line.x, line.top + dy);
        lines.push(serde_json::json!({
            "text": shown,
            "x": round2(origin.x),
            "y": round2(origin.y),
            "w": round2(line.width),
        }));
    }
    Some(lines)
}

fn num(value: f64) -> serde_json::Value {
    serde_json::json!(round2(value))
}

fn round2(value: f64) -> f64 {
    if value.is_finite() { (value * 100.0).round() / 100.0 } else { 0.0 }
}

#[cfg(test)]
mod tests;
