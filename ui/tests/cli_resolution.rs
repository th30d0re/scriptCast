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
