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
    let mut report = fixture();
    report.root = temp.clone();
    report
        .toml
        .values
        .insert("scripts".into(), "scripts".into());
    let rows = discover_episodes(&report).unwrap();
    assert_eq!(rows.len(), 2);
    let new = rows.iter().find(|r| r.id == "new").unwrap();
    assert_eq!(new.status, EpisodeStatus::NotRendered);
    assert_eq!(new.turns, Some(2));
    assert!(rows.iter().any(|r| r.status == EpisodeStatus::Rendered));
    std::fs::remove_dir_all(temp).unwrap();
}

#[test]
fn real_tool_partial_speed_edit_exposes_contract_conflict_without_writing() {
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
    let before = fs::read(root.join("voices.yaml")).unwrap();
    let result = edit(
        &resolved,
        &snapshot,
        "voices",
        &[json!({"file":"voices","op":"set_speaker","id":"aisha","fields":{"speed":1.04}})],
    );
    assert!(matches!(result, Err(ToolError::Validation(_))));
    assert_eq!(fs::read(root.join("voices.yaml")).unwrap(), before);
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
    assert_eq!(fs::read(root.join("voices.yaml")).unwrap(), before);
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
