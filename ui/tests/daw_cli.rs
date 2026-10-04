use eframe::egui;
use scriptcast_ui::{
    cli::{ExportCommand, ExportSource, find_dev_checkout, resolve_export_with},
    daw::{self, DawEvent, DawTask},
    episode::Episode,
};
use std::{
    ffi::OsStr,
    fs::{self, File, FileTimes},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, UNIX_EPOCH},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = PathBuf::from("/tmp/scriptcast-d/b2").join(format!(
            "test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn episode(&self) -> Episode {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small");
        let dir = self.0.join("episode copy with spaces");
        copy_tree(&fixture, &dir);
        Episode::load(&dir).unwrap()
    }

    fn executable(&self, folder: &str, script: &str) -> PathBuf {
        let dir = self.0.join(folder);
        fs::create_dir_all(&dir).unwrap();
        let program = dir.join("scriptcast-export");
        fs::write(&program, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        program
    }

    fn checkout(&self, name: &str) -> PathBuf {
        fs::write(
            self.0.join("pyproject.toml"),
            format!(
                "[project]\nname = '{name}'\n[project.scripts]\n\
                 scriptcast-export = 'scriptcast.tools.export_session:main'\n"
            ),
        )
        .unwrap();
        let executable = self.0.join("ui/target/release/scriptcast-ui");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        executable
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}
fn mtime(path: &Path, seconds: u64) {
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
        .unwrap();
}
fn fresh_xml(episode: &Episode) {
    fs::write(daw::fcpxml_path(episode), "<fcpxml/>").unwrap();
    mtime(&episode.dir.join("episode_manifest.json"), 100);
    mtime(&daw::fcpxml_path(episode), 101);
}
fn collect(task: DawTask) -> Vec<DawEvent> {
    let mut events = Vec::new();
    loop {
        let event = task
            .receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("task result");
        let done = matches!(event, DawEvent::Finished(_));
        events.push(event);
        if done {
            return events;
        }
    }
}
fn no_op(_: &Path) -> Command {
    Command::new("/usr/bin/true")
}

#[test]
fn live_prefers_existing_render_state_path_then_falls_back() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let fallback = episode.dir.join("episode_small.als");
    let preferred = scratch.0.join("rendered elsewhere.als");
    fs::write(&preferred, "preferred").unwrap();
    fs::write(&fallback, "fallback").unwrap();
    let state_path = episode.dir.join("render_state.json");
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    state["als_path"] = serde_json::json!(preferred);
    fs::write(&state_path, serde_json::to_vec(&state).unwrap()).unwrap();
    let episode = Episode::load(&episode.dir).unwrap();
    assert_eq!(daw::live_target(&episode), Some(preferred.clone()));
    fs::remove_file(preferred).unwrap();
    assert_eq!(daw::live_target(&episode), Some(fallback.clone()));
    fs::remove_file(fallback).unwrap();
    assert_eq!(daw::live_target(&episode), None);
}

#[test]
fn live_without_als_path_uses_episode_set() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    assert!(episode.als_path.is_none());
    assert_eq!(daw::live_target(&episode), None);
    let path = episode.dir.join("episode_small.als");
    fs::write(&path, "set").unwrap();
    assert_eq!(daw::live_target(&episode), Some(path));
}

#[test]
fn opener_argv_preserves_paths_with_spaces_without_launching() {
    let path = Path::new("/tmp/scriptcast-d/b2/episode space/file name.als");
    for (command, bundle) in [
        (daw::live_command(path), "com.ableton.live"),
        (daw::logic_command(path), "com.apple.logic10"),
    ] {
        assert_eq!(command.get_program(), "/usr/bin/open");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [OsStr::new("-b"), OsStr::new(bundle), path.as_os_str()]
        );
    }
}

#[test]
fn freshness_missing_older_equal_newer_and_missing_manifest() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let path = daw::fcpxml_path(&episode);
    assert_eq!(path, episode.dir.join("episode_small.fcpxml"));
    assert!(!daw::fcpxml_is_fresh(&episode));
    fs::write(&path, "xml").unwrap();
    let manifest = episode.dir.join("episode_manifest.json");
    mtime(&manifest, 100);
    for (seconds, expected) in [(99, false), (100, true), (101, true)] {
        mtime(&path, seconds);
        assert_eq!(daw::fcpxml_is_fresh(&episode), expected);
    }
    fs::remove_file(manifest).unwrap();
    assert!(!daw::fcpxml_is_fresh(&episode));
}

#[test]
fn resolver_order_folder_path_checkout_without_changing_environment() {
    let scratch = Scratch::new();
    let exe = scratch.checkout("scriptcast");
    let folder = scratch.executable("chosen bin", "exit 0");
    let path_program = scratch.executable("path bin", "exit 0");
    let path = std::env::join_paths([path_program.parent().unwrap()]).unwrap();
    let resolve = || resolve_export_with(folder.parent(), Some(&path), Some(&exe)).unwrap();
    let result = resolve();
    assert_eq!(result.source, ExportSource::Folder);
    assert_eq!(result.program, folder);
    assert!(result.description().starts_with("CLI folder:"));
    fs::remove_file(&folder).unwrap();
    let result = resolve();
    assert_eq!(result.source, ExportSource::Path);
    assert_eq!(result.program, path_program);
    assert!(result.description().starts_with("PATH:"));
    fs::remove_file(path_program).unwrap();
    let result = resolve();
    assert_eq!(result.source, ExportSource::DevCheckout(scratch.0.clone()));
    assert!(result.description().starts_with("Dev checkout:"));
}

#[test]
fn resolver_skips_non_executable_files_and_empty_setting() {
    let scratch = Scratch::new();
    let program = scratch.executable("bin", "exit 0");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o644)).unwrap();
    let exe = scratch.checkout("scriptcast");
    assert!(matches!(
        resolve_export_with(program.parent(), None, Some(&exe))
            .unwrap()
            .source,
        ExportSource::DevCheckout(_)
    ));
    let path = std::env::join_paths([program.parent().unwrap()]).unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        resolve_export_with(Some(Path::new("")), Some(&path), Some(&exe))
            .unwrap()
            .source,
        ExportSource::Path
    );
}

#[test]
fn checkout_detection_rejects_other_project_and_uses_first_matching_ancestor() {
    let scratch = Scratch::new();
    let exe = scratch.checkout("other-project");
    assert_eq!(find_dev_checkout(&exe), None);
    scratch.checkout("scriptcast");
    let nested = scratch.0.join("ui");
    fs::write(nested.join("pyproject.toml"), "[project]\nname='other'\n").unwrap();
    assert_eq!(find_dev_checkout(&exe), Some(scratch.0.clone()));
    fs::write(
        nested.join("pyproject.toml"),
        "[project]\nname='scriptcast'\n",
    )
    .unwrap();
    assert_eq!(find_dev_checkout(&exe), Some(nested.clone()));
    fs::write(nested.join("pyproject.toml"), "not valid toml").unwrap();
    assert_eq!(find_dev_checkout(&exe), Some(scratch.0.clone()));
}

#[test]
fn resolver_failure_names_all_three_locations() {
    let scratch = Scratch::new();
    let exe = scratch.checkout("other");
    let error = resolve_export_with(Some(&scratch.0), None, Some(&exe)).unwrap_err();
    for expected in [
        "CLI folder",
        "PATH",
        "dev-checkout",
        "scriptcast-export",
        "pyproject.toml",
    ] {
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn export_argv_cwd_and_python_environment() {
    let scratch = Scratch::new();
    let exe = scratch.checkout("scriptcast");
    let resolved = resolve_export_with(None, None, Some(&exe)).unwrap();
    let dir = scratch.0.join("episode with spaces");
    let command = resolved.command(&dir);
    assert_eq!(command.get_program(), "python3");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [
            OsStr::new("-m"),
            OsStr::new("scriptcast.tools.export_session"),
            dir.as_os_str(),
            OsStr::new("--fcpxml")
        ]
    );
    assert_eq!(command.get_current_dir(), Some(dir.as_path()));
    let env: Vec<_> = command.get_envs().collect();
    assert!(env.contains(&(OsStr::new("PYTHONPATH"), Some(scratch.0.as_os_str()))));
    assert!(env.contains(&(OsStr::new("PYTHONDONTWRITEBYTECODE"), Some(OsStr::new("1")))));
    let program = scratch.executable("bin", "exit 0");
    let command = resolve_export_with(program.parent(), None, None)
        .unwrap()
        .command(&dir);
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [dir.as_os_str(), OsStr::new("--fcpxml")]
    );
    assert_eq!(command.get_current_dir(), Some(dir.as_path()));
}

#[test]
fn disabled_reasons_allow_fresh_xml_without_exporter() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let unavailable = Err("Export command unavailable: checked all locations".into());
    assert_eq!(
        daw::live_disabled_reason(None).as_deref(),
        Some(daw::NO_EPISODE)
    );
    assert_eq!(
        daw::live_disabled_reason(Some(&episode)).as_deref(),
        Some(daw::NO_LIVE_SET)
    );
    assert_eq!(
        daw::logic_disabled_reason(None, &unavailable).as_deref(),
        Some(daw::NO_EPISODE)
    );
    assert_eq!(
        daw::logic_disabled_reason(Some(&episode), &unavailable),
        unavailable.as_ref().err().cloned()
    );
    let available = Ok(ExportCommand {
        program: "/unused/export".into(),
        source: ExportSource::Path,
    });
    assert_eq!(daw::logic_disabled_reason(Some(&episode), &available), None);
    fresh_xml(&episode);
    assert_eq!(
        daw::logic_disabled_reason(Some(&episode), &unavailable),
        None
    );
    fs::write(episode.dir.join("episode_small.als"), "set").unwrap();
    assert_eq!(daw::live_disabled_reason(Some(&episode)), None);
}

#[test]
fn real_export_from_dev_checkout_creates_xml_in_copy_and_reports_success() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let exe = checkout.join("ui/target/release/scriptcast-ui");
    let export = resolve_export_with(None, None, Some(&exe)).unwrap();
    assert_eq!(
        export.source,
        ExportSource::DevCheckout(checkout.to_owned())
    );
    let path = daw::fcpxml_path(&episode);
    let expected = path.clone();
    let opened = Arc::new(AtomicBool::new(false));
    let worker_opened = opened.clone();
    let task = DawTask::logic(
        &episode,
        Ok(export),
        move |path| {
            assert_eq!(path, expected);
            assert!(path.is_file());
            worker_opened.store(true, Ordering::SeqCst);
            no_op(path)
        },
        egui::Context::default(),
    );
    let events = collect(task);
    assert_eq!(
        events,
        [
            DawEvent::Progress("Writing episode_small.fcpxml…".into()),
            DawEvent::Finished(Ok("Opened in Logic".into()))
        ]
    );
    let xml = fs::read_to_string(path).unwrap();
    assert!(xml.contains("<fcpxml"));
    assert!(xml.contains("audioChannels=\"1\""));
    assert!(daw::fcpxml_is_fresh(&episode));
    assert!(opened.load(Ordering::SeqCst));
}

#[test]
fn fresh_xml_skips_export_and_opens_even_if_unavailable() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    fresh_xml(&episode);
    let events = collect(DawTask::logic(
        &episode,
        Err("no exporter".into()),
        no_op,
        egui::Context::default(),
    ));
    assert_eq!(events, [DawEvent::Finished(Ok("Opened in Logic".into()))]);
}

#[test]
fn failed_export_reports_stderr_tail_and_never_calls_opener() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let program = scratch.executable(
        "bin",
        "printf 'omitted-line\nline2\nline3\nline4\nline5\nline6\nexport broke\n' >&2\nexit 1",
    );
    let export = resolve_export_with(program.parent(), None, None);
    let events = collect(DawTask::logic(
        &episode,
        export,
        |_| panic!("must not open after failed export"),
        egui::Context::default(),
    ));
    let Some(DawEvent::Finished(Err(error))) = events.last() else {
        panic!("{events:?}");
    };
    assert!(error.contains("export broke"));
    assert!(error.contains("line2"));
    assert!(!error.contains("omitted-line"));
    assert!(!daw::fcpxml_path(&episode).exists());
}

#[test]
fn missing_app_error_is_reported_using_fake_opener() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    fresh_xml(&episode);
    let events = collect(DawTask::logic(
        &episode,
        Err("no exporter".into()),
        |_| {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "printf 'Unable to find application' >&2; exit 1"]);
            command
        },
        egui::Context::default(),
    ));
    let [DawEvent::Finished(Err(error))] = events.as_slice() else {
        panic!("{events:?}");
    };
    assert!(error.contains("Open in Logic failed"));
    assert!(error.contains("Unable to find application"));
}

#[test]
fn successful_export_without_fresh_output_does_not_open() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    let program = scratch.executable("bin", "exit 0");
    let export = resolve_export_with(program.parent(), None, None);
    let events = collect(DawTask::logic(
        &episode,
        export,
        |_| panic!("missing output"),
        egui::Context::default(),
    ));
    assert!(
        matches!(events.last(), Some(DawEvent::Finished(Err(error))) if error.contains("did not create a fresh"))
    );
}

#[test]
fn logic_opener_runs_off_the_calling_thread() {
    let scratch = Scratch::new();
    let episode = scratch.episode();
    fresh_xml(&episode);
    let calling_thread = std::thread::current().id();
    let (release, gate) = std::sync::mpsc::channel();
    let task = DawTask::logic(
        &episode,
        Err("no exporter".into()),
        move |path| {
            assert_ne!(std::thread::current().id(), calling_thread);
            gate.recv_timeout(Duration::from_secs(5)).unwrap();
            no_op(path)
        },
        egui::Context::default(),
    );
    // A blocking constructor would prevent this release and fail the worker.
    release.send(()).unwrap();
    assert_eq!(
        collect(task),
        [DawEvent::Finished(Ok("Opened in Logic".into()))]
    );
}
