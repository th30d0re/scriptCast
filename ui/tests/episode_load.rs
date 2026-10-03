use scriptcast_ui::episode::{
    Episode, VerificationState as V, find_root_with_override, parse_turn_index,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small")
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-scratch")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        copy_tree(fixture().parent().unwrap(), &dir);
        Self(dir)
    }
    fn episode(&self) -> PathBuf {
        self.0.join("episode_small")
    }
    fn json(&self, name: &str, change: impl FnOnce(&mut serde_json::Value)) {
        let path = self.episode().join(name);
        let mut data = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut data);
        fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &to.join(e.file_name()));
        } else {
            fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}
#[test]
fn fixture_loads_and_joins_by_id_in_chunk_order() {
    let e = Episode::load(&fixture()).unwrap();
    assert_eq!((e.turns.len(), e.speakers.len(), e.total_ms), (3, 2, 800));
    assert_eq!(e.turns[0].first_words, "Hello, world.");
    assert_eq!(e.script_in_sync, Some(true));
    assert_eq!(e.project_root.as_deref(), fixture().parent());
    assert_eq!(
        e.script_path,
        Some(fixture().parent().unwrap().join("script.md"))
    );
    assert_eq!(
        e.voices_path,
        Some(fixture().parent().unwrap().join("voices.yaml"))
    );
    assert_eq!(
        e.turns.iter().map(|t| t.verification).collect::<Vec<_>>(),
        [V::Passed, V::Failed, V::NotVerified]
    );
    assert!(
        e.turns
            .iter()
            .flat_map(|t| &t.wav_paths)
            .all(|p| p.is_absolute() && p.is_file())
    );
    assert!(e.turns[1].wav_paths[0].ends_with("turn_2_0.wav"));
    assert!(e.turns[1].wav_paths[1].ends_with("turn_2_1.wav"));
    assert_eq!(
        e.speakers.iter().map(|s| s.colour_idx).collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(e.warnings.is_empty(), "{:?}", e.warnings);
}
#[test]
fn missing_manifest_names_file() {
    let s = Scratch::new();
    fs::remove_file(s.episode().join("episode_manifest.json")).unwrap();
    let e = Episode::load(&s.episode()).unwrap_err();
    assert!(e.to_string().contains("episode_manifest.json"));
}
#[test]
fn invalid_manifest_preserves_serde_message() {
    let s = Scratch::new();
    fs::write(s.episode().join("episode_manifest.json"), "{").unwrap();
    let e = Episode::load(&s.episode()).unwrap_err().to_string();
    assert!(e.contains("episode_manifest.json") && e.contains("EOF") && e.contains("line 1"));
}
#[test]
fn missing_optional_files_leave_empty_dependent_fields() {
    let s = Scratch::new();
    for name in ["render_state.json", "TURN_INDEX.csv", "stress.json"] {
        fs::remove_file(s.episode().join(name)).unwrap();
    }
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.rendered_at, None);
    assert_eq!(e.script_in_sync, None);
    assert!(e.turns.iter().all(|t| t.verify.is_none()
        && t.first_words.is_empty()
        && t.verification == V::NotVerified));
    assert_eq!(e.warnings.len(), 3);
}
#[test]
fn malformed_optional_files_warn_and_keep_browsing() {
    let s = Scratch::new();
    for name in ["render_state.json", "TURN_INDEX.csv", "stress.json"] {
        fs::write(s.episode().join(name), "bad").unwrap();
    }
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.turns.len(), 3);
    assert_eq!(e.warnings.len(), 3);
    assert_eq!(e.count(V::NotVerified), 3);
}
#[test]
fn missing_audio_takes_precedence_without_erasing_verification() {
    let s = Scratch::new();
    fs::remove_file(s.episode().join("Samples/Processed/turn_2_1.wav")).unwrap();
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.turns[1].verification, V::MissingAudio);
    assert!(e.turns[1].verify.as_ref().unwrap().failed);
    assert_eq!(e.turns[0].verification, V::Passed);
}
#[test]
fn overview_dominance_handles_chunks_silence_and_zero_slices() {
    let e = Episode::load(&fixture()).unwrap();
    assert_eq!(e.overview_sections(4), [Some(0), Some(1), Some(1), Some(0)]);
    assert_eq!(e.overview_sections(1), [Some(0)]);
    assert!(e.overview_sections(0).is_empty());
    let s = Scratch::new();
    s.json("episode_manifest.json", |v| {
        v["turns"][2]["end_ms"] = 1600.into();
    });
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.overview_sections(2), [Some(0), None]);
}
#[test]
fn changed_and_unavailable_script_hashes() {
    let s = Scratch::new();
    fs::write(s.0.join("script.md"), "changed").unwrap();
    assert_eq!(
        Episode::load(&s.episode()).unwrap().script_in_sync,
        Some(false)
    );
    fs::remove_file(s.0.join("script.md")).unwrap();
    assert_eq!(Episode::load(&s.episode()).unwrap().script_in_sync, None);
}
#[test]
fn absolute_render_source_overrides_manifest() {
    let s = Scratch::new();
    let script = s.0.join("script.md");
    s.json("render_state.json", |v| {
        v["source_file"] = script.to_str().unwrap().into()
    });
    s.json("episode_manifest.json", |v| {
        v["source_file"] = "wrong.md".into()
    });
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.script_path, Some(script));
    assert_eq!(e.script_in_sync, Some(true));
}
#[test]
fn project_discovery_and_override() {
    let s = Scratch::new();
    assert_eq!(
        find_root_with_override(&s.episode(), None),
        Some(s.0.clone())
    );
    assert_eq!(
        find_root_with_override(&s.episode(), Some(&s.episode())),
        Some(s.episode())
    );
    fs::remove_file(s.0.join("scriptcast.toml")).unwrap();
    assert_eq!(find_root_with_override(&s.episode(), None), None);
}
#[test]
fn csv_handles_escaped_quotes_newlines_and_rejects_malformed_records() {
    let rows = parse_turn_index("turn_index,first_words\r\n1,\"Hello, \"\"world\"\"\nagain\"\r\n")
        .unwrap();
    assert_eq!(rows[0].first_words, "Hello, \"world\"\nagain");
    for csv in [
        "turn_index,first_words\n1,\"unfinished",
        "turn_index,first_words\n1,a,b",
        "turn_index,first_words\n1,a\n1,b",
        "turn_index,first_words\n1,\"x\"y",
    ] {
        assert!(parse_turn_index(csv).is_err());
    }
}
#[test]
fn rejects_invalid_time_bounds() {
    let s = Scratch::new();
    s.json("episode_manifest.json", |v| {
        v["turns"][1]["end_ms"] = 0.into()
    });
    assert!(Episode::load(&s.episode()).is_err());
}
#[test]
#[ignore = "requires real EP03 directory; read-only"]
fn ep03_loads() {
    let Some(path) = std::env::var_os("SCRIPTCAST_EP03") else {
        eprintln!("SCRIPTCAST_EP03 unset; skipped");
        return;
    };
    let start = Instant::now();
    let e = Episode::load(Path::new(&path)).unwrap();
    let elapsed = start.elapsed();
    println!("EP03 load time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
    assert_eq!(
        (e.turns.len(), e.speakers.len(), e.total_ms),
        (392, 4, 3861350)
    );
    assert_eq!(e.script_in_sync, Some(false));
    assert_eq!(
        [
            e.count(V::Passed),
            e.count(V::Failed),
            e.count(V::NotVerified),
            e.count(V::MissingAudio)
        ],
        // stress.json (2026-09-16) predates the 2026-09-18 re-render: 5 of its 18
        // entries name turn_ids no longer in the manifest, so only 13 join.
        [13, 0, 379, 0]
    );
}

#[test]
fn canceled_load_stops_before_opening_metadata() {
    let canceled = std::sync::atomic::AtomicBool::new(true);
    let error = Episode::load_cancellable(&fixture(), &canceled).unwrap_err();
    assert_eq!(error.cause, "Loading canceled");
}

#[test]
fn repetition_is_nullable_text_and_missing_fields_remain_optional() {
    let e = Episode::load(&fixture()).unwrap();
    assert_eq!(e.turns[0].verify.as_ref().unwrap().repetition, None);
    assert_eq!(
        e.turns[1].verify.as_ref().unwrap().repetition.as_deref(),
        Some("Two chunks")
    );
    let s = Scratch::new();
    s.json("stress.json", |v| {
        v[0].as_object_mut().unwrap().remove("repetition");
    });
    assert_eq!(Episode::load(&s.episode()).unwrap().count(V::Passed), 1);
}

#[test]
fn stale_verification_does_not_fall_back_to_numeric_turn_indices() {
    let s = Scratch::new();
    s.json("stress.json", |v| {
        v[0]["turn_id"] = "stale-id".into();
        v[0]["turn"] = 1.into();
        v[0]["manifest_turn"] = 1.into();
    });
    let e = Episode::load(&s.episode()).unwrap();
    assert_eq!(e.turns[0].verification, V::NotVerified);
    assert!(e.turns[0].verify.is_none());
    assert!(
        e.warnings
            .iter()
            .any(|w| w.contains("1 stale verification") && w.contains("stale-id"))
    );
}
