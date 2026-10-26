mod common;
use common::Scratch;
use scriptcast_ui::cli::*;
use std::{
    ffi::OsStr,
    fs,
    time::{Duration, Instant},
};
#[test]
fn four_step_resolution_uses_first_venv_with_command() {
    let t = Scratch::new();
    let project = t.0.join("project");
    fs::create_dir_all(project.join(".venv-a/bin")).unwrap();
    let b = t.exe("project/.venv-b/bin/scriptcast", "exit 0");
    let folder = t.exe("configured/scriptcast", "exit 0");
    let path = t.exe("path/scriptcast", "exit 0");
    fs::write(
        t.0.join("pyproject.toml"),
        "[project]\nname='scriptcast'\n[project.scripts]\nscriptcast='scriptcast.__main__:main'\n",
    )
    .unwrap();
    let exe = t.0.join("ui/target/release/app");
    let login = std::env::join_paths([path.parent().unwrap()]).unwrap();
    let r = || {
        resolve_with(
            "scriptcast",
            Some(&project),
            folder.parent(),
            &login,
            Some(&exe),
        )
        .unwrap()
    };
    assert_eq!(r().source, ExportSource::Folder);
    fs::remove_file(folder.clone()).unwrap();
    assert_eq!(r().program, b);
    assert_eq!(r().source, ExportSource::ProjectVenv);
    fs::remove_file(b).unwrap();
    assert_eq!(r().source, ExportSource::Path);
    fs::remove_file(path).unwrap();
    let result = r();
    assert!(matches!(result.source, ExportSource::DevCheckout(_)));
    assert_eq!(result.prefix, ["-m", "scriptcast.__main__"]);
    assert_eq!(result.path, login);
}
#[test]
fn dev_interpreter_skips_missing_dependencies() {
    let t = Scratch::new();
    t.exe("project/.venv-a/bin/python", "exit 1");
    let good = t.exe("project/.venv-b/bin/python", "exit 0");
    fs::write(
        t.0.join("pyproject.toml"),
        "[project]\nname='scriptcast'\n[project.scripts]\nscriptcast-export='custom.export:main'\n",
    )
    .unwrap();
    let r = resolve_with(
        "scriptcast-export",
        Some(&t.0.join("project")),
        None,
        OsStr::new("/usr/bin"),
        Some(&t.0.join("ui/target/app")),
    )
    .unwrap();
    assert_eq!(r.program, good);
    assert_eq!(r.prefix, ["-m", "custom.export"]);
    assert!(
        resolve_with(
            "scriptcast-repair",
            None,
            None,
            OsStr::new(""),
            Some(&t.0.join("ui/app"))
        )
        .unwrap_err()
        .contains("project.scripts")
    );
}
#[test]
fn clip_transcribe_prefers_engine_venv_over_installed_scriptcast() {
    let t = Scratch::new();
    let project = t.0.join("project");
    // Installed scriptcast first alphabetically would win under the old ordering.
    t.exe("project/.venv-voice/bin/scriptcast", "exit 0");
    t.exe(
        "project/.venv-voice/bin/python",
        r#"case "$*" in
  *scriptcast.tools.clip_transcribe*) exit 0 ;;
  *fermion*) exit 1 ;;
  *) exit 1 ;;
esac"#,
    );
    let fermion = t.exe(
        "project/.venv-fermion/bin/python",
        r#"case "$*" in
  *fermion*mlx_whisper*scriptcast.asr.engines*) exit 0 ;;
  *scriptcast.tools.clip_transcribe*) exit 0 ;;
  *) exit 1 ;;
esac"#,
    );
    fs::write(
        t.0.join("pyproject.toml"),
        "[project]\nname='scriptcast'\n[project.scripts]\n\
         scriptcast-clip-transcribe='scriptcast.tools.clip_transcribe:main'\n",
    )
    .unwrap();
    let r = resolve_with(
        "scriptcast-clip-transcribe",
        Some(&project),
        None,
        OsStr::new("/usr/bin"),
        Some(&t.0.join("ui/target/app")),
    )
    .unwrap();
    assert_eq!(r.program, fermion);
    assert_eq!(r.prefix, ["-m", "scriptcast.tools.clip_transcribe"]);
    assert!(matches!(r.source, ExportSource::DevCheckout(_)));
}

#[test]
fn stale_help_is_skipped_and_the_reason_names_the_flag() {
    let t = Scratch::new();
    let project = t.0.join("project");
    let origin = "/Users/emmanuel/Documents/Theory/scriptCast";
    let stale = t.exe(
        "project/.venv-voice/bin/scriptcast",
        &format!("# {origin}\nprintf '%s\\n' 'usage: scriptcast'"),
    );
    let good = t.exe(
        "project/.venv-z/bin/scriptcast",
        "printf '%s\\n' '--progress-jsonl --resume --export-only'",
    );
    let (resolved, skipped) = resolve_with_flags(
        "scriptcast",
        Some(&project),
        None,
        OsStr::new("/no-such-path"),
        None,
        &["--progress-jsonl", "--resume", "--export-only"],
    )
    .unwrap();
    assert_eq!(resolved.program, good);
    assert_eq!(resolved.source, ExportSource::ProjectVenv);
    assert!(skipped.iter().any(|s| s.program == stale));
    let reason = &skipped[0].reason;
    assert!(reason.contains("--progress-jsonl"), "{reason}");
    assert!(reason.contains(".venv-voice"), "{reason}");
    assert!(reason.contains(origin), "{reason}");
    assert!(reason.contains("older scriptCast"), "{reason}");
}

#[test]
fn login_path_shell_noise_timeout_and_failure() {
    let t = Scratch::new();
    let shell = t.exe("shell", "printf 'startup noise\\n/first:/second'");
    let r = read_login_path(
        Some(shell.as_os_str()),
        OsStr::new("/fallback"),
        Duration::from_secs(1),
    );
    assert_eq!(r.path, "/first:/second");
    assert!(r.note.contains("noise"));
    let shell = t.exe("slow", "/bin/sleep 10");
    let start = Instant::now();
    let r = read_login_path(
        Some(shell.as_os_str()),
        OsStr::new("/fallback"),
        Duration::from_millis(100),
    );
    assert_eq!(r.path, "/fallback");
    assert!(r.note.contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(1));
    let shell = t.exe("bad", "exit 1");
    assert_eq!(
        read_login_path(
            Some(shell.as_os_str()),
            OsStr::new("/fallback"),
            Duration::from_secs(1)
        )
        .path,
        "/fallback"
    );
}
