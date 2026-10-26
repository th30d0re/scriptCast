mod common;
use common::Scratch;
use scriptcast_ui::{
    cli::{ExportSource, Resolved},
    runner::*,
};
use std::{
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};
fn commands() -> Commands {
    [
        "scriptcast",
        "scriptcast-verify",
        "scriptcast-repair",
        "scriptcast-word-timings",
        "scriptcast-clip-transcribe",
        "scriptcast-verify-ensemble",
        "scriptcast-review",
        "scriptcast-review-stats",
    ]
    .into_iter()
    .map(|s| {
        (
            s.into(),
            Ok(Resolved {
                program: PathBuf::from("/fake bin").join(s),
                source: ExportSource::Folder,
                prefix: vec![],
                path: "/usr/bin:/bin".into(),
                pythonpath: None,
            }),
        )
    })
    .collect()
}
fn args(s: &JobSpec) -> Vec<String> {
    s.args
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect()
}
#[test]
fn exact_presets_and_no_overwrite() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let c = commands();
    let script = e.script_path.as_ref().unwrap().display().to_string();
    let dir = e.dir.display().to_string();
    let out = e.dir.parent().unwrap().display().to_string();
    let base = vec![
        "--transcript".into(),
        script.clone(),
        "--episode-id".into(),
        "episode_small".into(),
        "--out-dir".into(),
        out.clone(),
    ];
    for (p, tail) in [
        (Preset::DetectChanges, vec!["--detect-changes"]),
        (Preset::PrecisionInsert, vec!["--precision-insert"]),
        (
            Preset::RegenerateTurns(vec![1, 2, 3]),
            vec!["--regenerate-turns", "1-3"],
        ),
    ] {
        let s = JobSpec::from_preset(&p, &e, &c).unwrap();
        let mut want = base.clone();
        want.extend(tail.into_iter().map(str::to_string));
        assert_eq!(args(&s), want);
        assert_eq!(s.cwd, e.project_root.clone().unwrap());
        assert_eq!(s.synthesizes, p != Preset::DetectChanges);
        assert!(!args(&s).contains(&["--", "overwrite"].concat()));
    }
    let s = JobSpec::from_preset(&Preset::Verify, &e, &c).unwrap();
    assert_eq!(
        args(&s),
        [
            dir.clone(),
            "--transcript".into(),
            script.clone(),
            "--json-out".into(),
            e.dir.join("stress.json").display().to_string()
        ]
    );
    assert_eq!(s.overwrites, vec![e.dir.join("stress.json")]);
    std::fs::remove_file(e.dir.join("stress.json")).unwrap();
    assert!(
        JobSpec::from_preset(&Preset::Verify, &e, &c)
            .unwrap()
            .overwrites
            .is_empty()
    );
    for p in [Preset::Repair(None), Preset::Repair(Some(vec![2]))] {
        let s = JobSpec::from_preset(&p, &e, &c).unwrap();
        let mut expected = vec![
            dir.clone(),
            "--transcript".into(),
            script.clone(),
            "--episode-id".into(),
            "episode_small".into(),
            "--voices".into(),
            e.voices_path.as_ref().unwrap().display().to_string(),
            "--out-dir".into(),
            out.clone(),
        ];
        if !p.turns().is_empty() {
            expected.extend(["--turns".into(), "2".into()]);
        }
        assert_eq!(args(&s), expected);
        assert!(s.synthesizes);
    }
}
#[test]
fn metadata_and_hash_gates() {
    let tmp = Scratch::new();
    let mut e = tmp.fixture();
    let c = commands();
    assert!(JobSpec::from_preset(&Preset::RegenerateTurns(vec![1]), &e, &c).is_ok());
    std::fs::write(e.script_path.as_ref().unwrap(), "changed").unwrap();
    e = scriptcast_ui::episode::Episode::load(&e.dir).unwrap();
    for p in [
        Preset::RegenerateTurns(vec![1]),
        Preset::Repair(Some(vec![1])),
    ] {
        assert!(
            JobSpec::from_preset(&p, &e, &c)
                .unwrap_err()
                .contains("Script changed")
        );
    }
    assert!(JobSpec::from_preset(&Preset::PrecisionInsert, &e, &c).is_ok());
    e.project_root = None;
    assert!(
        JobSpec::from_preset(&Preset::Repair(None), &e, &c)
            .unwrap_err()
            .contains("scriptcast.toml")
    );
    e.project_root = Some(tmp.0.clone());
    e.voices_path = None;
    assert!(
        JobSpec::from_preset(&Preset::Repair(None), &e, &c)
            .unwrap_err()
            .contains("voices")
    );
}
#[test]
fn ranges_and_display_round_trip() {
    assert_eq!(compact_ranges(&[5, 3, 4, 9, 3]), "3-5,9");
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let mut s = JobSpec::from_preset(&Preset::DetectChanges, &e, &commands()).unwrap();
    s.program = "/usr/bin/printf".into();
    s.args = vec![
        "%s\\n".into(),
        "path with spaces / and 'quote' $notexpanded".into(),
    ];
    let output = std::process::Command::new("/bin/sh")
        .args(["-c", &s.display()])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "path with spaces / and 'quote' $notexpanded\n"
    );
}
fn process_spec(program: &str, args: Vec<OsString>) -> (Scratch, JobSpec) {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let mut s = JobSpec::from_preset(&Preset::DetectChanges, &e, &commands()).unwrap();
    s.program = program.into();
    s.args = args;
    (tmp, s)
}
#[test]
fn echo_streams_before_exit_and_sleep_cancels() {
    let (_tmp, s) = process_spec("/bin/echo", vec!["streamed".into()]);
    let job = Job::start(&s, eframe::egui::Context::default()).unwrap();
    assert!(matches!(
        job.receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        JobEvent::Line(Stream::Stdout, s) if s == "streamed"
    ));
    assert!(matches!(
        job.receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        JobEvent::Exited(Some(0))
    ));
    let (_tmp, s) = process_spec("/bin/sleep", vec!["10".into()]);
    let job = Job::start(&s, eframe::egui::Context::default()).unwrap();
    let start = Instant::now();
    job.cancel();
    assert!(matches!(
        job.receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
        JobEvent::Exited(_)
    ));
    assert!(start.elapsed() < Duration::from_secs(1));
}
#[test]
fn stdin_is_null_environment_is_injected_and_stderr_drains() {
    let (_tmp, s) = process_spec(
        "/bin/sh",
        vec![
            "-c".into(),
            "read value; test $? -ne 0 || exit 3; printf '%s\\n' \"$PATH\"; printf 'last error\\n' >&2".into(),
        ],
    );
    let job = Job::start(&s, eframe::egui::Context::default()).unwrap();
    let mut lines = vec![];
    loop {
        match job.receiver.recv_timeout(Duration::from_secs(2)).unwrap() {
            JobEvent::Line(_, s) => lines.push(s),
            JobEvent::Exited(code) => {
                assert_eq!(code, Some(0));
                break;
            }
        }
    }
    assert!(lines.contains(&"/usr/bin:/bin".into()));
    assert!(lines.contains(&"last error".into()));
}
#[test]
fn automation_refuses_every_synthesizing_preset() {
    for name in [
        "precision-insert",
        "regenerate-turns",
        "regenerate-selected",
        "repair",
    ] {
        assert!(Preset::safe_run(name).is_err());
    }
    assert_eq!(Preset::safe_run("verify").unwrap(), Preset::Verify);
    assert_eq!(
        Preset::safe_run("detect-changes").unwrap(),
        Preset::DetectChanges
    );
    assert_eq!(
        Preset::safe_run("clip-transcripts").unwrap(),
        Preset::ClipTranscripts
    );
    assert_eq!(
        Preset::safe_run("second-opinion").unwrap(),
        Preset::SecondOpinion
    );
}

#[test]
fn second_opinion_preset_matches_clip_shape() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let c = commands();
    let script = e.script_path.as_ref().unwrap().display().to_string();
    let dir = e.dir.display().to_string();
    let s = JobSpec::from_preset(&Preset::SecondOpinion, &e, &c).unwrap();
    assert_eq!(args(&s), [dir, "--script".into(), script]);
    assert!(!s.synthesizes);
    assert!(s.overwrites.is_empty());
    std::fs::write(e.dir.join("stress_ensemble.json"), "{}").unwrap();
    let s = JobSpec::from_preset(&Preset::SecondOpinion, &e, &c).unwrap();
    assert_eq!(s.overwrites, vec![e.dir.join("stress_ensemble.json")]);
}

#[test]
fn clip_transcripts_preset_matches_read_along_shape() {
    let tmp = Scratch::new();
    let e = tmp.fixture();
    let c = commands();
    let script = e.script_path.as_ref().unwrap().display().to_string();
    let dir = e.dir.display().to_string();
    let s = JobSpec::from_preset(&Preset::ClipTranscripts, &e, &c).unwrap();
    assert_eq!(args(&s), [dir, "--script".into(), script]);
    assert!(!s.synthesizes);
    assert!(s.overwrites.is_empty());
    std::fs::write(e.dir.join("clip_transcripts.json"), "{}").unwrap();
    let s = JobSpec::from_preset(&Preset::ClipTranscripts, &e, &c).unwrap();
    assert_eq!(s.overwrites, vec![e.dir.join("clip_transcripts.json")]);
}

#[test]
fn inherited_pipes_cannot_hold_exit_or_leave_readers_running() {
    let (_tmp, s) = process_spec(
        "/bin/sh",
        vec![
            "-c".into(),
            "sleep 2 & printf 'parent output\\n'; printf 'parent error\\n' >&2".into(),
        ],
    );
    let start = Instant::now();
    let job = Job::start(&s, eframe::egui::Context::default()).unwrap();
    let mut lines = vec![];
    loop {
        match job.receiver.recv_timeout(Duration::from_secs(1)).unwrap() {
            JobEvent::Line(_, line) => lines.push(line),
            JobEvent::Exited(code) => {
                assert_eq!(code, Some(0));
                break;
            }
        }
    }
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(lines.contains(&"parent output".into()));
    assert!(lines.contains(&"parent error".into()));
    assert!(matches!(
        job.receiver.recv_timeout(Duration::from_millis(50)),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
    ));
}

#[test]
fn all_buffered_lines_and_unterminated_tail_precede_exit() {
    let (_tmp, s) = process_spec(
        "/bin/sh",
        vec![
            "-c".into(),
            "i=0; while [ $i -lt 2000 ]; do echo $i; i=$((i+1)); done; printf final >&2".into(),
        ],
    );
    let job = Job::start(&s, eframe::egui::Context::default()).unwrap();
    let mut stdout = vec![];
    let mut stderr = vec![];
    loop {
        match job.receiver.recv_timeout(Duration::from_secs(2)).unwrap() {
            JobEvent::Line(Stream::Stdout, line) => stdout.push(line),
            JobEvent::Line(Stream::Stderr, line) => stderr.push(line),
            JobEvent::Exited(code) => {
                assert_eq!(code, Some(0));
                break;
            }
        }
    }
    assert_eq!(stdout, (0..2000).map(|n| n.to_string()).collect::<Vec<_>>());
    assert_eq!(stderr, ["final"]);
}
