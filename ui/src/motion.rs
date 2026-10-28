//! Plays one silent run under the episode playhead.
//!
//! The file stays at its own size. The texture is the preview the picture row
//! asked for: 1080p, or the lighter 720p. A 4K master is never decoded at 4K.
//!
//! ffmpeg stays open and emits frames in order. A jump backward, or a jump
//! more than a few frames forward, starts it again at the new frame. The
//! inspector keeps the still until the first of those frames arrives.

use crate::picture::frame_index;
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FPS: f64 = 30.0;
/// Larger than 4K, so a bad probe cannot allocate an unbounded frame.
const MAX_EDGE: usize = 7680;
/// Live preview ceiling. The cut itself is not limited by this, and a 4K
/// file is scaled down to it rather than copied onto the GPU at full size.
const PREVIEW_MAX_W: usize = 1920;
const PREVIEW_MAX_H: usize = 1080;
/// Seek again when playback has fallen further behind than this.
const CATCH_UP: u64 = 45;
/// A one-frame step backward is the clock settling, not a scrub. Restarting
/// ffmpeg for it flashes the picture.
const REWIND: u64 = 15;

#[derive(Clone)]
struct Target {
    path: PathBuf,
    frame: u64,
    view_w: usize,
    view_h: usize,
    seq: u64,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            frame: 0,
            view_w: 0,
            view_h: 0,
            seq: 0,
        }
    }
}

struct Decoded {
    path: PathBuf,
    frame: u64,
    width: usize,
    height: usize,
    source: [usize; 2],
    rgba: Arc<Vec<u8>>,
}

struct Shared {
    target: Mutex<Target>,
    latest: Mutex<Option<Decoded>>,
    child: Mutex<Option<Child>>,
    wake: Condvar,
    stop: AtomicBool,
}

struct Pipe {
    path: PathBuf,
    next_frame: u64,
    ended: bool,
    width: usize,
    height: usize,
    view_w: usize,
    view_h: usize,
    source: [usize; 2],
    stdout: std::process::ChildStdout,
}

#[derive(Default)]
pub struct Player {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    shown: Option<(PathBuf, u64, [usize; 2], TextureHandle)>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            target: Mutex::new(Target::default()),
            latest: Mutex::new(None),
            child: Mutex::new(None),
            wake: Condvar::new(),
            stop: AtomicBool::new(false),
        }
    }
}

impl Player {
    /// The frame of `path` at `clip_ms`, scaled into `view_w`×`view_h`.
    /// Asks for another paint while the decoder is still catching the playhead.
    pub fn present(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
        clip_ms: u64,
        view_w: f32,
        view_h: f32,
    ) -> Option<TextureHandle> {
        self.ensure_thread();
        let frame = frame_index(clip_ms);
        let view_w = (view_w.round() as usize).clamp(2, PREVIEW_MAX_W);
        let view_h = (view_h.round() as usize).clamp(2, PREVIEW_MAX_H);
        {
            let mut target = self.shared.target.lock().unwrap();
            let resized = target.view_w.abs_diff(view_w) > 32 || target.view_h.abs_diff(view_h) > 32;
            if target.path != path || target.frame != frame || resized {
                target.path = path.to_owned();
                target.frame = frame;
                if resized || target.view_w == 0 {
                    target.view_w = view_w;
                    target.view_h = view_h;
                }
                target.seq = target.seq.wrapping_add(1);
                self.shared.wake.notify_one();
            }
        }
        let decoded = self.shared.latest.lock().unwrap().as_ref().map(|decoded| {
            (
                decoded.path.clone(),
                decoded.frame,
                decoded.width,
                decoded.height,
                decoded.source,
                Arc::clone(&decoded.rgba),
            )
        });
        let Some((decoded_path, decoded_frame, width, height, source, rgba)) = decoded else {
            ctx.request_repaint_after(Duration::from_millis(33));
            return self.held(path);
        };
        if decoded_path != path {
            ctx.request_repaint_after(Duration::from_millis(33));
            return self.held(path);
        }
        if self
            .shown
            .as_ref()
            .is_some_and(|(shown_path, shown_frame, _, handle)| {
                shown_path == path
                    && *shown_frame == decoded_frame
                    && handle.size() == [width, height]
            })
        {
            if decoded_frame != frame {
                ctx.request_repaint_after(Duration::from_millis(33));
            }
            return self.shown.as_ref().map(|(_, _, _, handle)| handle.clone());
        }
        if rgba.len() != width.saturating_mul(height).saturating_mul(4) {
            return None;
        }
        let image = ColorImage::from_rgba_unmultiplied([width, height], rgba.as_slice());
        let handle = ctx.load_texture("picture-motion", image, TextureOptions::LINEAR);
        self.shown = Some((path.to_owned(), decoded_frame, source, handle.clone()));
        if decoded_frame != frame {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
        Some(handle)
    }

    /// The last frame of this clip, so a seek gap does not fall back to the still.
    fn held(&self, path: &Path) -> Option<TextureHandle> {
        self.shown
            .as_ref()
            .and_then(|(shown_path, _, _, handle)| (shown_path == path).then(|| handle.clone()))
    }

    /// The file's own pixel size, not the smaller texture on screen.
    pub fn source_size(&self) -> Option<[usize; 2]> {
        self.shown.as_ref().map(|(_, _, source, _)| *source)
    }

    /// Stop decoding. The next `present` starts again.
    pub fn idle(&mut self) {
        self.shown = None;
        let mut target = self.shared.target.lock().unwrap();
        if target.path.as_os_str().is_empty() {
            return;
        }
        target.path.clear();
        target.frame = 0;
        target.view_w = 0;
        target.view_h = 0;
        target.seq = target.seq.wrapping_add(1);
        self.shared.wake.notify_one();
    }

    fn ensure_thread(&mut self) {
        if self.thread.is_some() {
            return;
        }
        let shared = Arc::clone(&self.shared);
        self.thread = Some(thread::spawn(move || decode_loop(shared)));
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        self.shared.wake.notify_one();
        stop_child(&self.shared);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn decode_loop(shared: Arc<Shared>) {
    let mut pipe: Option<Pipe> = None;
    while !shared.stop.load(Ordering::Acquire) {
        let target = shared.target.lock().unwrap().clone();
        if target.path.as_os_str().is_empty() || target.view_w == 0 || target.view_h == 0 {
            pipe = None;
            stop_child(&shared);
            wait(&shared, target.seq);
            continue;
        }
        let restart = match &pipe {
            None => true,
            Some(pipe) => {
                pipe.path != target.path
                    || pipe.view_w.abs_diff(target.view_w) > 32
                    || pipe.view_h.abs_diff(target.view_h) > 32
                    || (!pipe.ended && target.frame.saturating_add(REWIND) < pipe.next_frame)
                    || (!pipe.ended && target.frame > pipe.next_frame.saturating_add(CATCH_UP))
                    || (pipe.ended && target.frame.saturating_add(REWIND) < pipe.next_frame)
            }
        };
        if restart {
            pipe = None;
            stop_child(&shared);
            if shared.stop.load(Ordering::Acquire) {
                break;
            }
            match open_pipe(&target.path, target.frame, target.view_w, target.view_h) {
                Some((child, stdout, width, height, source)) => {
                    *shared.child.lock().unwrap() = Some(child);
                    if shared.stop.load(Ordering::Acquire) {
                        stop_child(&shared);
                        break;
                    }
                    pipe = Some(Pipe {
                        path: target.path.clone(),
                        next_frame: target.frame,
                        ended: false,
                        width,
                        height,
                        view_w: target.view_w,
                        view_h: target.view_h,
                        source,
                        stdout,
                    });
                }
                None => {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
            }
        }
        let Some(pipe) = pipe.as_mut() else {
            wait(&shared, target.seq);
            continue;
        };
        if pipe.ended || pipe.next_frame > target.frame {
            wait(&shared, target.seq);
            continue;
        }
        let mut rgba = vec![0; pipe.width * pipe.height * 4];
        match pipe.stdout.read_exact(&mut rgba) {
            Ok(()) => {
                let frame = pipe.next_frame;
                pipe.next_frame += 1;
                *shared.latest.lock().unwrap() = Some(Decoded {
                    path: pipe.path.clone(),
                    frame,
                    width: pipe.width,
                    height: pipe.height,
                    source: pipe.source,
                    rgba: Arc::new(rgba),
                });
            }
            Err(_) => {
                pipe.ended = true;
                stop_child(&shared);
            }
        }
    }
    stop_child(&shared);
}

fn wait(shared: &Shared, seq: u64) {
    let guard = shared.target.lock().unwrap();
    if guard.seq != seq || shared.stop.load(Ordering::Acquire) {
        return;
    }
    let _ = shared.wake.wait_timeout(guard, Duration::from_millis(40));
}

fn open_pipe(
    path: &Path,
    frame: u64,
    view_w: usize,
    view_h: usize,
) -> Option<(Child, std::process::ChildStdout, usize, usize, [usize; 2])> {
    let (source_w, source_h) = frame_size(path)?;
    let (width, height) = preview_box(source_w, source_h, view_w, view_h);
    let seconds = format!("{:.3}", frame as f64 / FPS);
    let scale = format!("scale={width}:{height}:flags=fast_bilinear");
    let mut child = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-ss")
        .arg(seconds)
        .arg("-i")
        .arg(path)
        .arg("-an")
        .arg("-vf")
        .arg(scale)
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgba")
        .arg("pipe:1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    Some((child, stdout, width, height, [source_w, source_h]))
}

/// Fit the source inside the view, never enlarging it and never above 1080p.
/// A zero view means the source size, which the decode test uses.
fn preview_box(src_w: usize, src_h: usize, max_w: usize, max_h: usize) -> (usize, usize) {
    if src_w == 0 || src_h == 0 {
        return (2, 2);
    }
    if max_w == 0 || max_h == 0 {
        return (src_w, src_h);
    }
    let max_w = max_w.min(PREVIEW_MAX_W).min(src_w).max(2);
    let max_h = max_h.min(PREVIEW_MAX_H).min(src_h).max(2);
    let mut width = max_w;
    let mut height = width.saturating_mul(src_h) / src_w;
    if height > max_h || height == 0 {
        height = max_h;
        width = height.saturating_mul(src_w) / src_h;
    }
    (width.max(2), height.max(2))
}

fn frame_size(path: &Path) -> Option<(usize, usize)> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=width,height")
        .arg("-of")
        .arg("csv=p=0")
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let mut parts = text.trim().split(',');
    let width: usize = parts.next()?.parse().ok()?;
    let height: usize = parts.next()?.parse().ok()?;
    if width == 0 || height == 0 || width > MAX_EDGE || height > MAX_EDGE {
        return None;
    }
    width.checked_mul(height)?.checked_mul(4)?;
    Some((width, height))
}

fn stop_child(shared: &Shared) {
    if let Some(mut child) = shared.child.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_red_run_decodes_a_red_frame() {
        let ffmpeg = Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if ffmpeg.map(|status| !status.success()).unwrap_or(true) {
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "scriptcast-motion-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("red.mp4");
        let made = Command::new("ffmpeg")
            .arg("-y")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("color=c=red:s=320x180:r=30:d=0.2")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(made.success());
        let (mut child, mut stdout, width, height, source) = open_pipe(&path, 0, 0, 0).unwrap();
        assert_eq!(source, [320, 180]);
        assert_eq!((width, height), (320, 180));
        let mut rgba = vec![0; width * height * 4];
        stdout.read_exact(&mut rgba).unwrap();
        let _ = child.kill();
        let _ = child.wait();
        assert!(
            rgba[0] > 180 && rgba[1] < 80 && rgba[2] < 80,
            "pixel {:?}",
            &rgba[..4]
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_preview_stays_inside_the_stage_and_does_not_upscale() {
        assert_eq!(preview_box(1920, 1080, 1000, 500), (888, 500));
        assert_eq!(preview_box(3840, 2160, 1920, 1080), (1920, 1080));
        assert_eq!(preview_box(3840, 2160, 1280, 720), (1280, 720));
        assert_eq!(preview_box(3840, 2160, 2000, 1200), (1920, 1080));
        assert_eq!(preview_box(320, 180, 1000, 1000), (320, 180));
    }
}
