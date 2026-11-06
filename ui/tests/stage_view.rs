//! R8 interaction tests: Ctrl+R compact selection in the stage scroll
//! view, driven through the real egui widgets with `egui_kittest`, plus
//! the full Browser wiring (stage column, Ctrl+R, one JSONL line).
use std::{ffi::OsString, path::PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use scriptcast_ui::{
    Browser,
    router_client::{JsonlQueueClient, RouterRequest, SelectionContext, Source},
    stage_view::{StageAction, StageView},
};

fn fixture_stage() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stage")
}

fn scratch_queue(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scriptcast-stage-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("router-queue.jsonl")
}

fn read_requests(queue: &PathBuf) -> Vec<RouterRequest> {
    let text = std::fs::read_to_string(queue)
        .unwrap_or_else(|e| panic!("queue {} should exist: {e}", queue.display()));
    text.lines()
        .map(|line| serde_json::from_str(line).expect("each queue line is a RouterRequest"))
        .collect()
}

fn read_contexts(queue: &PathBuf) -> Vec<SelectionContext> {
    read_requests(queue)
        .iter()
        .map(|request| {
            assert_eq!(request.source, Source::Selection);
            request
                .selection_context()
                .expect("selection request text is a SelectionContext")
        })
        .collect()
}

// ---------- Widget level: the same `show` the Browser calls ----------

struct WidgetSlot {
    fonts: bool,
    view: StageView,
    client: JsonlQueueClient,
    selected_turn: Option<String>,
    submitted: Vec<SelectionContext>,
}

fn widget_harness(queue: PathBuf) -> Harness<'static, WidgetSlot> {
    let dir = fixture_stage();
    let view = StageView::load(&dir.join("stage_plan.json"), Some(&dir)).unwrap();
    // No `.wgpu()`: these tests only drive widgets (no snapshot), so they
    // also run where no GPU adapter is available.
    let mut harness = Harness::builder()
        .with_size(eframe::egui::vec2(520.0, 760.0))
        .with_pixels_per_point(1.0)
        .build_ui_state(
            move |ui, slot: &mut WidgetSlot| {
                if !slot.fonts {
                    scriptcast_ui::theme::apply(ui.ctx());
                    slot.fonts = true;
                    return;
                }
                eframe::egui::CentralPanel::default().show(ui, |ui| {
                    let action = scriptcast_ui::stage_view::show(
                        ui,
                        &mut slot.view,
                        slot.selected_turn.as_deref(),
                        Some(&mut slot.client),
                    );
                    match action {
                        StageAction::SelectTurn { turn_id, .. } => {
                            slot.selected_turn = Some(turn_id);
                        }
                        StageAction::Submitted(context) => slot.submitted.push(context),
                        StageAction::None => {}
                    }
                });
            },
            WidgetSlot {
                fonts: false,
                view,
                client: JsonlQueueClient::new(queue),
                selected_turn: Some("turn-1".into()),
                submitted: Vec::new(),
            },
        );
    harness.run_steps(4);
    harness
}

#[test]
fn ctrl_r_select_and_send_writes_one_jsonl_line() {
    let queue = scratch_queue("widget");
    let mut harness = widget_harness(queue.clone());
    let normal_height = harness.state().view.thumb_height();
    assert!(!harness.state().view.compact);

    // Ctrl+R (Control, not Cmd) enters compact selection mode and shrinks.
    harness.key_press_modifiers(eframe::egui::Modifiers::CTRL, eframe::egui::Key::R);
    harness.run_steps(3);
    assert!(harness.state().view.compact, "Ctrl+R toggles compact mode");
    assert!(
        harness.state().view.thumb_height() < normal_height,
        "compact view shrinks"
    );

    // Click two frames belonging to two different turns.
    harness.get_by_label("SceneA · intro · turn-1").click();
    harness.run_steps(3);
    harness.get_by_label("SceneA · detail · turn-2").click();
    harness.run_steps(3);
    assert_eq!(
        harness.state().view.selected_frames.len(),
        2,
        "both clicked frames stay selected"
    );
    assert!(
        !queue.exists(),
        "nothing is sent before Send is pressed: {}",
        queue.display()
    );

    harness.get_by_label("Send selection").click();
    harness.run_steps(3);

    let contexts = read_contexts(&queue);
    assert_eq!(contexts.len(), 1, "one Send appends exactly one line");
    let context = &contexts[0];
    assert_eq!(context.episode, "fixture-ep");
    assert_eq!(context.turn_ids, vec!["turn-1", "turn-2"]);
    assert_eq!(context.cue_ids, vec!["intro", "detail"]);
    assert_eq!(context.frame_paths.len(), 2);
    assert!(context.frame_paths[0].ends_with("SceneA_intro.png"));
    assert!(context.frame_paths[1].ends_with("SceneA_detail.png"));
    assert_eq!(harness.state().submitted.len(), 1);

    // A second Send appends a second line instead of rewriting the file.
    harness.get_by_label("Send selection").click();
    harness.run_steps(3);
    assert_eq!(read_contexts(&queue).len(), 2);
    let _ = std::fs::remove_dir_all(queue.parent().unwrap());
}

#[test]
fn clicking_a_frame_in_normal_mode_selects_its_turn() {
    let queue = scratch_queue("click");
    let mut harness = widget_harness(queue.clone());
    harness.get_by_label("SceneA · detail · turn-2").click();
    harness.run_steps(3);
    assert_eq!(harness.state().selected_turn.as_deref(), Some("turn-2"));
    assert!(
        harness.state().view.selected_frames.is_empty(),
        "normal-mode clicks move the turn, not the pick set"
    );
    let _ = std::fs::remove_dir_all(queue.parent().unwrap());
}

// ---------- Browser level: episode with a stage plan ----------

struct BrowserSlot {
    fonts: bool,
    app: Option<Browser>,
}

#[test]
fn browser_ctrl_r_selection_lands_in_the_queue_file() {
    let queue = scratch_queue("browser");
    let args: Vec<OsString> = vec![
        fixture_stage().into_os_string(),
        OsString::from("--select"),
        OsString::from("1"),
        OsString::from("--stage-queue"),
        queue.clone().into_os_string(),
    ];
    // Taller than the 1440x900 snapshot size so all three compact frames
    // stay inside the stage viewport for clicking.
    let mut harness = Harness::builder()
        .with_size(eframe::egui::vec2(1440.0, 1400.0))
        .with_pixels_per_point(1.0)
        .build_ui_state(
            move |ui, slot: &mut BrowserSlot| {
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
            BrowserSlot {
                fonts: false,
                app: None,
            },
        );
    std::thread::sleep(std::time::Duration::from_millis(200));
    harness.run_steps(4);

    // The stage column is present with all three derived frames.
    harness.get_by_label("SceneA · intro · turn-1");
    harness.get_by_label("SceneA · detail · turn-2");
    harness.get_by_label("SceneA · end · turn-2");

    harness.key_press_modifiers(eframe::egui::Modifiers::CTRL, eframe::egui::Key::R);
    harness.run_steps(3);
    harness.get_by_label("SceneA · intro · turn-1").click();
    harness.run_steps(3);
    harness.get_by_label("SceneA · end · turn-2").click();
    harness.run_steps(3);
    harness.get_by_label("Send selection").click();
    harness.run_steps(4);

    let contexts = read_contexts(&queue);
    assert_eq!(contexts.len(), 1, "one Send appends exactly one line");
    let context = &contexts[0];
    assert_eq!(context.episode, "fixture-ep");
    assert_eq!(context.turn_ids, vec!["turn-1", "turn-2"]);
    assert_eq!(context.cue_ids, vec!["intro", "end"]);
    assert_eq!(context.frame_paths.len(), 2);
    assert!(context.frame_paths[0].ends_with("SceneA_intro.png"));
    assert!(context.frame_paths[1].ends_with("SceneA_end.png"));
    let _ = std::fs::remove_dir_all(queue.parent().unwrap());
}
