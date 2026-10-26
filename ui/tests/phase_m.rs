use scriptcast_ui::phase_m::*;
use serde_json::{Map, Value, json};

fn fixture(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_m")
        .join(name);
    std::fs::read(path).unwrap()
}

#[test]
fn word_range_shift_click_and_handle_snap() {
    let words: WordsJson = parse_tool(&fixture("words.json")).unwrap();
    let span = WordSpan {
        anchor: 0,
        focus: 0,
    };
    let shifted = span.click(2, true);
    assert_eq!(shifted.inclusive(), (0, 2));
    let replaced = shifted.click(1, false);
    assert_eq!(replaced.inclusive(), (1, 1));
    assert_eq!(snap_start_index(&words.words, 1.6, 2), 1);
    assert_eq!(snap_end_index(&words.words, 1.6, 0), 0);
    assert_eq!(snap_end_index(&words.words, 3.9, 0), 2);
    let (start, end) = span_times(&words.words, shifted).unwrap();
    assert_eq!((start, end), (1.0, 4.4));
}

#[test]
fn aisha_words_keep_the_recorded_endpoints() {
    let words: WordsJson = parse_tool(&fixture("words-aisha.json")).unwrap();
    assert_eq!(words.duration, 15.41);
    assert_eq!(words.words[0].i, 0);
    assert_eq!(words.words[0].word, "The");
    assert_eq!((words.words[0].start, words.words[0].end), (0.0, 0.18));
    let last = words.words.last().unwrap();
    assert_eq!(last.i, 44);
    assert_eq!(last.word, "it.");
    assert_eq!((last.start, last.end), (15.12, 15.38));
}

#[test]
fn id_collision_is_exact_and_ignores_blank() {
    let ids = ["speech", "exact"];
    assert!(id_collides(&ids, "speech"));
    assert!(id_collides(&ids, " speech "));
    assert!(!id_collides(&ids, "Speech"));
    assert!(!id_collides(&ids, ""));
    assert!(!id_collides(&ids, "   "));
}

#[test]
fn tool_argv_matches_the_phase_m_contract() {
    assert_eq!(
        clip_words_invocation("/tmp/source.wav").argv(),
        vec![
            "scriptcast-clip",
            "/tmp/source.wav",
            "--words-json",
            "--model",
            "small"
        ]
    );
    let register = clip_register_invocation(&ClipRegister {
        source: "/tmp/source.wav".into(),
        id: "exact".into(),
        start: 1.5,
        end: 3.25,
        speaker: "Narrator".into(),
        citation: "A book".into(),
        url: "https://example.test".into(),
        content_note: "keep".into(),
        pad_ms: 100,
    });
    assert_eq!(
        register.argv(),
        vec![
            "scriptcast-clip",
            "/tmp/source.wav",
            "--id",
            "exact",
            "--start",
            "1.50",
            "--end",
            "3.25",
            "--speaker",
            "Narrator",
            "--citation",
            "A book",
            "--url",
            "https://example.test",
            "--content-note",
            "keep",
            "--pad",
            "0.10",
            "--json"
        ]
    );
    assert_eq!(
        reference_candidates_invocation("/tmp/raw.wav", 9.0).argv(),
        vec![
            "scriptcast-reference",
            "/tmp/raw.wav",
            "--candidates-json",
            "--seconds",
            "9.00"
        ]
    );
    assert_eq!(
        reference_write_invocation("/tmp/raw.wav", "host", 0.0, 9.5, 0.92).argv(),
        vec![
            "scriptcast-reference",
            "/tmp/raw.wav",
            "--name",
            "host",
            "--start",
            "0.00",
            "--end",
            "9.50",
            "--peak",
            "0.92",
            "--json"
        ]
    );
    assert_eq!(
        audition_invocation(
            &["/tmp/a.wav".into(), "/tmp/b.wav".into()],
            "guest",
            "mlx_chatterbox",
            "/tmp/keep",
            None
        )
        .argv(),
        vec![
            "scriptcast-audition",
            "/tmp/a.wav",
            "/tmp/b.wav",
            "--speaker",
            "guest",
            "--engine",
            "mlx_chatterbox",
            "--json",
            "--keep",
            "/tmp/keep"
        ]
    );
    assert_eq!(
        calibrate_invocation("record", "default", "host", "reckerd", "/tmp/keep", true).argv(),
        vec![
            "scriptcast-calibrate",
            "record",
            "--reading",
            "default",
            "--speaker",
            "host",
            "--candidates",
            "reckerd",
            "--json",
            "--keep",
            "/tmp/keep",
            "--write"
        ]
    );
    assert_eq!(
        outliers_invocation("/tmp/episode", "/tmp/episode.md").argv(),
        vec![
            "scriptcast-outliers",
            "/tmp/episode",
            "--transcript",
            "/tmp/episode.md",
            "--json"
        ]
    );
    assert_eq!(
        stress_invocation("/tmp/take.wav", "historical record").argv(),
        vec![
            "scriptcast-stress",
            "/tmp/take.wav",
            "--text",
            "historical record",
            "--json"
        ]
    );
}

#[test]
fn audition_speaker_defaults_to_the_voices_selection() {
    let people = ["emmanuel_theodore", "aisha"];
    assert_eq!(default_audition_speaker(&people, "aisha", "guest"), "aisha");
    assert_eq!(
        default_audition_speaker(&people, "", "guest"),
        "emmanuel_theodore"
    );
    assert_eq!(
        default_audition_speaker(&people, "aisha", "emmanuel_theodore"),
        "emmanuel_theodore"
    );
}

#[test]
fn audition_winner_is_the_first_highest_mean_and_heard_words_diff() {
    let audition: AuditionJson = parse_tool(&fixture("audition.json")).unwrap();
    let means: Vec<f64> = audition.candidates.iter().map(|c| c.mean).collect();
    assert_eq!(winner_index(&means), Some(1));
    assert_eq!(mean_band(0.936), MeanBand::High);
    assert_eq!(mean_band(0.872), MeanBand::Mid);
    assert_eq!(mean_band(0.794), MeanBand::Low);
    assert_eq!(winner_index(&[0.5, 0.5]), Some(0));
    let take = &audition.candidates[0].takes[0];
    let diff = heard_diff(&take.text, &take.heard);
    let changed: Vec<&str> = diff
        .iter()
        .filter(|word| word.differs)
        .map(|word| word.text.as_str())
        .collect();
    assert_eq!(changed, vec!["learn"]);
    assert!(diff.iter().all(|word| word.text != "We" || !word.differs));
}

#[test]
fn outlier_ratio_bands() {
    let report: OutliersJson = parse_tool(&fixture("outliers.json")).unwrap();
    assert_eq!(classify_ratio(report.outliers[0].ratio), RatioClass::Danger);
    assert_eq!(classify_ratio(report.outliers[1].ratio), RatioClass::Within);
    assert_eq!(classify_ratio(report.outliers[2].ratio), RatioClass::Danger);
    assert_eq!(classify_ratio(0.6), RatioClass::Within);
    assert_eq!(classify_ratio(1.0), RatioClass::Within);
}

#[test]
fn stress_card_parses_phoneme_heard() {
    let card: StressJson = parse_tool(&fixture("stress.json")).unwrap();
    assert_eq!(card.heteronyms.len(), 1);
    assert_eq!(card.heteronyms[0].word, "record");
    assert_eq!(card.heteronyms[0].expected, "abc");
    assert_eq!(card.heteronyms[0].heard, "r eh");
    assert!(!card.heteronyms[0].ok);
    let error = parse_tool::<StressJson>(br#"{"error":"missing wav"}"#);
    assert_eq!(error.unwrap_err(), "missing wav");
}

#[test]
fn use_for_speaker_changes_only_the_voices_draft() {
    let mut fields = Map::new();
    fields.insert("engine".into(), json!("omnivoice"));
    fields.insert("name".into(), json!("Aisha"));
    fields.insert("reference_audio".into(), json!("old.wav"));
    fields.insert("reference_text".into(), json!("old words"));
    fields.insert("speed".into(), json!(1.0));
    let before = fields.clone();
    assign_reference(
        &mut fields,
        "references/aisha_neutral.wav",
        "Trust is built.",
    );
    assert_eq!(
        fields.get("reference_audio").and_then(Value::as_str),
        Some("references/aisha_neutral.wav")
    );
    assert_eq!(
        fields.get("reference_text").and_then(Value::as_str),
        Some("Trust is built.")
    );
    for key in ["engine", "name", "speed"] {
        assert_eq!(fields.get(key), before.get(key));
    }
    assert_eq!(fields.len(), before.len());
}

#[test]
fn candidates_fixture_matches_the_python_shape() {
    let parsed: CandidatesJson = parse_tool(&fixture("candidates.json")).unwrap();
    assert_eq!(parsed.candidates[0].rank, 1);
    assert_eq!(parsed.candidates[0].score, 0.6667);
    assert_eq!(parsed.candidates[0].text, "Alpha sentence. Beta sentence.");
}
