//! Output-device management and per-turn playback over rodio.
//!
//! One `AudioOut` owns one `MixerDeviceSink` for the app's lifetime; each
//! `play()` creates a fresh `Player` on its mixer with the turn's segments
//! appended in chunk order. Position is tracked by wall clock (`base` +
//! `started`, frozen into `paused_at`), not `Player::get_pos()`: rodio wraps
//! every appended source in its own `track_position()`, so `get_pos()` resets
//! at each segment boundary (see phaseB-findings.md).

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{
    ChannelCount, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate, buffer::SamplesBuffer,
    cpal,
};
use std::{
    collections::VecDeque,
    fmt,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

/// Turns appended to one player, in order, with their durations. Pure
/// bookkeeping, so gapless boundaries can be tested without a device.
#[derive(Default)]
pub struct TurnQueue {
    entries: VecDeque<(usize, Duration)>,
}

impl TurnQueue {
    pub fn push(&mut self, turn: usize, duration: Duration) {
        self.entries.push_back((turn, duration));
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn current(&self, mut elapsed: Duration) -> Option<(usize, Duration, usize)> {
        if self.entries.is_empty() {
            return None;
        }
        for (i, &(turn, dur)) in self.entries.iter().enumerate() {
            if elapsed < dur || (elapsed == dur && i == self.entries.len() - 1) {
                return Some((turn, elapsed.min(dur), i));
            }
            elapsed -= dur;
        }
        None
    }
}

/// Stream info of one decoded WAV, for the inspector's FILE field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioInfo {
    pub channels: ChannelCount,
    pub sample_rate: SampleRate,
    pub duration: Option<Duration>,
}
impl AudioInfo {
    /// "mono" / "stereo" / "3 ch".
    pub fn channel_layout(&self) -> &'static str {
        match self.channels.get() {
            1 => "mono",
            2 => "stereo",
            _ => "multi",
        }
    }
}
impl fmt::Display for AudioInfo {
    /// "<mono|stereo> · <kHz>", for example "mono · 48 kHz".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let khz = self.sample_rate.get() as f64 / 1000.0;
        if khz.fract() == 0.0 {
            write!(f, "{} · {} kHz", self.channel_layout(), khz as u64)
        } else {
            write!(f, "{} · {khz:.1} kHz", self.channel_layout())
        }
    }
}

/// Decode a file's header. `None` when the file is missing or undecodable.
pub fn probe(path: &Path) -> Option<AudioInfo> {
    let bytes = std::fs::read(path).ok()?;
    let hint = path.extension().and_then(|ext| ext.to_str());
    let info = soundcraft_audio_io::probe(&bytes, hint).ok()?;
    let channels = ChannelCount::try_from(info.channels).ok()?;
    let sample_rate = SampleRate::try_from(info.sample_rate).ok()?;
    let duration = (info.sample_rate != 0).then(|| Duration::from_secs_f64(info.duration_secs()));
    Some(AudioInfo {
        channels,
        sample_rate,
        duration,
    })
}

/// The single audio output the app plays through.
pub struct AudioOut {
    sink: MixerDeviceSink,
    device_name: String,
    notice: Option<String>,
    player: Option<Player>,
    queue: TurnQueue,
    fallback_duration: Duration,
    /// Manifest timing for the current turn; used by `duration()` only when a
    /// decoder could not report a segment duration. Set before `play()`.
    muted: bool,
    base: Duration,
    started: Instant,
    paused_at: Option<Duration>,
    error_tx: Sender<String>,
    error_rx: Receiver<String>,
    stream_rate: u32,
}

impl AudioOut {
    /// Open `device` (`None` = the system default). A named device that is
    /// gone falls back to the default output with a notice in `take_notice()`.
    pub fn open(device: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();
        let mut notice = None;
        let selected = match device {
            Some(name) => {
                let found = host.output_devices().ok().and_then(|mut devices| {
                    devices.find(|d| d.description().is_ok_and(|desc| desc.name() == name))
                });
                match found {
                    Some(device) => device,
                    None => {
                        let fallback = host.default_output_device().ok_or_else(|| {
                            format!("output device \"{name}\" is gone and no default output exists")
                        })?;
                        notice = Some(format!(
                            "Output device \"{name}\" is not available; using \"{}\"",
                            fallback
                                .description()
                                .map(|d| d.name().to_string())
                                .unwrap_or_else(|_| "unknown".into())
                        ));
                        fallback
                    }
                }
            }
            None => host
                .default_output_device()
                .ok_or_else(|| "no audio output device found".to_string())?,
        };
        let device_name = selected
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "unknown".into());
        let (error_tx, error_rx) = mpsc::channel();
        // Same filter as rodio's examples/error_callback.rs: only actionable
        // losses reach the app; transient errors are logged, not fatal.
        let make_on_error = || {
            let callback_tx = error_tx.clone();
            move |err: cpal::StreamError| {
                if matches!(
                    err,
                    cpal::StreamError::DeviceNotAvailable | cpal::StreamError::StreamInvalidated
                ) {
                    let _ = callback_tx.send(err.to_string());
                } else {
                    eprintln!("scriptcast-ui: audio stream error: {err}");
                }
            }
        };

        let mut sink = None;
        let want_rate = 48_000;

        if let Ok(configs) = selected.supported_output_configs() {
            let configs: Vec<_> = configs.collect();
            let ranges: Vec<_> = configs
                .iter()
                .map(|r| (r.channels(), r.min_sample_rate(), r.max_sample_rate()))
                .collect();
            let default_channels = selected
                .default_output_config()
                .map(|c| c.channels())
                .unwrap_or(2);

            if let Some(idx) = pick_rate(&ranges, default_channels, want_rate) {
                let config = configs[idx].with_sample_rate(want_rate);
                if let Ok(builder) = DeviceSinkBuilder::from_device(selected.clone())
                    && let Ok(s) = builder
                        .with_supported_config(&config)
                        .with_error_callback(make_on_error())
                        .open_sink_or_fallback()
                {
                    sink = Some(s);
                }
            }
        }

        if sink.is_none() {
            sink = Some(
                DeviceSinkBuilder::from_device(selected.clone())
                    .map_err(|e| e.to_string())?
                    .with_error_callback(make_on_error())
                    .open_sink_or_fallback()
                    .map_err(|e| e.to_string())?,
            );
        }

        let mut sink = sink.unwrap();
        // What actually opened: open_sink_or_fallback may silently pick another config.
        let actual_rate = sink.config().sample_rate().get();
        sink.log_on_drop(false);
        Ok(Self {
            sink,
            device_name,
            notice,
            player: None,
            queue: TurnQueue::default(),
            fallback_duration: Duration::ZERO,
            muted: false,
            base: Duration::ZERO,
            started: Instant::now(),
            paused_at: None,
            error_tx,
            error_rx,
            stream_rate: actual_rate,
        })
    }

    /// Names of the currently available output devices (the null driver, if
    /// any, is excluded). Callable without an open output so the device menu
    /// still populates after an `open` failure.
    pub fn outputs() -> Vec<String> {
        let host = cpal::default_host();
        let mut names = Vec::new();
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                let Ok(description) = device.description() else {
                    continue;
                };
                if description.driver().is_some_and(|driver| driver == "null") {
                    continue;
                }
                names.push(description.name().to_string());
            }
        }
        names
    }

    /// One-time notice from `open` (for example a fallback to the default
    /// output because the requested device is gone).
    pub fn take_notice(&mut self) -> Option<String> {
        self.notice.take()
    }

    pub fn stream_rate(&self) -> u32 {
        self.stream_rate
    }
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Manifest timing for the turn about to be played; `duration()` falls
    /// back to it when a decoder reports no duration.
    pub fn set_fallback_duration(&mut self, duration: Duration) {
        self.fallback_duration = duration;
    }

    /// Stop the current player and play `wavs` (one per segment) in order.
    /// A missing or undecodable file is an `Err` naming the path.
    pub fn play(&mut self, turn: usize, wavs: &[PathBuf]) -> Result<(), String> {
        self.stop();
        if wavs.is_empty() {
            return Err(format!("turn {turn} has no audio segments"));
        }
        let player = Player::connect_new(self.sink.mixer());
        let mut total = Duration::ZERO;
        for path in wavs {
            let (source, duration) = pcm_source(path, None, None, self.stream_rate)?;
            total += duration;
            player.append(source);
        }
        if self.muted {
            player.set_volume(0.0);
        }
        player.play();
        let dur = if total > Duration::ZERO {
            total
        } else {
            self.fallback_duration
        };
        self.queue.clear();
        self.queue.push(turn, dur);
        self.player = Some(player);
        self.base = Duration::ZERO;
        self.started = Instant::now();
        self.paused_at = None;
        Ok(())
    }

    /// Play a WAV already held in memory. Used after ffmpeg extracts a clip.
    pub fn play_wav_bytes(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        self.stop();
        let pcm = crate::waveform::output_samples_bytes(&bytes, "wav", self.stream_rate)?;
        let (source, total) = source_from_pcm(pcm)?;
        let player = Player::connect_new(self.sink.mixer());
        if self.muted {
            player.set_volume(0.0);
        }
        player.append(source);
        player.play();
        self.queue.clear();
        self.queue.push(0, total);
        self.player = Some(player);
        self.base = Duration::ZERO;
        self.started = Instant::now();
        self.paused_at = None;
        Ok(())
    }

    /// Append `turn`'s segments behind the current audio in the same player,
    /// so auto-advance has no gap. No-op when nothing is playing.
    pub fn queue_next(&mut self, turn: usize, wavs: &[PathBuf]) -> Result<(), String> {
        let Some(player) = &self.player else {
            return Ok(());
        };
        if wavs.is_empty() {
            return Err(format!("turn {turn} has no audio segments"));
        }
        let mut total = Duration::ZERO;
        for path in wavs {
            let (source, duration) = pcm_source(path, None, None, self.stream_rate)?;
            total += duration;
            player.append(source);
        }
        let dur = if total > Duration::ZERO {
            total
        } else {
            self.fallback_duration
        };
        self.queue.push(turn, dur);
        Ok(())
    }

    pub fn queued_after_current(&self) -> bool {
        self.queue.entries.len() > 1
    }

    /// Drop turns whose audio has finished and rebase the wall clock onto the
    /// turn now playing. Called once per frame.
    pub fn sync_queue(&mut self) {
        if self.player.is_none() {
            return;
        }
        let elapsed = match self.paused_at {
            Some(at) => at,
            None => self.base + self.started.elapsed(),
        };
        if let Some((_, _, finished_count)) = self.queue.current(elapsed)
            && finished_count > 0
        {
            let mut dropped_duration = Duration::ZERO;
            for _ in 0..finished_count {
                if let Some((_, dur)) = self.queue.entries.pop_front() {
                    dropped_duration += dur;
                }
            }
            if let Some(at) = self.paused_at.as_mut() {
                *at = at.saturating_sub(dropped_duration);
            } else {
                self.base = elapsed.saturating_sub(dropped_duration);
                self.started = Instant::now();
            }
        }
    }

    pub fn pause(&mut self) {
        let pos = self.position();
        if let Some(player) = &self.player
            && self.paused_at.is_none()
        {
            self.paused_at = Some(pos);
            player.pause();
        }
    }
    pub fn resume(&mut self) {
        if let (Some(player), Some(at)) = (&self.player, self.paused_at.take()) {
            self.base = at;
            self.started = Instant::now();
            player.play();
        }
    }
    pub fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
        self.queue.clear();
        self.paused_at = None;
        self.base = Duration::ZERO;
    }
    pub fn is_paused(&self) -> bool {
        self.player.as_ref().is_some_and(Player::is_paused)
    }
    /// True when a turn was started and the whole player queue has drained.
    pub fn is_finished(&self) -> bool {
        !self.queue.entries.is_empty() && self.player.as_ref().is_some_and(Player::empty)
    }
    /// The turn whose audio is playing now (it follows gapless boundaries),
    /// `None` once everything has finished.
    pub fn playing(&self) -> Option<usize> {
        if self.is_finished() {
            None
        } else {
            let elapsed = match self.paused_at {
                Some(at) => at,
                None => self.base + self.started.elapsed(),
            };
            self.queue.current(elapsed).map(|(turn, _, _)| turn)
        }
    }
    /// Tracked position within the current turn (wall clock); `Player::get_pos()`
    /// is per-segment and not used.
    pub fn position(&self) -> Duration {
        if self.player.is_none() {
            return Duration::ZERO;
        }
        let elapsed = match self.paused_at {
            Some(at) => at,
            None => self.base + self.started.elapsed(),
        };
        if let Some((_, offset, _)) = self.queue.current(elapsed) {
            offset
        } else {
            self.duration()
        }
    }
    /// Duration of the current turn: the sum of decoded segment durations, or
    /// the manifest timing set with `set_fallback_duration` when a decoder
    /// could not report one.
    pub fn duration(&self) -> Duration {
        self.queue
            .entries
            .front()
            .map(|&(_, dur)| dur)
            .unwrap_or(Duration::ZERO)
    }
    /// Seek within the currently playing segment. No-op on failure (rodio
    /// seeks only the current source; multi-segment turns cannot seek past
    /// their first segment boundary).
    pub fn seek(&mut self, position: Duration) {
        if let Some(player) = &self.player
            && player.try_seek(position).is_ok()
        {
            self.base = position;
            self.started = Instant::now();
            if self.paused_at.is_some() {
                self.paused_at = Some(position);
            }
        }
    }
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(player) = &self.player {
            player.set_volume(if muted { 0.0 } else { 1.0 });
        }
    }
    /// The most recent output-device loss, if any, reported by the stream's
    /// error callback. Polled by the app once per frame.
    pub fn take_device_error(&self) -> Option<String> {
        self.error_rx.try_recv().ok()
    }
    /// Capture/test hook: inject a device loss through the same channel the
    /// real stream error callback uses.
    pub fn simulate_device_loss_for_capture(&self, message: &str) {
        let _ = self.error_tx.send(message.to_string());
    }
    /// Diagnostic: rodio's per-segment `Player::get_pos()`. Not used for the
    /// UI position; exists to document its behaviour across segment boundaries.
    pub fn raw_player_position(&self) -> Option<Duration> {
        self.player.as_ref().map(Player::get_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays the fixture's two-segment turn 2 (muted) end to end: asserts the
    /// player finishes and that `duration()` is the sum of both WAVs
    /// (0.4 s ± 20 ms). Also samples rodio's per-segment `get_pos()` so the
    /// findings file can show it resetting at the segment boundary.
    #[test]
    #[ignore = "needs an audio output device; plays muted"]
    fn fixture_two_segment_turn_plays_in_order() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small");
        let episode = crate::episode::Episode::load(&dir).unwrap();
        let turn = &episode.turns[1];
        assert_eq!(turn.index, 2);
        // Chunk order comes from the loader (manifest order is scrambled):
        // turn_2_0.wav (chunk 0) must play before turn_2_1.wav (chunk 1).
        assert!(turn.wav_paths[0].ends_with("turn_2_0.wav"));
        assert!(turn.wav_paths[1].ends_with("turn_2_1.wav"));

        let mut audio = AudioOut::open(None).expect("an audio output device");
        audio.set_muted(true);
        audio.set_fallback_duration(Duration::from_millis(400));
        audio.play(turn.index, &turn.wav_paths).unwrap();
        let duration = audio.duration();
        let drift = duration.as_millis().abs_diff(400);
        assert!(drift <= 20, "duration {duration:?}, expected ~400 ms");

        let deadline = Instant::now() + Duration::from_secs(10);
        let mut raw = Vec::new();
        while !audio.is_finished() {
            assert!(Instant::now() < deadline, "player did not finish");
            if let Some(pos) = audio.raw_player_position() {
                raw.push(pos.as_millis());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        eprintln!("rodio get_pos() across the two segments (ms): {raw:?}");
        assert!(audio.playing().is_none());
        assert!(audio.position() <= duration);
    }

    #[test]
    fn test_pick_rate() {
        let want = 48_000;

        // Match on the default channel count
        let ranges = [(2, 44100, 44100), (2, 48000, 96000), (1, 48000, 48000)];
        assert_eq!(super::pick_rate(&ranges, 2, want), Some(1));

        // Match only on another channel count
        let ranges = [(2, 44100, 44100), (1, 44100, 96000)];
        assert_eq!(super::pick_rate(&ranges, 2, want), Some(1));

        // No range containing 48 000
        let ranges = [(2, 44100, 44100), (1, 96000, 96000)];
        assert_eq!(super::pick_rate(&ranges, 2, want), None);

        // Empty list
        let ranges: [(u16, u32, u32); 0] = [];
        assert_eq!(super::pick_rate(&ranges, 2, want), None);
    }
    #[test]
    fn test_turn_queue() {
        let mut q = TurnQueue::default();
        assert_eq!(q.current(Duration::from_secs(0)), None);

        q.push(1, Duration::from_secs(10));
        q.push(2, Duration::from_secs(5));

        // before first boundary
        assert_eq!(
            q.current(Duration::from_secs(5)),
            Some((1, Duration::from_secs(5), 0))
        );
        // exactly at boundary
        assert_eq!(
            q.current(Duration::from_secs(10)),
            Some((2, Duration::from_secs(0), 1))
        );
        // past the last turn (returns None);
        assert_eq!(q.current(Duration::from_secs(16)), None);
    }

    #[test]
    fn audio_info_display_formats_layout_and_khz() {
        let mono = AudioInfo {
            channels: 1.try_into().unwrap(),
            sample_rate: 48_000.try_into().unwrap(),
            duration: None,
        };
        assert_eq!(mono.to_string(), "mono · 48 kHz");
        let stereo = AudioInfo {
            channels: 2.try_into().unwrap(),
            sample_rate: 44_100.try_into().unwrap(),
            duration: None,
        };
        assert_eq!(stereo.to_string(), "stereo · 44.1 kHz");
    }

    #[test]
    fn probe_reads_fixture_wav_headers() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/episode_small");
        let info = probe(&dir.join("Samples/Processed/turn_1_0.wav")).unwrap();
        assert_eq!(info.channels.get(), 1);
        assert_eq!(info.sample_rate.get(), 48_000);
        let duration = info.duration.unwrap();
        assert!(duration.as_millis().abs_diff(200) <= 20, "{duration:?}");
        assert!(probe(&dir.join("Samples/Processed/nope.wav")).is_none());
    }
}

pub fn is_builtin_speakers(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("macbook") && lower.contains("speakers")
}

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    #[test]
    fn test_is_builtin_speakers() {
        assert!(is_builtin_speakers("MacBook Pro Speakers"));
        assert!(is_builtin_speakers("macbook air speakers"));
        assert!(!is_builtin_speakers("Headphones"));
        assert!(!is_builtin_speakers("AirPods"));
        assert!(!is_builtin_speakers("External DAC"));
        assert!(!is_builtin_speakers(""));
    }
}

fn pcm_source(
    path: &Path,
    start: Option<f64>,
    end: Option<f64>,
    out_rate: u32,
) -> Result<(SamplesBuffer, Duration), String> {
    let pcm = crate::waveform::output_samples(path, start, end, out_rate)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    source_from_pcm(pcm)
}

fn source_from_pcm(pcm: crate::waveform::OutputPcm) -> Result<(SamplesBuffer, Duration), String> {
    let channels = ChannelCount::try_from(pcm.channels).map_err(|_| "audio has no channels")?;
    let rate = SampleRate::try_from(pcm.sample_rate).map_err(|_| "audio has a zero sample rate")?;
    let frames = pcm.interleaved.len() / usize::from(channels.get());
    let duration = Duration::from_secs_f64(frames as f64 / f64::from(rate.get()));
    Ok((
        SamplesBuffer::new(channels, rate, pcm.interleaved),
        duration,
    ))
}

/// Pick the config range to open at `want` Hz: the first range whose
/// min..=max contains `want`, preferring the device's default channel count.
fn pick_rate(ranges: &[(u16, u32, u32)], default_channels: u16, want: u32) -> Option<usize> {
    ranges
        .iter()
        .position(|&(c, min, max)| c == default_channels && want >= min && want <= max)
        .or_else(|| {
            ranges
                .iter()
                .position(|&(_, min, max)| want >= min && want <= max)
        })
}

impl AudioOut {
    /// Archive previews share this output device, with a separate finite player and the same mute setting.
    pub fn preview_range(&self, path: &Path, start: f64, end: f64) -> Result<Player, String> {
        if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
            return Err("Invalid clip range".into());
        }
        let player = Player::connect_new(self.sink.mixer());
        if self.muted {
            player.set_volume(0.0);
        }
        let (source, _) = pcm_source(path, Some(start), Some(end), self.stream_rate)?;
        player.append(source);
        player.play();
        Ok(player)
    }
}
