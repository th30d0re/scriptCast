use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let git_meta = manifest_dir.join("..").join(".git");
    if git_meta.is_dir() {
        println!("cargo:rerun-if-changed={}", git_meta.join("HEAD").display());
    } else {
        println!("cargo:rerun-if-changed={}", git_meta.display());
    }

    let short = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&manifest_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&manifest_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    let value = match short {
        Some(hash) if dirty => format!("{hash}-dirty"),
        Some(hash) => hash,
        None => "unknown".into(),
    };
    println!("cargo:rustc-env=SCRIPTCAST_GIT={value}");
}
