use scriptcast_ui::project::*;
use serde_json::json;

fn fixture() -> Report {
    Report::parse(include_bytes!("fixtures/phase_l/show.json")).unwrap()
}
#[test]
fn real_show_schema_preserves_readings_rates_and_problems() {
    let report = fixture();
    assert_eq!(report.schema, 1);
    assert_eq!(report.voices.speakers.len(), 18);
    assert_eq!(report.clips.clips.len(), 32);
    assert_eq!(report.pronunciations.entries.len(), 5);
    assert!(
        report
            .pronunciations
            .entries
            .iter()
            .all(|e| e["readings"].is_object())
    );
    assert!(
        report
            .speaker_rates
            .rates
            .values()
            .any(|rate| rate.is_object())
    );
    assert!(report.toml.problems.is_empty());
    for key in TOML_KEYS {
        assert!(report.toml.values.contains_key(key));
    }
    assert!(
        report.voices.speakers.iter().all(|speaker| {
            speaker["fields"].is_object() && speaker["fields"]["name"].is_string()
        })
    );
    assert!(
        report
            .clips
            .clips
            .iter()
            .all(|clip| { clip["fields"].is_object() && clip["fields"].get("source").is_some() })
    );
    assert!(
        report
            .pronunciations
            .entries
            .iter()
            .all(|entry| { entry["fields"].is_object() && entry["fields"] == entry["readings"] })
    );
}
#[test]
fn changed_fields_include_only_actual_changes() {
    let before = json!({"name":"Aisha", "engine":"omnivoice", "speed":1.0});
    let after = json!({"name":"Aisha", "engine":"omnivoice", "speed":1.1});
    assert_eq!(
        changed_fields(before.as_object().unwrap(), after.as_object().unwrap()),
        json!({"speed":1.1}).as_object().unwrap().clone()
    );
}
#[test]
fn small_list_ops_keep_untouched_entries() {
    let report = fixture();
    let mut toml = report.toml.clone();
    toml.values.insert("tail_ms".into(), json!(900));
    assert_eq!(
        toml_ops(&report.toml, &toml),
        vec![json!({"file":"toml","op":"set","key":"tail_ms","value":900})]
    );
    let mut pronunciations = report.pronunciations.clone();
    let removed = pronunciations.entries.remove(0);
    assert_eq!(
        pronunciation_ops(&report.pronunciations, &pronunciations),
        vec![json!({"file":"pronunciations","op":"delete_entry","word":removed["word"]})]
    );
    let mut hearings = report.accepted_hearings.clone();
    let removed = hearings.entries.remove(0);
    assert_eq!(
        hearing_ops(&report.accepted_hearings, &hearings),
        vec![
            json!({"file":"accepted_hearings","op":"delete","word":removed["word"],"heard":removed["heard"][0]})
        ]
    );
}
#[test]
fn tool_errors_are_actionable() {
    assert_eq!(
        tool_result(Some(3), b"", b"voices: sha256 mismatch", "/python"),
        Err(ToolError::Stale)
    );
    assert_eq!(
        tool_result(
            Some(1),
            b"",
            b"op[0]: invalid\nvoices: missing text\n",
            "/python"
        ),
        Err(ToolError::Validation(vec![
            "op[0]: invalid".into(),
            "voices: missing text".into()
        ]))
    );
    let error = tool_result(
        Some(1),
        b"",
        b"ModuleNotFoundError: No module named 'ruamel'",
        "/venv/bin/python",
    )
    .unwrap_err();
    assert_eq!(
        error.messages(),
        vec!["scriptcast-project needs ruamel.yaml in /venv/bin/python"]
    );
}
#[test]
fn engine_validation_and_removal_are_live() {
    let mut fields = json!({"engine":"omnivoice", "reference_audio":"a.wav", "reference_text":""})
        .as_object()
        .unwrap()
        .clone();
    assert!(required_errors(&fields).contains_key("reference_text"));
    fields.insert("reference_text".into(), json!("A reference."));
    assert!(required_errors(&fields).is_empty());
    fields.insert("reference_audio_expressive".into(), json!("b.wav"));
    assert!(required_errors(&fields).contains_key("reference_text_expressive"));
    assert_eq!(
        removed_engine_fields(&fields, "archive"),
        vec![
            "reference_audio",
            "reference_audio_expressive",
            "reference_text"
        ]
    );
    for engine in [
        "kokoro",
        "mlx_kokoro",
        "mlx_chatterbox",
        "elevenlabs",
        "omnivoice",
        "archive",
    ] {
        assert!(engine_fields(engine).contains(&"engine"));
    }
}
#[test]
fn tone_engine_has_only_its_shipped_fields_and_requires_instruct() {
    assert_eq!(
        engine_fields("tone"),
        &["name", "engine", "instruct", "character_profile"]
    );
    let mut fields = json!({
        "name": "Narrator",
        "engine": "tone",
        "instruct": "   ",
        "character_profile": "A measured documentary narrator."
    })
    .as_object()
    .unwrap()
    .clone();
    assert_eq!(
        required_errors(&fields).keys().collect::<Vec<_>>(),
        vec!["instruct"]
    );
    fields.insert("instruct".into(), json!("Warm, precise, and unhurried."));
    assert!(required_errors(&fields).is_empty());
}
#[test]
fn interrupted_status_takes_precedence() {
    assert_eq!(episode_status(true, true), EpisodeStatus::Interrupted);
    assert_eq!(episode_status(true, false), EpisodeStatus::Rendered);
    assert_eq!(episode_status(false, false), EpisodeStatus::NotRendered);
}

#[test]
fn discovery_reads_rendered_outputs_and_unrendered_scripts() {
    let temp = std::env::temp_dir().join(format!("scriptcast-l-discovery-{}", std::process::id()));
    std::fs::create_dir_all(temp.join("outputs/one")).unwrap();
    std::fs::create_dir_all(temp.join("scripts")).unwrap();
    let source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small");
    for name in ["episode_manifest.json", "stress.json"] {
        if source.join(name).is_file() {
            std::fs::copy(source.join(name), temp.join("outputs/one").join(name)).unwrap();
        }
    }
    std::fs::write(
        temp.join("scripts/new.md"),
        "Aisha (00:00)\nHello.\nAisha (00:01)\nAgain.\n",
    )
    .unwrap();
    std::fs::write(temp.join("scripts/other.md"), "Bob (00:00)\nHi.\n").unwrap();
    let manifest = temp.join("outputs/one/episode_manifest.json");
    let text = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(
        manifest,
        text.replace("\"script.md\"", "\"scripts/new.md\""),
    )
    .unwrap();
    let mut report = fixture();
    report.root = temp.clone();
    report
        .toml
        .values
        .insert("scripts".into(), "scripts".into());
    let rows = discover_episodes(&report).unwrap();
    assert_eq!(rows.len(), 2);
    let rendered = rows
        .iter()
        .find(|r| r.status == EpisodeStatus::Rendered)
        .unwrap();
    assert!(same_file(&rendered.script, &temp.join("scripts/new.md")));
    assert!(rendered.turns.is_some());
    let other = rows.iter().find(|r| r.id == "other").unwrap();
    assert_eq!(other.status, EpisodeStatus::NotRendered);
    assert!(
        !rows
            .iter()
            .any(|r| r.status == EpisodeStatus::NotRendered && r.id == "new")
    );
    assert_eq!(format_review_date("2026-10-05T01:33:56Z"), "05 Oct 2026");
    let rendered_at = format_rendered("2026-10-05T14:32:20Z");
    assert!(rendered_at.contains("Oct 2026"));
    assert!(rendered_at.contains('\n'));
    std::fs::remove_dir_all(temp).unwrap();
}

#[test]
fn speaker_and_clip_ops_send_the_complete_mapping() {
    let report = fixture();
    let before = report.voices.clone();
    let mut after = before.clone();
    let speaker = after
        .speakers
        .iter_mut()
        .find(|speaker| speaker["id"] == "aisha")
        .unwrap();
    let mut fields = speaker["fields"].as_object().unwrap().clone();
    fields.insert("speed".into(), json!(1.04));
    let reference_text = fields["reference_text"].clone();
    speaker["fields"] = json!(fields);
    let ops = voice_ops(&before, &after).unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["op"], "set_speaker");
    assert_eq!(ops[0]["id"], "aisha");
    assert_eq!(ops[0]["fields"]["speed"], 1.04);
    assert_eq!(
        ops[0]["fields"]["name"],
        before.speakers.iter().find(|s| s["id"] == "aisha").unwrap()["fields"]["name"]
    );
    assert_eq!(ops[0]["fields"]["reference_text"], reference_text);
    assert!(ops[0]["fields"].as_object().unwrap().len() > 2);

    let clips = report.clips.clone();
    let mut edited = clips.clone();
    edited.clips[0]["fields"]["content_note"] = "Kept with the source.".into();
    let ops = clip_ops(&clips, &edited).unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["op"], "set_clip");
    assert_eq!(ops[0]["fields"]["content_note"], "Kept with the source.");
    assert_eq!(
        ops[0]["fields"]["source"],
        clips.clips[0]["fields"]["source"]
    );
}

fn one_added_line(before: &str, after: &str) -> Option<String> {
    let before: Vec<&str> = before.lines().collect();
    let after: Vec<&str> = after.lines().collect();
    if after.len() != before.len() + 1 {
        return None;
    }
    let mut index = 0;
    let mut added = None;
    for line in &after {
        if index < before.len() && *line == before[index] {
            index += 1;
        } else if added.is_none() {
            added = Some((*line).to_owned());
        } else {
            return None;
        }
    }
    (index == before.len()).then_some(added).flatten()
}

#[test]
fn real_tool_speed_edit_changes_exactly_one_line() {
    use scriptcast_ui::cli::{ExportSource, Resolved};
    use std::{fs, path::Path};
    let python = Path::new("/tmp/scriptcast-l/venv/bin/python");
    if !python.is_file() {
        eprintln!("Skipping real tool test: /tmp/scriptcast-l/venv/bin/python is absent.");
        return;
    }
    let root =
        Path::new("/tmp/scriptcast-l").join(format!("ui-contract-test-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("scriptcast.toml"),
        "# test project\nvoices = \"voices.yaml\"\n",
    )
    .unwrap();
    let source = fixture().voices.path;
    fs::copy(source, root.join("voices.yaml")).unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let resolved = Resolved {
        source: ExportSource::DevCheckout(checkout.clone()),
        program: python.into(),
        prefix: vec!["-m".into(), "scriptcast.tools.project_config".into()],
        path: std::env::var_os("PATH").unwrap_or_default(),
        pythonpath: Some(checkout),
    };
    let snapshot = load(&resolved, &root).unwrap();
    let voices = root.join("voices.yaml");
    let before = fs::read(&voices).unwrap();
    let mut stale = snapshot.clone();
    stale.hashes.insert("voices".into(), "0".repeat(64));
    assert!(matches!(
        edit(
            &resolved,
            &stale,
            "voices",
            &[json!({"file":"voices","op":"rename_speaker","id":"aisha","to":"reader"})]
        ),
        Err(ToolError::Stale)
    ));
    assert_eq!(fs::read(&voices).unwrap(), before);
    let speaker = snapshot
        .report
        .voices
        .speakers
        .iter()
        .find(|speaker| speaker["id"] == "aisha")
        .expect("aisha");
    let mut fields = speaker["fields"].as_object().unwrap().clone();
    fields.insert("speed".into(), json!(1.04));
    let saved = edit(
        &resolved,
        &snapshot,
        "voices",
        &[json!({"file":"voices","op":"set_speaker","id":"aisha","fields":fields})],
    )
    .unwrap_or_else(|error| panic!("speed edit failed: {}", error.messages().join("; ")));
    let after = String::from_utf8(fs::read(&voices).unwrap()).unwrap();
    let before_text = String::from_utf8(before).unwrap();
    let added = one_added_line(&before_text, &after)
        .unwrap_or_else(|| panic!("expected one inserted line\n{before_text}\n---\n{after}"));
    assert!(
        added.contains("speed") && added.contains("1.04"),
        "inserted line was {added}"
    );
    assert_eq!(
        saved
            .report
            .voices
            .speakers
            .iter()
            .find(|speaker| speaker["id"] == "aisha")
            .unwrap()["fields"]["speed"],
        1.04
    );
    let empty = root.join("empty-init");
    fs::create_dir_all(&empty).unwrap();
    let created = init_project(&resolved, &empty).unwrap();
    let toml = fs::read_to_string(empty.join("scriptcast.toml")).unwrap();
    assert_eq!(
        toml,
        "# scriptCast project settings. Every key is optional; see the README Configuration table.\n"
    );
    assert!(!created.report.toml.missing);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rename_draft_builds_the_real_rename_operation() {
    let before = fixture().voices;
    let mut after = before.clone();
    after.speakers[0]["original_id"] = after.speakers[0]["id"].clone();
    after.speakers[0]["id"] = "host".into();
    assert_eq!(
        rename_ops(&before, &after).unwrap(),
        vec![
            json!({"file":"voices", "op":"rename_speaker", "id":before.speakers[0]["id"], "to":"host"})
        ]
    );
}

#[test]
fn pronunciation_edits_use_the_real_readings_mapping() {
    let before = fixture().pronunciations;
    let mut after = before.clone();
    after.entries[0]["readings"]["noun"] = "REC-ord".into();
    let ops = pronunciation_ops(&before, &after);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["op"], "set_entry");
    assert_eq!(ops[0]["readings"]["noun"], "REC-ord");
    assert_eq!(
        ops[0]["readings"]["default"],
        before.entries[0]["readings"]["default"]
    );
}
