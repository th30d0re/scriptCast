mod common;
use common::Scratch;
use scriptcast_ui::{checkpoint::*, runner::Preset};
use std::{fs, os::unix::fs::PermissionsExt};
#[test]
fn exact_scopes_versions_and_restore() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let files = plan_files(&Preset::RegenerateTurns(vec![2]), &e).unwrap();
    let cp = create(&e.dir, "regenerate-turns", vec![], vec![2], &files).unwrap();
    assert_eq!(cp.version, 1);
    assert_eq!(cp.files.len(), 6);
    for p in [
        "episode_manifest.json",
        "render_state.json",
        "TURN_INDEX.csv",
        "episode_small.als",
        "Samples/Processed/turn_2_0.wav",
        "Samples/Processed/turn_2_1.wav",
    ] {
        assert!(cp.dir.join(p).is_file());
    }
    assert!(cp.dir.join("checkpoint.json").is_file());
    let wav = e.dir.join("Samples/Processed/turn_2_0.wav");
    let original = fs::read(&wav).unwrap();
    fs::write(&wav, b"changed wav").unwrap();
    let before = restore(&e.dir, &cp).unwrap();
    assert_eq!(before.version, 2);
    assert_eq!(before.preset, "pre-restore");
    assert_eq!(fs::read(&wav).unwrap(), original);
    assert_eq!(
        fs::read(before.dir.join("Samples/Processed/turn_2_0.wav")).unwrap(),
        b"changed wav"
    );
    let verify = plan_files(&Preset::Verify, &e).unwrap();
    assert_eq!(verify.len(), 1);
    assert_eq!(verify[0].stored_as.to_str(), Some("stress.json"));
    let cp = create(&e.dir, "verify", vec![], vec![], &verify).unwrap();
    assert_eq!(cp.version, 3);
    assert_eq!(
        list(&e.dir).iter().map(|c| c.version).collect::<Vec<_>>(),
        [3, 2, 1]
    );
}
#[test]
fn all_samples_skip_asd_and_incomplete_checkpoints() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    fs::write(e.dir.join("Samples/cache.asd"), "cache").unwrap();
    let files = plan_files(&Preset::PrecisionInsert, &e).unwrap();
    assert_eq!(files.len(), 8);
    assert!(
        files
            .iter()
            .all(|f| f.path.extension().is_none_or(|s| s != "asd"))
    );
    let cp = create(&e.dir, "precision-insert", vec![], vec![], &files).unwrap();
    let root = e.dir.join("_backups/ui");
    fs::create_dir(root.join("v10-incomplete")).unwrap();
    fs::create_dir(root.join("v11-bad")).unwrap();
    fs::write(root.join("v11-bad/checkpoint.json"), "broken").unwrap();
    assert_eq!(list(&e.dir).len(), 1);
    let next = create(
        &e.dir,
        "verify",
        vec![],
        vec![],
        &plan_files(&Preset::Verify, &e).unwrap(),
    )
    .unwrap();
    assert_eq!(next.version, 12);
    assert!(cp.dir.exists());
}
#[test]
fn readonly_backup_fails_without_changing_sources() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let backups = e.dir.join("_backups");
    fs::create_dir(&backups).unwrap();
    let original = fs::read(e.dir.join("stress.json")).unwrap();
    fs::set_permissions(&backups, fs::Permissions::from_mode(0o555)).unwrap();
    let result = create(
        &e.dir,
        "verify",
        vec![],
        vec![],
        &plan_files(&Preset::Verify, &e).unwrap(),
    );
    fs::set_permissions(backups, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read(e.dir.join("stress.json")).unwrap(), original);
}
#[test]
fn failed_copy_is_not_published_and_traversal_is_refused() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let files = vec![PlannedFile {
        path: e.dir.join("absent"),
        stored_as: "stress.json".into(),
    }];
    assert!(create(&e.dir, "verify", vec![], vec![], &files).is_err());
    assert!(list(&e.dir).is_empty());
    let files = vec![PlannedFile {
        path: e.dir.join("stress.json"),
        stored_as: "../escape".into(),
    }];
    assert!(create(&e.dir, "verify", vec![], vec![], &files).is_err());
    assert!(list(&e.dir).is_empty());
}
#[test]
#[ignore = "explicit scratch-clone checkpoint timing and restore; no synthesis"]
fn ep03_checkpoint_timing_restore() {
    let dir = std::path::PathBuf::from("/tmp/scriptcast-c/outputs/ATO_EP03_five_levels");
    let e = scriptcast_ui::episode::Episode::load(&dir).unwrap();
    let files = plan_files(&Preset::PrecisionInsert, &e).unwrap();
    let start = std::time::Instant::now();
    let cp = create(&dir, "precision-insert", vec![], vec![], &files).unwrap();
    println!(
        "Precision scope: {:.3}s, {} files, {} bytes, {}",
        start.elapsed().as_secs_f64(),
        cp.files.len(),
        cp.bytes,
        cp.dir.display()
    );
}
#[test]
#[ignore = "restore newest Verify checkpoint on scratch clone only"]
fn ep03_restore_verify() {
    let dir = std::path::Path::new("/tmp/scriptcast-c/outputs/ATO_EP03_five_levels");
    let cp = list(dir)
        .into_iter()
        .find(|c| c.preset == "verify")
        .unwrap();
    let bytes = fs::read(cp.dir.join("stress.json")).unwrap();
    let before = restore(dir, &cp).unwrap();
    assert_eq!(fs::read(dir.join("stress.json")).unwrap(), bytes);
    println!(
        "Restored {} byte-for-byte; prior state {}",
        cp.dir.display(),
        before.dir.display()
    );
}
