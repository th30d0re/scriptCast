fn main() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let doc = stage_layout::apa_document_json();
    let fixtures = root.join("fixtures");
    std::fs::create_dir_all(&fixtures).expect("fixtures dir");
    std::fs::write(fixtures.join("apa_t21.json"), doc.as_bytes()).expect("write document");
    let laid = stage_layout::layout(&doc, &[]);
    let value: serde_json::Value = serde_json::from_str(&laid).expect("layout json");
    if value.get("error").is_some() {
        eprintln!("{laid}");
        std::process::exit(1);
    }
    let pretty = serde_json::to_string_pretty(&value).expect("pretty");
    std::fs::write(fixtures.join("apa_t21.layout.json"), pretty.as_bytes()).expect("write layout");
    std::fs::write(root.join("episode/layout.json"), pretty.as_bytes()).expect("write episode layout");
    let body = &value["definition-body"];
    println!("elements {}", value.as_object().map(|o| o.len()).unwrap_or(0));
    println!("definition-body {}x{} at {},{}", body["w"], body["h"], body["x"], body["y"]);
    if let Some(lines) = body["lines"].as_array() {
        for line in lines {
            println!("  {}", line["text"].as_str().unwrap_or(""));
        }
    }
}
