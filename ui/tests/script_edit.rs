mod common;
use common::Scratch;
use scriptcast_ui::{
    checkpoint::{self, PlannedFile},
    cli::{ExportCommand, ExportSource},
    script::{self, Changes, ScriptTurn},
};
use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn turn(body: &str) -> ScriptTurn {
    ScriptTurn {
        index: 1,
        turn_id: "loaded-id".into(),
        speaker_id: "alice".into(),
        display_name: "Alice".into(),
        header_line: 3,
        body_start: 5,
        body_end: 7,
        body: body.into(),
    }
}

fn original(ending: &str) -> Vec<u8> {
    "# Préamble\n\nAlice (00:01)\n \nOriginal **body**.\n<!-- comment -->\nlast line\n\n \nBob (00:02)\nNext."
        .replace('\n', ending).into_bytes()
}

#[test]
fn splice_preserves_every_surrounding_byte_and_blank_line() {
    for ending in ["\n", "\r\n"] {
        let bytes = original(ending);
        let body = ["Original **body**.", "<!-- comment -->", "last line"].join(ending);
        let draft = "Edited café.\n[beat] More.";
        let changed = script::splice(&bytes, &turn(&body), draft).unwrap();
        let expected = String::from_utf8(bytes)
            .unwrap()
            .replace(&body, &draft.replace('\n', ending));
        assert_eq!(changed, expected.as_bytes());
    }
}

#[test]
fn splice_handles_eof_empty_spans_and_invalid_input() {
    let mut t = turn("old");
    t.header_line = 1;
    t.body_start = 2;
    t.body_end = 2;
    assert_eq!(
        script::splice(b"Alice (00:00)\nold", &t, "new").unwrap(),
        b"Alice (00:00)\nnew"
    );
    assert!(script::splice(b"Alice (00:00)\nold", &t, " \n\t").is_err());
    t.body_end = 100;
    assert!(script::splice(b"Alice (00:00)\nold", &t, "new").is_err());
    t.body_end = 1;
    t.body.clear();
    for source in ["Alice (00:00)", "Alice (00:00)\n"] {
        assert_eq!(
            script::splice(source.as_bytes(), &t, "new").unwrap(),
            b"Alice (00:00)\nnew"
        );
    }
    assert_eq!(
        script::splice(b"Alice (00:00)\nBob (00:01)\nbody", &t, "new").unwrap(),
        b"Alice (00:00)\nnew\nBob (00:01)\nbody"
    );
}

#[test]
fn save_conflict_empty_draft_and_checkpoint_failure_touch_no_script() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let path = e.script_path.as_ref().unwrap();
    let bytes = original("\n");
    fs::write(path, &bytes).unwrap();
    let t = turn("Original **body**.\n<!-- comment -->\nlast line");
    let error = script::save(&e, &t, "new", "stale-hash").unwrap_err();
    assert_eq!(error, "The script changed on disk. Reload to edit.");
    assert!(!e.dir.join("_backups").exists());
    assert!(script::save(&e, &t, " \n", &hash(&bytes)).is_err());
    assert!(!e.dir.join("_backups").exists());
    fs::write(e.dir.join("_backups"), "blocks backup directory creation").unwrap();
    assert!(script::save(&e, &t, "new", &hash(&bytes)).is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert!(!path.parent().unwrap().read_dir().unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".scriptcast-edit")
    }));
}

#[test]
fn checkpoint_precedes_save_and_restore_is_reversible_with_modes() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let path = e.script_path.as_ref().unwrap();
    let bytes = original("\r\n");
    fs::write(path, &bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o640)).unwrap();
    let t = turn("Original **body**.\r\n<!-- comment -->\r\nlast line");
    let cp = script::save(&e, &t, "new\nbody", &hash(&bytes)).unwrap();
    assert_eq!(cp.kind, "script-edit");
    assert_eq!(cp.preset, "script-edit");
    assert_eq!(cp.turns, [1]);
    assert_eq!(cp.external_files[0].path, fs::canonicalize(path).unwrap());
    assert_eq!(
        cp.external_files[0].stored_as,
        PathBuf::from("external/script.md")
    );
    assert_eq!(fs::read(cp.dir.join(&cp.files[0])).unwrap(), bytes);
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    let edited = fs::read(path).unwrap();
    assert_ne!(edited, bytes);
    let listed = checkpoint::list(&e.dir);
    assert_eq!(listed.len(), 1);
    let before = checkpoint::restore(&e.dir, &listed[0]).unwrap();
    assert_eq!(before.preset, "pre-restore");
    assert_eq!(before.kind, "script-edit");
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert_eq!(fs::read(before.dir.join(&before.files[0])).unwrap(), edited);
    checkpoint::restore(&e.dir, &before).unwrap();
    assert_eq!(fs::read(path).unwrap(), edited);
}

#[test]
fn external_names_cannot_collide_and_audio_cannot_escape() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let path = e.script_path.as_ref().unwrap();
    let files = [PlannedFile {
        path: path.clone(),
        stored_as: "script.md".into(),
    }];
    assert!(checkpoint::create(&e.dir, "audio", vec![], vec![], &files).is_err());
    fs::create_dir(tmp.0.join("other")).unwrap();
    let other = tmp.0.join("other/script.md");
    fs::write(&other, "different").unwrap();
    assert!(
        checkpoint::create_external(&e.dir, "script-edit", &[path.clone(), other], vec![]).is_err()
    );
    assert!(checkpoint::list(&e.dir).is_empty());
    let cp = checkpoint::create_external(&e.dir, "script-edit", std::slice::from_ref(path), vec![])
        .unwrap();
    let mut forged = cp.clone();
    forged.kind = "audio".into();
    assert!(checkpoint::restore(&e.dir, &forged).is_err());
    forged = cp;
    forged.external_files[0].stored_as = "../escape".into();
    assert!(checkpoint::restore(&e.dir, &forged).is_err());
}

#[test]
fn legacy_audio_checkpoint_is_listed_and_restored() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let files = [PlannedFile {
        path: e.dir.join("stress.json"),
        stored_as: "stress.json".into(),
    }];
    let cp = checkpoint::create(&e.dir, "verify", vec![], vec![], &files).unwrap();
    let manifest = cp.dir.join("checkpoint.json");
    let mut data: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    data.as_object_mut().unwrap().remove("kind");
    data.as_object_mut().unwrap().remove("external_files");
    fs::write(&manifest, serde_json::to_vec(&data).unwrap()).unwrap();
    let listed = checkpoint::list(&e.dir);
    assert_eq!(listed[0].kind, "audio");
    checkpoint::restore(&e.dir, &listed[0]).unwrap();
}

#[test]
fn edited_rows_require_equal_counts_and_deduplicate() {
    let changes = Changes {
        inserted: vec![2, 1],
        modified: vec![1],
        deleted: vec!["old".into()],
        unchanged: 1,
    };
    assert_eq!(script::edited_turns(&changes, 3, 4), None);
    assert_eq!(script::edited_turns(&changes, 3, 3), Some(vec![1, 2]));
    assert_eq!(script::edited_turns(&changes, 2, 2), None);
}

#[test]
fn real_cli_edit_changes_and_restore_on_copied_fixture() {
    let tmp = phase_i_scratch();
    let mut e = tmp.fixture();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();
    // Force the actual dev-checkout fallback, independent of installed scriptcast-export commands.
    let export = ExportCommand {
        source: ExportSource::DevCheckout(root.clone()),
        program: "python3".into(),
    };
    let setup = r#"
import hashlib, json
from pathlib import Path
from scriptcast.parser import parse_transcript
from scriptcast.markup import tokenize_markup
from scriptcast.render_state import RenderState, compute_turn_fingerprint
from scriptcast.voices import load_voices
p = Path('script.md')
p.write_bytes(b'Alice (00:00)\r\nHello.\r\n\r\nBob (00:01)\r\nOriginal.\r\n\r\nAlice (00:02)\r\nEnd.\r\n')
voices = load_voices(Path('voices.yaml'))
turns = tokenize_markup(parse_transcript(p))
state = RenderState('2.0', 'script.md', hashlib.sha256(p.read_bytes()).hexdigest(), 'today',
                    [compute_turn_fingerprint(t, voices.get(t.speaker_id)) for t in turns], [])
Path('episode_small/render_state.json').write_text(json.dumps(state.to_dict()))
"#;
    let output = Command::new("python3")
        .args(["-c", setup])
        .current_dir(&tmp.0)
        .env("PYTHONPATH", &root)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("SCRIPTCAST_PROJECT", &tmp.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    e = scriptcast_ui::episode::Episode::load(&e.dir).unwrap();
    let loaded = script::load_turns(&export, &e).unwrap();
    assert_eq!(loaded.turns.len(), 3);
    assert_eq!(loaded.line_ending, "crlf");
    assert_eq!(
        loaded.script_path,
        fs::canonicalize(e.script_path.as_ref().unwrap()).unwrap()
    );
    let before = script::load_changes(&export, &e).unwrap();
    assert_eq!(before.unchanged, 3);
    assert!(before.inserted.is_empty() && before.modified.is_empty() && before.deleted.is_empty());
    let original = fs::read(&loaded.script_path).unwrap();
    let cp = script::save(
        &e,
        &loaded.turns[1],
        "Edited second turn.",
        &loaded.script_sha256,
    )
    .unwrap();
    let changes = script::load_changes(&export, &e).unwrap();
    assert_eq!(changes.inserted, [1]);
    assert_eq!(changes.deleted, [loaded.turns[1].turn_id.clone()]);
    assert!(changes.modified.is_empty());
    assert_eq!(changes.unchanged, 2);
    checkpoint::restore(&e.dir, &cp).unwrap();
    assert_eq!(fs::read(&loaded.script_path).unwrap(), original);
    assert_eq!(script::load_changes(&export, &e).unwrap().unchanged, 3);
}

#[test]
fn checkpoint_paths_refuse_symlink_escapes_before_writing() {
    use std::os::unix::fs::symlink;
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let outside = tmp.0.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, e.dir.join("_backups")).unwrap();
    let script = e.script_path.as_ref().unwrap();
    assert!(
        checkpoint::create_external(&e.dir, "script-edit", std::slice::from_ref(script), vec![])
            .is_err()
    );
    assert!(!outside.join("ui").exists());
    fs::remove_file(e.dir.join("_backups")).unwrap();
    let cp =
        checkpoint::create_external(&e.dir, "script-edit", std::slice::from_ref(script), vec![])
            .unwrap();
    let destination = outside.join("other.md");
    fs::write(&destination, "keep me").unwrap();
    fs::remove_file(script).unwrap();
    symlink(&destination, script).unwrap();
    assert!(checkpoint::restore(&e.dir, &cp).is_err());
    assert_eq!(fs::read_to_string(&destination).unwrap(), "keep me");
    let files = [PlannedFile {
        path: e.dir.join("stress.json"),
        stored_as: "stress.json".into(),
    }];
    let mut audio = checkpoint::create(&e.dir, "verify", vec![], vec![], &files).unwrap();
    audio.files[0] = destination;
    assert!(checkpoint::restore(&e.dir, &audio).is_err());
}

/// Operator-only live acceptance: refuses any destination outside the cloned D2 project.
#[test]
#[ignore = "requires the EP03 clone and export shim under /tmp/scriptcast-d2/live"]
fn phase_d2_live_copy_save_changes_and_restore() {
    let root = fs::canonicalize("/tmp/scriptcast-d2/live").unwrap();
    let dir = root.join("outputs/ATO_EP03_five_levels");
    let e = scriptcast_ui::episode::Episode::load(&dir).unwrap();
    let export = ExportCommand {
        source: ExportSource::Folder,
        program: root.join(".venv-d2/bin/scriptcast-export"),
    };
    let loaded = script::load_turns(&export, &e).unwrap();
    let script_path = fs::canonicalize(&loaded.script_path).unwrap();
    assert!(script_path.starts_with(&root));
    assert_eq!(
        fs::canonicalize(e.script_path.as_ref().unwrap()).unwrap(),
        script_path
    );
    let original = fs::read(&script_path).unwrap();
    let turn = &loaded.turns[6];
    let draft = format!(
        "{} This sentence is a reversible Phase D2 copy-only test.",
        turn.body
    );
    let cp = script::save(&e, turn, &draft, &loaded.script_sha256).unwrap();
    assert_eq!(cp.kind, "script-edit");
    let after_save = hash(&fs::read(&script_path).unwrap());
    assert_ne!(after_save, loaded.script_sha256);
    eprintln!("copy_script_before_save={}", loaded.script_sha256);
    eprintln!("copy_script_after_save={after_save}");
    let output = Command::new(&export.program)
        .arg(&dir)
        .arg("--changes-json")
        .env("SCRIPTCAST_PROJECT", &root)
        .output()
        .unwrap();
    fs::write(
        "/tmp/scriptcast-d2/live-changes-after-save.json",
        &output.stdout,
    )
    .unwrap();
    // Restore even if the read-only export failed, before checking its outcome.
    let before_restore = checkpoint::restore(&dir, &cp).unwrap();
    let after_restore = hash(&fs::read(&script_path).unwrap());
    eprintln!("copy_script_after_restore={after_restore}");
    assert_eq!(fs::read(&script_path).unwrap(), original);
    assert_eq!(after_restore, loaded.script_sha256);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let changes: Changes = serde_json::from_slice(&output.stdout).unwrap();
    assert!(changes.inserted.contains(&6) || changes.modified.contains(&6));
    let restored = script::load_turns(&export, &e).unwrap();
    assert_eq!(restored.script_sha256, loaded.script_sha256);
    let summary = serde_json::json!({
        "turn": 6, "script": script_path, "checkpoint": cp.dir,
        "pre_restore_checkpoint": before_restore.dir, "restored_bytes_equal": true,
        "original_sha256": loaded.script_sha256, "restored_sha256": restored.script_sha256,
        "changes_after_save": serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
    });
    fs::write(
        "/tmp/scriptcast-d2/live-roundtrip.json",
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
    println!("{summary}");
}

#[test]
fn batch_bytes_one_checkpoint_conflict_and_restore() {
    for ending in ["\n", "\r\n"] {
        let tmp = phase_i_scratch();
        let e = tmp.fixture();
        let path = e.script_path.as_ref().unwrap();
        let bytes = original(ending);
        fs::write(path, &bytes).unwrap();
        let body = ["Original **body**.", "<!-- comment -->", "last line"].join(ending);
        let second = ScriptTurn {
            index: 2,
            turn_id: "second".into(),
            speaker_id: "bob".into(),
            display_name: "Bob".into(),
            header_line: 10,
            body_start: 11,
            body_end: 11,
            body: "Next.".into(),
        };
        let edits = vec![
            (turn(&body), body.replace("Original", "New")),
            (second, "Last.".into()),
        ];
        assert!(script::save_many(&e, &edits, "wrong hash").is_err());
        assert_eq!(fs::read(path).unwrap(), bytes);
        assert!(!e.dir.join("_backups").exists());
        let cp = script::save_many(&e, &edits, &hash(&bytes)).unwrap();
        assert_eq!(checkpoint::list(&e.dir).len(), 1);
        assert_eq!(cp.turns, vec![1, 2]);
        assert_eq!(cp.kind, "script-edit");
        let expected = String::from_utf8(bytes.clone())
            .unwrap()
            .replace("Original", "New")
            .replace("Next.", "Last.");
        assert_eq!(fs::read(path).unwrap(), expected.as_bytes());
        checkpoint::restore(&e.dir, &cp).unwrap();
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn batch_validates_all_spans_and_clip_tags_before_checkpointing() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let path = e.script_path.as_ref().unwrap();
    let bytes = original("\n");
    fs::write(path, &bytes).unwrap();
    let t = turn("Original **body**.\n<!-- comment -->\nlast line");
    let mut bad = t.clone();
    bad.index = 2;
    bad.body_end = 999;
    assert!(
        script::save_many(
            &e,
            &[(t.clone(), "new".into()), (bad, "bad".into())],
            &hash(&bytes)
        )
        .is_err()
    );
    assert!(
        script::save_many(
            &e,
            &[(t.clone(), "new".into()), (t, "duplicate".into())],
            &hash(&bytes)
        )
        .is_err()
    );
    assert!(script::save_many(&e, &[], &hash(&bytes)).is_err());
    assert!(!e.dir.join("_backups").exists());
    assert_eq!(fs::read(path).unwrap(), bytes);
}

fn phase_i_scratch() -> Scratch {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = PathBuf::from("/tmp/scriptcast-i").join(format!(
        "script-edit-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&path).unwrap();
    Scratch(path)
}

/// Live acceptance only operates on the explicitly prepared Phase I copy.
#[test]
#[ignore = "requires the EP03 project copy under /tmp/scriptcast-i/live"]
fn phase_i_live_replace_all_changes_and_restore() {
    use scriptcast_ui::find::{self, Matcher, Options};
    let root = fs::canonicalize("/tmp/scriptcast-i/live").unwrap();
    let dir = root.join("outputs/ATO_EP03_five_levels");
    let e = scriptcast_ui::episode::Episode::load(&dir).unwrap();
    assert!(
        fs::canonicalize(e.script_path.as_ref().unwrap())
            .unwrap()
            .starts_with(&root)
    );
    let export = ExportCommand {
        source: ExportSource::DevCheckout(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .into(),
        ),
        program: std::env::var_os("SCRIPTCAST_I_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| "python3".into()),
    };
    let loaded = script::load_turns(&export, &e).unwrap();
    let path = fs::canonicalize(&loaded.script_path).unwrap();
    assert!(path.starts_with(&root));
    let original = fs::read(&path).unwrap();
    let options = Options {
        whole_word: true,
        ..Default::default()
    };
    let matcher = Matcher::new("racism", &options).unwrap();
    let plan = find::plan(&loaded.turns, &matcher, &options, "bias", None);
    assert!(plan.edits.len() > 1);
    let before_checkpoints = checkpoint::list(&dir).len();
    let cp = script::save_many(&e, &plan.edits, &loaded.script_sha256).unwrap();
    let saved = fs::read(&path).unwrap();
    let changes_result = script::load_changes(&export, &e);
    let checkpoint_count = checkpoint::list(&dir).len();
    // Always restore before checking the post-save exports or byte assertions.
    checkpoint::restore(&dir, &cp).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(checkpoint_count, before_checkpoints + 1);
    let changes = changes_result.unwrap();
    for (turn, _) in &plan.edits {
        assert!(changes.inserted.contains(&turn.index) || changes.modified.contains(&turn.index));
    }
    let before_text = String::from_utf8(original.clone()).unwrap();
    let after_text = String::from_utf8(saved.clone()).unwrap();
    let before_lines: Vec<_> = before_text.split_inclusive('\n').collect();
    let after_lines: Vec<_> = after_text.split_inclusive('\n').collect();
    assert_eq!(before_lines.len(), after_lines.len());
    let regex = regex::Regex::new(r"(?i)\bracism\b").unwrap();
    for (index, (before, after)) in before_lines.iter().zip(&after_lines).enumerate() {
        if before != after {
            assert!(
                plan.edits
                    .iter()
                    .any(|(t, _)| index + 1 >= t.body_start && index < t.body_end)
            );
            assert_eq!(regex.replace_all(before, "bias"), *after);
        }
    }
    let summary = serde_json::json!({
        "script": path, "query": "racism", "replacement": "bias",
        "occurrences": plan.occurrences, "turns": plan.edits.len(), "skipped": plan.skipped,
        "indices": plan.edits.iter().map(|(t, _)| t.index).collect::<Vec<_>>(),
        "checkpoint": cp.dir, "checkpoint_version": cp.version,
        "original_sha256": hash(&original), "replaced_sha256": hash(&saved),
        "restored_sha256": hash(&fs::read(&path).unwrap()), "restored_bytes_equal": true,
        "only_matched_words_changed": true, "one_checkpoint": true,
        "changes_after_save": { "inserted": changes.inserted, "modified": changes.modified,
            "deleted": changes.deleted, "unchanged": changes.unchanged },
    });
    fs::write("/tmp/scriptcast-i/script-before.md", original).unwrap();
    fs::write("/tmp/scriptcast-i/script-after.md", saved).unwrap();
    fs::write(
        "/tmp/scriptcast-i/live-roundtrip.json",
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
    println!("{summary}");
}

#[test]
fn batch_preserves_mixed_line_endings_and_accepts_changed_line_counts() {
    let tmp = phase_i_scratch();
    let e = tmp.fixture();
    let path = e.script_path.as_ref().unwrap();
    let bytes = b"Alice (00:00)\r\nFirst.\nKeep.\r\n\r\nBob (00:01)\nLast.";
    fs::write(path, bytes).unwrap();
    let mut first = turn("First.\nKeep.");
    first.header_line = 1;
    first.body_start = 2;
    first.body_end = 3;
    let mut last = turn("Last.");
    last.index = 2;
    last.header_line = 5;
    last.body_start = 6;
    last.body_end = 6;
    let edits = [
        (last, "New last.\nExtra.".into()),
        (first, "New first.\nKeep.".into()),
    ];
    let cp = script::save_many(&e, &edits, &hash(bytes)).unwrap();
    assert_eq!(
        fs::read(path).unwrap(),
        b"Alice (00:00)\r\nNew first.\nKeep.\r\n\r\nBob (00:01)\nNew last.\nExtra."
    );
    checkpoint::restore(&e.dir, &cp).unwrap();
    assert_eq!(fs::read(path).unwrap(), bytes);
}
