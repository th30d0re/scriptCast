mod common;
use common::Scratch;
use scriptcast_ui::{
    cli::{ExportSource, Resolved},
    detached::{self, Classification, CommandPlan, Record},
    new_episode::{Options, Preview},
    pipeline::{Pipeline, State, relayout_length},
    progress::{Event, Progress, Tail},
    runner::{JobSpec, Preset},
};
use std::{fs, io::Write, path::PathBuf};

fn preview() -> Preview {
    Preview::parse(include_bytes!("fixtures/phase_j/dry-run.json")).unwrap()
}
#[test]
fn preview_schema_unknown_and_blocking_gate() {
    let mut p = preview();
    assert!(!p.can_render());
    assert!(p.speakers[2].engine.is_none());
    p.warnings.retain(|w| !w.blocking);
    assert!(p.can_render());
    p.turns.clear();
    assert!(!p.can_render());
    let mut json: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/phase_j/dry-run.json")).unwrap();
    json["schema"] = 2.into();
    assert!(Preview::parse(&serde_json::to_vec(&json).unwrap()).is_err());
}
#[test]
fn tail_partial_growth_terminal_and_truncation() {
    let scratch = Scratch::new();
    let path = scratch.0.join("progress.jsonl");
    fs::write(&path, b"{\"event\":\"stage\",\"name\":\"layout\"}").unwrap();
    let mut tail = Tail::default();
    assert!(tail.read(&path).unwrap().is_empty());
    let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(file).unwrap();
    assert_eq!(
        tail.read(&path).unwrap(),
        vec![Event::Stage {
            name: "layout".into()
        }]
    );
    assert!(tail.read(&path).unwrap().is_empty());
    writeln!(
        file,
        "{{\"event\":\"done\",\"ok\":true,\"outputs\":{{\"mp3\":\"episode.mp3\"}}}}"
    )
    .unwrap();
    let mut p = Progress::default();
    for e in tail.read(&path).unwrap() {
        p.apply(&e);
    }
    assert_eq!(p.terminal, Some(Ok(())));
    fs::write(&path, "{\"event\":\"error\",\"message\":\"cancelled\"}\n").unwrap();
    for e in tail.read(&path).unwrap() {
        p.apply(&e);
    }
    assert_eq!(p.terminal, Some(Err("cancelled".into())));
}
#[test]
fn eta_is_mean_completed_turn_seconds_and_resume_aware() {
    let mut p = Progress::default();
    assert!(p.eta().is_none());
    let event = Event::TurnDone {
        turn_index: 8,
        turn_id: "8".into(),
        n: 9,
        total: 12,
        seconds: 4.0,
    };
    p.apply(&event);
    p.apply(&event);
    assert_eq!(p.eta().unwrap().as_secs_f64(), 12.0);
    p.apply(&Event::TurnDone {
        turn_index: 9,
        turn_id: "9".into(),
        n: 10,
        total: 12,
        seconds: 8.0,
    });
    assert_eq!(p.eta().unwrap().as_secs_f64(), 12.0);
    assert!(
        p.status(std::time::Duration::from_secs(60))
            .contains("01:00 elapsed")
    );
}
#[test]
fn pipeline_skips_and_completes() {
    let mut p = Pipeline {
        enabled: [false, true, false, true, false, false],
        ..Default::default()
    };
    p.next();
    assert_eq!(p.current, Some(1));
    p.finish(Some(0), None, 14);
    assert_eq!(p.current, Some(3));
    p.finish(Some(0), None, 0);
    assert_eq!(p.state, State::Complete);
}
#[test]
fn pipeline_repair_failure_continue_and_error() {
    let mut p = Pipeline {
        enabled: [false, false, true, true, true, true],
        ..Default::default()
    };
    p.next();
    p.finish(Some(0), None, 3);
    assert_eq!(p.state, State::RepairStopped);
    assert_eq!(p.current, Some(2));
    p.continue_anyway();
    assert_eq!(p.current, Some(3));
    p.finish(Some(0), Some("bad layout".into()), 0);
    assert_eq!(p.state, State::Failed("bad layout".into()));
    p.continue_anyway();
    assert_eq!(p.current, Some(3));
}
#[test]
fn pipeline_nonzero_exit_stops_even_if_no_error_event() {
    let mut p = Pipeline::default();
    p.next();
    p.finish(Some(2), None, 0);
    assert!(matches!(p.state, State::Failed(_)));
    assert_eq!(p.current, Some(0));
}
#[test]
fn relayout_length_uses_n_minus_one_gaps() {
    assert_eq!(relayout_length([], 250), 0);
    assert_eq!(relayout_length([1000], 250), 1000);
    assert_eq!(relayout_length([1000, 2000, 3500], 250), 7000);
    assert_eq!(relayout_length([1000, 2000], 0), 3000);
}
fn record() -> Record {
    Record {
        pid: 42,
        pgid: 42,
        preset: "render".into(),
        argv: vec!["scriptcast".into(), "--resume".into()],
        log: "a.log".into(),
        log_offset: 0,
        progress: "a.jsonl".into(),
        started: "2026-10-07T12:00:00Z".into(),
        episode: "/tmp/scriptcast-j/episode".into(),
        commands: vec![],
        pipeline: None,
        current_step: None,
        exit: None,
        interrupted: false,
    }
}
#[test]
fn record_roundtrip_and_fake_pid_classifications() {
    let scratch = Scratch::new();
    let path = scratch.0.join("job.json");
    detached::write(&path, &record()).unwrap();
    let r = detached::read(&path).unwrap();
    assert_eq!(r.argv, record().argv);
    assert_eq!(r.pid, 42);
    assert_eq!(
        detached::classify(&r, false, |p| p == 42),
        Classification::Alive
    );
    assert_eq!(
        detached::classify(&r, true, |_| false),
        Classification::Finished
    );
    assert_eq!(
        detached::classify(&r, false, |_| false),
        Classification::Interrupted
    );
    assert!(!detached::alive(0));
    assert!(!detached::alive(u32::MAX));
}
fn resolved(name: &str) -> Resolved {
    Resolved {
        source: ExportSource::Folder,
        program: PathBuf::from(name),
        prefix: vec![],
        path: "/usr/bin:/bin".into(),
        pythonpath: None,
    }
}
#[test]
fn every_new_preset_argv() {
    let scratch = Scratch::new();
    let episode = scratch.fixture();
    let commands = [
        "scriptcast",
        "scriptcast-relayout",
        "scriptcast-stitch",
        "scriptcast-turn-index",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), Ok(resolved(name))))
    .collect();
    for (preset, expected) in [
        (
            Preset::Relayout(750),
            vec![
                episode.dir.display().to_string(),
                "--gap-ms".into(),
                "750".into(),
            ],
        ),
        (Preset::Stitch, vec![episode.dir.display().to_string()]),
        (
            Preset::TurnIndex,
            vec![
                episode.dir.display().to_string(),
                "--transcript".into(),
                episode.script_path.as_ref().unwrap().display().to_string(),
            ],
        ),
    ] {
        let spec = JobSpec::from_preset(&preset, &episode, &commands).unwrap();
        assert_eq!(
            spec.args
                .iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(spec.program.to_string_lossy(), preset.command());
    }
    let export = JobSpec::from_preset(
        &Preset::ReExport {
            fcpxml: true,
            logic_dry_run: true,
        },
        &episode,
        &commands,
    )
    .unwrap();
    for flag in ["--export-only", "--fcpxml", "--logic-dry-run"] {
        assert!(export.args.contains(&flag.into()));
    }
    assert!(!export.synthesizes);
    assert!(export.needs_confirmation());
    let render = JobSpec::from_preset(&Preset::Render, &episode, &commands).unwrap();
    assert!(render.synthesizes);
    assert!(render.args.contains(&"--resume".into()));
}
#[test]
fn new_episode_argv_resume_excludes_overwrite_and_dry_run_json_path() {
    let options = Options {
        script: "script with spaces.md".into(),
        voices: "voices.yaml".into(),
        episode_id: "e".into(),
        output: "outputs".into(),
        overwrite: true,
        no_trim: true,
        skip_als: true,
        fcpxml: true,
        max_turns: 12,
        ..Default::default()
    };
    let render = options.spec(resolved("scriptcast"), "/tmp".into(), None, true);
    assert!(render.args.contains(&"--resume".into()));
    assert!(!render.args.contains(&"--overwrite".into()));
    for flag in ["--no-trim", "--skip-als", "--fcpxml", "--max-turns"] {
        assert!(render.args.contains(&flag.into()));
    }
    let dry = options.spec(
        resolved("scriptcast"),
        "/tmp".into(),
        Some(std::path::Path::new("preview.json")),
        false,
    );
    assert!(!dry.synthesizes);
    assert!(dry.args.contains(&"--json-out".into()));
    assert!(!dry.args.contains(&"--overwrite".into()));
    let plan = CommandPlan::new(&render);
    assert_eq!(plan.args, render.args);
}

#[test]
fn detached_parent_helper() {
    let Some(dir) = std::env::var_os("SCRIPTCAST_J_LIFETIME_TEST") else {
        return;
    };
    let dir = PathBuf::from(dir);
    let plan = CommandPlan {
        program: "/bin/sh".into(),
        args: vec![
            "-c".into(),
            "echo started; sleep 1; echo survived; echo stderr-survived >&2".into(),
        ],
        cwd: dir.clone(),
        path: "/usr/bin:/bin".into(),
        pythonpath: None,
        checkpoint: vec![],
    };
    let (path, _) = detached::launch_with_executable(
        std::path::Path::new(env!("CARGO_BIN_EXE_scriptcast-ui")),
        &dir,
        "lifetime-test",
        vec![plan],
        None,
    )
    .unwrap();
    fs::write(dir.join("record-path"), path.as_os_str().as_encoded_bytes()).unwrap();
}
#[test]
fn detached_job_survives_parent_exit_with_both_streams() {
    let scratch = Scratch::new();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "detached_parent_helper"])
        .env("SCRIPTCAST_J_LIFETIME_TEST", &scratch.0)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
    let path = PathBuf::from(fs::read_to_string(scratch.0.join("record-path")).unwrap());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let record = detached::read(&path).unwrap();
        if record.exit == Some(0) {
            let log = fs::read_to_string(&record.log).unwrap();
            assert!(log.contains("survived"));
            assert!(log.contains("stderr-survived"));
            assert_eq!(record.pid, record.pgid);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Detached child did not complete after parent exit"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

#[test]
fn detached_pipeline_stops_at_repair_and_continues_after_decision() {
    let scratch = Scratch::new();
    let plan = |message: &str, failing: usize| {
        CommandPlan {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), format!(
            "echo {message}; printf '%s\\n' '{{\"event\":\"result\",\"passed\":1,\"failed\":{failing}}}' \
             '{{\"event\":\"done\",\"ok\":true,\"outputs\":{{}}}}' > \"$2\""
        ).into(), "fixture-step".into()],
        cwd: scratch.0.clone(), path: "/usr/bin:/bin".into(), pythonpath: None, checkpoint: vec![],
    }
    };
    let commands = vec![
        plan("skipped-render", 0),
        plan("verify", 3),
        plan("repair", 3),
        plan("relayout", 0),
        plan("skipped-stitch", 0),
        plan("index", 0),
    ];
    let pipeline = Pipeline {
        enabled: [false, true, true, true, false, true],
        ..Default::default()
    };
    let (path, _) = detached::launch_with_executable(
        std::path::Path::new(env!("CARGO_BIN_EXE_scriptcast-ui")),
        &scratch.0,
        "pipeline",
        commands,
        Some(pipeline),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let record = detached::read(&path).unwrap();
        if record.pipeline.as_ref().unwrap().state == State::RepairStopped {
            assert!(record.exit.is_none());
            let log = fs::read_to_string(&record.log).unwrap();
            assert!(log.contains("repair"));
            assert!(!log.contains("relayout"));
            fs::write(path.with_extension("continue"), b"continue").unwrap();
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    loop {
        let record = detached::read(&path).unwrap();
        if record.exit.is_some() {
            assert_eq!(record.exit, Some(0));
            assert_eq!(record.pipeline.unwrap().state, State::Complete);
            let log = fs::read_to_string(&record.log).unwrap();
            assert!(log.contains("relayout"));
            assert!(log.contains("index"));
            assert!(!log.contains("skipped"));
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
#[test]
fn utility_presets_do_not_require_a_transcript() {
    let scratch = Scratch::new();
    let mut episode = scratch.fixture();
    episode.script_path = None;
    let commands = [
        "scriptcast",
        "scriptcast-relayout",
        "scriptcast-stitch",
        "scriptcast-turn-index",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), Ok(resolved(name))))
    .collect();
    for preset in [
        Preset::ReExport {
            fcpxml: false,
            logic_dry_run: false,
        },
        Preset::Relayout(0),
        Preset::Stitch,
    ] {
        let spec = JobSpec::from_preset(&preset, &episode, &commands).unwrap();
        assert!(!spec.args.contains(&"--transcript".into()));
    }
}
#[test]
fn planned_episode_only_exposes_completed_audio() {
    let scratch = Scratch::new();
    let episode = scratch.fixture();
    let preview = preview();
    let planned = preview.episode(&episode.dir, &[1].into());
    assert!(!planned.turns[0].wav_paths.is_empty());
    assert!(planned.turns[1].wav_paths.is_empty());
    assert!(planned.turns[2].wav_paths.is_empty());
    assert!(scriptcast_ui::waveform::envelope(&planned.turns[0].wav_paths).is_ok());
}
#[test]
fn eta_includes_the_current_unfinished_turn() {
    let mut progress = Progress::default();
    for line in include_str!("fixtures/phase_j/progress.jsonl").lines() {
        progress.apply(&serde_json::from_str::<Event>(line).unwrap());
    }
    assert_eq!(progress.eta().unwrap().as_secs_f64(), 8.4);
}

#[test]
fn minimal_contract_record_can_reattach() {
    let scratch = Scratch::new();
    fs::create_dir(scratch.0.join("_jobs")).unwrap();
    let path = scratch.0.join("_jobs/job.json");
    fs::write(
        &path,
        r#"{"pid":42,"pgid":42,"preset":"render","argv":["scriptcast","--resume"],
        "log":"a.log","progress":"a.jsonl","started":"2026-10-07T12:00:00Z"}"#,
    )
    .unwrap();
    let record = detached::read(&path).unwrap();
    assert_eq!(record.episode, scratch.0);
    assert_eq!(
        detached::classify(&record, false, |_| true),
        Classification::Alive
    );
}
#[test]
fn live_preview_reads_speaker_chunk_layout_and_can_reopen_without_manifest() {
    let scratch = Scratch::new();
    let episode = scratch.fixture();
    let mut preview = preview();
    preview.turns[0].turn_id = "stable-turn-id".into();
    let speaker = episode.dir.join("Samples/Processed/alex");
    fs::create_dir(&speaker).unwrap();
    fs::copy(
        episode.dir.join("Samples/Processed/turn_1_0.wav"),
        speaker.join("stable-turn-id_chunk_0000.wav"),
    )
    .unwrap();
    let planned = preview.episode(&episode.dir, &[1].into());
    assert!(
        planned.turns[0]
            .wav_paths
            .iter()
            .any(|p| p.ends_with("stable-turn-id_chunk_0000.wav"))
    );
    fs::remove_file(episode.dir.join("episode_manifest.json")).unwrap();
    fs::create_dir(episode.dir.join("_jobs")).unwrap();
    let path = episode.dir.join("_jobs/fixture.json");
    let mut record = record();
    record.episode = episode.dir.clone();
    record.progress = path.with_extension("jsonl");
    detached::write(&path, &record).unwrap();
    fs::write(
        path.with_extension("preview.json"),
        serde_json::to_vec(&preview).unwrap(),
    )
    .unwrap();
    fs::write(
        &record.progress,
        include_bytes!("fixtures/phase_j/progress.jsonl"),
    )
    .unwrap();
    let reopened = scriptcast_ui::new_episode::load_live(&episode.dir).unwrap();
    assert_eq!(reopened.turns.len(), 3);
    assert!(!reopened.turns[0].wav_paths.is_empty());
    assert!(reopened.turns[1].wav_paths.is_empty());
}
