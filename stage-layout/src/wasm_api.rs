//! `layout(docJson, fonts?)` for Stage. `fonts` is an array of `Uint8Array` (TTF/OTF), or omitted.

use wasm_bindgen::prelude::*;

use crate::FontBytes;

#[wasm_bindgen(js_name = layout)]
pub fn layout_wasm(doc_json: &str, fonts: Option<JsValue>) -> String {
    crate::layout(doc_json, &font_list(fonts))
}

fn font_list(fonts: Option<JsValue>) -> Vec<FontBytes> {
    let Some(value) = fonts else { return Vec::new() };
    if value.is_null() || value.is_undefined() {
        return Vec::new();
    }
    if let Some(bytes) = value.dyn_ref::<js_sys::Uint8Array>() {
        return vec![FontBytes::new(bytes.to_vec())];
    }
    let Some(array) = value.dyn_ref::<js_sys::Array>() else { return Vec::new() };
    let mut out = Vec::new();
    for item in array.iter() {
        if let Some(bytes) = item.dyn_ref::<js_sys::Uint8Array>() {
            out.push(FontBytes::new(bytes.to_vec()));
        }
    }
    out
}
