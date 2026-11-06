mod common;
use scriptcast_ui::mix::{self, Gain, Mix, Overlay};
use scriptcast_ui::{
    cli::{ExportSource, Resolved},
    runner::{JobSpec, Preset},
};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mix.yaml")
}

#[test]
fn loads_fixture_and_missing_file_is_empty() {
    let loaded = mix::load(&fixture()).unwrap();
    assert_eq!(loaded.overlays.len(), 1);
    assert_eq!(loaded.overlays[0].gain_db, -22.0);
    assert_eq!(loaded.gains[0].turn_equals.as_deref(), Some("Interesting."));
    assert_eq!(
        mix::load(&fixture().with_file_name("absent-mix.yaml")).unwrap(),
        Mix::default()
    );
}

#[test]
fn saves_yaml_the_python_tool_reads() {
    let dir = std::env::temp_dir().join(format!("scriptcast-mix-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mix.yaml");
    let expected = mix::load(&fixture()).unwrap();
    mix::save(&path, &expected).unwrap();
    assert_eq!(mix::load(&path).unwrap(), expected);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("overlays:"));
    assert!(text.contains("turn_contains:"));
    assert!(!text.contains("null"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn gains_are_bounded_on_load_and_save() {
    let dir = std::env::temp_dir().join(format!("scriptcast-mix-bounds-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mix.yaml");
    std::fs::write(&path, "overlays:\n  - file: x.wav\n    turn_contains: hello\n    after_word: hello\n    gain_db: -90\ngains:\n  - turn_contains: hello\n    gain_db: 30\n").unwrap();
    let loaded = mix::load(&path).unwrap();
    assert_eq!(loaded.overlays[0].gain_db, mix::OVERLAY_GAIN_MIN);
    assert_eq!(loaded.gains[0].gain_db, mix::TURN_GAIN_MAX);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unmatched_word_uses_tool_sentence_and_normalization() {
    let overlay = Overlay {
        file: "nice.wav".into(),
        turn_equals: None,
        turn_contains: Some("signed".into()),
        after_word: "^(69|sixtynine)$".into(),
        delay_ms: 0,
        gain_db: -22.0,
    };
    assert_eq!(
        mix::unmatched_word_error(&overlay, 42, "already signed the note"),
        Some("no recognised word matches '^(69|sixtynine)$' in turn 42".into())
    );
    assert_eq!(
        mix::unmatched_word_error(&overlay, 42, "already sixty-nine days"),
        None
    );
}

#[test]
fn public_bounds_are_the_requested_ranges() {
    assert_eq!(
        (
            mix::OVERLAY_GAIN_MIN,
            mix::OVERLAY_GAIN_MAX,
            mix::OVERLAY_GAIN_DEFAULT
        ),
        (-40.0, 0.0, -22.0)
    );
    assert_eq!((mix::TURN_GAIN_MIN, mix::TURN_GAIN_MAX), (-12.0, 12.0));
    let _ = Gain {
        turn_equals: None,
        turn_contains: Some("x".into()),
        gain_db: 0.0,
    };
}

#[test]
fn render_preset_requires_documented_asr_and_builds_exact_mix_argv() {
    let scratch = common::Scratch::new();
    let episode = scratch.fixture();
    let command = Resolved {
        program: PathBuf::from("/fake/scriptcast-mix"),
        source: ExportSource::Folder,
        prefix: vec![],
        path: "/usr/bin:/bin".into(),
        pythonpath: None,
    };
    let commands = [("scriptcast-mix".into(), Ok(command))]
        .into_iter()
        .collect();
    let missing = JobSpec::from_preset(&Preset::Mix, &episode, &commands).unwrap_err();
    assert!(missing.contains("Recognised words are missing"));
    let asr = mix::asr_path(&episode.dir, &episode.manifest.episode_id);
    std::fs::write(&asr, "[]").unwrap();
    let spec = JobSpec::from_preset(&Preset::Mix, &episode, &commands).unwrap();
    let args: Vec<_> = spec
        .args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        args,
        vec![
            episode.dir.display().to_string(),
            "--script".into(),
            episode.script_path.as_ref().unwrap().display().to_string(),
            "--config".into(),
            episode
                .project_root
                .as_ref()
                .unwrap()
                .join("mix.yaml")
                .display()
                .to_string(),
            "--out".into(),
            mix::mixed_path(&episode.dir, &episode.manifest.episode_id)
                .display()
                .to_string(),
            "--asr".into(),
            asr.display().to_string()
        ]
    );
}
