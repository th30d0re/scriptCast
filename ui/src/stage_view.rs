//! Turn-synced vertical scroll view of Stage stills (R8).
//!
//! The still list is re-derived from `stage_plan.json` exactly as
//! `scriptcast/tools/stage_stills.py:resolve_scene_stills` derives it:
//! per segment, cues are gathered by `shot_id.` prefix (falling back to
//! bare cues inside the segment range), sorted by ms; one still per cue
//! at cue + 700 ms named `<scene>_<cue>.png`, plus one `<scene>_end.png`
//! at segment end − 1 frame. Sync keys on `turn_id` in both directions:
//! selecting a turn scrolls to its first cue frame, and scrolling (or
//! clicking) to a frame selects its turn. Ctrl+R toggles a compact
//! selection mode whose picks are sent through [`crate::router_client`].

use crate::{
    router_client::{Delivery, RouterClient, RouterRequest, SelectionContext},
    theme as t,
};
use eframe::egui::{self, Align, Rect, Sense, Ui, vec2};
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap},
    fs::OpenOptions,
    io::Write,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

/// One still in plan order, with the turn its cue belongs to.
#[derive(Clone, Debug, PartialEq)]
pub struct StageFrame {
    pub scene: String,
    pub shot_id: String,
    pub turn_id: String,
    pub turn_index: usize,
    /// Bare cue name (`intro`, `detail`, or `end` for the end frame).
    pub cue_id: String,
    /// Canonical plan key for cue frames (`<shot_id>.<cue>`); `end` for the end frame.
    pub cue_key: String,
    /// Still time in ms (cue ms + 700, or the segment-end time).
    pub ms: u64,
    pub frame_index: u64,
    pub file_name: String,
    pub path: PathBuf,
}

/// A parsed `stage_plan.json` plus its derived still list.
#[derive(Clone, Debug)]
pub struct StagePlan {
    pub episode: String,
    pub fps: f64,
    pub frames: Vec<StageFrame>,
    pub plan_path: PathBuf,
}

#[derive(Deserialize)]
struct RawPlan {
    episode: String,
    #[serde(default = "default_fps")]
    fps: f64,
    #[serde(default)]
    range: RawRange,
    #[serde(default)]
    cues: HashMap<String, serde_json::Value>,
    #[serde(default)]
    segments: Vec<RawSegment>,
}

fn default_fps() -> f64 {
    30.0
}

#[derive(Default, Deserialize)]
struct RawRange {
    #[serde(default)]
    start_ms: u64,
}

#[derive(Deserialize)]
struct RawSegment {
    #[serde(default)]
    scene: Option<String>,
    #[serde(default)]
    instance: Option<String>,
    #[serde(default)]
    start_ms: u64,
    #[serde(default)]
    end_ms: u64,
    #[serde(default)]
    shots: Vec<RawShot>,
}

#[derive(Deserialize)]
struct RawShot {
    shot_id: String,
    turn_id: String,
    #[serde(default)]
    turn_index: usize,
    #[serde(default)]
    start_ms: u64,
    #[serde(default)]
    end_ms: u64,
    #[serde(default)]
    cues: HashMap<String, serde_json::Value>,
}

fn cue_ms(value: &serde_json::Value) -> Option<u64> {
    value
        .as_f64()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(|v| v as u64)
}

fn frame_fn(ms: u64, range_start_ms: u64, fps: f64) -> u64 {
    if ms <= range_start_ms || fps <= 0.0 {
        return 0;
    }
    ((ms - range_start_ms) as f64 * fps / 1000.0).ceil() as u64
}

fn shot_for_ms(shots: &[RawShot], ms: u64) -> Option<&RawShot> {
    if let Some(shot) = shots.iter().find(|s| s.start_ms <= ms && ms < s.end_ms) {
        return Some(shot);
    }
    if let Some(last) = shots.last().filter(|s| ms >= s.end_ms) {
        return Some(last);
    }
    shots
        .iter()
        .find(|s| ms < s.end_ms)
        .or_else(|| shots.first())
}

/// Derive the ordered still list from a parsed plan, mirroring
/// `resolve_scene_stills`. Segments without shots contribute nothing:
/// a frame with no turn could never sync.
fn derive_frames(raw: &RawPlan, stills_dir: &Path) -> Vec<StageFrame> {
    let fps = if raw.fps > 0.0 { raw.fps } else { 30.0 };
    let range_start = raw.range.start_ms;
    let mut frames = Vec::new();
    for seg in &raw.segments {
        let scene = seg
            .scene
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| seg.instance.clone())
            .unwrap_or_default();
        if scene.is_empty() || seg.shots.is_empty() {
            continue;
        }
        let mut scene_cues: HashMap<String, u64> = HashMap::new();
        for shot in &seg.shots {
            let prefix = format!("{}.", shot.shot_id);
            for (key, value) in &raw.cues {
                if let Some(cue_name) = key.strip_prefix(&prefix)
                    && let Some(ms) = cue_ms(value)
                {
                    scene_cues.insert(cue_name.to_owned(), ms);
                }
            }
            for (cue_name, value) in &shot.cues {
                if scene_cues.contains_key(cue_name) {
                    continue;
                }
                let resolved = cue_ms(value)
                    .or_else(|| {
                        raw.cues
                            .get(&format!("{}.{}", shot.shot_id, cue_name))
                            .and_then(cue_ms)
                    })
                    .or_else(|| raw.cues.get(cue_name).and_then(cue_ms));
                if let Some(ms) = resolved {
                    scene_cues.insert(cue_name.clone(), ms);
                }
            }
        }
        if scene_cues.is_empty() {
            for (key, value) in &raw.cues {
                if !key.contains('.')
                    && let Some(ms) =
                        cue_ms(value).filter(|ms| seg.start_ms <= *ms && *ms <= seg.end_ms)
                {
                    scene_cues.insert(key.clone(), ms);
                }
            }
        }
        let mut sorted: Vec<(String, u64)> = scene_cues.into_iter().collect();
        sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        let end_frame = frame_fn(seg.end_ms, range_start, fps).saturating_sub(1);
        for (cue_name, cue_at) in &sorted {
            let target_ms = cue_at + 700;
            let Some(shot) = shot_for_ms(&seg.shots, *cue_at) else {
                continue;
            };
            let file_name = format!("{scene}_{cue_name}.png");
            frames.push(StageFrame {
                scene: scene.clone(),
                shot_id: shot.shot_id.clone(),
                turn_id: shot.turn_id.clone(),
                turn_index: shot.turn_index,
                cue_id: cue_name.clone(),
                cue_key: format!("{}.{}", shot.shot_id, cue_name),
                ms: target_ms,
                frame_index: frame_fn(target_ms, range_start, fps).min(end_frame),
                path: stills_dir.join(&file_name),
                file_name,
            });
        }
        let last = seg.shots.last().expect("shots checked non-empty");
        let end_ms = (range_start as f64 + end_frame as f64 * 1000.0 / fps).round() as u64;
        let file_name = format!("{scene}_end.png");
        frames.push(StageFrame {
            scene: scene.clone(),
            shot_id: last.shot_id.clone(),
            turn_id: last.turn_id.clone(),
            turn_index: last.turn_index,
            cue_id: "end".into(),
            cue_key: "end".into(),
            ms: end_ms,
            frame_index: end_frame,
            path: stills_dir.join(&file_name),
            file_name,
        });
    }
    frames
}

impl StagePlan {
    /// Load a plan and derive its stills. `stills_dir` defaults to the
    /// plan's own directory, which is where the fixture and the usual
    /// stills output layout keep the PNGs beside the plan.
    pub fn load(plan_path: &Path, stills_dir: Option<&Path>) -> Result<Self, String> {
        let bytes =
            std::fs::read(plan_path).map_err(|e| format!("{}: {e}", plan_path.display()))?;
        let raw: RawPlan =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", plan_path.display()))?;
        if raw.segments.is_empty() {
            return Err(format!(
                "{}: stage plan has no segments",
                plan_path.display()
            ));
        }
        let dir = stills_dir
            .map(Path::to_owned)
            .or_else(|| plan_path.parent().map(Path::to_owned))
            .unwrap_or_else(|| PathBuf::from("."));
        let frames = derive_frames(&raw, &dir);
        if frames.is_empty() {
            return Err(format!(
                "{}: stage plan has no cue frames",
                plan_path.display()
            ));
        }
        Ok(Self {
            episode: raw.episode,
            fps: if raw.fps > 0.0 { raw.fps } else { 30.0 },
            frames,
            plan_path: plan_path.to_owned(),
        })
    }

    /// Frame indices belonging to a turn, in plan order.
    pub fn frames_for_turn(&self, turn_id: &str) -> Vec<usize> {
        self.frames
            .iter()
            .enumerate()
            .filter(|(_, f)| f.turn_id == turn_id)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn turn_for_frame(&self, frame: usize) -> Option<&str> {
        self.frames.get(frame).map(|f| f.turn_id.as_str())
    }

    /// Build the router payload for a set of picked frames: frame order,
    /// turns deduplicated, paths resolved against the stills directory.
    pub fn selection_context(&self, selected: &BTreeSet<usize>) -> SelectionContext {
        let mut turn_ids: Vec<String> = Vec::new();
        let mut cue_ids = Vec::new();
        let mut frame_paths = Vec::new();
        for idx in selected {
            let Some(frame) = self.frames.get(*idx) else {
                continue;
            };
            if !turn_ids.contains(&frame.turn_id) {
                turn_ids.push(frame.turn_id.clone());
            }
            cue_ids.push(frame.cue_id.clone());
            frame_paths.push(frame.path.clone());
        }
        SelectionContext {
            episode: self.episode.clone(),
            turn_ids,
            cue_ids,
            frame_paths,
        }
    }
}

/// First frame whose rect touches the viewport, i.e. the frame the user
/// has scrolled to. Pure so the scroll→turn rule is testable headless.
pub fn first_visible_frame(rects: &[(usize, Rect)], viewport: Rect) -> Option<usize> {
    rects
        .iter()
        .find(|(_, rect)| rect.intersects(viewport))
        .map(|(idx, _)| *idx)
}

#[derive(Clone, Debug, PartialEq)]
pub enum StageAction {
    None,
    OpenKitReview,
    SelectTurn {
        turn_id: String,
        turn_index: usize,
        frame: usize,
    },
    Submitted(SelectionContext),
}

/// Open the local Kit Review site, starting Vite when it is not already listening.
pub fn open_kit_review() -> Result<(), String> {
    let address: SocketAddr = "127.0.0.1:5173".parse().expect("valid local address");
    if TcpStream::connect_timeout(&address, Duration::from_millis(150)).is_err() {
        let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
        let root = cwd
            .ancestors()
            .find(|path| path.join("site/package.json").is_file())
            .ok_or_else(|| "Could not find the ScriptCast site workspace".to_string())?;
        Command::new("npm")
            .args(["run", "dev", "-w", "site", "--", "--host", "127.0.0.1"])
            .current_dir(root)
            .spawn()
            .map_err(|error| format!("Could not start Kit Review site: {error}"))?;
    }
    Command::new("/usr/bin/open")
        .arg("http://localhost:5173/#kit")
        .spawn()
        .map_err(|error| format!("Could not open Kit Review: {error}"))?;
    Ok(())
}

/// Append a rejection in the shared Stage review format used by the CLI.
pub fn append_rejection(
    stage_dir: &Path,
    frame: &StageFrame,
    reason: &str,
    element_id: Option<&str>,
) -> Result<PathBuf, String> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("Rejection reason cannot be empty".into());
    }
    let path = stage_dir.join("review/rejections.jsonl");
    std::fs::create_dir_all(path.parent().expect("review log has a parent"))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut record = serde_json::json!({
        "shot": frame.shot_id,
        "still_path": frame.path,
        "frame": frame.frame_index,
        "reason": reason,
        "ts": crate::router_client::now_string(),
    });
    if let Some(element_id) = element_id.filter(|id| !id.is_empty()) {
        record["element_id"] = serde_json::Value::String(element_id.to_owned());
    }
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    writeln!(
        output,
        "{}",
        serde_json::to_string(&record).expect("rejection record serializes")
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    output
        .flush()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    output
        .sync_data()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(path)
}

fn stage_dir(plan_path: &Path) -> PathBuf {
    plan_path
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "stage"))
        .map(Path::to_path_buf)
        .or_else(|| plan_path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// State for the stage column. Texture handles cache per still path.
pub struct StageView {
    pub plan: StagePlan,
    pub compact: bool,
    pub selected_frames: BTreeSet<usize>,
    pub notice: Option<String>,
    /// Next `RouterRequest` id for selection sends; starts at 1.
    next_request_id: u64,
    scroll_to_frame: Option<usize>,
    synced_turn: Option<String>,
    last_offset: f32,
    textures: HashMap<PathBuf, Option<egui::TextureHandle>>,
    rejecting_frame: Option<usize>,
    rejection_reason: String,
}

impl StageView {
    pub fn new(plan: StagePlan) -> Self {
        Self {
            plan,
            compact: false,
            selected_frames: BTreeSet::new(),
            notice: None,
            next_request_id: 1,
            scroll_to_frame: None,
            synced_turn: None,
            last_offset: 0.0,
            textures: HashMap::new(),
            rejecting_frame: None,
            rejection_reason: String::new(),
        }
    }

    pub fn load(plan_path: &Path, stills_dir: Option<&Path>) -> Result<Self, String> {
        Ok(Self::new(StagePlan::load(plan_path, stills_dir)?))
    }

    pub fn toggle_compact(&mut self) {
        self.compact = !self.compact;
    }

    /// Thumbnail height; compact mode shrinks the view so more cues fit.
    pub fn thumb_height(&self) -> f32 {
        if self.compact { 64.0 } else { 150.0 }
    }

    /// Column width the browser carves out for this view.
    pub fn column_width(&self) -> f32 {
        if self.compact { 180.0 } else { 340.0 }
    }

    /// Record the browser's selected turn. A change scrolls to the
    /// turn's first cue frame; an echo of the already-synced turn (for
    /// example a scroll-driven selection coming back) does nothing.
    pub fn sync_to_turn(&mut self, selected_turn_id: Option<&str>) {
        let turn = selected_turn_id.map(str::to_owned);
        if turn == self.synced_turn {
            return;
        }
        self.synced_turn = turn.clone();
        self.scroll_to_frame = turn
            .as_deref()
            .and_then(|id| self.plan.frames_for_turn(id).into_iter().next());
    }

    pub fn selection_context(&self) -> SelectionContext {
        self.plan.selection_context(&self.selected_frames)
    }
}

fn texture(
    ctx: &egui::Context,
    cache: &mut HashMap<PathBuf, Option<egui::TextureHandle>>,
    path: &Path,
) -> Option<egui::TextureHandle> {
    if let Some(hit) = cache.get(path) {
        return hit.clone();
    }
    let loaded = (|| {
        let bytes = std::fs::read(path).ok()?;
        let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        Some(ctx.load_texture(
            path.display().to_string(),
            color,
            egui::TextureOptions::LINEAR,
        ))
    })();
    cache.insert(path.to_owned(), loaded.clone());
    loaded
}

/// Render the stage column. `selected_turn_id` is the browser's selected
/// turn; `client` receives the selection on Send (a `None` client still
/// returns the `Submitted` action without writing anywhere).
pub fn show(
    ui: &mut Ui,
    view: &mut StageView,
    selected_turn_id: Option<&str>,
    client: Option<&mut dyn RouterClient>,
) -> StageAction {
    if !ui.ctx().text_edit_focused()
        && ui
            .ctx()
            .input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::R))
    {
        view.toggle_compact();
    }
    view.sync_to_turn(selected_turn_id);

    let mut action = StageAction::None;
    ui.label(t::text("Stage stills", t::BODY, t::TEXT_PRIMARY).font(t::bold(t::BODY)));
    ui.label(t::text(
        format!("{} · {} frames", view.plan.episode, view.plan.frames.len()),
        t::SECONDARY,
        t::TEXT_MUTED,
    ));
    if t::secondary_button(ui, "Kit review", None, true).clicked() {
        action = StageAction::OpenKitReview;
    }
    if view.compact {
        ui.label(t::text(
            format!("Selection mode · {} selected", view.selected_frames.len()),
            t::SECONDARY,
            t::TEXT_MUTED,
        ));
        let can_send = !view.selected_frames.is_empty();
        if t::secondary_button(ui, "Send selection", None, can_send).clicked() {
            let context = view.selection_context();
            match client {
                Some(client) => {
                    let episode_dir = view.plan.plan_path.parent().map(Path::to_path_buf);
                    let request = RouterRequest::selection(
                        view.next_request_id,
                        &context,
                        episode_dir,
                        crate::router_client::now_string(),
                    );
                    view.next_request_id += 1;
                    match client.submit(&request) {
                        Delivery::Queued => {
                            view.notice = Some(format!(
                                "Sent {} frame(s) for {} turn(s)",
                                context.frame_paths.len(),
                                context.turn_ids.len()
                            ));
                            action = StageAction::Submitted(context);
                        }
                        Delivery::Failed(error) => {
                            view.notice = Some(format!("Send failed: {error}"));
                        }
                    }
                }
                None => {
                    action = StageAction::Submitted(context);
                }
            }
        }
        if t::secondary_button(ui, "Clear selection", None, can_send).clicked() {
            view.selected_frames.clear();
        }
    } else {
        ui.label(t::text(
            "Ctrl+R to select frames",
            t::SECONDARY,
            t::TEXT_SUBTLE,
        ));
    }
    if let Some(notice) = &view.notice {
        ui.label(t::text(notice.clone(), t::SECONDARY, t::TEXT_MUTED));
    }

    let thumb_h = view.thumb_height();
    let frames: Vec<StageFrame> = view.plan.frames.clone();
    let mut rects: Vec<(usize, Rect)> = Vec::new();
    let mut clicked: Option<usize> = None;
    let mut reject_clicked: Option<usize> = None;
    let output = egui::ScrollArea::vertical()
        .id_salt("stage_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for (idx, frame) in frames.iter().enumerate() {
                let picked = view.selected_frames.contains(&idx);
                let turn_selected = selected_turn_id == Some(frame.turn_id.as_str());
                let width = ui.available_width();
                let (rect, image_response) =
                    ui.allocate_exact_size(vec2(width, thumb_h), Sense::click());
                if let Some(handle) = texture(ui.ctx(), &mut view.textures, &frame.path) {
                    ui.painter().image(
                        handle.id(),
                        rect,
                        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                } else {
                    ui.painter()
                        .rect_filled(rect, t::RADIUS_CONTROL, t::BG_RAISED);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &frame.file_name,
                        t::font(t::SECONDARY),
                        t::TEXT_MUTED,
                    );
                }
                if image_response.clicked() {
                    clicked = Some(idx);
                }
                let label = format!("{} · {} · {}", frame.scene, frame.cue_id, frame.turn_id);
                let selected = if view.compact { picked } else { turn_selected };
                let button = ui.add(egui::Button::new(label).selected(selected));
                if button.clicked() {
                    clicked = Some(idx);
                }
                ui.horizontal(|ui| {
                    // Approval remains a handoff decision; rejection is the
                    // actionable, machine-readable feedback path.
                    let _ = ui.small_button("Approve");
                    if ui.small_button("Reject").clicked() {
                        reject_clicked = Some(idx);
                    }
                });
                rects.push((idx, rect.union(button.rect)));
                if view.scroll_to_frame == Some(idx) {
                    button.scroll_to_me(Some(Align::TOP));
                }
                ui.add_space(t::SPACE_8);
            }
        });
    view.scroll_to_frame = None;

    if let Some(idx) = reject_clicked {
        view.rejecting_frame = Some(idx);
        view.rejection_reason.clear();
    }
    if let Some(idx) = view.rejecting_frame {
        let frame = &frames[idx];
        let mut close = false;
        egui::Window::new("Reject still")
            .id(egui::Id::new("stage_rejection"))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("{} · {}", frame.shot_id, frame.file_name));
                ui.add(
                    egui::TextEdit::singleline(&mut view.rejection_reason)
                        .hint_text("Short reason"),
                );
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    if ui.button("Record rejection").clicked() {
                        match append_rejection(
                            &stage_dir(&view.plan.plan_path),
                            frame,
                            &view.rejection_reason,
                            None,
                        ) {
                            Ok(path) => {
                                view.notice = Some(format!(
                                    "Rejected {} · {}",
                                    frame.file_name,
                                    path.display()
                                ));
                                close = true;
                            }
                            Err(error) => view.notice = Some(format!("Reject failed: {error}")),
                        }
                    }
                });
            });
        if close {
            view.rejecting_frame = None;
            view.rejection_reason.clear();
        }
    }

    // Scroll → turn: only when the user actually scrolled this frame.
    let offset_changed = (output.state.offset.y - view.last_offset).abs() > 0.5;
    view.last_offset = output.state.offset.y;
    if offset_changed && let Some(idx) = first_visible_frame(&rects, output.inner_rect) {
        let frame = &frames[idx];
        if selected_turn_id != Some(frame.turn_id.as_str()) {
            view.synced_turn = Some(frame.turn_id.clone());
            action = StageAction::SelectTurn {
                turn_id: frame.turn_id.clone(),
                turn_index: frame.turn_index,
                frame: idx,
            };
        }
    }

    if let Some(idx) = clicked {
        if view.compact {
            if !view.selected_frames.remove(&idx) {
                view.selected_frames.insert(idx);
            }
        } else {
            let frame = &frames[idx];
            view.synced_turn = Some(frame.turn_id.clone());
            action = StageAction::SelectTurn {
                turn_id: frame.turn_id.clone(),
                turn_index: frame.turn_index,
                frame: idx,
            };
        }
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_plan() -> StagePlan {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stage");
        StagePlan::load(&dir.join("stage_plan.json"), Some(&dir)).unwrap()
    }

    #[test]
    fn derivation_mirrors_stage_stills_naming_timing_and_turns() {
        let plan = fixture_plan();
        assert_eq!(plan.episode, "fixture-ep");
        // Two shot cues + one segment-end frame; the bare aliases in the
        // fixture (`intro`, `detail`) must not double the list.
        assert_eq!(plan.frames.len(), 3);
        let intro = &plan.frames[0];
        assert_eq!(intro.cue_id, "intro");
        assert_eq!(intro.cue_key, "shotA.intro");
        assert_eq!(intro.turn_id, "turn-1");
        assert_eq!(intro.ms, 1700, "cue at 1000 ms + 700 ms");
        assert_eq!(intro.frame_index, 51, "ceil(1700 * 30 / 1000)");
        assert_eq!(intro.file_name, "SceneA_intro.png");
        assert!(intro.path.ends_with("SceneA_intro.png"));
        let detail = &plan.frames[1];
        assert_eq!(detail.cue_id, "detail");
        assert_eq!(detail.turn_id, "turn-2");
        assert_eq!(detail.ms, 3200);
        assert_eq!(detail.frame_index, 96);
        let end = &plan.frames[2];
        assert_eq!(end.cue_id, "end");
        assert_eq!(
            end.turn_id, "turn-2",
            "end frame belongs to the last shot's turn"
        );
        assert_eq!(end.frame_index, 119, "ceil(4000 * 30 / 1000) - 1");
        assert_eq!(end.ms, 3967, "round(119 * 1000 / 30)");
        assert_eq!(end.file_name, "SceneA_end.png");
    }

    #[test]
    fn turn_mapping_and_selection_context_keep_frame_order() {
        let plan = fixture_plan();
        assert_eq!(plan.frames_for_turn("turn-1"), vec![0]);
        assert_eq!(plan.frames_for_turn("turn-2"), vec![1, 2]);
        assert_eq!(plan.frames_for_turn("turn-9"), Vec::<usize>::new());
        assert_eq!(plan.turn_for_frame(2), Some("turn-2"));
        // Picked out of order in the set; context still follows frame order
        // and names turn-2 once for its two frames.
        let context = plan.selection_context(&BTreeSet::from([2, 0, 1]));
        assert_eq!(context.episode, "fixture-ep");
        assert_eq!(context.turn_ids, vec!["turn-1", "turn-2"]);
        assert_eq!(context.cue_ids, vec!["intro", "detail", "end"]);
        assert_eq!(context.frame_paths.len(), 3);
        assert!(context.frame_paths[0].ends_with("SceneA_intro.png"));
        let empty = plan.selection_context(&BTreeSet::new());
        assert!(empty.turn_ids.is_empty() && empty.frame_paths.is_empty());
    }

    #[test]
    fn first_visible_frame_is_the_topmost_intersecting_one() {
        let viewport = Rect::from_min_size(egui::pos2(0.0, 100.0), vec2(200.0, 100.0));
        let rects = vec![
            (
                0,
                Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(200.0, 90.0)),
            ),
            (
                1,
                Rect::from_min_size(egui::pos2(0.0, 95.0), vec2(200.0, 90.0)),
            ),
            (
                2,
                Rect::from_min_size(egui::pos2(0.0, 190.0), vec2(200.0, 90.0)),
            ),
        ];
        assert_eq!(first_visible_frame(&rects, viewport), Some(1));
        let below = Rect::from_min_size(egui::pos2(0.0, 500.0), vec2(200.0, 100.0));
        assert_eq!(first_visible_frame(&rects, below), None);
    }

    #[test]
    fn compact_mode_shrinks_thumbnails_and_column() {
        let mut view = StageView::new(fixture_plan());
        let (normal_h, normal_w) = (view.thumb_height(), view.column_width());
        view.toggle_compact();
        assert!(view.compact);
        assert!(view.thumb_height() < normal_h, "compact thumbnails shrink");
        assert!(view.column_width() < normal_w, "compact column shrinks");
        view.toggle_compact();
        assert!(!view.compact);
    }

    #[test]
    fn rejection_writer_appends_the_shared_record_shape() {
        let temp =
            std::env::temp_dir().join(format!("scriptcast-stage-rejection-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        let mut plan = fixture_plan();
        let frame = plan.frames.remove(0);
        let path = append_rejection(&temp, &frame, "text overlaps", Some("caption"))
            .expect("rejection writes");
        let record: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("rejection log exists"))
                .expect("rejection log is JSONL");
        assert_eq!(record["shot"], frame.shot_id);
        assert_eq!(record["still_path"], frame.path.to_string_lossy().as_ref());
        assert_eq!(record["frame"], frame.frame_index);
        assert_eq!(record["reason"], "text overlaps");
        assert_eq!(record["element_id"], "caption");
        assert!(record["ts"].as_str().is_some_and(|ts| ts.ends_with('Z')));
        let _ = std::fs::remove_dir_all(temp);
    }
}
