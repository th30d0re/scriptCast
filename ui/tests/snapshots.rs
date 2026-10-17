//! Headless snapshots of the states `capture.rs` already knows how to open.
use std::ffi::OsString;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use scriptcast_ui::Browser;

const SIZE: eframe::egui::Vec2 = eframe::egui::vec2(1440.0, 900.0);

fn fixture(name: &str) -> OsString {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .into_os_string()
}

struct Slot {
    fonts: bool,
    app: Option<Browser>,
}

fn harness(args: Vec<OsString>) -> Harness<'static, Slot> {
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .with_pixels_per_point(1.0)
        .wgpu()
        .build_ui_state(
            move |ui, slot: &mut Slot| {
                // Named faces are only visible on the frame after `set_fonts`.
                if !slot.fonts {
                    scriptcast_ui::theme::apply(ui.ctx());
                    slot.fonts = true;
                    return;
                }
                if slot.app.is_none() {
                    slot.app = Some(Browser::for_harness(ui.ctx(), args.clone()));
                }
                slot.app.as_mut().unwrap().harness_frame(ui);
            },
            Slot {
                fonts: false,
                app: None,
            },
        );
    // Waveform peaks arrive on a worker. Wait, then paint the settled frame.
    std::thread::sleep(std::time::Duration::from_millis(200));
    harness.run_steps(4);
    harness
}

fn episode_args(extra: &[&str]) -> Vec<OsString> {
    let mut args = vec![fixture("episode_small")];
    args.extend(extra.iter().map(|flag| OsString::from(*flag)));
    args
}

fn assert_inside_window(harness: &Harness<'_, Slot>) {
    let screen = eframe::egui::Rect::from_min_size(eframe::egui::Pos2::ZERO, SIZE);
    let mut nodes = vec![harness.root()];
    nodes.extend(harness.root().children_recursive());
    for node in nodes {
        if node.accesskit_node().bounding_box().is_none() {
            continue;
        }
        let rect = node.rect();
        if !rect.is_finite() || rect.width() < 1.0 || rect.height() < 1.0 {
            continue;
        }
        // Off-screen scroll content keeps its layout box. A widget that still
        // meets the window must keep its center within one row of the edge.
        if !rect.intersects(screen) {
            continue;
        }
        let center = rect.center();
        let edge = 40.0;
        let spill = center.x < screen.min.x - edge
            || center.y < screen.min.y - edge
            || center.x > screen.max.x + edge
            || center.y > screen.max.y + edge;
        let label = node
            .accesskit_node()
            .label()
            .map(|label| label.to_string())
            .unwrap_or_default();
        assert!(
            !spill,
            "widget {label:?} role {:?} {rect:?} extends past the window",
            node.accesskit_node().role()
        );
    }
}

fn shot(name: &str, args: Vec<OsString>) {
    let mut harness = harness(args);
    harness.snapshot(name);
    assert_inside_window(&harness);
}

#[test]
fn episode_browser_idle() {
    shot("episode_browser_idle", episode_args(&[]));
}

#[test]
fn selected_turn() {
    shot("selected_turn", episode_args(&["--select", "2"]));
}

#[test]
fn inline_row_editor() {
    let mut harness = harness(episode_args(&["--tag-toolbar", "1", "--edit-mode", "row"]));
    let inputs: Vec<_> = harness
        .get_all_by_role(eframe::egui::accesskit::Role::TextInput)
        .collect();
    let editors: Vec<_> = harness
        .get_all_by_role(eframe::egui::accesskit::Role::MultilineTextInput)
        .collect();
    let short = inputs
        .iter()
        .map(|node| node.rect().height())
        .fold(f32::MAX, f32::min);
    let tall = editors
        .iter()
        .map(|node| node.rect().height())
        .fold(0.0, f32::max);
    assert!(
        short.is_finite() && tall + 0.5 >= short * 3.0,
        "row editor {tall} is shorter than 3 lines of {short}"
    );
    harness.snapshot("inline_row_editor");
    assert_inside_window(&harness);
}

#[test]
fn context_menu() {
    shot("context_menu", episode_args(&["--context-menu", "1"]));
}

#[test]
fn new_episode_sheet() {
    let mut args = episode_args(&["--new-episode"]);
    args.push(fixture("script.md"));
    args.push(OsString::from("--dry-run-json"));
    args.push(fixture("phase_j/dry-run.json"));
    shot("new_episode_sheet", args);
}

#[test]
fn pipeline_stopped() {
    let mut args = episode_args(&["--pipeline-fixture"]);
    args.push(fixture("phase_j/pipeline.json"));
    shot("pipeline_stopped", args);
}

#[test]
fn voices() {
    let mut harness = harness(vec![
        OsString::from("--project-data"),
        OsString::from("voices"),
        OsString::from("--select"),
        OsString::from("aisha"),
    ]);
    let mut chips: Vec<_> = harness.get_all_by_label("OmniVoice").collect();
    chips.extend(harness.get_all_by_label("archive"));
    assert!(!chips.is_empty(), "speaker chips are missing from the tree");
    for name in ["Emmanuel Theodore", "Toussaint", "Aisha", "Lee Atwater"] {
        let name_rect = harness.get_by_label(name).rect();
        let shares = chips.iter().any(|chip| {
            let rect = chip.rect();
            name_rect.top() < rect.bottom()
                && rect.top() < name_rect.bottom()
                && rect.left() + 1.0 >= name_rect.left()
        });
        assert!(
            shares,
            "{name} {name_rect:?} does not share a row with a chip"
        );
    }
    harness.snapshot("voices");
    assert_inside_window(&harness);
}

#[test]
fn clips() {
    shot(
        "clips",
        vec![OsString::from("--project-data"), OsString::from("clips")],
    );
}

#[test]
fn audition() {
    let mut args = vec![
        OsString::from("--project-data"),
        OsString::from("voices"),
        OsString::from("--audition-fixture"),
    ];
    args.push(fixture("phase_m/audition.json"));
    shot("audition", args);
}

#[test]
fn failed_job_banner() {
    let mut args = episode_args(&["--failed-job-fixture"]);
    args.push(fixture("failed-job.log"));
    shot("failed_job_banner", args);
}
