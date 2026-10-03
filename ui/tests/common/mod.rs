#![allow(dead_code)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
pub struct Scratch(pub PathBuf);
impl Scratch {
    pub fn new() -> Self {
        let p = PathBuf::from("/tmp/scriptcast-c").join(format!(
            "test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    pub fn fixture(&self) -> scriptcast_ui::episode::Episode {
        copy_tree(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
            &self.0,
        );
        fs::write(self.0.join("voices.yaml"), "speakers: {}\n").unwrap();
        fs::write(self.0.join("episode_small/episode_small.als"), b"set").unwrap();
        scriptcast_ui::episode::Episode::load(&self.0.join("episode_small")).unwrap()
    }
    pub fn exe(&self, relative: &str, body: &str) -> PathBuf {
        let p = self.0.join(relative);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn copy_tree(from: &Path, to: &Path) {
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
