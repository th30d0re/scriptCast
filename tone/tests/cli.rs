use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run(python: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_scriptcast-tone"))
        .env("SCRIPTCAST_TONE_PYTHON", python)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn missing_instruct_fails_before_python_starts_and_writes_nothing() {
    let marker = scratch("python-started");
    let out = scratch("missing-instruct.wav");
    let fake_python = scratch("fake-python.sh");
    let _ = fs::remove_file(&marker);
    let _ = fs::remove_file(&out);
    fs::write(
        &fake_python,
        format!("#!/bin/sh\ntouch '{}'\nexit 99\n", marker.display()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_python).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(&fake_python, permissions).unwrap();

    let result = run(
        &fake_python,
        &["speak", "--text", "Hello.", "--out", out.to_str().unwrap()],
    );
    assert!(!result.status.success());
    assert!(!marker.exists());
    assert!(!out.exists());
}

#[test]
fn blank_instruct_fails_before_python_starts() {
    let marker = scratch("blank-python-started");
    let fake_python = scratch("blank-fake-python.sh");
    let _ = fs::remove_file(&marker);
    fs::write(
        &fake_python,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_python).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(&fake_python, permissions).unwrap();

    let result = run(
        &fake_python,
        &[
            "speak",
            "--text",
            "Hello.",
            "--instruct",
            "  ",
            "--out",
            "x.wav",
        ],
    );
    assert!(!result.status.success());
    assert!(!marker.exists());
}
