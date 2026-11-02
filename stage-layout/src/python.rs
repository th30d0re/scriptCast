//! `stage_layout.layout(doc_json, font_paths)` for scriptCast's tools.

use pyo3::prelude::*;

use crate::FontBytes;

#[pyfunction]
#[pyo3(name = "layout")]
fn layout_py(doc_json: &str, font_paths: Vec<String>) -> PyResult<String> {
    let mut fonts = Vec::with_capacity(font_paths.len());
    for path in &font_paths {
        let bytes = std::fs::read(path).map_err(|err| {
            PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("{path}: {err}"))
        })?;
        fonts.push(FontBytes::new(bytes));
    }
    Ok(crate::layout(doc_json, &fonts))
}

#[pymodule]
fn stage_layout(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(layout_py, module)?)?;
    Ok(())
}
