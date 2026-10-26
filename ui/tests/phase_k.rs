mod common;
use common::{Scratch, copy_tree};
use scriptcast_ui::{
    cli::{ExportCommand, ExportSource},
    episode::Episode,
    retime, script,
    structure::{EditOp, archive_ready, edit_error},
    tags,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn fixture() -> (Scratch, Episode, ExportCommand) {
    let dir = PathBuf::from("/tmp/scriptcast-k").join(format!(
        "test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &dir,
    );
    fs::write(
        dir.join("scriptcast.toml"),
        "voices = \"voices.yaml\"\nclips = \"clips.yaml\"\n",
    )
    .unwrap();
    fs::write(dir.join("voices.yaml"), concat!(
        "speakers:\n  alice:\n    name: Alice\n    engine: kokoro\n    kokoro_voice: af_heart\n    lang_code: a\n",
        "  bob:\n    name: Bob\n    engine: omnivoice\n    reference_audio: bob.wav\n    reference_text: words\n",
        "  archive:\n    name: Archive\n    engine: archive\n")).unwrap();
    fs::write(
        dir.join("clips.yaml"),
        "clips:\n  sample:\n    source: source.wav\n    start: 0.0\n    end: 1.5\n    citation: Sample citation\n",
    )
    .unwrap();
    fs::write(
        dir.join("script.md"),
        "# preserved\n\nAlice (00:01)\nFirst.\n\nBob (00:05)\nSecond.\n\nAlice (00:09)\nThird.\n",
    )
    .unwrap();
    let mut episode = Episode::load(&dir.join("episode_small")).unwrap();
    episode.script_path = Some(dir.join("script.md"));
    episode.voices_path = Some(dir.join("voices.yaml"));
    episode.project_root = Some(dir.clone());
    let root = std::env::var_os("SCRIPTCAST_K_DEV")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .to_owned()
        });
    let export = ExportCommand {
        source: ExportSource::DevCheckout(root),
        program: std::env::var_os("SCRIPTCAST_K_PYTHON")
            .map(PathBuf::from)
            .unwrap_or("python3".into()),
    };
    (Scratch(dir), episode, export)
}
#[test]
fn operations_serialize_the_exact_contract() {
    let cases = [
        (
            EditOp::Insert {
                position: 1,
                speaker_id: "alice".into(),
                timestamp: None,
                body: "Hi".into(),
            },
            serde_json::json!({"op":"insert","position":1,"speaker_id":"alice","timestamp":null,"body":"Hi"}),
        ),
        (
            EditOp::Delete { index: 2 },
            serde_json::json!({"op":"delete","index":2}),
        ),
        (
            EditOp::Move { index: 2, to: 1 },
            serde_json::json!({"op":"move","index":2,"to":1}),
        ),
        (
            EditOp::SetSpeaker {
                index: 0,
                speaker_id: "bob".into(),
            },
            serde_json::json!({"op":"set_speaker","index":0,"speaker_id":"bob"}),
        ),
    ];
    for (op, json) in cases {
        assert_eq!(
            serde_json::to_value(vec![op]).unwrap(),
            serde_json::json!([json])
        );
    }
    assert_eq!(
        edit_error(Some(3), "stale script"),
        "The script changed on disk. Reload to edit."
    );
    let error = edit_error(Some(1), "op[0]: unknown speaker\nop[1]: empty body\n");
    assert!(error.contains("op[0]: unknown speaker\nop[1]: empty body"));
}
#[test]
fn real_python_structural_round_trips_and_stale_atomic_failure() {
    let (_tmp, episode, export) = fixture();
    let mut loaded = script::load_turns(&export, &episode).unwrap();
    let initial_hash = loaded.script_sha256.clone();
    for (op, expected) in [
        (
            EditOp::Move { index: 0, to: 2 },
            vec!["Second.", "Third.", "First."],
        ),
        (EditOp::Delete { index: 1 }, vec!["Second.", "First."]),
        (
            EditOp::SetSpeaker {
                index: 0,
                speaker_id: "alice".into(),
            },
            vec!["Second.", "First."],
        ),
        (
            EditOp::Insert {
                position: 1,
                speaker_id: "bob".into(),
                timestamp: None,
                body: "New.".into(),
            },
            vec!["Second.", "New.", "First."],
        ),
    ] {
        let before = fs::read(episode.script_path.as_ref().unwrap()).unwrap();
        let (checkpoint, response) =
            script::apply_edits(&export, &episode, &[op], &loaded.script_sha256).unwrap();
        assert_eq!(
            fs::read(checkpoint.dir.join("external/script.md")).unwrap(),
            before
        );
        loaded = script::load_turns(&export, &episode).unwrap();
        assert_eq!(response.script_sha256, loaded.script_sha256);
        assert_eq!(
            loaded
                .turns
                .iter()
                .map(|s| s.body.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let before = fs::read(episode.script_path.as_ref().unwrap()).unwrap();
    assert!(
        script::apply_edits(
            &export,
            &episode,
            &[EditOp::Delete { index: 0 }],
            &initial_hash
        )
        .unwrap_err()
        .contains("changed on disk")
    );
    let error = script::apply_edits(
        &export,
        &episode,
        &[EditOp::SetSpeaker {
            index: 0,
            speaker_id: "missing".into(),
        }],
        &loaded.script_sha256,
    )
    .unwrap_err();
    assert!(error.contains("op[0]"), "{error}");
    assert_eq!(
        before,
        fs::read(episode.script_path.as_ref().unwrap()).unwrap()
    );
}
#[test]
fn local_tags_agree_with_real_python_and_archive_gating() {
    let (_tmp, episode, export) = fixture();
    let registry = script::load_registry(&export, &episode).unwrap();
    assert_eq!(registry.speakers.len(), 3);
    assert_eq!(registry.clips[0].end - registry.clips[0].start, 1.5);
    for body in [
        "[pause:800ms] [beat] [emphasis] [tone:quiet]",
        "[pause:0] [pause:10001ms]",
        "[pause:abc] [unknown] [tone: ]",
        "[clip:sample] [clip:sample]",
        "[clip:] [clip:missing]",
        "café [pause:2] 🙂 [beat]\n[tone:]",
        "[pause:999999999999999999999999999]",
    ] {
        fs::write(
            episode.script_path.as_ref().unwrap(),
            format!("Alice (00:00)\n{body}\n"),
        )
        .unwrap();
        let remote = script::load_tags(&export, &episode).unwrap();
        let local: Vec<_> = tags::spans(body, &registry)
            .into_iter()
            .flat_map(|span| {
                span.messages
                    .into_iter()
                    .map(move |message| (body[span.range.clone()].to_owned(), message))
            })
            .collect();
        assert_eq!(
            local,
            remote
                .problems
                .into_iter()
                .map(|p| (p.tag, p.message))
                .collect::<Vec<_>>(),
            "{body}"
        );
    }
    assert!(!tags::can_save("[pause:abc]", &registry));
    assert!(!tags::can_save("[pause:800", &registry));
    assert!(tags::can_save("[unknown]", &registry));
    assert!(!archive_ready("archive", "free text", &registry));
    assert!(!archive_ready("archive", "[clip:missing] words", &registry));
    assert!(archive_ready("archive", "[clip:sample] words", &registry));
    assert!(archive_ready("kokoro", "free text", &registry));
}
#[test]
fn layouter_keeps_every_character_and_paints_backgrounds() {
    use eframe::egui;
    let text = "Café 🙂 [beat] [pause:abc] fin";
    let registry = Default::default();
    let spans = tags::spans(text, &registry);
    assert_eq!(&text[spans[0].range.clone()], "[beat]");
    assert!(spans[1].blocks_save);
    let job = tags::layout(text, &registry, false, 300.0);
    assert_eq!(job.text, text);
    assert!(
        job.sections
            .iter()
            .any(|s| s.format.background == scriptcast_ui::theme::DANGER_PILL)
    );
    let raw = tags::layout(text, &registry, true, 300.0);
    assert_eq!(raw.text, text);
    assert_eq!(raw.sections.len(), 1);
    let ctx = egui::Context::default();
    scriptcast_ui::theme::apply(&ctx);
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let galley = ui.fonts_mut(|f| f.layout_job(job.clone()));
        for index in 0..=text.chars().count() {
            let cursor = egui::text::CCursor::new(index);
            let rect = galley.pos_from_cursor(cursor);
            let hit = galley.cursor_from_pos(rect.center().to_vec2());
            assert_eq!(hit.index.0, index, "caret at {index}");
        }
    });
    output.textures_delta.clear();
}
#[test]
fn retime_preview_is_read_only_apply_checks_hash_and_has_checkpoint() {
    let (_tmp, episode, export) = fixture();
    let path = episode.script_path.as_ref().unwrap();
    let before = fs::read(path).unwrap();
    let preview = retime::preview(&export, &episode).unwrap();
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!preview.rows.is_empty());
    let cp = retime::apply(&episode, &preview).unwrap();
    assert_eq!(fs::read(cp.dir.join("external/script.md")).unwrap(), before);
    assert_eq!(fs::read(path).unwrap(), preview.bytes);
    assert!(
        retime::apply(&episode, &preview)
            .unwrap_err()
            .contains("changed on disk")
    );
    assert!(retime::parse_diff(b"not json").is_err());
    assert!(
        retime::parse_diff(br#"[{"turn_index":1,"display_name":"A","old":"00:00","new":"00:00"}]"#)
            .unwrap()
            .is_empty()
    );
}
