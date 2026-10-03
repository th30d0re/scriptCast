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
  --screenshot PATH  after the UI settles, save the window to PATH as PNG and exit
  --size WxH         inner window size in points (default: 1440x900)
  --fresh            ignore persisted state and do not save state on exit
  --select N         select manifest turn N after loading, scrolled into view
  -h, --help         print this help

--screenshot implies --fresh unless EPISODE_DIR is given; either way a
screenshot run never writes persisted state. A saved screenshot exits 0;
any failure (including a 20 s timeout) exits non-zero.";

#[derive(Clone, Debug)]
pub struct Cli {
    pub episode: Option<PathBuf>,
    pub screenshot: Option<PathBuf>,
    pub size: [f32; 2],
    pub fresh: bool,
    pub select: Option<usize>,
    pub help: bool,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut cli = Self {
            episode: None,
            screenshot: None,
            size: t::WINDOW_SIZE,
            fresh: false,
            select: None,
            help: false,
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
                    return Err(format!("unexpected extra argument {}", arg.to_string_lossy()));
                }
                cli.episode = Some(PathBuf::from(arg));
                continue;
            }
            match arg.to_string_lossy().as_ref() {
                "--" => positional_only = true,
                "-h" | "--help" => cli.help = true,
                "--fresh" => cli.fresh = true,
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
                "--select" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--select needs a turn number".to_string())?;
                    cli.select = Some(parse_select(&value.to_string_lossy())?);
                }
                other => return Err(format!("unknown option {other}")),
            }
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

fn parse_select(text: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .map_err(|_| format!("--select expects a manifest turn number, got {text}"))
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
}
impl Capture {
    pub fn new(path: PathBuf) -> Self {
        let now = Instant::now();
        Self {
            path,
            phase: Phase::Loading,
            frames: 0,
            quiet_since: now,
            deadline: now + TIMEOUT,
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
        assert!(!cli.effective_fresh());
        assert!(cli.persist());
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
        let cli = parse(&["/tmp/episode", "--select", "3", "--screenshot", "/tmp/o.png"]).unwrap();
        assert_eq!(cli.episode, Some(PathBuf::from("/tmp/episode")));
        assert_eq!(cli.select, Some(3));
        assert_eq!(cli.screenshot, Some(PathBuf::from("/tmp/o.png")));
    }
    #[test]
    fn size_parses_points() {
        assert_eq!(parse(&["--size", "1440x900"]).unwrap().size, [1440.0, 900.0]);
        assert_eq!(parse(&["--size", "2880X1800"]).unwrap().size, [2880.0, 1800.0]);
        assert_eq!(parse(&["--size", "1200.5x800"]).unwrap().size, [1200.5, 800.0]);
    }
    #[test]
    fn size_rejects_garbage() {
        for bad in ["1440", "x900", "1440x", "axb", "0x900", "1440x0", "-5x100", "NaNx900", "1440x900x1"] {
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
        let path = std::env::temp_dir().join(format!(
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
