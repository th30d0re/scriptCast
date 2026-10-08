//! Self-screenshot support: the `--screenshot`/`--size`/`--fresh`/`--select`
//! flags and the capture state machine behind them. The app photographs itself
//! through egui, so visual verification needs no macOS Screen Recording
//! permission.

use eframe::egui;
use scriptcast_ui::theme as t;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Painted frames required after loading before the UI may be called settled.
pub const MIN_FRAMES: usize = 3;
/// Extra idle time required after the last frame that requested a repaint, so
/// asynchronously loaded icons have landed before the capture.
pub const QUIET: Duration = Duration::from_millis(500);
/// Keep-alive repaint interval while a capture is pending, so the deadline and
/// the settle condition are re-evaluated even when the app itself is idle.
pub const TICK: Duration = Duration::from_millis(50);
/// A capture that takes longer than this is a failure.
pub const TIMEOUT: Duration = Duration::from_secs(20);

pub const USAGE: &str = "Usage: scriptcast-ui [EPISODE_DIR] [OPTIONS]

Open a rendered scriptCast episode directory in the episode browser.

Options:
  --new-episode PATH open the New Episode sheet
  --dry-run-json PATH preview fixture (with --new-episode)
  --progress-fixture PATH load a progress JSONL fixture
  --pipeline-fixture PATH load a pipeline JSON fixture
  --quit-alert       show the active-render quit alert
  --screenshot PATH  after the UI settles, save the window to PATH as PNG and exit
  --size WxH         inner window size in points (default: 1440x900)
  --inspector MODE   selected, verification, or jobs
  --confirm PRESET   show confirmation; never auto-confirm
  --run PRESET       run detect-changes, verify, read-along-timings,
                     clip-transcripts or second-opinion; capture after exit
  --running-screenshot PATH  also save the active job (requires --run)
  --fresh            ignore persisted state and do not save state on exit
  --select N         select manifest turn N after loading, scrolled into view
  --review N         open discrepancy review on failed turn N (implies --select N)
  --review-mode MODE whisper or ensemble (with --review; default whisper)
  --find QUERY       find words after the episode loads
  --replace TEXT     open Replace with TEXT (never saves automatically)
  --edit             open the transcript editor (requires --select)
  --context-menu N   show turn menu and speaker choices
  --new-turn-after N show an unsaved insertion draft
  --new-turn-speaker ID choose the insertion speaker
  --tag-toolbar N    open the body editor with its tag toolbar
  --clip-picker      open the registered clip picker
  --raw-script       use plain monospace text in the body editor
  --edit-mode MODE   panel (default) or row
  --play             after --select, start playback of the selected turn
                     (requires --select)
  --at MS            after --play starts, seek the player to MS milliseconds
                     (requires --play)
  --mute             play at volume 0; --screenshot together with --play always
                     implies --mute, so a capture run never makes sound
  --simulate-device-loss
                     after --play, inject an output-device error through the
                     same channel the real error callback uses, to capture the
                     Unavailable device state (requires --play)
  -h, --help         print this help

--screenshot implies --fresh unless EPISODE_DIR is given; either way a
screenshot run never writes persisted state. With --play the capture waits
until the playback position has advanced at least 200 ms past --at (or until
the Missing audio / Unavailable device state is showing), then photographs the
window while playing. A saved screenshot exits 0; any failure (including a
20 s timeout) exits non-zero.";

#[derive(Clone, Debug)]
pub struct Cli {
    pub new_episode: Option<PathBuf>,
    pub dry_run_json: Option<PathBuf>,
    pub progress_fixture: Option<PathBuf>,
    pub pipeline_fixture: Option<PathBuf>,
    pub quit_alert: bool,
    pub episode: Option<PathBuf>,
    pub screenshot: Option<PathBuf>,
    pub size: [f32; 2],
    pub fresh: bool,
    pub select: Option<usize>,
    /// Open discrepancy review after selecting this turn (capture aid).
    pub review: Option<usize>,
    /// Whisper / Ensemble mode when opening `--review`.
    pub review_mode: Option<scriptcast_ui::discrepancy::ReviewMode>,
    pub edit: bool,
    pub context_menu: Option<usize>,
    pub new_turn_after: Option<usize>,
    pub new_turn_speaker: Option<String>,
    pub tag_toolbar: Option<usize>,
    pub clip_picker: bool,
    pub raw_script: bool,
    pub find: Option<String>,
    pub replace: Option<String>,
    pub edit_mode: Option<crate::editing::EditMode>,
    pub play: bool,
    pub at: Option<u64>,
    pub mute: bool,
    pub simulate_device_loss: bool,
    pub help: bool,
    pub inspector: Option<crate::inspector::InspectorMode>,
    pub confirm: Option<String>,
    pub run: Option<String>,
    pub running_screenshot: Option<PathBuf>,
    pub no_gapless: bool,
    /// Hidden capture aid: `(timeline_height, inspector_width)` in points.
    pub layout: Option<(f32, f32)>,
    /// Hidden capture aid: open the Settings window.
    pub settings: bool,
    /// Hidden capture aid: start with the side panel collapsed.
    pub inspector_hidden: bool,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut cli = Self {
            new_episode: None,
            dry_run_json: None,
            progress_fixture: None,
            pipeline_fixture: None,
            quit_alert: false,
            episode: None,
            screenshot: None,
            size: t::WINDOW_SIZE,
            fresh: false,
            select: None,
            review: None,
            review_mode: None,
            edit: false,
            context_menu: None,
            new_turn_after: None,
            new_turn_speaker: None,
            tag_toolbar: None,
            clip_picker: false,
            raw_script: false,
            find: None,
            replace: None,
            edit_mode: None,
            play: false,
            at: None,
            mute: false,
            simulate_device_loss: false,
            help: false,
            inspector: None,
            confirm: None,
            run: None,
            running_screenshot: None,
            no_gapless: false,
            layout: None,
            settings: false,
            inspector_hidden: false,
        };
        let mut positional_only = false;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let is_flag = !positional_only && {
                let text = arg.to_string_lossy();
                text.len() > 1 && text.starts_with('-')
            };
            if !is_flag {
                if cli.episode.is_some() {
                    return Err(format!(
                        "unexpected extra argument {}",
                        arg.to_string_lossy()
                    ));
                }
                cli.episode = Some(PathBuf::from(arg));
                continue;
            }
            match arg.to_string_lossy().as_ref() {
                "--new-episode" => {
                    cli.new_episode = Some(args.next().ok_or("Missing script")?.into())
                }
                "--dry-run-json" => {
                    cli.dry_run_json = Some(args.next().ok_or("Missing JSON")?.into())
                }
                "--progress-fixture" => {
                    cli.progress_fixture = Some(args.next().ok_or("Missing JSONL")?.into())
                }
                "--pipeline-fixture" => {
                    cli.pipeline_fixture = Some(args.next().ok_or("Missing JSON")?.into())
                }
                "--quit-alert" => cli.quit_alert = true,
                "--" => positional_only = true,

                "-h" | "--help" => cli.help = true,
                "--inspector" => {
                    cli.inspector = Some(
                        match args
                            .next()
                            .ok_or("--inspector needs selected, verification or jobs")?
                            .to_string_lossy()
                            .as_ref()
                        {
                            "selected" => crate::inspector::InspectorMode::SelectedTurn,
                            "verification" => crate::inspector::InspectorMode::Verification,
                            "jobs" => crate::inspector::InspectorMode::Jobs,
                            _ => return Err("Unknown inspector".into()),
                        },
                    );
                }
                "--confirm" => {
                    let name = args
                        .next()
                        .ok_or("--confirm needs a preset")?
                        .to_string_lossy()
                        .into_owned();
                    scriptcast_ui::runner::Preset::parse(&name, vec![])?;
                    cli.confirm = Some(name);
                }
                "--run" => {
                    let name = args
                        .next()
                        .ok_or("--run needs detect-changes or verify")?
                        .to_string_lossy()
                        .into_owned();
                    scriptcast_ui::runner::Preset::safe_run(&name)?;
                    cli.run = Some(name);
                }
                "--fresh" => cli.fresh = true,
                "--edit" => cli.edit = true,
                "--context-menu" | "--new-turn-after" | "--tag-toolbar" => {
                    let n = args
                        .next()
                        .ok_or("Turn number required")?
                        .to_string_lossy()
                        .parse::<usize>()
                        .map_err(|_| "Invalid turn number")?;
                    if n == 0 {
                        return Err("Turn numbers start at 1".into());
                    }
                    match arg.to_string_lossy().as_ref() {
                        "--context-menu" => cli.context_menu = Some(n),
                        "--new-turn-after" => cli.new_turn_after = Some(n),
                        _ => {
                            cli.tag_toolbar = Some(n);
                            cli.edit = true;
                        }
                    }
                    cli.select = Some(n);
                }
                "--new-turn-speaker" => {
                    cli.new_turn_speaker = Some(
                        args.next()
                            .ok_or("Speaker id required")?
                            .to_string_lossy()
                            .into_owned(),
                    )
                }
                "--clip-picker" => cli.clip_picker = true,
                "--raw-script" => cli.raw_script = true,
                "--find" => {
                    cli.find = Some(
                        args.next()
                            .ok_or("--find needs a query")?
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
                "--replace" => {
                    cli.replace = Some(
                        args.next()
                            .ok_or("--replace needs text")?
                            .to_string_lossy()
                            .into_owned(),
                    );
                }

                "--edit-mode" => {
                    cli.edit_mode = Some(
                        match args
                            .next()
                            .ok_or("--edit-mode needs panel or row")?
                            .to_string_lossy()
                            .as_ref()
                        {
                            "panel" => crate::editing::EditMode::Panel,
                            "row" => crate::editing::EditMode::Row,
                            _ => return Err("--edit-mode needs panel or row".into()),
                        },
                    );
                }
                "--play" => cli.play = true,

                "--mute" => cli.mute = true,
                "--simulate-device-loss" => cli.simulate_device_loss = true,
                "--no-gapless" => cli.no_gapless = true,
                "--settings" => cli.settings = true,
                "--inspector-hidden" => cli.inspector_hidden = true,
                "--running-screenshot" => {
                    cli.running_screenshot = Some(PathBuf::from(
                        args.next().ok_or("--running-screenshot needs a path")?,
                    ));
                }
                "--screenshot" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--screenshot needs a PNG path".to_string())?;
                    cli.screenshot = Some(PathBuf::from(value));
                }
                "--size" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--size needs a value like 1440x900".to_string())?;
                    cli.size = parse_size(&value.to_string_lossy())?;
                }
                "--layout" => {
                    let value = args.next().ok_or_else(|| {
                        "--layout needs timeline_height x inspector_width (e.g. 420x480)"
                            .to_string()
                    })?;
                    cli.layout = Some(parse_layout(&value.to_string_lossy())?);
                }
                "--select" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--select needs a turn number".to_string())?;
                    cli.select = Some(parse_select(&value.to_string_lossy())?);
                }
                "--review" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--review needs a turn number".to_string())?;
                    let turn = parse_select(&value.to_string_lossy())?;
                    cli.review = Some(turn);
                    if cli.select.is_none() {
                        cli.select = Some(turn);
                    }
                }
                "--review-mode" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--review-mode needs whisper or ensemble".to_string())?;
                    cli.review_mode = Some(match value.to_string_lossy().as_ref() {
                        "whisper" | "whisper-only" | "whisper_only" => {
                            scriptcast_ui::discrepancy::ReviewMode::WhisperOnly
                        }
                        "ensemble" => scriptcast_ui::discrepancy::ReviewMode::Ensemble,
                        _ => return Err("--review-mode needs whisper or ensemble".into()),
                    });
                }
                "--at" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--at needs a millisecond value".to_string())?;
                    cli.at = Some(parse_at(&value.to_string_lossy())?);
                }
                other => return Err(format!("unknown option {other}")),
            }
        }
        if cli.edit && cli.select.is_none() {
            return Err("--edit needs --select".into());
        }
        if cli.review.is_some() && cli.select.is_none() {
            return Err("--review needs a turn number".into());
        }
        if cli.review_mode.is_some() && cli.review.is_none() {
            return Err("--review-mode needs --review".into());
        }
        if cli.play && cli.select.is_none() {
            return Err("--play needs --select (there is no default turn to play)".into());
        }
        if cli.at.is_some() && !cli.play {
            return Err("--at needs --play".into());
        }
        if cli.simulate_device_loss && !cli.play {
            return Err("--simulate-device-loss needs --play".into());
        }
        if cli.running_screenshot.is_some() && cli.run.is_none() {
            return Err("--running-screenshot requires --run".into());
        }
        if cli.run.is_some() && cli.confirm.is_some() {
            return Err("--run and --confirm are mutually exclusive".into());
        }
        if (cli.run.is_some() || cli.confirm.is_some()) && cli.episode.is_none() {
            return Err("--run / --confirm require an explicit episode".into());
        }
        Ok(cli)
    }
    /// `--screenshot` implies `--fresh` unless an episode dir is given.
    pub fn effective_fresh(&self) -> bool {
        self.fresh || (self.screenshot.is_some() && self.episode.is_none())
    }
    /// Fresh runs and every screenshot run leave persisted state untouched.
    pub fn persist(&self) -> bool {
        !self.effective_fresh() && self.screenshot.is_none()
    }
    /// A capture that plays is always muted; --mute mutes any run.
    pub fn effective_muted(&self) -> bool {
        self.mute || self.screenshot.is_some() || self.running_screenshot.is_some()
    }
}

fn parse_size(text: &str) -> Result<[f32; 2], String> {
    let invalid = || format!("--size expects WxH in points (for example 1440x900), got {text}");
    let (w, h) = text.split_once(['x', 'X']).ok_or_else(invalid)?;
    let width: f32 = w.parse().map_err(|_| invalid())?;
    let height: f32 = h.parse().map_err(|_| invalid())?;
    if !width.is_finite() || !height.is_finite() || width < 1.0 || height < 1.0 {
        return Err(invalid());
    }
    Ok([width, height])
}

fn parse_layout(text: &str) -> Result<(f32, f32), String> {
    let invalid =
        || format!("--layout expects timeline_height x inspector_width (e.g. 420x480), got {text}");
    let (h, w) = text.split_once(['x', 'X']).ok_or_else(invalid)?;
    let timeline_height: f32 = h.parse().map_err(|_| invalid())?;
    let inspector_width: f32 = w.parse().map_err(|_| invalid())?;
    if !timeline_height.is_finite()
        || !inspector_width.is_finite()
        || timeline_height < 1.0
        || inspector_width < 1.0
    {
        return Err(invalid());
    }
    Ok((timeline_height, inspector_width))
}

fn parse_select(text: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .map_err(|_| format!("--select expects a manifest turn number, got {text}"))
}

fn parse_at(text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| format!("--at expects milliseconds, got {text}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for the initial episode load (if any) to finish.
    Loading,
    /// Load done; counting painted frames and waiting for the UI to go idle.
    Settling,
    /// Screenshot command sent; waiting for the `Event::Screenshot` reply.
    Requested,
}
impl Phase {
    pub fn describe(self, frames: usize) -> String {
        match self {
            Self::Loading => "waiting for the episode to load".into(),
            Self::Settling => format!("waiting for the ui to settle ({frames} frames painted)"),
            Self::Requested => "waiting for the screenshot event".into(),
        }
    }
}

pub struct Capture {
    pub path: PathBuf,
    pub phase: Phase,
    pub frames: usize,
    pub quiet_since: Instant,
    pub deadline: Instant,
    /// --play: start the selected turn when the load settles.
    pub play: bool,
    /// --at: seek target; the capture fires once position is ≥ at + 200 ms.
    pub at: Duration,
    /// --simulate-device-loss: inject a device error right after starting.
    pub simulate_device_loss: bool,
    /// Set once playback has actually been observed in the Playing state, so a
    /// turn that played to completion (transport back to Idle) still counts as
    /// ready instead of looking like a failed start.
    pub play_started: bool,
    /// When the screenshot was last requested. eframe only drains the
    /// screenshot action on a frame the OS reports as visible; a request made
    /// before the window is visible can sit in the queue forever, so the
    /// command is re-sent while no event arrives.
    pub requested_at: Instant,
}
impl Capture {
    pub fn new(path: PathBuf, play: bool, at: Option<u64>, simulate_device_loss: bool) -> Self {
        let now = Instant::now();
        Self {
            path,
            phase: Phase::Loading,
            frames: 0,
            quiet_since: now,
            deadline: now + TIMEOUT,
            play,
            at: Duration::from_millis(at.unwrap_or(0)),
            simulate_device_loss,
            play_started: false,
            requested_at: now,
        }
    }
}

/// Print the failure and exit non-zero; a capture run must never hang.
pub fn fail(message: &str) -> ! {
    eprintln!("scriptcast-ui: {message}");
    std::process::exit(1);
}

/// Encode the screenshot (already in full physical pixels) as PNG.
pub fn write_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let width = image.width() as u32;
    let height = image.height() as u32;
    if width == 0 || height == 0 {
        return Err("screenshot has zero size".into());
    }
    let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
    for pixel in &image.pixels {
        rgba.extend_from_slice(&pixel.to_array());
    }
    let Some(buffer) = image::RgbaImage::from_raw(width, height, rgba) else {
        return Err(format!(
            "screenshot buffer mismatch: {width}x{height} but {} pixels",
            image.pixels.len()
        ));
    };
    buffer
        .save_with_format(path, image::ImageFormat::Png)
        .map_err(|e| format!("could not write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Cli, String> {
        Cli::parse(args.iter().map(OsString::from))
    }
    #[test]
    fn no_args_keeps_defaults() {
        let cli = parse(&[]).unwrap();
        assert_eq!(cli.episode, None);
        assert_eq!(cli.screenshot, None);
        assert_eq!(cli.size, t::WINDOW_SIZE);
        assert!(!cli.fresh && !cli.help && cli.select.is_none());
        assert!(cli.layout.is_none());
        assert!(!cli.effective_fresh());
        assert!(cli.persist());
    }
    #[test]
    fn layout_parses_timeline_by_inspector() {
        let cli = parse(&["--layout", "420x480"]).unwrap();
        assert_eq!(cli.layout, Some((420.0, 480.0)));
        assert!(parse(&["--layout", "x480"]).is_err());
        assert!(parse(&["--layout"]).is_err());
    }
    #[test]
    fn settings_flag_opens_hidden_capture_aid() {
        let cli = parse(&["--settings"]).unwrap();
        assert!(cli.settings);
        assert!(!parse(&[] as &[&str]).unwrap().settings);
    }
    #[test]
    fn positional_episode_dir_still_works() {
        let cli = parse(&["/tmp/episode"]).unwrap();
        assert_eq!(cli.episode, Some(PathBuf::from("/tmp/episode")));
        assert!(cli.persist());
    }
    #[test]
    fn screenshot_alone_implies_fresh_and_never_persists() {
        let cli = parse(&["--screenshot", "/tmp/out.png"]).unwrap();
        assert_eq!(cli.screenshot, Some(PathBuf::from("/tmp/out.png")));
        assert!(cli.effective_fresh());
        assert!(!cli.persist());
    }
    #[test]
    fn screenshot_with_episode_is_not_fresh_but_never_persists() {
        let cli = parse(&["--screenshot", "/tmp/out.png", "/tmp/episode"]).unwrap();
        assert_eq!(cli.episode, Some(PathBuf::from("/tmp/episode")));
        assert!(!cli.effective_fresh());
        assert!(!cli.persist());
    }
    #[test]
    fn fresh_alone_suppresses_persistence() {
        let cli = parse(&["--fresh", "/tmp/episode"]).unwrap();
        assert!(cli.fresh);
        assert!(cli.effective_fresh());
        assert!(!cli.persist());
    }
    #[test]
    fn flags_may_follow_the_episode_dir() {
        let cli = parse(&[
            "/tmp/episode",
            "--select",
            "3",
            "--screenshot",
            "/tmp/o.png",
        ])
        .unwrap();
        assert_eq!(cli.episode, Some(PathBuf::from("/tmp/episode")));
        assert_eq!(cli.select, Some(3));
        assert_eq!(cli.screenshot, Some(PathBuf::from("/tmp/o.png")));
    }
    #[test]
    fn size_parses_points() {
        assert_eq!(
            parse(&["--size", "1440x900"]).unwrap().size,
            [1440.0, 900.0]
        );
        assert_eq!(
            parse(&["--size", "2880X1800"]).unwrap().size,
            [2880.0, 1800.0]
        );
        assert_eq!(
            parse(&["--size", "1200.5x800"]).unwrap().size,
            [1200.5, 800.0]
        );
    }
    #[test]
    fn size_rejects_garbage() {
        for bad in [
            "1440",
            "x900",
            "1440x",
            "axb",
            "0x900",
            "1440x0",
            "-5x100",
            "NaNx900",
            "1440x900x1",
        ] {
            assert!(parse(&["--size", bad]).is_err(), "accepted {bad}");
        }
    }
    #[test]
    fn select_parses_turn_numbers() {
        assert_eq!(parse(&["--select", "0"]).unwrap().select, Some(0));
        assert_eq!(parse(&["--select", "391"]).unwrap().select, Some(391));
        assert!(parse(&["--select", "-1"]).is_err());
        assert!(parse(&["--select", "three"]).is_err());
    }
    #[test]
    fn review_implies_select() {
        let cli = parse(&["/tmp/e", "--review", "119"]).unwrap();
        assert_eq!(cli.review, Some(119));
        assert_eq!(cli.select, Some(119));
        assert!(parse(&["--review"]).is_err());
        let cli = parse(&["/tmp/e", "--review", "119", "--review-mode", "ensemble"]).unwrap();
        assert_eq!(
            cli.review_mode,
            Some(scriptcast_ui::discrepancy::ReviewMode::Ensemble)
        );
        assert!(parse(&["--review-mode", "ensemble"]).is_err());
    }
    #[test]
    fn play_flags_parse_and_validate_combinations() {
        let cli = parse(&["/tmp/ep", "--select", "4", "--play", "--at", "4620"]).unwrap();
        assert!(cli.play);
        assert_eq!(cli.at, Some(4620));
        assert!(!cli.mute && !cli.simulate_device_loss);
        assert!(
            parse(&["--play", "/tmp/ep"]).is_err(),
            "--play needs --select"
        );
        assert!(
            parse(&["--at", "100", "/tmp/ep"]).is_err(),
            "--at needs --play"
        );
        assert!(
            parse(&["--simulate-device-loss", "/tmp/ep"]).is_err(),
            "--simulate-device-loss needs --play"
        );
        assert!(parse(&["--play", "--select", "1", "--at", "x", "/tmp/e"]).is_err());
        assert!(parse(&["--play", "--select", "1", "--at", "-5", "/tmp/e"]).is_err());
    }
    #[test]
    fn screenshot_with_play_is_always_muted() {
        let cli = parse(&[
            "/tmp/e",
            "--select",
            "0",
            "--play",
            "--screenshot",
            "/tmp/o.png",
        ])
        .unwrap();
        assert!(cli.effective_muted(), "capture + play must imply mute");
        let cli = parse(&["/tmp/e", "--select", "0", "--play"]).unwrap();
        assert!(!cli.effective_muted(), "plain --play is audible");
        let cli = parse(&["/tmp/e", "--mute"]).unwrap();
        assert!(cli.effective_muted());
    }
    #[test]
    fn missing_values_and_unknown_options_fail() {
        assert!(parse(&["--screenshot"]).is_err());
        assert!(parse(&["--size"]).is_err());
        assert!(parse(&["--select"]).is_err());
        assert!(parse(&["--bogus"]).is_err());
        assert!(parse(&["/tmp/a", "/tmp/b"]).is_err());
    }
    #[test]
    fn double_dash_allows_dash_prefixed_paths() {
        let cli = parse(&["--", "-odd-dir"]).unwrap();
        assert_eq!(cli.episode, Some(PathBuf::from("-odd-dir")));
    }
    #[test]
    fn help_is_recognised() {
        assert!(parse(&["--help"]).unwrap().help);
        assert!(parse(&["-h"]).unwrap().help);
    }
    #[test]
    fn png_round_trip_has_full_dimensions() {
        let image = egui::ColorImage::new([8, 4], vec![egui::Color32::RED; 32]);
        let path = PathBuf::from("/tmp/scriptcast-f").join(format!(
            "scriptcast-ui-capture-test-{}.png",
            std::process::id()
        ));
        write_png(&path, &image).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes.is_empty());
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (8, 4));
        std::fs::remove_file(&path).unwrap();
    }
}
