//! The YouTube frame in the inspector. A run clip
//! (`video/runs/3840x2160@30` or `1920x1080@30`) plays on the episode clock.
//! A still is the stand-in until that frame is decoded, and the only picture
//! when the run is missing. A missing file is an empty slot, not a banner.

use crate::theme as t;
use eframe::egui::{self, Align, Align2, Layout, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// How large a run is decoded for the picture. The file may be 4K; the
/// preview is not. 1080p is the sharp check. 720p is the lighter one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preview {
    /// Follow the on-screen box, and never decode above 1280×720.
    P720,
    /// Decode at 1920×1080 even when the stage is smaller. A 4K file
    /// scales down to this instead of playing at 4K.
    #[default]
    P1080,
}

impl Preview {
    /// The box ffmpeg scales into. 1080p ignores a smaller stage so the
    /// animation is actually full HD. 720p stays inside the stage.
    pub fn decode_size(self, view_w: f32, view_h: f32) -> (f32, f32) {
        match self {
            Preview::P720 => (view_w.clamp(2.0, 1280.0), view_h.clamp(2.0, 720.0)),
            Preview::P1080 => (1920.0, 1080.0),
        }
    }
}

const REVEAL_S: f64 = 0.25;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shot {
    pub id: String,
    pub seg: String,
    index: usize,
    holds: Option<String>,
    /// The shot list's Render line, such as `ext:…png` or `manim:…`.
    render: String,
}

impl Shot {
    pub fn primary(&self) -> &str {
        self.holds.as_deref().unwrap_or(&self.id)
    }

    pub fn is_hold(&self) -> bool {
        self.holds.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Index {
    pub shots: Vec<Shot>,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub shot_id: String,
    pub primary: String,
    pub hold: bool,
    pub duration_ms: u64,
    pub playhead_ms: u64,
    pub still: Option<PathBuf>,
    /// Silent run for this shot's primary, when the 30 fps deliverable is on disk.
    pub clip: Option<PathBuf>,
    /// Offset into `clip`. Zero at the primary turn, so a hold keeps playing the same run.
    pub clip_ms: u64,
}

#[derive(Clone, Debug, Default)]
pub enum Source {
    #[default]
    Absent,
    Ready(Index),
    Failed(String),
}

/// `youtube.shots` from the project file. `Ok(None)` when the table is absent.
pub fn shots_path(root: &Path) -> Result<Option<PathBuf>, String> {
    let path = root.join("scriptcast.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let Some(shots) = value.get("youtube").and_then(|table| table.get("shots")) else {
        return Ok(None);
    };
    let Some(shots) = shots.as_str().filter(|s| !s.trim().is_empty()) else {
        return Err(format!("{}: youtube.shots must be a path", path.display()));
    };
    let raw = PathBuf::from(shots);
    let resolved = if raw.is_absolute() {
        raw
    } else {
        root.join(raw)
    };
    if resolved.is_file() {
        Ok(Some(resolved))
    } else {
        Err(format!("Shot list not found: {}", resolved.display()))
    }
}

pub fn load_index(root: &Path) -> Result<Source, String> {
    match shots_path(root)? {
        Some(path) => Ok(Source::Ready(Index {
            shots: load_shots(&path)?,
        })),
        None => Ok(Source::Absent),
    }
}

pub fn load_shots(path: &Path) -> Result<Vec<Shot>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut shots = Vec::new();
    let mut heading: Option<(String, String)> = None;
    let mut body = String::new();
    for line in text.lines() {
        if let Some(next) = shot_heading(line) {
            if let Some((id, seg)) = heading.take() {
                push_shot(&mut shots, id, seg, &body);
            }
            heading = Some(next);
            body.clear();
        } else if heading.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((id, seg)) = heading {
        push_shot(&mut shots, id, seg, &body);
    }
    if shots.is_empty() {
        return Err(format!("{}: no shots", path.display()));
    }
    Ok(shots)
}

fn shot_heading(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("## ")?;
    let id: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let bytes = id.as_bytes();
    if bytes.len() < 6 || bytes[0] != b'S' {
        return None;
    }
    let seg = id[1..3].to_owned();
    if !seg.bytes().all(|b| b.is_ascii_digit()) || bytes[3] != b'-' {
        return None;
    }
    if !bytes[4..6].iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if bytes.len() == 7 && !bytes[6].is_ascii_lowercase() {
        return None;
    }
    if bytes.len() > 7 {
        return None;
    }
    Some((id, seg))
}

fn push_shot(shots: &mut Vec<Shot>, id: String, seg: String, body: &str) {
    let holds = field(body, "Holds");
    shots.push(Shot {
        id,
        seg,
        index: shots.len(),
        holds: (!holds.is_empty()).then_some(holds),
        render: field(body, "Render"),
    });
}

fn field(body: &str, name: &str) -> String {
    let marker = format!("**{name}:**");
    for line in body.lines() {
        if let Some((_, rest)) = line.split_once(&marker) {
            return rest.trim().trim_matches('`').to_owned();
        }
    }
    String::new()
}

/// The frame covering `turn`, at `playhead_ms` on the episode clock.
pub fn at(
    index: &Index,
    windows: &[(u64, u64)],
    turn: usize,
    playhead_ms: u64,
    episode_dir: &Path,
    youtube: bool,
) -> Option<Frame> {
    if windows.len() != index.shots.len() {
        return None;
    }
    let shot = index.shots.get(turn)?;
    let primary = shot.primary().to_owned();
    let beat = beat_index(&index.shots, windows, &primary, playhead_ms);
    let origin = run_origin(&index.shots, windows, &primary);
    Some(Frame {
        shot_id: shot.id.clone(),
        primary: primary.clone(),
        hold: shot.is_hold(),
        duration_ms: shot_duration(windows, turn),
        playhead_ms,
        still: still(
            episode_dir,
            &primary,
            &shot.id,
            beat,
            youtube,
            &shot.render,
        ),
        // A paper scan is the picture. The conformed clip is the same page on a
        // dark field, and swapping the two makes the window pulse bright and dim.
        clip: (youtube && !shot.render.starts_with("ext:"))
            .then(|| run_clip(episode_dir, &primary))
            .flatten(),
        clip_ms: playhead_ms.saturating_sub(origin),
    })
}

/// Same rounding as the cutter: `floor(seconds × 30 + 0.5)`.
pub fn frame_index(ms: u64) -> u64 {
    ms.saturating_mul(30).saturating_add(500) / 1000
}

fn run_origin(shots: &[Shot], windows: &[(u64, u64)], primary: &str) -> u64 {
    shots
        .iter()
        .find(|shot| shot.primary() == primary)
        .map(|shot| windows[shot.index].0)
        .unwrap_or(0)
}

fn run_clip(dir: &Path, primary: &str) -> Option<PathBuf> {
    ["3840x2160@30", "1920x1080@30"].into_iter().find_map(|res| {
        let path = dir.join(format!("video/runs/{res}/{primary}.mp4"));
        path.is_file().then_some(path)
    })
}

fn shot_duration(windows: &[(u64, u64)], index: usize) -> u64 {
    let start = windows[index].0;
    if let Some(next) = windows.get(index + 1) {
        next.0.saturating_sub(start)
    } else {
        windows[index].1.saturating_sub(start)
    }
}

fn beat_index(shots: &[Shot], windows: &[(u64, u64)], primary: &str, playhead_ms: u64) -> usize {
    let run: Vec<&Shot> = shots.iter().filter(|shot| shot.primary() == primary).collect();
    if run.is_empty() {
        return 0;
    }
    let t0 = windows[run[0].index].0 as f64 / 1000.0;
    let offset = playhead_ms as f64 / 1000.0 - t0;
    let mut chosen = 0;
    for (i, shot) in run.iter().enumerate().skip(1) {
        let reveal = windows[shot.index].0 as f64 / 1000.0 - t0 + REVEAL_S;
        if offset + 1e-6 >= reveal {
            chosen = i;
        }
    }
    chosen
}

fn still(
    dir: &Path,
    primary: &str,
    shot_id: &str,
    beat: usize,
    youtube: bool,
    render: &str,
) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if youtube {
        candidates.push(dir.join(format!("video/figcraft/{primary}/beat-{beat:02}.png")));
        // A chart run has one poster. Later beats keep it until the clip frame arrives.
        if render.starts_with("manim:") && beat > 0 {
            candidates.push(dir.join(format!("video/figcraft/{primary}/beat-00.png")));
        }
        if let Some(page) = ext_image(dir, render) {
            candidates.push(page);
        }
        candidates.push(dir.join(format!("video/preview/1920x1080/{shot_id}.png")));
        candidates.push(dir.join(format!("video/preview/960x540/{shot_id}.png")));
    } else {
        candidates.push(dir.join(format!("video/preview/1080x1920/{shot_id}.png")));
        candidates.push(dir.join(format!("video/reel/{shot_id}.png")));
    }
    candidates.into_iter().find(|path| path.is_file())
}

/// `ext:` paths in the shot list are relative to the project, which may be
/// the episode directory or a parent of it.
fn ext_image(episode_dir: &Path, render: &str) -> Option<PathBuf> {
    let rel = render.strip_prefix("ext:")?.trim();
    let rel = Path::new(rel);
    if rel.is_absolute()
        || rel
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return None;
    }
    let mut dir = episode_dir.to_path_buf();
    for _ in 0..6 {
        let candidate = dir.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn clock(ms: u64) -> String {
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) as f64 / 1000.0;
    format!("{minutes:02}:{seconds:06.3}")
}

pub fn seconds(ms: u64) -> String {
    format!("{:.3} s", ms as f64 / 1000.0)
}

/// Width of the picture stage. The default is most of the row. A dragged
/// width stays between a clear majority and enough room for the turn list.
pub fn stage_width(row: f32, custom: Option<f32>) -> f32 {
    let (min, max) = stage_bounds(row);
    custom.unwrap_or(row * 0.72).clamp(min, max)
}

pub fn stage_bounds(row: f32) -> (f32, f32) {
    if row <= 0.0 {
        return (0.0, 0.0);
    }
    let max = (row * 0.85).min((row - 320.0).max(row * 0.5));
    let min = (row * 0.60).min(max);
    (min, max)
}

pub(crate) fn fit_16_9(max_w: f32, max_h: f32) -> (f32, f32) {
    let max_w = max_w.max(1.0);
    let mut width = max_w;
    let mut height = width * 9.0 / 16.0;
    if max_h.is_finite() && height > max_h {
        height = max_h.max(1.0);
        width = height * 16.0 / 9.0;
    }
    (width, height)
}

/// Draw the slot. `motion` is the run frame under the playhead, when one has
/// been decoded. `youtube` only chooses which files the next frame reads.
pub fn show(
    ui: &mut Ui,
    frame: &Frame,
    youtube: &mut bool,
    texture: &mut Option<(PathBuf, SystemTime, egui::TextureHandle)>,
    motion: Option<&egui::TextureHandle>,
    file_px: Option<[usize; 2]>,
    preview: &mut Preview,
) {
    ui.horizontal(|ui| {
        let youtube_label = "YouTube  1920×1080";
        let reel_label = "Reel  1080×1920";
        let youtube_button = if *youtube {
            t::primary_button(ui, youtube_label, None, true)
        } else {
            t::secondary_button(ui, youtube_label, None, true)
        };
        if youtube_button.clicked() {
            *youtube = true;
        }
        let reel_button = if *youtube {
            t::secondary_button(ui, reel_label, None, true)
        } else {
            t::primary_button(ui, reel_label, None, true)
        };
        if reel_button.clicked() {
            *youtube = false;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            for (setting, label) in [(Preview::P1080, "1080p"), (Preview::P720, "720p")] {
                let response = if *preview == setting {
                    t::primary_button(ui, label, None, true)
                } else {
                    t::secondary_button(ui, label, None, true)
                };
                if response.clicked() {
                    *preview = setting;
                }
            }
            ui.label(t::text("Preview", t::FIELD_LABEL, t::TEXT_MUTED).font(t::bold(t::FIELD_LABEL)));
        });
    });
    let caption_h = t::BODY + t::SPACE_8;
    let avail_h = ui.available_height() - caption_h - t::SPACE_8;
    let (width, height) = fit_16_9(ui.available_width(), avail_h);
    let mut drawn_size = file_px;
    if let Some(handle) = motion {
        drawn_size = drawn_size.or(Some(handle.size()));
        ui.add(
            egui::Image::from_texture(handle)
                .maintain_aspect_ratio(true)
                .fit_to_exact_size(vec2(width, height))
                .corner_radius(t::RADIUS_CONTROL),
        );
    } else if let Some(path) = frame.still.clone()
        && let Some(handle) = cached(ui.ctx(), texture, &path)
    {
        drawn_size = Some(handle.size());
        ui.add(
            egui::Image::from_texture(&handle)
                .maintain_aspect_ratio(true)
                .fit_to_exact_size(vec2(width, height))
                .corner_radius(t::RADIUS_CONTROL),
        );
    } else {
        empty(ui, frame, width, height);
    }
    if frame.hold {
        ui.label(t::text("SHOT", t::FIELD_LABEL, t::TEXT_MUTED).font(t::bold(t::FIELD_LABEL)));
        let size = drawn_size
            .map(|[w, h]| format!("  ·  {w}×{h}"))
            .unwrap_or_default();
        ui.label(t::text(
            format!("{}{size}  ·  holds {}", frame.shot_id, frame.primary),
            t::BODY,
            t::TEXT_PRIMARY,
        ));
    } else if motion.is_some() || frame.still.is_some() {
        let size = drawn_size
            .map(|[w, h]| format!("  ·  {w}×{h}"))
            .unwrap_or_default();
        ui.label(t::text(
            format!("{}{size}  ·  {}", frame.shot_id, clock(frame.playhead_ms)),
            t::SECONDARY,
            t::TEXT_MUTED,
        ));
    }
}

fn cached(
    ctx: &egui::Context,
    texture: &mut Option<(PathBuf, SystemTime, egui::TextureHandle)>,
    path: &Path,
) -> Option<egui::TextureHandle> {
    let modified = std::fs::metadata(path).and_then(|meta| meta.modified()).ok()?;
    if let Some((cached_path, cached_time, handle)) = texture
        && cached_path == path
        && *cached_time == modified
    {
        return Some(handle.clone());
    }
    let bytes = std::fs::read(path).ok()?;
    let image = fit_still(image::load_from_memory(&bytes).ok()?);
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    let handle = ctx.load_texture(path.display().to_string(), color, egui::TextureOptions::LINEAR);
    *texture = Some((path.to_owned(), modified, handle.clone()));
    Some(handle)
}

/// A scan can be several thousand pixels on a side. Uploading that stalls the
/// window and it flashes. Cap the long edge.
fn fit_still(image: image::DynamicImage) -> image::RgbaImage {
    const MAX_EDGE: u32 = 1920;
    let (w, h) = (image.width(), image.height());
    if w <= MAX_EDGE && h <= MAX_EDGE {
        return image.to_rgba8();
    }
    let scale = MAX_EDGE as f32 / w.max(h) as f32;
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    image
        .resize(nw, nh, image::imageops::FilterType::Triangle)
        .to_rgba8()
}

fn empty(ui: &mut Ui, frame: &Frame, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, t::RADIUS_CONTROL, t::BG_WINDOW);
    dash_rect(painter, rect, Stroke::new(1.0, t::BORDER_DEFAULT));
    let center = rect.center();
    painter.text(
        pos2(center.x, center.y - 18.0),
        Align2::CENTER_CENTER,
        &frame.shot_id,
        t::bold(t::BODY),
        t::TEXT_PRIMARY,
    );
    painter.text(
        center,
        Align2::CENTER_CENTER,
        seconds(frame.duration_ms),
        t::font(t::BODY),
        t::TEXT_MUTED,
    );
    painter.text(
        pos2(center.x, center.y + 18.0),
        Align2::CENTER_CENTER,
        "No picture yet",
        t::font(t::BODY),
        t::TEXT_SUBTLE,
    );
}

fn dash_rect(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    dash(painter, rect.left_top(), rect.right_top(), stroke);
    dash(painter, rect.right_top(), rect.right_bottom(), stroke);
    dash(painter, rect.right_bottom(), rect.left_bottom(), stroke);
    dash(painter, rect.left_bottom(), rect.left_top(), stroke);
}

fn dash(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, stroke: Stroke) {
    let delta = to - from;
    let length = delta.length();
    if length < 1.0 {
        return;
    }
    let step = delta / length;
    let mut cursor = 0.0;
    while cursor < length {
        let end = (cursor + 4.0).min(length);
        painter.line_segment([from + step * cursor, from + step * end], stroke);
        cursor += 8.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "scriptcast-picture-{n}-{:?}",
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_png(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([15, 27, 51, 255]));
        image.save(path).unwrap();
    }

    const LIST: &str = "\
## S00-02 — Dictionary order
- **Type:** card
- **Render:** `figma-card:cards/S00-06.json`

## S00-03 — Sense 2
- **Type:** hold
- **Render:** `figma-card:cards/S00-06.json`
- **Holds:** `S00-02`
";

    #[test]
    fn a_hold_names_its_primary_and_the_second_beat_waits_for_the_reveal() {
        let dir = scratch();
        let list = dir.join("shots.md");
        std::fs::write(&list, LIST).unwrap();
        let shots = load_shots(&list).unwrap();
        assert_eq!(shots[1].primary(), "S00-02");
        assert!(shots[1].is_hold());
        let windows = [(13_150, 26_000), (26_010, 39_270)];
        let index = Index { shots };
        let early = at(&index, &windows, 0, 14_000, &dir, true).unwrap();
        assert_eq!(early.shot_id, "S00-02");
        assert!(early.still.is_none());
        write_png(&dir.join("video/figcraft/S00-02/beat-00.png"));
        write_png(&dir.join("video/figcraft/S00-02/beat-01.png"));
        let first = at(&index, &windows, 0, 14_000, &dir, true).unwrap();
        assert!(first.still.unwrap().ends_with("beat-00.png"));
        // The next item is revealed a quarter second after the hold turn starts.
        let at_boundary = at(&index, &windows, 1, 26_010, &dir, true).unwrap();
        assert!(at_boundary.still.unwrap().ends_with("beat-00.png"));
        let second = at(&index, &windows, 1, 26_260, &dir, true).unwrap();
        assert!(second.hold);
        assert_eq!(second.primary, "S00-02");
        assert!(second.still.unwrap().ends_with("beat-01.png"));
        let reel = at(&index, &windows, 0, 14_000, &dir, false).unwrap();
        assert!(reel.still.is_none());
        write_png(&dir.join("video/preview/1080x1920/S00-02.png"));
        let reel = at(&index, &windows, 0, 14_000, &dir, false).unwrap();
        assert!(reel.still.unwrap().ends_with("1080x1920/S00-02.png"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_slot_stays_absent_until_the_project_names_a_shot_list() {
        let dir = scratch();
        std::fs::write(dir.join("scriptcast.toml"), "tail_ms = 400\n").unwrap();
        assert!(matches!(load_index(&dir).unwrap(), Source::Absent));
        std::fs::write(dir.join("shots.md"), LIST).unwrap();
        std::fs::write(
            dir.join("scriptcast.toml"),
            "tail_ms = 400\n\n[youtube]\nshots = \"shots.md\"\n",
        )
        .unwrap();
        let Source::Ready(index) = load_index(&dir).unwrap() else {
            panic!("shot list should load");
        };
        assert_eq!(index.shots.len(), 2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_run_clip_is_timed_from_the_primary_even_on_a_hold() {
        let dir = scratch();
        let list = dir.join("shots.md");
        std::fs::write(&list, LIST).unwrap();
        let clip = dir.join("video/runs/1920x1080@30/S00-02.mp4");
        if let Some(parent) = clip.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&clip, b"clip").unwrap();
        let shots = load_shots(&list).unwrap();
        let windows = [(13_150, 26_000), (26_010, 39_270)];
        let index = Index { shots };
        let first = at(&index, &windows, 0, 14_000, &dir, true).unwrap();
        assert_eq!(first.clip_ms, 850);
        assert_eq!(frame_index(850), 26);
        assert_eq!(first.clip.as_deref(), Some(clip.as_path()));
        let four_k = dir.join("video/runs/3840x2160@30/S00-02.mp4");
        if let Some(parent) = four_k.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&four_k, b"4k").unwrap();
        let preferred = at(&index, &windows, 0, 14_000, &dir, true).unwrap();
        assert_eq!(preferred.clip.as_deref(), Some(four_k.as_path()));
        let hold = at(&index, &windows, 1, 26_260, &dir, true).unwrap();
        assert_eq!(hold.clip_ms, 13_110);
        assert_eq!(hold.clip.as_deref(), Some(four_k.as_path()));
        let reel = at(&index, &windows, 0, 14_000, &dir, false).unwrap();
        assert!(reel.clip.is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_picture_stage_takes_most_of_the_row() {
        let width = stage_width(1440.0, None);
        assert!(width > 1440.0 * 0.7, "{width}");
        let (min, max) = stage_bounds(1440.0);
        assert!(stage_width(1440.0, Some(200.0)) >= min);
        assert!(stage_width(1440.0, Some(4000.0)) <= max);
        let (fitted_w, fitted_h) = fit_16_9(1000.0, 400.0);
        assert!((fitted_h - 400.0).abs() < 0.1);
        assert!((fitted_w - 400.0 * 16.0 / 9.0).abs() < 0.1);
    }

    #[test]
    fn a_paper_and_a_chart_use_the_shot_list_image() {
        let root = scratch();
        let episode = root.join("episode");
        std::fs::create_dir_all(&episode).unwrap();
        let page = root.join("archive/hening_p226.png");
        write_png(&page);
        let poster = episode.join("video/figcraft/S05-05/beat-00.png");
        write_png(&poster);
        let list = "\
## S10-30 — Primary source scan
- **Render:** `ext:archive/hening_p226.png`

## S10-31 — Primary source scan
- **Render:** `ext:archive/hening_p226.png`
- **Holds:** `S10-30`

## S05-05 — Five tiers
- **Render:** `manim:yt02/tier_cone.py:TierCone`

## S05-06 — The cone holds
- **Render:** `manim:yt02/tier_cone.py:TierCone`
- **Holds:** `S05-05`
";
        let shots = load_shots_from(list);
        let windows = [
            (0, 1_000),
            (10_000, 20_000),
            (30_000, 40_000),
            (40_000, 50_000),
        ];
        let index = Index { shots };
        let paper = at(&index, &windows, 0, 0, &episode, true).unwrap();
        assert_eq!(paper.still.as_deref(), Some(page.as_path()));
        assert!(paper.clip.is_none());
        assert!(paper.clip.is_none());
        let clip_dir = episode.join("video/runs/1920x1080@30");
        std::fs::create_dir_all(&clip_dir).unwrap();
        std::fs::write(clip_dir.join("S10-30.mp4"), b"clip").unwrap();
        let with_clip = at(&index, &windows, 0, 0, &episode, true).unwrap();
        assert!(
            with_clip.clip.is_none(),
            "a paper scan is not swapped for its clip"
        );
        let held = at(&index, &windows, 1, 12_000, &episode, true).unwrap();
        assert_eq!(held.still.as_deref(), Some(page.as_path()));
        let chart = at(&index, &windows, 3, 45_000, &episode, true).unwrap();
        assert_eq!(chart.still.as_deref(), Some(poster.as_path()));
        let _ = std::fs::remove_dir_all(root);
    }

    fn load_shots_from(text: &str) -> Vec<Shot> {
        let dir = scratch();
        let path = dir.join("shots.md");
        std::fs::write(&path, text).unwrap();
        let shots = load_shots(&path).unwrap();
        let _ = std::fs::remove_dir_all(dir);
        shots
    }

    #[test]
    fn the_preview_defaults_to_1080p_and_keeps_4k_off_the_screen() {
        assert_eq!(Preview::default(), Preview::P1080);
        assert_eq!(Preview::P1080.decode_size(900.0, 500.0), (1920.0, 1080.0));
        assert_eq!(Preview::P720.decode_size(1600.0, 900.0), (1280.0, 720.0));
        assert_eq!(Preview::P720.decode_size(800.0, 450.0), (800.0, 450.0));
    }
}
