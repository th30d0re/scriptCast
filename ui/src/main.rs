mod table;
mod timeline;
mod view;

use eframe::egui::{self, Align, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};
use scriptcast_ui::{
    episode::{Episode, LoadError, VerificationState as V},
    theme as t,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use timeline::TimelineView;

const STORAGE_KEY: &str = "scriptcast_browser";
const LOAD_NOTICE_DELAY: Duration = Duration::from_millis(150);
const OVERVIEW_SECTIONS: usize = 120;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RecentEpisode {
    id: String,
    path: PathBuf,
    opened: u64,
    turns: usize,
    total_ms: u64,
}
#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    last_episode: Option<PathBuf>,
    recent: Vec<RecentEpisode>,
}
struct PendingLoad {
    path: PathBuf,
    started: Instant,
    receiver: Receiver<Result<Episode, LoadError>>,
    canceled: Arc<AtomicBool>,
}
impl Drop for PendingLoad {
    fn drop(&mut self) {
        self.canceled.store(true, Ordering::Relaxed);
    }
}
#[derive(Default)]
struct Filters {
    search: String,
    speaker: Option<usize>,
    failed_only: bool,
}
impl Filters {
    fn matches(&self, episode: &Episode) -> Vec<bool> {
        let query = self.search.to_lowercase();
        episode
            .turns
            .iter()
            .map(|turn| {
                self.speaker.is_none_or(|s| s == turn.speaker_idx)
                    && (!self.failed_only || turn.verification == V::Failed)
                    && (query.is_empty()
                        || turn.first_words.to_lowercase().contains(&query)
                        || episode.speakers[turn.speaker_idx]
                            .name
                            .to_lowercase()
                            .contains(&query)
                        || format!("{:03}", turn.index).contains(&query))
            })
            .collect()
    }
}
#[derive(Default)]
struct Browser {
    episode: Option<Episode>,
    pending: Option<PendingLoad>,
    error: Option<(PathBuf, String)>,
    preferences: Preferences,
    selected: Option<usize>,
    filters: Filters,
    timeline: TimelineView,
    overview: Vec<Option<usize>>,
    scroll_to: Option<usize>,
    focus_search: bool,
}
impl Browser {
    fn new(cc: &eframe::CreationContext<'_>, argument: Option<PathBuf>) -> Self {
        t::apply(&cc.egui_ctx);
        let preferences = cc
            .storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_default();
        let mut app = Self {
            preferences,
            ..Self::default()
        };
        if let Some(path) = argument.or_else(|| app.preferences.last_episode.clone()) {
            app.open(path, &cc.egui_ctx);
        }
        app
    }
    fn open(&mut self, path: PathBuf, ctx: &egui::Context) {
        let (sender, receiver) = mpsc::channel();
        let worker_path = path.clone();
        let canceled = Arc::new(AtomicBool::new(false));
        let worker_canceled = Arc::clone(&canceled);
        let repaint = ctx.clone();
        // Replacing the receiver invalidates any previous request. Late results
        // can never overwrite a newer open or a user's cancellation.
        std::thread::spawn(move || {
            let result = Episode::load_cancellable(&worker_path, &worker_canceled);
            let _ = sender.send(result);
            repaint.request_repaint();
        });
        self.pending = Some(PendingLoad {
            path,
            started: Instant::now(),
            receiver,
            canceled,
        });
        self.error = None;
        ctx.request_repaint_after(LOAD_NOTICE_DELAY);
    }
    fn pick(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open rendered episode directory")
            .pick_folder()
        {
            self.open(path, ctx);
        }
    }
    fn reload(&mut self, ctx: &egui::Context) {
        if let Some(e) = &self.episode {
            self.open(e.dir.clone(), ctx);
        }
    }
    fn poll_load(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending else {
            return;
        };
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err(LoadError {
                path: pending.path.clone(),
                cause: "Episode loader stopped unexpectedly".into(),
            }),
        };
        let path = self.pending.take().expect("pending request").path.clone();
        match result {
            Ok(episode) => self.accept_episode(episode, ctx),
            Err(error) => self.error = Some((path, error.to_string())),
        }
    }
    fn accept_episode(&mut self, episode: Episode, ctx: &egui::Context) {
        let same = self
            .episode
            .as_ref()
            .is_some_and(|old| old.dir == episode.dir);
        let selected_id = if same {
            self.selected
                .and_then(|i| self.episode.as_ref()?.turns.get(i))
                .map(|t| t.id.clone())
        } else {
            None
        };
        self.selected = selected_id.and_then(|id| episode.turns.iter().position(|t| t.id == id));
        if !same {
            self.filters = Filters::default();
            self.timeline = TimelineView::initial(episode.total_ms);
        } else {
            self.timeline.zoom(1.0, episode.total_ms);
        }
        if episode.count(V::Failed) == 0 {
            self.filters.failed_only = false;
        }
        if self
            .filters
            .speaker
            .is_some_and(|s| s >= episode.speakers.len())
        {
            self.filters.speaker = None;
        }
        self.overview = episode.overview_sections(OVERVIEW_SECTIONS);
        self.preferences.last_episode = Some(episode.dir.clone());
        self.preferences.recent.retain(|r| r.path != episode.dir);
        self.preferences.recent.insert(
            0,
            RecentEpisode {
                id: episode.manifest.episode_id.clone(),
                path: episode.dir.clone(),
                opened: now(),
                turns: episode.turns.len(),
                total_ms: episode.total_ms,
            },
        );
        self.preferences.recent.truncate(5);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "scriptCast — {}",
            episode.manifest.episode_id
        )));
        self.episode = Some(episode);
        self.error = None;
        // Reset virtual scroll on a new episode; preserve/reveal selection on reload.
        self.scroll_to = self.selected.or(Some(0));
    }
    fn select(&mut self, index: usize) {
        if let Some(e) = &self.episode
            && let Some(turn) = e.turns.get(index)
        {
            self.selected = Some(index);
            self.scroll_to = Some(index);
            self.timeline.reveal(turn.start_ms, turn.end_ms, e.total_ms);
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.text_edit_focused() {
            return;
        }
        let (open, reload, search, up, down, escape) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::O),
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::R),
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::F),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
            )
        });
        if open {
            self.pick(ctx);
        }
        if reload {
            self.reload(ctx);
        }
        if search {
            self.focus_search = true;
        }
        if escape {
            self.pending = None;
            self.error = None;
        }
        if (up || down)
            && let Some(e) = &self.episode
        {
            let visible: Vec<_> = self
                .filters
                .matches(e)
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.then_some(i))
                .collect();
            if !visible.is_empty() {
                let position = self
                    .selected
                    .and_then(|s| visible.iter().position(|i| *i == s));
                let next = match position {
                    Some(i) if up => i.saturating_sub(1),
                    Some(i) => (i + 1).min(visible.len() - 1),
                    None if up => visible.len() - 1,
                    None => 0,
                };
                self.select(visible[next]);
            }
        }
    }
    fn header(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        view::rule(ui, rect);
        let mut ui = ui.new_child(
            UiBuilder::new()
                .max_rect(view::inset(rect, t::SPACE_20, 0.0))
                .layout(Layout::left_to_right(Align::Center)),
        );
        ui.spacing_mut().item_spacing.x = t::SPACE_22;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = t::SPACE_8;
            ui.add(
                egui::Image::new(t::icon("audio-lines-22-accent"))
                    .fit_to_exact_size(egui::Vec2::splat(t::SPACE_22)),
            );
            ui.label(t::heading("scriptCast", t::BRAND));
        });
        let (divider, _) =
            ui.allocate_exact_size(vec2(t::BORDER, t::BUTTON_HEIGHT), Sense::hover());
        ui.painter().rect_filled(divider, 0, t::BORDER_DEFAULT);
        let mut reload = false;
        let mut open = false;
        // Lay out content-sized controls at the right, then give identity the remainder.
        let remaining = ui.available_rect_before_wrap();
        let mut actions = ui.new_child(
            UiBuilder::new()
                .id_salt("header_actions")
                .max_rect(remaining)
                .layout(Layout::right_to_left(Align::Center)),
        );
        reload |= t::secondary_button(
            &mut actions,
            "Reload",
            Some("rotate-cw-16-text"),
            self.episode.is_some() && self.pending.is_none(),
        )
        .clicked();
        open |=
            t::secondary_button(&mut actions, "Open", Some("folder-open-16-text"), true).clicked();
        let identity = Rect::from_min_max(
            remaining.min,
            pos2(actions.min_rect().left() - t::SPACE_22, remaining.bottom()),
        );
        let (name, metadata, warnings) = if let Some(e) = &self.episode {
            (
                e.manifest.episode_id.as_str(),
                format!(
                    "{} / {} kHz / {} turns / {} speakers / Rendered {}",
                    e.manifest.engine,
                    e.manifest.sample_rate as f64 / 1000.0,
                    e.turns.len(),
                    e.speakers.len(),
                    e.rendered_at.as_deref().unwrap_or("unknown")
                ),
                e.warnings.join("\n"),
            )
        } else {
            (
                "No episode open",
                "Open a rendered episode directory to begin".into(),
                String::new(),
            )
        };
        let title = Rect::from_min_max(
            pos2(identity.left(), identity.center().y - t::SPACE_22),
            pos2(identity.right(), identity.center().y + t::SPACE_5),
        );
        let meta = Rect::from_min_max(
            pos2(identity.left(), title.bottom()),
            pos2(identity.right(), identity.center().y + t::SPACE_22),
        );
        view::line(&ui, title, name, t::bold(t::EPISODE_NAME), t::TEXT_PRIMARY);
        view::line(&ui, meta, &metadata, t::font(t::SECONDARY), t::TEXT_MUTED);
        ui.interact(identity, ui.id().with("metadata"), Sense::hover())
            .on_hover_text(if warnings.is_empty() {
                metadata
            } else {
                format!("{metadata}\n\nWarnings:\n{warnings}")
            });
        let ctx = ui.ctx().clone();
        if reload {
            self.reload(&ctx);
        }
        if open {
            self.pick(&ctx);
        }
    }
    fn filter_bar(&mut self, ui: &mut Ui) {
        let Some(e) = &self.episode else {
            return;
        };
        let rect = view::inset(ui.max_rect(), t::SPACE_16, 0.0);
        let mut ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        ui.spacing_mut().item_spacing.x = t::SPACE_10;
        let (search_rect, _) =
            ui.allocate_exact_size(vec2(t::SEARCH_WIDTH, t::SEARCH_HEIGHT), Sense::hover());
        ui.painter()
            .rect_filled(search_rect, t::RADIUS_CONTROL, t::BG_WINDOW);
        ui.painter().rect_stroke(
            search_rect,
            t::RADIUS_CONTROL,
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
            egui::StrokeKind::Inside,
        );
        let icon = Rect::from_center_size(
            pos2(search_rect.left() + t::SPACE_14, search_rect.center().y),
            egui::Vec2::splat(t::SPACE_14),
        );
        egui::Image::new(t::icon("search-14-text-muted")).paint_at(&ui, icon);
        let edit = Rect::from_min_max(
            pos2(icon.right() + t::SPACE_7, search_rect.top()),
            pos2(
                search_rect.right() - t::TABLE_COL_DURATION - t::SPACE_8,
                search_rect.bottom(),
            ),
        );
        let response = ui.put(
            edit,
            egui::TextEdit::singleline(&mut self.filters.search)
                .id(egui::Id::new("turn_search"))
                .font(t::font(t::BODY))
                .hint_text("Search turns")
                .frame(egui::Frame::NONE),
        );
        if self.focus_search {
            response.request_focus();
            self.focus_search = false;
        }
        view::line(
            &ui,
            Rect::from_min_max(pos2(edit.right(), edit.top()), search_rect.max),
            "Cmd+F",
            t::font(t::CAPTION),
            t::TEXT_SUBTLE,
        );
        let old_speaker = self.filters.speaker;
        let old_failed = self.filters.failed_only;
        egui::ComboBox::from_id_salt("speaker_filter")
            .selected_text(
                self.filters
                    .speaker
                    .map(|s| e.speakers[s].name.as_str())
                    .unwrap_or("All speakers"),
            )
            .show_ui(&mut ui, |ui| {
                ui.selectable_value(&mut self.filters.speaker, None, "All speakers");
                for (i, speaker) in e.speakers.iter().enumerate() {
                    ui.selectable_value(&mut self.filters.speaker, Some(i), &speaker.name);
                }
            });
        let failures = e.count(V::Failed);
        let label = format!(
            "{}Failed only · {}",
            if self.filters.failed_only { "✓ " } else { "" },
            failures
        );
        let button = egui::Button::new(t::heading(label, t::BUTTON).color(
            if self.filters.failed_only {
                t::STATUS_FAILED
            } else {
                t::TEXT_PRIMARY
            },
        ))
        .min_size(vec2(0.0, t::BUTTON_HEIGHT))
        .fill(if self.filters.failed_only {
            t::BG_DANGER
        } else {
            t::BG_CONTROL
        })
        .stroke(egui::Stroke::new(
            t::BORDER,
            if self.filters.failed_only {
                t::STATUS_FAILED
            } else {
                t::BORDER_DEFAULT
            },
        ));
        if ui.add_enabled(failures > 0, button).clicked() {
            self.filters.failed_only = !self.filters.failed_only;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(
                egui::Image::new(t::icon("list-15-text-muted"))
                    .fit_to_exact_size(egui::Vec2::splat(t::TEXT_15)),
            );
            ui.label(t::text("Turn view", t::SECONDARY, t::TEXT_MUTED));
        });
        if response.changed()
            || old_speaker != self.filters.speaker
            || old_failed != self.filters.failed_only
        {
            self.scroll_to = Some(0);
        }
    }
    fn empty(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        let width = t::EMPTY_LEFT + t::EMPTY_RIGHT + t::EMPTY_GAP * 2.0 + t::BORDER;
        let x = rect.center().x - width / 2.0;
        let height = (t::EMPTY_DIVIDER + t::SPACE_22 * 4.0).min(rect.height() - t::SPACE_20 * 2.0);
        let top = rect.center().y - height / 2.0;
        let left = Rect::from_min_size(pos2(x, top), vec2(t::EMPTY_LEFT, height));
        let divider = Rect::from_center_size(
            pos2(left.right() + t::EMPTY_GAP, rect.center().y),
            vec2(t::BORDER, t::EMPTY_DIVIDER),
        );
        ui.painter().rect_filled(divider, 0, t::BORDER_DEFAULT);
        let right = Rect::from_min_size(
            pos2(divider.right() + t::EMPTY_GAP, top),
            vec2(t::EMPTY_RIGHT, height),
        );
        let mut open = false;
        let mut left_ui = view::region(ui, "empty_guidance", left, t::BG_WINDOW);
        egui::ScrollArea::vertical().id_salt("empty_guidance_scroll").show(&mut left_ui,|ui| {
            ui.spacing_mut().item_spacing.y=t::SPACE_20;
            ui.add(egui::Image::new(t::icon("folder-open-42-accent")).fit_to_exact_size(egui::Vec2::splat(t::HERO_ICON)));
            ui.label(t::heading("Your episode starts here.",t::HERO_TITLE));
            ui.label(t::text("Open an episode directory containing rendered audio and metadata. scriptCast loads the timeline, speakers, turns, and verification results in one local workspace.",t::TEXT_14,t::TEXT_MUTED));
            ui.horizontal(|ui| { open |= t::primary_button(ui,"Open Episode",Some("folder-open-16-on-accent"),true).clicked(); ui.label(t::text("Cmd+O",t::BODY,t::TEXT_MUTED)); });
            ui.add_space(t::SPACE_16); t::field_label(ui,"Expected directory contents");
            ui.spacing_mut().item_spacing.y=t::SPACE_7;
            for text in ["episode/","  episode_manifest.json","  Samples/Processed/","  TURN_INDEX.csv + stress.json (optional)"] { ui.label(egui::RichText::new(text).font(t::mono(t::BODY)).color(t::TEXT_MUTED)); }
        });
        let mut path = None;
        let mut recent_ui = view::region(ui, "recent_episodes", right, t::BG_WINDOW);
        egui::ScrollArea::vertical()
            .id_salt("recent_scroll")
            .show(&mut recent_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t::heading("Recent episodes", t::TEXT_13));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .link(t::text("Open directory ↗", t::SECONDARY, t::TEXT_SUBTLE))
                            .clicked()
                        {
                            open = true;
                        }
                    });
                });
                ui.add_space(t::SPACE_12);
                if self.preferences.recent.is_empty() {
                    ui.label(t::text("No recent episodes yet.", t::BODY, t::TEXT_MUTED));
                }
                for recent in &self.preferences.recent {
                    let (row, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), t::EPISODE_HEADER_HEIGHT),
                        Sense::click(),
                    );
                    let title = Rect::from_min_size(row.min, vec2(row.width(), t::SPACE_22));
                    let meta = title.translate(vec2(0.0, t::SPACE_22 + t::SPACE_5));
                    let path_rect = meta.translate(vec2(0.0, t::SPACE_22));
                    view::line(ui, title, &recent.id, t::font(t::TEXT_14), t::TEXT_PRIMARY);
                    view::line(
                        ui,
                        meta,
                        &format!(
                            "{} · {} turns · {}",
                            opened_when(recent.opened),
                            recent.turns,
                            view::duration(recent.total_ms)
                        ),
                        t::font(t::BODY),
                        t::TEXT_MUTED,
                    );
                    view::line(
                        ui,
                        path_rect,
                        &recent.path.display().to_string(),
                        t::mono(t::CAPTION),
                        t::TEXT_SUBTLE,
                    );
                    view::rule(ui, row);
                    if response
                        .on_hover_text(recent.path.display().to_string())
                        .clicked()
                    {
                        path = Some(recent.path.clone());
                    }
                }
                ui.add_space(t::SPACE_16);
                ui.label(t::text(
                    "● Local files  Nothing uploaded. No connection required.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
            });
        let ctx = ui.ctx().clone();
        if open {
            self.pick(&ctx);
        }
        if let Some(path) = path {
            self.open(path, &ctx);
        }
    }
    fn playback(&self, ui: &mut Ui) {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        );
        let mut ui = ui.new_child(
            UiBuilder::new()
                .max_rect(view::inset(rect, t::SPACE_20, 0.0))
                .layout(Layout::left_to_right(Align::Center)),
        );
        ui.spacing_mut().item_spacing.x = t::SPACE_18;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = t::SPACE_7;
            t::primary_button(ui, "Play", Some("play-16-on-accent"), false);
            t::secondary_button(ui, "Stop", Some("square-16-text"), false);
        });
        let (current, _) = ui.allocate_exact_size(
            vec2(
                t::PLAYBACK_CURRENT_AUDIO_WIDTH,
                t::BUTTON_HEIGHT + t::SPACE_12,
            ),
            Sense::hover(),
        );
        let label = self
            .selected
            .and_then(|i| {
                self.episode.as_ref()?.turns.get(i).map(|turn| {
                    (
                        turn,
                        &self.episode.as_ref().unwrap().speakers[turn.speaker_idx],
                    )
                })
            })
            .map(|(turn, speaker)| format!("Turn {:03} · {}", turn.index, speaker.name))
            .unwrap_or_else(|| "Select a turn to audition".into());
        let top = Rect::from_min_size(current.min, vec2(current.width(), current.height() / 2.0));
        view::line(&ui, top, &label, t::font(t::BODY), t::TEXT_PRIMARY);
        view::line(
            &ui,
            top.translate(vec2(0.0, top.height())),
            "00:00.000 / —",
            t::mono(t::NUMERIC),
            t::TEXT_MUTED,
        );
        let remaining = ui.available_rect_before_wrap();
        let mut options = ui.new_child(
            UiBuilder::new()
                .max_rect(remaining)
                .layout(Layout::right_to_left(Align::Center)),
        );
        options.add_enabled_ui(false, |ui| {
            ui.label(t::text("Output unavailable", t::SECONDARY, t::TEXT_MUTED));
            ui.label(t::text("Space  Play / pause", t::SECONDARY, t::TEXT_SUBTLE));
            let mut auto = false;
            ui.checkbox(&mut auto, "Auto-advance");
        });
        let track = Rect::from_min_max(
            pos2(
                remaining.left(),
                remaining.center().y - t::SEEK_TRACK_HEIGHT / 2.0,
            ),
            pos2(
                (options.min_rect().left() - t::SPACE_18).max(remaining.left()),
                remaining.center().y + t::SEEK_TRACK_HEIGHT / 2.0,
            ),
        );
        ui.painter()
            .rect_filled(track, t::RADIUS_SEEK_TRACK, t::BG_CONTROL);
    }
    fn status(&self, ui: &mut Ui) {
        let rect = view::inset(ui.max_rect(), t::SPACE_20, 0.0);
        let status = if self.pending.is_some() {
            "Loading episode…".into()
        } else if self.error.is_some() {
            "Couldn’t open episode · see error details".into()
        } else if let Some(e) = &self.episode {
            format!(
                "Ready · {} turns · {} passed · {} failed · {} not verified · {} missing audio{}",
                e.turns.len(),
                e.count(V::Passed),
                e.count(V::Failed),
                e.count(V::NotVerified),
                e.count(V::MissingAudio),
                if e.script_in_sync == Some(false) {
                    " · Script changed since render"
                } else {
                    ""
                }
            )
        } else {
            "No episode open".into()
        };
        let mut row = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        row.label(t::text("Local workspace", t::CAPTION, t::TEXT_SUBTLE));
        let left = Rect::from_min_max(
            rect.min,
            pos2(row.min_rect().left() - t::SPACE_20, rect.bottom()),
        );
        view::line(ui, left, &status, t::font(t::CAPTION), t::TEXT_MUTED);
    }
    fn workspace_card(&mut self, ctx: &egui::Context) {
        let loading = self
            .pending
            .as_ref()
            .filter(|p| p.started.elapsed() >= LOAD_NOTICE_DELAY)
            .map(|p| p.path.clone());
        if loading.is_none() && self.error.is_none() {
            return;
        }
        let mut choose = false;
        let mut retry = None;
        let mut dismiss = false;
        let mut cancel = false;
        egui::Window::new("Workspace state").id(egui::Id::new("workspace_state"))
            .title_bar(false).resizable(false).collapsible(false).anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO)
            .fixed_size(vec2(t::STATE_CARD_WIDTH,0.0))
            .frame(egui::Frame::new().fill(t::BG_SURFACE).stroke(egui::Stroke::new(t::BORDER,t::BORDER_DEFAULT)).corner_radius(t::RADIUS_CARD as u8).inner_margin(t::SPACE_16))
            .show(ctx,|ui| {
                ui.spacing_mut().item_spacing.y=t::SPACE_18;
                if let Some(path)=&loading {
                    ui.label(t::heading("Workspace state / Loading",t::TEXT_13));
                    ui.horizontal(|ui| { ui.add(egui::Image::new(t::icon("loader-circle-22-accent")).fit_to_exact_size(egui::Vec2::splat(t::SPACE_22))); ui.label(t::heading(format!("Loading {}…",path.file_name().unwrap_or_default().to_string_lossy()),t::STATE_TITLE)); });
                    ui.label(t::text("Reading turn metadata, audio paths and verification results.",t::TEXT_13,t::TEXT_MUTED));
                    let (bar,_)=ui.allocate_exact_size(vec2(ui.available_width(),t::PROGRESS_HEIGHT),Sense::hover());
                    ui.painter().rect_filled(bar,0,t::BG_CONTROL);
                    let phase=(ui.input(|i| i.time)*0.6).fract() as f32;
                    let width=bar.width()/4.0;
                    ui.painter().rect_filled(Rect::from_min_size(pos2(bar.left()+(bar.width()-width)*phase,bar.top()),vec2(width,bar.height())),0,t::ACCENT);
                    cancel=t::secondary_button(ui,"Cancel",None,true).clicked();
                    ui.label(t::text("Your files remain local. Cancel keeps the current episode open.",t::SECONDARY,t::TEXT_MUTED));
                    ctx.request_repaint_after(Duration::from_millis(50));
                } else if let Some((path,error))=&self.error {
                    ui.label(t::heading("Workspace state / Error",t::TEXT_13));
                    ui.horizontal(|ui| { ui.add(egui::Image::new(t::icon("circle-alert-22-danger")).fit_to_exact_size(egui::Vec2::splat(t::SPACE_22))); ui.label(t::heading("Couldn’t open this episode",t::STATE_TITLE)); });
                    ui.label(t::text("Turn metadata could not be read. Choose a rendered episode directory or fix the metadata and try again.",t::TEXT_13,t::TEXT_MUTED));
                    ui.label(t::text(error,t::SECONDARY,t::STATUS_FAILED));
                    ui.horizontal(|ui| {
                        choose=t::primary_button(ui,"Choose directory",None,true).clicked();
                        if t::secondary_button(ui,"Try again",None,true).clicked() { retry=Some(path.clone()); }
                        dismiss=t::secondary_button(ui,"Dismiss",None,true).clicked();
                    });
                    ui.label(t::text(if self.episode.is_some() { "The current episode is still open. A malformed directory does not erase it." } else { "Choose a directory containing episode_manifest.json." },t::SECONDARY,t::TEXT_MUTED));
                }
            });
        if cancel {
            self.pending = None;
        }
        if dismiss {
            self.error = None;
        }
        if choose {
            self.pick(ctx);
        }
        if let Some(path) = retry {
            self.open(path, ctx);
        }
    }
}
impl Browser {
    fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.shortcuts(&ctx);
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0, t::BG_WINDOW);
        let header = Rect::from_min_size(rect.min, vec2(rect.width(), t::EPISODE_HEADER_HEIGHT));
        let status = Rect::from_min_max(
            pos2(rect.left(), rect.bottom() - t::STATUS_BAR_HEIGHT),
            rect.max,
        );
        let playback = Rect::from_min_max(
            pos2(rect.left(), status.top() - t::PLAYBACK_BAR_HEIGHT),
            status.right_top(),
        );
        let workspace = Rect::from_min_max(header.left_bottom(), playback.right_top());
        self.header(&mut view::region(ui, "header", header, t::BG_SURFACE));
        self.playback(&mut view::region(ui, "playback", playback, t::BG_RAISED));
        self.status(&mut view::region(ui, "status", status, t::BG_WINDOW));
        if self.episode.is_some() {
            let timeline =
                Rect::from_min_size(workspace.min, vec2(workspace.width(), t::TIMELINE_HEIGHT));
            let filter = Rect::from_min_size(
                timeline.left_bottom(),
                vec2(workspace.width(), t::FILTER_BAR_HEIGHT),
            );
            let table = Rect::from_min_max(filter.left_bottom(), workspace.max);
            self.filter_bar(&mut view::region(ui, "filters", filter, t::BG_RAISED));
            let e = self.episode.as_ref().unwrap();
            let matches = self.filters.matches(e);
            let visible = matches
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.then_some(i))
                .collect::<Vec<_>>();
            let clicked = timeline::show(
                &mut view::region(ui, "timeline", timeline, t::BG_WINDOW),
                e,
                &self.overview,
                &mut self.timeline,
                &matches,
                self.selected,
            );
            if let Some(i) = clicked {
                self.select(i);
            }
            let clicked = table::show(
                &mut view::region(ui, "table", table, t::BG_SURFACE),
                self.episode.as_ref().unwrap(),
                &visible,
                self.selected,
                self.scroll_to.take(),
                self.filters.failed_only,
            );
            if let Some(i) = clicked {
                self.select(i);
            }
        } else {
            self.empty(&mut view::region(ui, "empty", workspace, t::BG_WINDOW));
        }
        self.workspace_card(&ctx);
    }
}
impl eframe::App for Browser {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_load(ctx);
    }
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.preferences);
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn opened_when(timestamp: u64) -> String {
    match now().saturating_sub(timestamp) / 86400 {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        days => format!("{days} days ago"),
    }
}
fn main() -> eframe::Result {
    let argument = std::env::args_os().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("scriptCast — Episode workspace")
            .with_inner_size(t::WINDOW_SIZE)
            .with_min_inner_size(t::WINDOW_MIN)
            .with_decorations(true),
        ..Default::default()
    };
    eframe::run_native(
        "scriptcast-ui",
        options,
        Box::new(move |cc| Ok(Box::new(Browser::new(cc, argument)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Episode {
        Episode::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small"),
        )
        .unwrap()
    }
    #[test]
    fn filters_intersect_and_never_manufacture_verification() {
        let e = fixture();
        let mut f = Filters::default();
        assert_eq!(f.matches(&e), [true, true, true]);
        f.search = "WORLD".into();
        assert_eq!(f.matches(&e), [true, false, false]);
        f.search.clear();
        f.failed_only = true;
        assert_eq!(f.matches(&e), [false, true, false]);
        f.speaker = Some(0);
        assert_eq!(f.matches(&e), [false, false, false]);
    }
    #[test]
    fn failed_open_keeps_episode_and_recent_history() {
        let ctx = egui::Context::default();
        let mut app = Browser::default();
        app.accept_episode(fixture(), &ctx);
        app.select(1);
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Err(LoadError {
                path: "bad/episode_manifest.json".into(),
                cause: "invalid JSON".into(),
            }))
            .unwrap();
        app.pending = Some(PendingLoad {
            path: "bad".into(),
            started: Instant::now(),
            receiver,
            canceled: Arc::new(AtomicBool::new(false)),
        });
        app.poll_load(&ctx);
        assert_eq!(app.episode.as_ref().unwrap().turns.len(), 3);
        assert_eq!(app.selected, Some(1));
        assert_eq!(app.preferences.recent.len(), 1);
        assert!(app.error.is_some());
    }
    #[test]
    fn canceled_result_cannot_replace_current_episode() {
        let ctx = egui::Context::default();
        let mut app = Browser::default();
        app.accept_episode(fixture(), &ctx);
        let (sender, receiver) = mpsc::channel();
        app.pending = Some(PendingLoad {
            path: "canceled".into(),
            started: Instant::now(),
            receiver,
            canceled: Arc::new(AtomicBool::new(false)),
        });
        app.pending = None;
        assert!(sender.send(Ok(fixture())).is_err());
        app.poll_load(&ctx);
        assert_eq!(app.preferences.recent.len(), 1);
    }

    #[test]
    fn workspace_states_layout_headlessly_without_panics() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut app = Browser::default();
        for size in [t::WINDOW_SIZE, t::WINDOW_MIN] {
            for state in 0..4 {
                app.pending = None;
                app.error = None;
                if state == 0 {
                    app.episode = None;
                } else {
                    app.accept_episode(fixture(), &ctx);
                }
                let mut sender = None;
                if state == 2 {
                    let (tx, receiver) = mpsc::channel();
                    sender = Some(tx);
                    app.pending = Some(PendingLoad {
                        path: "fixture".into(),
                        started: Instant::now() - LOAD_NOTICE_DELAY,
                        receiver,
                        canceled: Arc::new(AtomicBool::new(false)),
                    });
                }
                if state == 3 {
                    app.error = Some((
                        "bad".into(),
                        "episode_manifest.json: invalid JSON at line 1".into(),
                    ));
                }
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size.into())),
                        ..Default::default()
                    },
                    |ui| app.show(ui),
                );
                assert!(!output.shapes.is_empty());
                output.textures_delta.clear();
                drop(sender);
            }
        }
    }
}
