//! Text measurement for [`figcraft_layout::Measure`].
//!
//! Copied from FigCraft `crates/engine/src/measure.rs` at tag v0.14.0.
//! `stage-layout` does not depend on `figcraft-engine`.

use figcraft_doc::{Document, NodeId, NodeKind, TextAutoResize};

pub struct TextMeasure;

impl figcraft_layout::Measure for TextMeasure {
    fn text_height(&self, doc: &Document, id: NodeId, width: f64) -> Option<f64> {
        let n = doc.node(id)?;
        let NodeKind::Text(t) = &n.kind else { return None };
        let mut t = t.clone();
        if t.auto_resize == TextAutoResize::WidthAndHeight {
            t.auto_resize = TextAutoResize::Height;
        }
        Some(figcraft_text::layout(&t, Some(width), None).height.max(1.0))
    }

    fn text_size(&self, doc: &Document, id: NodeId) -> Option<(f64, f64)> {
        let n = doc.node(id)?;
        let NodeKind::Text(t) = &n.kind else { return None };
        if t.auto_resize == TextAutoResize::WidthAndHeight {
            return figcraft_text::auto_size(n);
        }
        let mut t = t.clone();
        t.auto_resize = TextAutoResize::WidthAndHeight;
        let laid = figcraft_text::layout_cached(&t, None, None);
        Some((laid.width.ceil().max(1.0), laid.height.max(1.0)))
    }

    fn text_baseline(&self, doc: &Document, id: NodeId) -> Option<f64> {
        let n = doc.node(id)?;
        let (laid, dy) = figcraft_text::layout_node(n)?;
        laid.lines.first().map(|line| line.baseline + dy).filter(|b| b.is_finite())
    }
}
