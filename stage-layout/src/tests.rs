use std::collections::BTreeMap;

use figcraft_doc::{Document, NodeKind, TextAutoResize};

use crate::{apa_document_json, layout, FontBytes};

fn value(doc_json: &str) -> serde_json::Value {
    let out = layout(doc_json, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(parsed.get("error").is_none(), "{out}");
    parsed
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1.0, "{actual} != {expected}");
}

fn box_of(doc: &serde_json::Value, id: &str) -> (f64, f64, f64, f64) {
    let item = doc.get(id).unwrap_or_else(|| panic!("missing {id}"));
    (
        item["x"].as_f64().expect("x"),
        item["y"].as_f64().expect("y"),
        item["w"].as_f64().expect("w"),
        item["h"].as_f64().expect("h"),
    )
}

fn lines_of<'a>(doc: &'a serde_json::Value, id: &str) -> Vec<&'a str> {
    doc[id]["lines"].as_array().unwrap_or_else(|| panic!("{id} has no lines")).iter().map(|line| line["text"].as_str().expect("text")).collect()
}

#[test]
fn bundled_fonts_register_plex_families() {
    crate::fonts::register(&[]);
    let sans = figcraft_text::family_styles("IBM Plex Sans");
    let mono = figcraft_text::family_styles("IBM Plex Mono");
    assert!(sans.iter().any(|style| style == "Regular"), "{sans:?}");
    assert!(sans.iter().any(|style| style == "Medium"), "{sans:?}");
    assert!(sans.iter().any(|style| style == "SemiBold"), "{sans:?}");
    assert!(mono.iter().any(|style| style == "Regular"), "{mono:?}");
    assert!(mono.iter().any(|style| style == "Medium"), "{mono:?}");
}

#[test]
fn garbage_font_bytes_do_not_replace_the_bundle() {
    let mut doc = Document::new("t");
    let page = doc.pages[0];
    let frame = doc.create(page, figcraft_doc::NodeKind::Frame, "card", 0.0, 0.0, 300.0, 80.0).unwrap();
    doc.get_mut(frame).unwrap().layout = Some(figcraft_doc::AutoLayout {
        mode: figcraft_doc::LayoutMode::Vertical,
        padding: [0.0; 4],
        gap: 0.0,
        strokes_included: false,
        ..figcraft_doc::AutoLayout::default()
    });
    let mut text = figcraft_doc::TextData::new("Hello");
    text.style.font_family = "IBM Plex Sans".into();
    text.style.font_style = "Regular".into();
    text.style.font_size = 32.0;
    text.auto_resize = TextAutoResize::WidthAndHeight;
    let id = doc.create(frame, NodeKind::Text(text), "greeting", 0.0, 0.0, 10.0, 10.0).unwrap();
    doc.get_mut(id).unwrap().child_layout.sizing_h = figcraft_doc::Sizing::Hug;
    doc.get_mut(id).unwrap().child_layout.sizing_v = figcraft_doc::Sizing::Hug;
    let json = serde_json::to_string(&doc).unwrap();
    let plain = layout(&json, &[]);
    let with_junk = layout(&json, &[FontBytes::new(b"wOFF not a font".to_vec())]);
    assert_eq!(plain, with_junk);
    let parsed = value(&json);
    assert_eq!(lines_of(&parsed, "greeting"), ["Hello"]);
    assert!(parsed["greeting"]["lines"][0]["w"].as_f64().unwrap() > 40.0);
}

#[test]
fn apa_names_are_unique_and_regions_match_the_scene() {
    let built: Document = serde_json::from_str(&apa_document_json()).unwrap();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for node in built.nodes() {
        if !node.name.is_empty() {
            *counts.entry(node.name.as_str()).or_insert(0) += 1;
        }
    }
    let dups: Vec<_> = counts.iter().filter(|(_, count)| **count > 1).collect();
    assert!(dups.is_empty(), "{dups:?}");

    let doc = value(&apa_document_json());
    let (x, y, w, h) = box_of(&doc, "definition-card");
    close(x, 110.0);
    close(y, 190.0);
    close(w, 650.0);
    close(h, 400.0);
    let (x, y, w, h) = box_of(&doc, "apa-field");
    close(x, 830.0);
    close(y, 130.0);
    close(w, 960.0);
    close(h, 520.0);
    let (x, y, w, h) = box_of(&doc, "allocation-tray");
    close(x, 830.0);
    close(y, 690.0);
    close(w, 960.0);
    close(h, 220.0);
    let (x, y, w, h) = box_of(&doc, "group-boundary");
    close(x, 985.0);
    close(y, 165.0);
    close(w, 430.0);
    close(h, 430.0);
    let (x, y, _, _) = box_of(&doc, "definition-source");
    close(x, 148.0);
    close(y, 512.0);
    let (x, y, _, _) = box_of(&doc, "definition-paraphrased");
    close(x, 148.0);
    close(y, 550.0);
    let (x, y, _, _) = box_of(&doc, "allocation-panel-1");
    close(x, 842.0);
    close(y, 734.0);
    let (x, y, _, _) = box_of(&doc, "resource-four");
    close(x, 1092.0);
    close(y, 866.0);
    let (x, y, _, _) = box_of(&doc, "behavior-favorable");
    close(x, 1492.0);
    close(y, 814.0);
    let (x, y, _, _) = box_of(&doc, "conduct-warm");
    close(x, 1174.0);
    close(y, 812.0);

    assert_eq!(lines_of(&doc, "definition-term"), ["INGROUP BIAS"]);
    assert_eq!(lines_of(&doc, "tray-heading"), ["DERIVED ILLUSTRATION"]);
    let body = lines_of(&doc, "definition-body");
    assert!(body.len() >= 2, "{body:?}");
    let source = "the tendency to favor one\u{2019}s own group and its members when dividing resources, judging conduct, and explaining behavior";
    assert_eq!(body.join(" ").split_whitespace().collect::<Vec<_>>(), source.split_whitespace().collect::<Vec<_>>());
    for line in &body {
        assert!(!line.ends_with('-'), "hyphenated break: {line}");
    }
}

#[test]
fn fixture_file_matches_the_builder() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/apa_t21.json");
    let on_disk: Document = serde_json::from_str(&std::fs::read_to_string(path).expect("fixtures/apa_t21.json")).expect("fixture json");
    let built: Document = serde_json::from_str(&apa_document_json()).unwrap();
    assert_eq!(on_disk, built);
}
