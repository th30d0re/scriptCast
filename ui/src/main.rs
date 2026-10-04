mod capture;
mod editing;
mod inspector;
mod jobs;
mod settings;
mod table;
mod timeline;
mod view;

use eframe::egui::{self, Align, Layout, Rect, Sense, StrokeKind, Ui, UiBuilder, pos2, vec2};
use inspector::InspectorMode;
use scriptcast_ui::{
    audio::{self, AudioOut},
    cli::{self, ExportCommand},
    daw::{self, DawEvent, DawTask},
    episode::{Episode, LoadError, VerificationState as V, overview_section_count},
    theme as t,
    waveform::{self, Waveforms},
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
/// Repaint cadence while audio is playing; no repaints are requested when idle.
const REPLAY_TICK: Duration = Duration::from_millis(50);

// Empty-workspace copy, factored out so the content height measurement in
// `empty_left_height`/`empty_right_height` cannot drift from what is rendered.
const HERO_TITLE_TEXT: &str = "Your episode starts here.";
const HERO_BODY_TEXT: &str = "Open an episode directory containing rendered audio and metadata. \
    scriptCast loads the timeline, speakers, turns, and verification results in one local workspace.";

const DIR_LIST_TITLE: &str = "Expected directory contents";
const DIR_LIST: [&str; 4] = [
    "episode/",
    "  episode_manifest.json",
    "  Samples/Processed/",
    "  TURN_INDEX.csv + stress.json (optional)",
];
const LOCAL_FILES_NOTE: &str = "● Local files  Nothing uploaded. No connection required.";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RecentEpisode {
    id: String,
    path: PathBuf,
    opened: u64,
    turns: usize,
    total_ms: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    /// Folder with scriptCast commands; None (or empty) uses discovery.
    cli_dir: Option<PathBuf>,
    last_episode: Option<PathBuf>,
    recent: Vec<RecentEpisode>,
    /// Play the next visible turn when the current one finishes (screen 03).
    auto_advance: bool,
    #[serde(default = "default_true")]
    gapless: bool,
    /// Output device choice, persisted by name; a gone device falls back to
    /// the default output with a notice.
    output_device: Option<String>,
    /// Timeline pane height; `None` uses `t::TIMELINE_HEIGHT`.
    #[serde(default)]
    timeline_height: Option<f32>,
    /// Inspector pane width for every mode; `None` uses each mode's default.
    #[serde(default)]
    inspector_width: Option<f32>,
    /// Accent, speaker colours, and text scale.
    #[serde(default)]
    appearance: t::Appearance,
    #[serde(default)]
    edit_mode: editing::EditMode,
}
fn default_true() -> bool {
    true
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            cli_dir: None,
            last_episode: None,
            recent: Vec::new(),
            auto_advance: true,
            gapless: true,
            output_device: None,
            timeline_height: None,
            inspector_width: None,
            appearance: t::Appearance::default(),
            edit_mode: editing::EditMode::default(),
        }
    }
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

/// Playback state of the transport. `AudioOut` mirrors it; the transition
/// functions below are pure and unit-tested without audio hardware.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Transport {
    #[default]
    Idle,
    Playing(usize),
    Paused(usize),
}
impl Transport {
    fn turn(self) -> Option<usize> {
        match self {
            Self::Idle => None,
            Self::Playing(t) | Self::Paused(t) => Some(t),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpaceAction {
    Play(usize),
    Pause,
    Resume,
    /// The selected turn has no audio; the caller shows the Missing audio state.
    Missing(usize),
    Nothing,
}
/// Space: play the selection, toggle pause on the playing turn, or switch to
/// the newly selected turn. A missing-audio turn never starts playing.
fn space_action(
    transport: Transport,
    selected: Option<usize>,
    playable: impl Fn(usize) -> bool,
) -> (Transport, SpaceAction) {
    let Some(turn) = selected else {
        return (transport, SpaceAction::Nothing);
    };
    if !playable(turn) {
        return (Transport::Idle, SpaceAction::Missing(turn));
    }
    match transport {
        Transport::Idle => (Transport::Playing(turn), SpaceAction::Play(turn)),
        Transport::Playing(t) if t == turn => (Transport::Paused(t), SpaceAction::Pause),
        Transport::Playing(_) => (Transport::Playing(turn), SpaceAction::Play(turn)),
        Transport::Paused(t) if t == turn => (Transport::Playing(t), SpaceAction::Resume),
        Transport::Paused(_) => (Transport::Playing(turn), SpaceAction::Play(turn)),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EscAction {
    Stop,
    Nothing,
}
/// Esc stops playback; with nothing playing it leaves the transport alone.
fn esc_action(transport: Transport) -> (Transport, EscAction) {
    if transport.turn().is_some() {
        (Transport::Idle, EscAction::Stop)
    } else {
        (transport, EscAction::Nothing)
    }
}
/// Auto-advance target: the next visible turn after `current` that has audio
/// (a missing-audio turn cannot play, so it is skipped). `None` at the end of
/// the visible list — playback stops there — or when `current` is not visible.
fn advance_target(episode: &Episode, visible: &[usize], current: usize) -> Option<usize> {
    visible
        .iter()
        .skip_while(|i| **i != current)
        .skip(1)
        .find(|i| episode.turns[**i].verification != V::MissingAudio)
        .copied()
}
/// Previous/next visible turn relative to the selection, for the inspector's
/// buttons; with no visible selection, Previous wraps to the last visible turn
/// and Next to the first (matching Up/Down).
fn neighbour_targets(visible: &[usize], current: Option<usize>) -> (Option<usize>, Option<usize>) {
    match current.and_then(|c| visible.iter().position(|i| *i == c)) {
        Some(pos) => (
            pos.checked_sub(1).map(|p| visible[p]),
            visible.get(pos + 1).copied(),
        ),
        None => (visible.last().copied(), visible.first().copied()),
    }
}
#[derive(Default)]
struct Browser {
    editing: editing::Editing,
    jobs: jobs::Jobs,
    export_command: Option<Result<ExportCommand, String>>,
    live_task: Option<DawTask>,
    logic_task: Option<DawTask>,
    daw_notice: Option<Result<String, String>>,
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
    persist: bool,
    capture: Option<capture::Capture>,
    pending_select: Option<usize>,
    select_error: Option<String>,
    /// The audio output; `None` only in headless tests, `Some(Err)` when no
    /// output device could be opened.
    audio: Option<Result<AudioOut, String>>,
    transport: Transport,
    muted: bool,
    /// Last output-device loss from the stream error callback. While set, the
    /// transport is paused and playback controls stay disabled until a working
    /// output is chosen.
    device_error: Option<String>,
    /// The Unavailable device card is hidden (but the error stays active).
    device_card_dismissed: bool,
    waveforms: Option<Waveforms>,
    /// Show the Missing audio card for the selected turn.
    missing_audio_card: bool,
    /// Transient playback/open notices, shown in the status bar.
    audio_notice: Option<String>,
    /// Cached `AudioOut::outputs()`; refreshed at startup, on device errors
    /// and whenever a device menu opens.
    device_list: Vec<String>,
    /// Probe cache for the inspector FILE field and the idle duration, keyed
    /// by turn id so it is recomputed only when the selection changes.
    selected_media: Option<(String, SelectedMedia)>,
    /// Settings window (⌘, / gear).
    settings_open: bool,
}
struct SelectedMedia {
    file_line: String,
    duration: Option<Duration>,
}
impl Browser {
    fn new(cc: &eframe::CreationContext<'_>, cli: &capture::Cli) -> Self {
        let mut preferences = if cli.effective_fresh() {
            Preferences::default()
        } else {
            cc.storage
                .and_then(|s| eframe::get_value(s, STORAGE_KEY))
                .unwrap_or_default()
        };
        if cli.no_gapless {
            preferences.gapless = false;
        }
        if let Some((timeline_height, inspector_width)) = cli.layout {
            preferences.timeline_height = Some(timeline_height);
            preferences.inspector_width = Some(inspector_width);
        }
        if let Some(mode) = cli.edit_mode {
            preferences.edit_mode = mode;
        }
        t::apply(&cc.egui_ctx);
        t::set_appearance(&cc.egui_ctx, preferences.appearance);
        let muted = cli.effective_muted();
        let (audio, audio_notice) = match AudioOut::open(preferences.output_device.as_deref()) {
            Ok(mut audio) => {
                audio.set_muted(muted);
                let notice = audio.take_notice();
                (Some(Ok(audio)), notice)
            }
            Err(message) => (
                Some(Err(message.clone())),
                Some(format!("Audio output unavailable: {message}")),
            ),
        };
        let mut app = Self {
            jobs: jobs::Jobs {
                login: Some(cli::start_login_path()),
                mode: cli.inspector,
                auto_confirm: cli.confirm.clone(),
                auto_run: cli.run.clone(),
                running_screenshot: cli.running_screenshot.clone(),
                ..Default::default()
            },
            preferences,
            persist: cli.persist(),
            capture: cli.screenshot.clone().map(|path| {
                capture::Capture::new(path, cli.play, cli.at, cli.simulate_device_loss)
            }),
            pending_select: cli.select,
            audio,
            audio_notice,
            muted,
            device_list: AudioOut::outputs(),
            settings_open: cli.settings,
            ..Self::default()
        };
        app.editing.capture_open = cli.edit;
        if let Some(path) = cli
            .episode
            .clone()
            .or_else(|| app.preferences.last_episode.clone())
        {
            app.open(path, &cc.egui_ctx);
        }
        app
    }
    fn open(&mut self, path: PathBuf, ctx: &egui::Context) {
        if self.jobs.busy() || self.editing.busy() {
            return;
        }
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
        self.export_command = None;
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
        self.export_command = None;
        self.daw_notice = None;
        // Reloading replaces the WAV paths under any playing player; stop first.
        self.stop_audio();
        self.missing_audio_card = false;
        self.selected_media = None;
        self.waveforms = None; // Drop old ones before starting new
        self.waveforms = Some(Waveforms::start(&episode, ctx.clone()));
        let same = self
            .episode
            .as_ref()
            .is_some_and(|old| old.dir == episode.dir);
        let previous_row = self.selected;
        let selected_id = if same {
            self.selected
                .and_then(|i| self.episode.as_ref()?.turns.get(i))
                .map(|t| t.id.clone())
        } else {
            None
        };
        self.selected = selected_id
            .and_then(|id| episode.turns.iter().position(|t| t.id == id))
            .or_else(|| previous_row.filter(|i| same && *i < episode.turns.len()));
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
        self.overview = episode.overview_sections(overview_section_count(episode.total_ms));
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
        self.editing.cancel();
        self.editing.invalidate();
        self.editing.manifest_len = Some(episode.turns.len());
        self.episode = Some(episode);
        self.error = None;
        self.jobs.selected.clear();
        self.refresh_checkpoints();
        self.refresh_commands(ctx);
        // Reset virtual scroll on a new episode; preserve/reveal selection on reload.
        self.scroll_to = self.selected.or(Some(0));
        if let Some(want) = self.pending_select.take() {
            let episode = self.episode.as_ref().expect("just accepted");
            let position = episode.turns.iter().position(|turn| turn.index == want);
            let count = episode.turns.len();
            match position {
                Some(position) => self.select(position),
                None => {
                    self.select_error = Some(format!(
                        "episode has no manifest turn {want} ({count} turns loaded)"
                    ));
                }
            }
        }
    }
    fn select(&mut self, index: usize) {
        if !self.editing.allow_selection(index, false) {
            return;
        }
        let mut switch = false;
        if let Some(e) = &self.episode
            && let Some(turn) = e.turns.get(index)
        {
            self.jobs.selected.clear();
            self.jobs.selected.insert(index);
            self.selected = Some(index);
            self.scroll_to = Some(index);
            self.timeline.reveal(turn.start_ms, turn.end_ms, e.total_ms);
            switch = self.transport.turn().is_some_and(|t| t != index);
        }
        // Selecting a different turn while playing switches playback
        // immediately; a turn with missing audio stops playback instead.
        if switch {
            let playable = self
                .episode
                .as_ref()
                .and_then(|e| e.turns.get(index))
                .is_some_and(|turn| turn.verification != V::MissingAudio);
            if playable {
                self.start_audio(index);
            } else {
                self.stop_audio();
            }
        }
    }
    /// Space / the Play-Pause button.
    fn play_selected(&mut self) {
        // A lost device must be replaced through the device menu first;
        // resuming a dead stream would look like playback but stay silent.
        if self.device_error.is_some() {
            return;
        }
        let Some(e) = &self.episode else {
            return;
        };
        let playable = |i: usize| {
            e.turns
                .get(i)
                .is_some_and(|t| t.verification != V::MissingAudio)
        };
        let (_, action) = space_action(self.transport, self.selected, playable);
        match action {
            SpaceAction::Play(i) => self.start_audio(i),
            SpaceAction::Pause => {
                if let Some(Ok(audio)) = self.audio.as_mut() {
                    audio.pause();
                }
                if let Some(t) = self.transport.turn() {
                    self.transport = Transport::Paused(t);
                }
            }
            SpaceAction::Resume => {
                if let Some(Ok(audio)) = self.audio.as_mut() {
                    audio.resume();
                }
                if let Some(t) = self.transport.turn() {
                    self.transport = Transport::Playing(t);
                }
            }
            SpaceAction::Missing(_) => {
                self.stop_audio();
                self.missing_audio_card = true;
            }
            SpaceAction::Nothing => {}
        }
    }
    /// Start playback of turn `idx` (a playable turn) on the audio output.
    fn start_audio(&mut self, idx: usize) {
        let target = self
            .episode
            .as_ref()
            .and_then(|e| e.turns.get(idx))
            .map(|turn| {
                (
                    turn.wav_paths.clone(),
                    Duration::from_millis(turn.end_ms.saturating_sub(turn.start_ms)),
                )
            });
        let Some((wavs, fallback)) = target else {
            return;
        };
        match self.audio.as_mut() {
            Some(Ok(audio)) => {
                audio.set_fallback_duration(fallback);
                match audio.play(idx, &wavs) {
                    Ok(()) => {
                        self.transport = Transport::Playing(idx);
                        self.audio_notice = None;
                    }
                    Err(message) => {
                        self.transport = Transport::Idle;
                        self.audio_notice = Some(message);
                    }
                }
            }
            Some(Err(message)) => {
                let message = message.clone();
                self.audio_notice = Some(format!("Audio output unavailable: {message}"));
            }
            None => {}
        }
    }
    fn stop_audio(&mut self) {
        self.transport = Transport::Idle;
        if let Some(Ok(audio)) = self.audio.as_mut() {
            audio.stop();
        }
    }
    /// Natural end of the playing turn: with auto-advance, select and play the
    /// next visible turn with audio; otherwise (or at the list end) stop.
    fn on_playback_finished(&mut self, turn: usize) {
        self.stop_audio();
        if !self.preferences.auto_advance {
            return;
        }
        let visible = self.visible_indices();
        let next = self
            .episode
            .as_ref()
            .and_then(|e| advance_target(e, &visible, turn));
        if let Some(next) = next {
            self.select(next);
            self.start_audio(next);
        }
    }
    /// The stream error callback reported the output device as gone: pause at
    /// the current position, keep the selection, show the recovery state.
    fn on_device_error(&mut self, message: String) {
        if let Some(Ok(audio)) = self.audio.as_mut() {
            audio.pause();
        }
        if let Some(t) = self.transport.turn() {
            self.transport = Transport::Paused(t);
        }
        self.device_list = AudioOut::outputs();
        self.device_card_dismissed = false;
        self.audio_notice = Some(format!("Output device lost: {message}"));
        self.device_error = Some(message);
    }
    /// Open a (possibly new) output device by name. A turn that was playing or
    /// paused resumes from its tracked position on the new device.
    fn choose_device(&mut self, name: String) {
        let resume = self.transport.turn().map(|t| (t, self.audio_position()));
        match AudioOut::open(Some(&name)) {
            Ok(mut audio) => {
                audio.set_muted(self.muted);
                if let Some(notice) = audio.take_notice() {
                    self.audio_notice = Some(notice);
                }
                self.audio = Some(Ok(audio));
                self.device_error = None;
                self.preferences.output_device = Some(name);
                if let Some((turn, position)) = resume {
                    self.start_audio(turn);
                    if matches!(self.transport, Transport::Playing(_))
                        && !position.is_zero()
                        && let Some(Ok(audio)) = self.audio.as_mut()
                    {
                        audio.seek(position);
                    }
                }
            }
            Err(message) => {
                self.audio_notice = Some(format!("Audio output unavailable: {message}"));
                self.audio = Some(Err(message));
            }
        }
    }
    fn audio_position(&self) -> Duration {
        self.audio
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or(Duration::ZERO, AudioOut::position)
    }
    fn visible_indices(&self) -> Vec<usize> {
        match &self.episode {
            Some(e) => self
                .filters
                .matches(e)
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.then_some(i))
                .collect(),
            None => Vec::new(),
        }
    }
    /// Refresh the inspector FILE field / idle duration probe cache when the
    /// selection moved to a different turn.
    fn refresh_selected_media(&mut self) {
        let current = self
            .selected
            .and_then(|i| self.episode.as_ref()?.turns.get(i))
            .map(|turn| (turn.id.clone(), turn.wav_paths.clone()));
        let stale = match (&self.selected_media, &current) {
            (Some((cached, _)), Some((id, _))) => cached != id,
            (None, Some(_)) => true,
            (Some(_), None) => {
                self.selected_media = None;
                false
            }
            (None, None) => false,
        };
        if stale && let Some((id, wavs)) = current {
            self.selected_media = Some((id, probe_turn(&wavs)));
        }
    }
    /// Device-error poll, natural-finish detection and the playing-only
    /// repaint cadence. Runs in `logic`, before any capture handling.
    fn poll_audio(&mut self, ctx: &egui::Context) {
        let device_error = match self.audio.as_mut() {
            Some(Ok(audio)) => {
                audio.sync_queue();
                audio.take_device_error()
            }
            _ => None,
        };
        if let Some(message) = device_error {
            self.on_device_error(message);
        }
        if let Transport::Playing(turn) = self.transport {
            let finished = matches!(&self.audio, Some(Ok(audio)) if audio.is_finished());
            let new_playing = self
                .audio
                .as_ref()
                .and_then(|a| a.as_ref().ok())
                .and_then(|a| a.playing());
            if let Some(new) = new_playing {
                if new != turn {
                    self.transport = Transport::Playing(new);
                    self.select(new);
                    let visible = self.visible_indices();
                    let target = self
                        .episode
                        .as_ref()
                        .and_then(|e| advance_target(e, &visible, turn));
                    if Some(new) != target {
                        if let Some(target) = target {
                            self.start_audio(target);
                        } else {
                            self.stop_audio();
                        }
                    }
                } else if self.preferences.auto_advance && self.preferences.gapless {
                    let queued_after = self
                        .audio
                        .as_ref()
                        .and_then(|a| a.as_ref().ok())
                        .is_some_and(|a| a.queued_after_current());
                    if !queued_after {
                        let visible = self.visible_indices();
                        let target = self
                            .episode
                            .as_ref()
                            .and_then(|e| advance_target(e, &visible, turn));
                        if let Some(next) = target {
                            let wavs = self
                                .episode
                                .as_ref()
                                .and_then(|e| e.turns.get(next))
                                .map(|t| t.wav_paths.clone());
                            if let (Some(wavs), Some(Ok(audio))) = (wavs, self.audio.as_mut()) {
                                let _ = audio.queue_next(next, &wavs);
                            }
                        }
                    }
                }
            }
            if finished {
                self.on_playback_finished(turn);
            }
        }
        if matches!(self.transport, Transport::Playing(_)) {
            ctx.request_repaint_after(REPLAY_TICK);
        }
    }
    fn capture_logic(&mut self, ctx: &egui::Context) {
        if self.capture.is_some()
            && self.pending.is_none()
            && let Some((path, error)) = &self.error
        {
            capture::fail(&format!("could not open {}: {error}", path.display()));
        }
        if let Some(message) = self.select_error.take() {
            if self.capture.is_some() {
                capture::fail(&message);
            }
            eprintln!("scriptcast-ui: {message}");
        }
        if self.jobs.busy()
            || self.editing.loading()
            || self.editing.capture_open
            || self.jobs.auto_run.is_some()
            || self.jobs.auto_confirm.is_some()
            || self.pending.is_some()
        {
            if let Some(cap) = &mut self.capture {
                cap.deadline = Instant::now() + capture::TIMEOUT;
            }
            ctx.request_repaint_after(capture::TICK);
            return;
        }
        let Some(cap) = &mut self.capture else {
            return;
        };
        if Instant::now() >= cap.deadline {
            capture::fail(&format!(
                "timed out after {} s while {}",
                capture::TIMEOUT.as_secs(),
                cap.phase.describe(cap.frames)
            ));
        }
        let mut received = None;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Screenshot {
                    image, user_data, ..
                } = event
                    && user_data.data.is_none()
                {
                    received = Some(Arc::clone(image));
                }
            }
        });
        if let Some(image) = received {
            match capture::write_png(&cap.path, &image) {
                Ok(()) => {
                    println!(
                        "screenshot: {} {}x{}",
                        cap.path.display(),
                        image.width(),
                        image.height()
                    );
                    std::process::exit(0);
                }
                Err(message) => capture::fail(&message),
            }
        }
        if cap.phase == capture::Phase::Loading && self.pending.is_none() {
            if let Some((path, error)) = &self.error {
                capture::fail(&format!("could not open {}: {error}", path.display()));
            }
            if self.pending_select.is_some() {
                capture::fail("--select needs an episode, but none was loaded");
            }
            cap.phase = capture::Phase::Settling;
            cap.quiet_since = Instant::now();
            // Copy the flags out so the play calls can take `&mut self`.
            let (play, at, simulate_loss) = (cap.play, cap.at, cap.simulate_device_loss);
            if play {
                // Play the selected turn, seek, then optionally inject the
                // device loss through the same channel real errors use.
                self.play_selected();
                if matches!(self.transport, Transport::Playing(_))
                    && let Some(Ok(audio)) = self.audio.as_mut()
                {
                    if at > Duration::ZERO {
                        audio.seek(at);
                    }
                    if simulate_loss {
                        audio.simulate_device_loss_for_capture(
                            "simulated output device loss (cpal StreamError::DeviceNotAvailable)",
                        );
                    }
                }
            }
        }
    }
    fn capture_settle(&mut self, ctx: &egui::Context) {
        if self.jobs.busy()
            || self.editing.loading()
            || self.editing.capture_open
            || self.jobs.auto_run.is_some()
            || self.jobs.auto_confirm.is_some()
            || self.pending.is_some()
        {
            if let Some(cap) = &mut self.capture {
                cap.deadline = Instant::now() + capture::TIMEOUT;
            }
            ctx.request_repaint_after(capture::TICK);
            return;
        }
        let Some(cap) = &mut self.capture else {
            return;
        };
        // Requests made during this frame (scroll reveals, finished icon loads)
        // mean the UI is still moving; only an idle frame advances the clock.
        let repaint_pending = ctx.has_requested_repaint();
        if cap.phase == capture::Phase::Settling {
            cap.frames += 1;
            if repaint_pending {
                cap.quiet_since = Instant::now();
            }
            // While playing, the 50 ms progress repaint never goes quiet, so
            // play runs use the position/card condition instead of the quiet
            // window (icon loads finish well inside the 200 ms audio advance).
            // Terminal states (Missing audio or Unavailable device card) must
            // still be captured; a turn that never started is a failure, not
            // a picture. Field-wise borrows keep `cap` usable throughout.
            let ready = if cap.play {
                if cap.frames < capture::MIN_FRAMES {
                    false
                } else if self.device_error.is_some() || self.missing_audio_card {
                    true
                } else {
                    match (self.transport, &self.audio) {
                        (Transport::Playing(_), Some(Ok(audio))) => {
                            cap.play_started = true;
                            audio.position() >= cap.at + Duration::from_millis(200)
                        }
                        _ if cap.play_started => true,
                        _ => capture::fail(&format!(
                            "--play could not start playback: {}",
                            self.audio_notice
                                .as_deref()
                                .unwrap_or("no audio output is available")
                        )),
                    }
                }
            } else {
                cap.frames >= capture::MIN_FRAMES
                    && !repaint_pending
                    && cap.quiet_since.elapsed() >= capture::QUIET
            };
            if ready {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                cap.phase = capture::Phase::Requested;
                cap.requested_at = Instant::now();
            }
        } else if cap.phase == capture::Phase::Requested
            && cap.requested_at.elapsed() >= Duration::from_secs(1)
        {
            // The screenshot action is only drained on frames the OS reports
            // as visible (eframe glow_integration.rs); a request issued before
            // the window became visible would wait forever, so re-request.
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            cap.requested_at = Instant::now();
        }
        ctx.request_repaint_after(capture::TICK);
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        let editor_focused = ctx.memory(|m| m.has_focus(editing::editor_id()));
        let dialog = self.jobs.confirmation.is_some()
            || self.jobs.restore.is_some()
            || self.editing.pending_selection.is_some();
        if !dialog && self.editing.draft.is_some() {
            if ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)
                    || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)
            }) {
                self.save_draft(ctx);
            }
            if editor_focused
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            {
                self.editing.cancel();
                ctx.memory_mut(|m| m.surrender_focus(editing::editor_id()));
                return;
            }
        }
        if editing::suppress_shortcuts(editor_focused, ctx.text_edit_focused(), dialog) {
            return;
        }
        if ctx.text_edit_focused() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::J)) {
            self.jobs.mode = Some(InspectorMode::Jobs);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
            self.settings_open = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::E))
            && let Some(row) = self.selected
        {
            self.open_editor(row);
        }
        let (open, reload, search, up, down, escape, space) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::O),
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::R),
                i.consume_key(egui::Modifiers::COMMAND, egui::Key::F),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space),
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
            if self.settings_open {
                self.settings_open = false;
            } else {
                self.pending = None;
                self.error = None;
                self.missing_audio_card = false;
                self.device_card_dismissed = true;
                if esc_action(self.transport).1 == EscAction::Stop {
                    self.stop_audio();
                }
            }
        }
        if space {
            self.play_selected();
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
                let next = visible[next];
                self.select(next);
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
        actions.spacing_mut().item_spacing.x = t::SPACE_8;
        // This row is laid out right-to-left: Settings, Logic, Live, Reload, Open.
        let settings =
            t::icon_button(&mut actions, "settings-16-text", true).on_hover_text("Settings (⌘,)");
        if settings.clicked() || self.jobs.configure {
            self.settings_open = true;
            self.jobs.configure = false;
        }
        // Show a placeholder while discovery runs, but never store it: the script
        // loader waits for a real result in `export_command`.
        let export = self
            .export_command
            .clone()
            .unwrap_or_else(|| Err("Resolving CLI commands…".into()));
        let logic_reason = if self.logic_task.is_some() {
            Some("Preparing for Logic…".into())
        } else {
            daw::logic_disabled_reason(self.episode.as_ref(), &export)
        };
        let source = match &export {
            Ok(command) => command.description(),
            Err(error) => error.clone(),
        };
        let logic_hint = if self.episode.as_ref().is_some_and(daw::fcpxml_is_fresh) {
            format!("Open existing fresh FCPXML; no export needed.\n{source}")
        } else {
            format!("Export FCPXML, then open in Logic.\n{source}")
        };
        let logic_hint = logic_reason.as_ref().map_or(logic_hint.clone(), |reason| {
            format!("{reason}\n{logic_hint}")
        });
        let logic = t::secondary_button(
            &mut actions,
            if self.logic_task.is_some() {
                "Preparing for Logic…"
            } else {
                "Open in Logic"
            },
            None,
            logic_reason.is_none(),
        )
        .on_hover_text(&logic_hint)
        .on_disabled_hover_text(&logic_hint)
        .clicked();
        let live_reason = if self.live_task.is_some() {
            Some("Opening in Live…".into())
        } else {
            daw::live_disabled_reason(self.episode.as_ref())
        };
        let live_hint = live_reason.clone().unwrap_or_else(|| {
            self.episode
                .as_ref()
                .and_then(daw::live_target)
                .map_or_else(String::new, |p| format!("Open {} in Live", p.display()))
        });
        let live = t::secondary_button(&mut actions, "Open in Live", None, live_reason.is_none())
            .on_hover_text(&live_hint)
            .on_disabled_hover_text(&live_hint)
            .clicked();
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
                    e.rendered_at.as_deref().map_or_else(
                        || "unknown".to_string(),
                        |iso| rendered_when(iso).into_owned(),
                    )
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
        if live && let Some(path) = self.episode.as_ref().and_then(daw::live_target) {
            self.daw_notice = Some(Ok(format!("Opening {} in Live", daw::filename(&path))));
            self.live_task = Some(DawTask::live(path, ctx.clone()));
        }
        if logic && let Some(episode) = &self.episode {
            self.daw_notice = Some(Ok(if daw::fcpxml_is_fresh(episode) {
                "Opening in Logic…".into()
            } else {
                format!("Writing {}…", daw::filename(&daw::fcpxml_path(episode)))
            }));
            self.logic_task = Some(DawTask::logic(
                episode,
                export,
                daw::logic_command,
                ctx.clone(),
            ));
        }
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
        let prompt = "Search turns";
        let hint = "Cmd+F";
        let hint_width = t::text_width(&ui, hint, t::font(t::CAPTION));
        // The field grows so prompt and hint always fit; the token is the floor.
        let search_width = t::SEARCH_WIDTH.max(
            t::SPACE_7
                + t::SPACE_14
                + t::SPACE_7
                + t::text_width(&ui, prompt, t::font(t::BODY))
                + t::SPACE_8
                + hint_width
                + t::SPACE_8,
        );
        let (search_rect, _) =
            ui.allocate_exact_size(vec2(search_width, t::SEARCH_HEIGHT), Sense::hover());
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
        let hint_rect = Rect::from_min_max(
            pos2(
                search_rect.right() - t::SPACE_8 - hint_width,
                search_rect.top(),
            ),
            pos2(search_rect.right() - t::SPACE_8, search_rect.bottom()),
        );
        let edit = Rect::from_min_max(
            pos2(icon.right() + t::SPACE_7, search_rect.top()),
            pos2(hint_rect.left() - t::SPACE_8, search_rect.bottom()),
        );
        // A child ui, not `ui.put`: `put` advances the parent cursor past the
        // edit rect, which pushed the following ComboBox into the search field.
        let mut edit_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(edit)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let response = edit_ui.add(
            egui::TextEdit::singleline(&mut self.filters.search)
                .id(egui::Id::new("turn_search"))
                .font(t::font(t::BODY))
                .hint_text(prompt)
                .desired_width(edit.width())
                .frame(egui::Frame::NONE),
        );
        if self.focus_search {
            response.request_focus();
            self.focus_search = false;
        }
        view::line(&ui, hint_rect, hint, t::font(t::CAPTION), t::TEXT_SUBTLE);
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
            if t::secondary_button(ui, "Jobs", None, true)
                .on_hover_text("Job runner · Cmd+J")
                .clicked()
            {
                self.jobs.mode = Some(InspectorMode::Jobs);
            }
            if t::secondary_button(ui, "Verification", None, self.selected.is_some()).clicked() {
                self.jobs.mode = Some(InspectorMode::Verification);
            }
            if t::secondary_button(ui, "Selected", None, self.selected.is_some()).clicked() {
                self.jobs.mode = Some(InspectorMode::SelectedTurn);
            }
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
        let left_height = empty_left_height(ui);
        let right_height = self.empty_right_height(ui);
        // Content height, centred vertically; an inner scroll only appears when
        // the window is shorter than the column's content.
        let height = left_height
            .max(right_height)
            .min((rect.height() - t::SPACE_20 * 2.0).max(t::SPACE_20));
        let top = rect.center().y - height / 2.0;
        let left = Rect::from_min_size(pos2(x, top), vec2(t::EMPTY_LEFT, height));
        let divider = Rect::from_min_size(
            pos2(left.right() + t::EMPTY_GAP, top),
            vec2(t::BORDER, height),
        );
        ui.painter().rect_filled(divider, 0, t::BORDER_DEFAULT);
        let right = Rect::from_min_size(
            pos2(divider.right() + t::EMPTY_GAP, top),
            vec2(t::EMPTY_RIGHT, height),
        );
        let mut open = false;
        let mut left_ui = view::region(ui, "empty_guidance", left, t::BG_WINDOW);
        if left_height > height {
            egui::ScrollArea::vertical()
                .id_salt("empty_guidance_scroll")
                .auto_shrink([false, false])
                .show(&mut left_ui, |ui| open |= empty_left(ui));
        } else {
            open |= empty_left(&mut left_ui);
        }
        let mut path = None;
        let mut right_ui = view::region(ui, "recent_episodes", right, t::BG_WINDOW);
        if right_height > height {
            egui::ScrollArea::vertical()
                .id_salt("recent_scroll")
                .auto_shrink([false, false])
                .show(&mut right_ui, |ui| {
                    let (open_dir, selected) = self.empty_right(ui);
                    open |= open_dir;
                    path = selected;
                });
        } else {
            let (open_dir, selected) = self.empty_right(&mut right_ui);
            open |= open_dir;
            path = selected;
        }
        let ctx = ui.ctx().clone();
        if open {
            self.pick(&ctx);
        }
        if let Some(path) = path {
            self.open(path, &ctx);
        }
    }
    /// Content height of the recent-episodes column; mirrors `empty_right`.
    fn empty_right_height(&self, ui: &Ui) -> f32 {
        // The heading row shares a line with the "Open directory ↗" link, an
        // interactive widget, so it can grow to the theme's interact height.
        let mut height = t::line_height(ui, t::bold(t::TEXT_13)).max(t::BUTTON_HEIGHT)
            + t::SPACE_6
            + t::SPACE_12;
        if self.preferences.recent.is_empty() {
            height += t::wrapped_height(
                ui,
                "No recent episodes yet.",
                t::font(t::BODY),
                t::EMPTY_RIGHT,
            );
        } else {
            let rows = self.preferences.recent.len() as f32;
            height += rows * t::EPISODE_HEADER_HEIGHT + (rows - 1.0) * t::SPACE_6;
        }
        height
            + t::SPACE_6
            + t::SPACE_16
            + t::wrapped_height(ui, LOCAL_FILES_NOTE, t::font(t::CAPTION), t::EMPTY_RIGHT)
    }
    /// Right column of the empty workspace. Returns (open-directory clicked,
    /// selected recent path).
    fn empty_right(&self, ui: &mut Ui) -> (bool, Option<PathBuf>) {
        let mut open = false;
        let mut path = None;
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
        ui.label(t::text(LOCAL_FILES_NOTE, t::CAPTION, t::TEXT_MUTED));
        (open, path)
    }
    fn playback(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        );
        self.refresh_selected_media();
        let audio_ok = matches!(&self.audio, Some(Ok(_)));
        // The bar (except the device menu) is disabled at 45% when nothing is
        // selected or there is no usable audio; a lost device disables it too.
        let live = self.episode.is_some()
            && self.selected.is_some()
            && audio_ok
            && self.device_error.is_none();
        let playing_turn = self.transport.turn();
        let shown = playing_turn.or(self.selected).and_then(|i| {
            let e = self.episode.as_ref()?;
            let turn = e.turns.get(i)?;
            Some((turn.index, e.speakers[turn.speaker_idx].name.clone()))
        });
        let (position_ms, duration_ms) = match self.transport {
            Transport::Playing(_) | Transport::Paused(_) => {
                let audio = self.audio.as_ref().and_then(|r| r.as_ref().ok());
                let position = audio.map_or(0, |a| a.position().as_millis() as u64);
                let duration = audio
                    .map(|a| a.duration())
                    .filter(|d| !d.is_zero())
                    .or_else(|| self.selected_media.as_ref().and_then(|(_, m)| m.duration));
                (position, duration)
            }
            Transport::Idle => (
                0,
                self.selected_media.as_ref().and_then(|(_, m)| m.duration),
            ),
        };
        let mut bar = ui.new_child(
            UiBuilder::new()
                .max_rect(view::inset(rect, t::SPACE_20, 0.0))
                .layout(Layout::left_to_right(Align::Center)),
        );
        bar.spacing_mut().item_spacing.x = t::SPACE_18;
        let mut play_clicked = false;
        let mut stop_clicked = false;
        let mut chosen_device: Option<String> = None;
        bar.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = t::SPACE_7;
            let playing = matches!(self.transport, Transport::Playing(_));
            let (label, icon) = if playing {
                ("Pause", "pause-16-on-accent")
            } else {
                ("Play", "play-16-on-accent")
            };
            play_clicked = t::primary_button(ui, label, Some(icon), live).clicked();
            stop_clicked =
                t::secondary_button(ui, "Stop", Some("square-16-text"), playing_turn.is_some())
                    .clicked();
        });
        let (label, position) = match &shown {
            Some((index, name)) => (
                format!("Turn {index:03} · {name}"),
                format!(
                    "{} / {}",
                    view::time(position_ms),
                    duration_ms.map_or("—".into(), |d| view::time(d.as_millis() as u64))
                ),
            ),
            None => ("Select a turn to audition".into(), "00:00.000 / —".into()),
        };
        let current_width = t::PLAYBACK_CURRENT_AUDIO_WIDTH
            .max(t::text_width(&bar, label.as_str(), t::font(t::BODY)))
            .max(t::text_width(&bar, position.as_str(), t::mono(t::NUMERIC)));
        let (current, _) = bar.allocate_exact_size(
            vec2(current_width, t::BUTTON_HEIGHT + t::SPACE_12),
            Sense::hover(),
        );
        let top = Rect::from_min_size(current.min, vec2(current.width(), current.height() / 2.0));
        view::line(&bar, top, &label, t::font(t::BODY), t::TEXT_PRIMARY);
        view::line(
            &bar,
            top.translate(vec2(0.0, top.height())),
            &position,
            t::mono(t::NUMERIC),
            t::TEXT_MUTED,
        );
        let remaining = bar.available_rect_before_wrap();
        let mut options = bar.new_child(
            UiBuilder::new()
                .max_rect(remaining)
                .layout(Layout::right_to_left(Align::Center)),
        );
        options.spacing_mut().item_spacing.x = t::SPACE_18;
        // The output device menu stays enabled even when the rest of the bar
        // is disabled, so a lost device can be recovered from here too.
        let current_device = match &self.audio {
            Some(Ok(audio)) => audio.device_name().to_string(),
            Some(Err(_)) | None => "No output".to_string(),
        };
        let combo = egui::ComboBox::from_id_salt("output_device")
            .selected_text(t::text(current_device.clone(), t::SECONDARY, t::TEXT_MUTED))
            .show_ui(&mut options, |ui| {
                if self.device_list.is_empty() {
                    ui.label(t::text(
                        "No output devices found",
                        t::SECONDARY,
                        t::TEXT_MUTED,
                    ));
                }
                for name in &self.device_list {
                    if ui.selectable_label(*name == current_device, name).clicked() {
                        chosen_device = Some(name.clone());
                    }
                }
            });

        let mut response = combo.response;
        if let Some(Ok(audio)) = &self.audio {
            let khz = audio.stream_rate() as f64 / 1000.0;
            let tip = if khz.fract() == 0.0 {
                format!("Output at {} kHz", khz as u64)
            } else {
                format!("Output at {:.1} kHz", khz)
            };
            response = response.on_hover_text(tip);
        }

        if response.clicked() {
            self.device_list = AudioOut::outputs();
        }
        options.add_enabled_ui(live, |ui| {
            ui.label(t::text("Space  Play / pause", t::SECONDARY, t::TEXT_SUBTLE));
            let mut auto = self.preferences.auto_advance;
            if ui
                .checkbox(
                    &mut auto,
                    egui::RichText::new("Auto-advance")
                        .font(t::font(t::BODY))
                        .color(t::TEXT_MUTED),
                )
                .changed()
            {
                self.preferences.auto_advance = auto;
            }
        });
        let track = Rect::from_min_max(
            pos2(
                remaining.left(),
                remaining.center().y - t::WAVE_VIEWPORT_HEIGHT / 2.0,
            ),
            pos2(
                (options.min_rect().left() - t::SPACE_18).max(remaining.left()),
                remaining.center().y + t::WAVE_VIEWPORT_HEIGHT / 2.0,
            ),
        );
        bar.painter().rect(
            track,
            t::RADIUS_WAVE_VIEWPORT,
            t::BG_SURFACE,
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
            StrokeKind::Inside,
        );
        let centre_x = track.center().x;
        let pos_sec = position_ms as f32 / 1000.0;
        let turn_idx = playing_turn.or(self.selected);
        let envelope = turn_idx.and_then(|i| self.waveforms.as_ref().and_then(|w| w.get(i)));
        let mut seek_to = None;
        if let Some(env) = envelope {
            let dur_sec = duration_ms.map(|d| d.as_secs_f32()).unwrap_or(0.0);
            let window_start = pos_sec - t::WAVE_VIEWPORT_SECONDS / 2.0;
            let bar_space = t::WAVE_BAR + t::WAVE_GAP;
            let n = (track.width() / bar_space) as usize;
            let max_h = track.height() * t::WAVE_VIEWPORT_FILL;
            let painter = bar.painter().with_clip_rect(track);
            if n > 0 && track.width() > 0.0 {
                for i in 0..n {
                    let t0 = window_start + (i as f32) * t::WAVE_VIEWPORT_SECONDS / n as f32;
                    let t1 = window_start + ((i + 1) as f32) * t::WAVE_VIEWPORT_SECONDS / n as f32;
                    let Some(peak) = waveform::peak_in_range(env, t0, t1, dur_sec) else {
                        continue;
                    };
                    let h = (peak * max_h).max(t::WAVE_MIN);
                    let x = track.left() + i as f32 * bar_space;
                    let y = track.center().y - h / 2.0;
                    let colour = if x + t::WAVE_BAR / 2.0 < centre_x {
                        t::accent()
                    } else {
                        t::TEXT_MUTED
                    };
                    painter.rect_filled(
                        Rect::from_min_size(pos2(x, y), vec2(t::WAVE_BAR, h)),
                        0.0,
                        colour,
                    );
                }
            }
            let playhead = Rect::from_center_size(
                pos2(centre_x, track.center().y),
                vec2(t::WAVE_PLAYHEAD, track.height()),
            );
            painter.rect_filled(playhead, 0.0, t::TEXT_PRIMARY);
            let clear = egui::Color32::TRANSPARENT;
            let fade = t::WAVE_FADE.min(track.width() / 2.0);
            let mut mesh = egui::Mesh::default();
            let left_inner = track.left() + fade;
            mesh.colored_vertex(pos2(track.left(), track.top()), t::BG_SURFACE);
            mesh.colored_vertex(pos2(left_inner, track.top()), clear);
            mesh.colored_vertex(pos2(track.left(), track.bottom()), t::BG_SURFACE);
            mesh.colored_vertex(pos2(left_inner, track.bottom()), clear);
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(1, 2, 3);
            let right_inner = track.right() - fade;
            let base = 4;
            mesh.colored_vertex(pos2(track.right(), track.top()), t::BG_SURFACE);
            mesh.colored_vertex(pos2(right_inner, track.top()), clear);
            mesh.colored_vertex(pos2(track.right(), track.bottom()), t::BG_SURFACE);
            mesh.colored_vertex(pos2(right_inner, track.bottom()), clear);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 2, base + 3);
            painter.add(egui::Shape::mesh(mesh));
        }
        let response = bar.interact(
            track,
            bar.id().with("wave_viewport"),
            Sense::click_and_drag(),
        );
        if (response.clicked() || response.dragged())
            && let Some(pointer) = response.interact_pointer_pos()
            && track.width() > 0.0
            && let Some(duration) = duration_ms.filter(|d| !d.is_zero())
        {
            let delta_sec = (pointer.x - centre_x) / track.width() * t::WAVE_VIEWPORT_SECONDS;
            let new_sec = (pos_sec + delta_sec).clamp(0.0, duration.as_secs_f32());
            seek_to = Some(Duration::from_secs_f32(new_sec));
        }
        if play_clicked {
            self.play_selected();
        }
        if stop_clicked {
            self.stop_audio();
        }
        if let Some(name) = chosen_device {
            self.choose_device(name);
        }
        if let Some(pos) = seek_to {
            match self.transport {
                Transport::Playing(_) | Transport::Paused(_) => {
                    if let Some(Ok(audio)) = self.audio.as_mut() {
                        audio.seek(pos);
                    }
                }
                Transport::Idle if live => {
                    self.play_selected();
                    if matches!(self.transport, Transport::Playing(_))
                        && let Some(Ok(audio)) = self.audio.as_mut()
                    {
                        audio.seek(pos);
                    }
                }
                Transport::Idle => {}
            }
        }
    }
    fn status(&self, ui: &mut Ui) {
        let rect = view::inset(ui.max_rect(), t::SPACE_20, 0.0);
        let status = if self.jobs.confirmation.is_some() || self.jobs.restore.is_some() {
            "Awaiting confirmation · no job has started".into()
        } else if let Some(notice) = &self.editing.notice {
            notice.clone()
        } else if let Some(notice) = &self.jobs.notice {
            notice.clone()
        } else if let Some(notice) = &self.daw_notice {
            match notice {
                Ok(message) | Err(message) => message.clone(),
            }
        } else if let Some(turn) = self.transport.turn() {
            let index = self
                .episode
                .as_ref()
                .and_then(|e| e.turns.get(turn))
                .map(|t| t.index)
                .unwrap_or(turn);
            let verb = match self.transport {
                Transport::Playing(_) => "Playing",
                Transport::Paused(_) => "Paused",
                Transport::Idle => unreachable!(),
            };
            format!("{verb} turn {index:03} · selected in timeline and table")
        } else if self.pending.is_some() {
            "Loading episode…".into()
        } else if self.error.is_some() {
            "Couldn’t open episode · see error details".into()
        } else if self.episode.is_some() && self.device_error.is_some() {
            self.audio_notice
                .clone()
                .unwrap_or_else(|| "Output device lost".into())
        } else if let Some(e) = &self.episode {
            let ready = format!(
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
            );
            match &self.audio_notice {
                Some(notice) => format!("{ready} · {notice}"),
                None => ready,
            }
        } else if let Some(notice) = &self.audio_notice {
            notice.clone()
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
        let color = if self.daw_notice.as_ref().is_some_and(Result::is_err) {
            t::STATUS_FAILED
        } else {
            t::TEXT_MUTED
        };
        view::line(
            ui,
            left,
            &status.replace('\n', " · "),
            t::font(t::CAPTION),
            color,
        );
        ui.interact(left, ui.id().with("status_details"), Sense::hover())
            .on_hover_text(&status);
    }
    fn poll_daw(&mut self) {
        for task in [&mut self.live_task, &mut self.logic_task] {
            while let Some(pending) = task.as_ref() {
                match pending.receiver.try_recv() {
                    Ok(DawEvent::Progress(message)) => self.daw_notice = Some(Ok(message)),
                    Ok(DawEvent::Finished(result)) => {
                        self.daw_notice = Some(result);
                        *task = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.daw_notice = Some(Err("DAW task stopped unexpectedly".into()));
                        *task = None;
                    }
                }
            }
        }
    }
    /// File name of the selected turn's first missing WAV, for the Missing
    /// audio card; `None` when nothing is missing.
    fn missing_file_name(&self, turn_idx: usize) -> Option<String> {
        let turn = self.episode.as_ref()?.turns.get(turn_idx)?;
        turn.wav_paths.iter().find(|p| !p.is_file()).map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string())
        })
    }
    fn workspace_card(&mut self, ctx: &egui::Context) {
        let loading = self
            .pending
            .as_ref()
            .filter(|p| p.started.elapsed() >= LOAD_NOTICE_DELAY)
            .map(|p| p.path.clone());
        // Missing audio: only while the selected turn still lacks its WAV.
        let missing = if self.missing_audio_card {
            self.selected
                .filter(|i| {
                    self.episode
                        .as_ref()
                        .and_then(|e| e.turns.get(*i))
                        .is_some_and(|t| t.verification == V::MissingAudio)
                })
                .and_then(|i| {
                    let e = self.episode.as_ref()?;
                    Some((e.turns[i].index, self.missing_file_name(i)?))
                })
        } else {
            None
        };
        let device = self
            .device_error
            .as_ref()
            .filter(|_| !self.device_card_dismissed)
            .map(|message| {
                let position = self.transport.turn().map(|_| self.audio_position());
                (message.clone(), position)
            });
        if loading.is_none() && self.error.is_none() && missing.is_none() && device.is_none() {
            return;
        }
        let mut choose = false;
        let mut retry = None;
        let mut dismiss = false;
        let mut cancel = false;
        let mut chosen_device: Option<String> = None;
        egui::Window::new("Workspace state")
            .id(egui::Id::new("workspace_state"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .fixed_size(vec2(t::STATE_CARD_WIDTH, 0.0))
            .frame(
                egui::Frame::new()
                    .fill(t::BG_SURFACE)
                    .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                    .corner_radius(t::RADIUS_CARD as u8)
                    .inner_margin(t::SPACE_16),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = t::SPACE_18;
                if let Some(path) = &loading {
                    let title = format!("Loading {}…", path.file_name().unwrap_or_default().to_string_lossy());
                    state_heading(ui, "loader-circle-22-accent", title);
                    ui.label(t::text(
                        "Reading turn metadata, audio paths and verification results.",
                        t::TEXT_13, t::TEXT_MUTED,
                    ));
                    let (bar, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), t::PROGRESS_HEIGHT), Sense::hover(),
                    );
                    ui.painter().rect_filled(bar, 0, t::BG_CONTROL);
                    let phase = (ui.input(|i| i.time) * 0.6).fract() as f32;
                    let width = bar.width() / 4.0;
                    let progress = Rect::from_min_size(
                        pos2(bar.left() + (bar.width() - width) * phase, bar.top()),
                        vec2(width, bar.height()),
                    );
                    ui.painter().rect_filled(progress, 0, t::accent());
                    cancel = t::secondary_button(ui, "Cancel", None, true).clicked();
                    ui.label(t::text(
                        "Your files remain local. Cancel keeps the current episode open.",
                        t::SECONDARY, t::TEXT_MUTED,
                    ));
                    ctx.request_repaint_after(Duration::from_millis(50));
                } else if let Some((path, error)) = &self.error {
                    state_heading(ui, "circle-alert-22-danger", "Couldn’t open this episode");
                    ui.label(t::text(
                        "Turn metadata could not be read. Choose a rendered episode directory \
                         or fix the metadata and try again.",
                        t::TEXT_13, t::TEXT_MUTED,
                    ));
                    ui.label(t::text(error, t::SECONDARY, t::STATUS_FAILED));
                    ui.horizontal(|ui| {
                        choose = t::primary_button(ui, "Choose directory", None, true).clicked();
                        if t::secondary_button(ui, "Try again", None, true).clicked() {
                            retry = Some(path.clone());
                        }
                        dismiss = t::secondary_button(ui, "Dismiss", None, true).clicked();
                    });
                    let hint = if self.episode.is_some() {
                        "The current episode is still open. A malformed directory does not erase it."
                    } else {
                        "Choose a directory containing episode_manifest.json."
                    };
                    ui.label(t::text(hint, t::SECONDARY, t::TEXT_MUTED));
                } else if let Some((message, position)) = &device {
                    state_heading(ui, "headphones-22-accent", "Audio output unavailable")
                        .on_hover_text(message);
                    let body = match position {
                        Some(position) => format!(
                            "The selected device was disconnected. Playback is paused at {}; \
                             your selection is unchanged. Choose another output to resume.",
                            view::time(position.as_millis() as u64)
                        ),
                        None => "The selected device was disconnected. Choose another output to continue.".to_owned(),
                    };
                    ui.label(t::text(body, t::TEXT_13, t::TEXT_MUTED));
                    ui.horizontal(|ui| {
                        let button = t::primary_button(ui, "Choose output", None, true);
                        egui::Popup::menu(&button).show(|ui| {
                            if self.device_list.is_empty() {
                                ui.label(t::text("No output devices found", t::SECONDARY, t::TEXT_MUTED));
                            }
                            for name in &self.device_list {
                                if ui.selectable_label(false, name).clicked() {
                                    chosen_device = Some(name.clone());
                                }
                            }
                        });
                        if button.clicked() { self.device_list = AudioOut::outputs(); }
                        dismiss = t::secondary_button(ui, "Dismiss", None, true).clicked();
                    });
                } else if let Some((index, file)) = &missing {
                    state_heading(ui, "volume-x-22-danger", format!("Audio missing · Turn {index:03}"));
                    ui.label(t::text(
                        format!("{file} was not found. Transcript and verification history are available. \
                                 Restore the file or regenerate the selected turn."),
                        t::TEXT_13, t::TEXT_MUTED,
                    ));
                    ui.horizontal(|ui| {
                        let preset = scriptcast_ui::runner::Preset::RegenerateTurns(self.selected_turns());
                        let spec = self.spec(&preset);
                        let reason = spec.as_ref().err().cloned().unwrap_or_default();
                        if t::primary_button(ui, "Regenerate selected", None, spec.is_ok())
                            .on_disabled_hover_text(reason).clicked()
                        {
                            self.request_job(preset, None, ctx);
                            dismiss = true;
                        }
                        dismiss |= t::secondary_button(ui, "Dismiss", None, true).clicked();
                    });
                }
            });
        if cancel {
            self.pending = None;
        }
        if dismiss {
            self.error = None;
            self.missing_audio_card = false;
            if device.is_some() {
                self.device_card_dismissed = true;
            }
        }
        if choose {
            self.pick(ctx);
        }
        if let Some(path) = retry {
            self.open(path, ctx);
        }
        if let Some(name) = chosen_device {
            self.choose_device(name);
        }
    }
}
impl Browser {
    fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if let Some(w) = self.waveforms.as_mut() {
            w.poll();
        }
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
            let timeline_max = (workspace.height() * t::TIMELINE_MAX_FRACTION).max(t::TIMELINE_MIN);
            let timeline_h = self
                .preferences
                .timeline_height
                .unwrap_or(t::TIMELINE_HEIGHT)
                .clamp(t::TIMELINE_MIN, timeline_max);
            let timeline = Rect::from_min_size(workspace.min, vec2(workspace.width(), timeline_h));
            let filter = Rect::from_min_size(
                timeline.left_bottom(),
                vec2(workspace.width(), t::FILTER_BAR_HEIGHT),
            );
            let review = Rect::from_min_max(filter.left_bottom(), workspace.max);
            self.filter_bar(&mut view::region(ui, "filters", filter, t::BG_RAISED));
            let visible = self.visible_indices();
            let (matches, playing, selected_turn) = {
                let e = self.episode.as_ref().unwrap();
                let playing = self.transport.turn().and_then(|turn| {
                    let start = e.turns.get(turn)?.start_ms;
                    Some((turn, start + self.audio_position().as_millis() as u64))
                });
                (
                    self.filters.matches(e),
                    playing,
                    self.selected.filter(|i| e.turns.get(*i).is_some()),
                )
            };
            let markers = self
                .editing
                .markers(self.episode.as_ref().unwrap().turns.len());
            let edited = markers.clone().unwrap_or_default();
            let clicked = timeline::show(
                &mut view::region(ui, "timeline", timeline, t::BG_WINDOW),
                self.episode.as_ref().unwrap(),
                &self.overview,
                &mut self.timeline,
                &matches,
                self.selected,
                playing,
                self.waveforms.as_ref(),
                &edited,
            );
            if let Some((i, double)) = clicked {
                if double {
                    self.open_editor(i);
                } else {
                    self.multi_select(i, ctx.input(|i| i.modifiers));
                }
            }
            let h_split = view::splitter_horizontal(
                ui,
                "timeline_splitter",
                timeline.bottom(),
                workspace.left(),
                workspace.right(),
            );
            if h_split.reset {
                self.preferences.timeline_height = None;
            } else if h_split.drag_delta != 0.0 {
                let next = (timeline_h + h_split.drag_delta).clamp(t::TIMELINE_MIN, timeline_max);
                self.preferences.timeline_height = Some(next);
            }
            // The inspector takes the right side of the review row whenever a
            // turn is selected (or Jobs mode is open); width is resizable.
            // The inspector takes the right of the review row whenever a turn is
            // selected: its mode's default width, or the operator's dragged width.
            let selected_turn = self.selected.or(selected_turn);
            let mode = self.jobs.mode.unwrap_or(InspectorMode::SelectedTurn);
            let (table_rect, inspector_rect) = match selected_turn.is_some()
                || mode == InspectorMode::Jobs
            {
                true => {
                    let width = mode.panel_width(self.preferences.inspector_width, review.width());
                    let inspector =
                        Rect::from_min_max(pos2(review.right() - width, review.top()), review.max);
                    (
                        Rect::from_min_max(review.min, pos2(inspector.left(), review.bottom())),
                        Some(inspector),
                    )
                }
                false => (review, None),
            };
            let (clicked, edit_action) = table::show(
                &mut view::region(ui, "table", table_rect, t::BG_SURFACE),
                self.episode.as_ref().unwrap(),
                &visible,
                self.selected,
                self.transport.turn(),
                self.scroll_to.take(),
                self.filters.failed_only,
                &self.jobs.selected,
                &edited,
                markers.is_none(),
                &mut self.editing,
                self.preferences.edit_mode,
                !self.jobs.busy() && self.pending.is_none(),
            );
            self.edit_action(edit_action, &ctx);
            if let Some((i, double)) = clicked {
                if double {
                    self.open_editor(i);
                } else {
                    self.multi_select(i, ctx.input(|i| i.modifiers));
                }
            }
            let selected_turn = self.selected.or(selected_turn);
            if let Some(rect) = inspector_rect
                && mode == InspectorMode::Jobs
            {
                self.jobs_panel(&mut view::region(
                    ui,
                    "jobs",
                    view::inset(rect, t::SPACE_12, 0.0),
                    t::BG_SURFACE,
                ));
            } else if let (Some(turn), Some(rect)) = (selected_turn, inspector_rect) {
                if mode == InspectorMode::Verification {
                    self.verification_panel(
                        &mut view::region(
                            ui,
                            "verification",
                            view::inset(rect, t::SPACE_16, 0.0),
                            t::BG_SURFACE,
                        ),
                        turn,
                    );
                } else {
                    let (can_previous, can_next) = neighbour_targets(&visible, Some(turn));
                    let (can_previous, can_next) = (can_previous.is_some(), can_next.is_some());
                    let file_line = self
                        .selected_media
                        .as_ref()
                        .map(|(_, m)| m.file_line.clone())
                        .unwrap_or_default();
                    let position = self.audio_position();
                    let (action, edit_action) = inspector::show(
                        &mut view::region(ui, "inspector", rect, t::BG_SURFACE),
                        InspectorMode::SelectedTurn,
                        self.episode.as_ref().unwrap(),
                        turn,
                        self.transport,
                        position,
                        self.preferences.auto_advance,
                        can_previous,
                        can_next,
                        &file_line,
                        &mut self.editing,
                        self.preferences.edit_mode,
                        edited.contains(&turn),
                        !self.jobs.busy() && self.pending.is_none(),
                    );
                    self.edit_action(edit_action, &ctx);
                    match action {
                        Some(inspector::Action::Previous) => {
                            if let Some(t) = neighbour_targets(&visible, Some(turn)).0 {
                                self.select(t);
                            }
                        }
                        Some(inspector::Action::Next) => {
                            if let Some(t) = neighbour_targets(&visible, Some(turn)).1 {
                                self.select(t);
                            }
                        }
                        Some(inspector::Action::Review) => {
                            self.jobs.mode = Some(InspectorMode::Verification)
                        }
                        None => {}
                    }
                }
            }
            if let Some(inspector) = inspector_rect {
                let width = inspector.width();
                let max_w = (review.width() * t::INSPECTOR_MAX_FRACTION).max(t::INSPECTOR_MIN);
                let v_split = view::splitter_vertical(
                    ui,
                    "inspector_splitter",
                    inspector.left(),
                    review.top(),
                    review.bottom(),
                );
                if v_split.reset {
                    self.preferences.inspector_width = None;
                } else if v_split.drag_delta != 0.0 {
                    // Inspector is on the right: drag left widens it.
                    let next = (width - v_split.drag_delta)
                        .clamp(t::INSPECTOR_MIN, max_w)
                        .min(review.width());
                    self.preferences.inspector_width = Some(next);
                }
            }
        } else {
            self.empty(&mut view::region(ui, "empty", workspace, t::BG_WINDOW));
        }
        self.workspace_card(&ctx);
        self.confirmation_dialog(&ctx);
        self.settings_window(&ctx);
        self.draft_guard(&ctx);
    }
}
impl eframe::App for Browser {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_load(ctx);
        self.poll_jobs(ctx);
        self.poll_editing(ctx);
        self.poll_daw();
        self.poll_audio(ctx);
        self.capture_logic(ctx);
    }
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
        self.running_capture(ui.ctx());
        self.capture_settle(ui.ctx());
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.persist {
            eframe::set_value(storage, STORAGE_KEY, &self.preferences);
        }
    }
    fn persist_egui_memory(&self) -> bool {
        self.persist
    }
}
/// Content height of the guidance column; mirrors `empty_left` item for item.
/// Item spacing is applied after each widget with the spacing in effect when
/// that widget was added, so the gap after the field label is still SPACE_20.
fn state_heading(ui: &mut Ui, icon: &str, title: impl Into<String>) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add(egui::Image::new(t::icon(icon)).fit_to_exact_size(egui::Vec2::splat(t::SPACE_22)));
        ui.label(t::heading(title, t::STATE_TITLE))
    })
    .inner
}
fn empty_left_height(ui: &Ui) -> f32 {
    let mut height = t::HERO_ICON;
    height +=
        t::SPACE_20 + t::wrapped_height(ui, HERO_TITLE_TEXT, t::bold(t::HERO_TITLE), t::EMPTY_LEFT);
    height +=
        t::SPACE_20 + t::wrapped_height(ui, HERO_BODY_TEXT, t::font(t::TEXT_14), t::EMPTY_LEFT);
    height += t::SPACE_20 + t::BUTTON_HEIGHT;
    height += t::SPACE_20 + t::SPACE_16 + t::line_height(ui, t::bold(t::FIELD_LABEL));
    let lines = DIR_LIST.len() as f32;
    height +=
        t::SPACE_20 + t::line_height(ui, t::mono(t::BODY)) * lines + t::SPACE_7 * (lines - 1.0);
    height
}
/// Left column of the empty workspace. Returns true when Open Episode was clicked.
fn empty_left(ui: &mut Ui) -> bool {
    let mut open = false;
    ui.spacing_mut().item_spacing.y = t::SPACE_20;
    ui.add(
        egui::Image::new(t::icon("folder-open-42-accent"))
            .fit_to_exact_size(egui::Vec2::splat(t::HERO_ICON)),
    );
    ui.label(t::heading(HERO_TITLE_TEXT, t::HERO_TITLE));
    ui.label(t::text(HERO_BODY_TEXT, t::TEXT_14, t::TEXT_MUTED));
    ui.horizontal(|ui| {
        open |=
            t::primary_button(ui, "Open Episode", Some("folder-open-16-on-accent"), true).clicked();
        ui.label(t::text("Cmd+O", t::BODY, t::TEXT_MUTED));
    });
    ui.add_space(t::SPACE_16);
    t::field_label(ui, DIR_LIST_TITLE);
    ui.spacing_mut().item_spacing.y = t::SPACE_7;
    for text in DIR_LIST {
        ui.label(
            egui::RichText::new(text)
                .font(t::mono(t::BODY))
                .color(t::TEXT_MUTED),
        );
    }
    open
}
/// Inspector FILE field content and probed total duration for one turn: WAV
/// names plus channel layout and sample rate decoded from the first readable
/// segment. Missing or undecodable files are named, never invented.
fn probe_turn(wavs: &[PathBuf]) -> SelectedMedia {
    let names: Vec<String> = wavs
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect();
    if names.is_empty() {
        return SelectedMedia {
            file_line: "no audio segments".into(),
            duration: None,
        };
    }
    let mut duration = Some(Duration::ZERO);
    let mut info = None;
    let mut missing = false;
    for path in wavs {
        match audio::probe(path) {
            Some(probed) => {
                info = info.or(Some(probed));
                duration = duration.and_then(|total| probed.duration.map(|d| total + d));
            }
            None => {
                missing = true;
                duration = None;
            }
        }
    }
    let suffix = if missing {
        "file missing".to_string()
    } else if let Some(info) = info {
        info.to_string()
    } else {
        "unreadable".to_string()
    };
    SelectedMedia {
        file_line: format!("{} · {suffix}", names.join(" + ")),
        duration,
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
/// ISO 8601 timestamp → the design's `18 Sep 2026, 10:26` in local time.
/// Unparseable values pass through unchanged.
fn rendered_when(iso: &str) -> std::borrow::Cow<'_, str> {
    use chrono::{Datelike, Timelike};
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    match chrono::DateTime::parse_from_rfc3339(iso) {
        Ok(dt) => {
            let local = dt.with_timezone(&chrono::Local);
            format!(
                "{} {} {}, {:02}:{:02}",
                local.day(),
                MONTHS[(local.month() - 1) as usize],
                local.year(),
                local.hour(),
                local.minute()
            )
            .into()
        }
        Err(_) => iso.into(),
    }
}
fn main() -> eframe::Result {
    let cli = match capture::Cli::parse(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("scriptcast-ui: {message}\n\n{}", capture::USAGE);
            std::process::exit(2);
        }
    };
    if cli.help {
        println!("{}", capture::USAGE);
        std::process::exit(0);
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("scriptCast — Episode workspace")
            .with_inner_size(cli.size)
            .with_min_inner_size(t::WINDOW_MIN)
            .with_decorations(true),
        persist_window: cli.persist(),
        ..Default::default()
    };
    let capturing = cli.screenshot.is_some();
    let result = eframe::run_native(
        "scriptcast-ui",
        options,
        Box::new(move |cc| Ok(Box::new(Browser::new(cc, &cli)))),
    );
    if capturing {
        // A finished capture exits the process itself; reaching this point
        // means the window closed before the screenshot arrived.
        eprintln!("scriptcast-ui: window closed before the screenshot was captured");
        std::process::exit(1);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_folder_preferences_round_trip_and_old_preferences_still_load() {
        let old: Preferences = serde_json::from_str("{}").unwrap();
        assert!(old.cli_dir.is_none());
        assert!(old.timeline_height.is_none());
        assert!(old.inspector_width.is_none());
        assert_eq!(old.appearance, t::Appearance::default());
        let preferences = Preferences {
            cli_dir: Some(PathBuf::from("/tmp/scriptcast-b2/my cli folder")),
            timeline_height: Some(420.0),
            inspector_width: Some(480.0),
            appearance: t::Appearance {
                accent: [1, 2, 3],
                text_scale: 1.1,
                ..t::Appearance::default()
            },
            ..Preferences::default()
        };
        let saved = serde_json::to_string(&preferences).unwrap();
        let restored: Preferences = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.cli_dir, preferences.cli_dir);
        assert_eq!(restored.timeline_height, Some(420.0));
        assert_eq!(restored.inspector_width, Some(480.0));
        assert_eq!(restored.appearance, preferences.appearance);
    }

    #[test]
    fn daw_progress_completion_and_worker_loss_update_status() {
        let mut app = Browser::default();
        let (sender, receiver) = mpsc::channel();
        app.logic_task = Some(DawTask { receiver });
        sender
            .send(DawEvent::Progress("Writing episode.fcpxml…".into()))
            .unwrap();
        app.poll_daw();
        assert!(app.logic_task.is_some());
        assert_eq!(app.daw_notice, Some(Ok("Writing episode.fcpxml…".into())));
        sender
            .send(DawEvent::Finished(Err("export failed".into())))
            .unwrap();
        app.poll_daw();
        assert!(app.logic_task.is_none());
        assert_eq!(app.daw_notice, Some(Err("export failed".into())));

        let (sender, receiver) = mpsc::channel();
        app.logic_task = Some(DawTask { receiver });
        drop(sender);
        app.poll_daw();
        assert!(app.logic_task.is_none());
        assert_eq!(
            app.daw_notice,
            Some(Err("DAW task stopped unexpectedly".into()))
        );
    }

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
            for state in 0..6 {
                app.pending = None;
                app.error = None;
                app.transport = Transport::Idle;
                app.missing_audio_card = false;
                app.device_error = None;
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
                if state == 4 {
                    // Missing audio card for the selected turn: point a WAV at
                    // a nonexistent path so the card's real-file lookup runs.
                    app.select(1);
                    let turn = &mut app.episode.as_mut().unwrap().turns[1];
                    turn.wav_paths[0] = PathBuf::from("/nonexistent/turn_2_0.wav");
                    turn.verification = V::MissingAudio;
                    app.missing_audio_card = true;
                }
                if state == 5 {
                    // Unavailable device card with a paused transport.
                    app.select(0);
                    app.transport = Transport::Paused(0);
                    app.device_error = Some("simulated loss".into());
                    app.device_list = vec!["Test Speaker".into()];
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
    #[test]
    fn space_and_esc_drive_the_transport_state_machine() {
        use SpaceAction as A;
        let all_playable = |_| true;
        // No selection: Space does nothing.
        assert_eq!(
            space_action(Transport::Idle, None, all_playable),
            (Transport::Idle, A::Nothing)
        );
        // Idle + selection: play it.
        assert_eq!(
            space_action(Transport::Idle, Some(4), all_playable),
            (Transport::Playing(4), A::Play(4))
        );
        // Playing the selected turn: pause; paused: resume.
        assert_eq!(
            space_action(Transport::Playing(4), Some(4), all_playable),
            (Transport::Paused(4), A::Pause)
        );
        assert_eq!(
            space_action(Transport::Paused(4), Some(4), all_playable),
            (Transport::Playing(4), A::Resume)
        );
        // A different selection switches immediately, from playing or paused.
        assert_eq!(
            space_action(Transport::Playing(4), Some(7), all_playable),
            (Transport::Playing(7), A::Play(7))
        );
        assert_eq!(
            space_action(Transport::Paused(4), Some(7), all_playable),
            (Transport::Playing(7), A::Play(7))
        );
        // A missing-audio turn never starts; the caller shows the card.
        let missing_two = |i: usize| i != 2;
        assert_eq!(
            space_action(Transport::Idle, Some(2), missing_two),
            (Transport::Idle, A::Missing(2))
        );
        assert_eq!(
            space_action(Transport::Playing(1), Some(2), missing_two),
            (Transport::Idle, A::Missing(2))
        );
        // Esc stops anything playing and is a no-op when idle.
        assert_eq!(
            esc_action(Transport::Playing(4)),
            (Transport::Idle, EscAction::Stop)
        );
        assert_eq!(
            esc_action(Transport::Paused(4)),
            (Transport::Idle, EscAction::Stop)
        );
        assert_eq!(
            esc_action(Transport::Idle),
            (Transport::Idle, EscAction::Nothing)
        );
    }
    #[test]
    fn auto_advance_targets_next_visible_turn_and_stops_at_the_end() {
        let mut e = fixture();
        // Speaker filter leaves turns 1 and 3 (indices 0 and 2) visible:
        // the next visible turn under the filter after 0 is 2, not 1.
        let filters = Filters {
            speaker: Some(0),
            ..Filters::default()
        };
        let visible: Vec<usize> = filters
            .matches(&e)
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.then_some(i))
            .collect();
        assert_eq!(visible, [0, 2]);
        assert_eq!(advance_target(&e, &visible, 0), Some(2));
        assert_eq!(
            advance_target(&e, &visible, 2),
            None,
            "end of the list stops"
        );
        assert_eq!(
            advance_target(&e, &visible, 1),
            None,
            "a turn filtered out mid-play has no advance target"
        );
        // A missing-audio turn cannot play and is skipped over.
        e.turns[1].verification = V::MissingAudio;
        let all: Vec<usize> = (0..e.turns.len()).collect();
        assert_eq!(advance_target(&e, &all, 0), Some(2));
        // All remaining turns missing: stop.
        e.turns[2].verification = V::MissingAudio;
        assert_eq!(advance_target(&e, &all, 0), None);
    }
    #[test]
    fn inspector_labels_transcript_only_with_real_expected_text() {
        let e = fixture();
        // Turn 1 has a passed stress.json entry: TRANSCRIPT with its text.
        assert_eq!(
            inspector::transcript_field(&e.turns[0]),
            ("Transcript", "Hello, world.")
        );
        // Turn 2's failed entry also carries expected text.
        assert_eq!(
            inspector::transcript_field(&e.turns[1]),
            ("Transcript", "Two chunks in order.")
        );
        // Turn 3 has no entry: FIRST WORDS from TURN_INDEX.csv, never
        // labelled a transcript.
        assert_eq!(
            inspector::transcript_field(&e.turns[2]),
            ("First words", "Not checked yet.")
        );
        // An entry with empty expected text falls back to first words too.
        let mut hollow = e.turns[0].clone();
        hollow.verify.as_mut().unwrap().expected = "  ".into();
        assert_eq!(
            inspector::transcript_field(&hollow),
            ("First words", "Hello, world.")
        );
    }
    #[test]
    fn inspector_renders_headlessly_in_all_transport_states() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        for transport in [Transport::Idle, Transport::Playing(0), Transport::Paused(0)] {
            let mut app = Browser::default();
            app.accept_episode(fixture(), &ctx);
            app.select(0);
            app.transport = transport;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, t::WINDOW_SIZE.into())),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
    }
    #[test]
    fn rendered_when_formats_local_time_and_passes_garbage_through() {
        let formatted = rendered_when("2026-09-18T12:00:00.000000Z");
        assert!(
            formatted.contains("Sep 2026, "),
            "unexpected format: {formatted}"
        );
        assert!(!formatted.contains('T') && !formatted.ends_with('Z'));
        assert!(formatted.contains(':'), "no clock time in {formatted}");
        assert_eq!(rendered_when("not a timestamp"), "not a timestamp");
        assert_eq!(rendered_when(""), "");
    }
    #[test]
    fn empty_columns_fit_the_default_window_without_scrolling() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let app = Browser::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let workspace = t::WINDOW_SIZE[1]
                - t::EPISODE_HEADER_HEIGHT
                - t::PLAYBACK_BAR_HEIGHT
                - t::STATUS_BAR_HEIGHT;
            let available = workspace - t::SPACE_20 * 2.0;
            assert!(empty_left_height(ui) <= available);
            assert!(app.empty_right_height(ui) <= available);
        });
        output.textures_delta.clear();
    }
}
