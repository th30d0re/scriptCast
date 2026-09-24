# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Unreleased

### Added
- `scriptcast-refit` remeasures existing WAVs, backs up the manifest, updates render state, and optionally relays out clips with `--gap-ms`.
- Project settings `tail_ms` (400) and `speech_threshold` (0.03), shared by CLI defaults. Explicit CLI flags override saved refit settings, which override project settings on subsequent renders.

### Fixed
- Preserve the requested speech tail when reloading unchanged WAV segments, including resumed renders.

## [0.1.0] - 2026-09-17

scriptCast was extracted as a standalone tool from an earlier in-project `voice_pipeline/` with its full history preserved.

### Added
- **13 Console Scripts**: `scriptcast` for rendering, plus `scriptcast-verify`, `scriptcast-repair`, `scriptcast-relayout`, `scriptcast-stitch`, `scriptcast-retime`, `scriptcast-turn-index`, `scriptcast-outliers`, `scriptcast-stress`, `scriptcast-clip`, `scriptcast-reference`, `scriptcast-audition`, and `scriptcast-calibrate`.
- **4 TTS Engines**: Kokoro (MLX), Chatterbox (MLX), OmniVoice, and ElevenLabs.
- **Project Structure**: `scriptcast.toml` configuration for managing voices, pronunciations, outputs, and clips.
- **Export Formats**: Exports to Ableton Live sets (`.als`), Final Cut Pro timelines (`.fcpxml`), and Logic Pro sessions.
- **Verification & Repair Loop**: Transcription and phoneme stress checks via Whisper, with automatic re-synthesis of failed turns.
- **Archival Clips**: A clips registry (`scriptcast-clip`) that splices raw recordings directly into the generated timeline.
- Neutral test fixtures.
- Licensed under **GPL-3.0-or-later**.

### Changed
- The environment variable for custom Ableton templates is now `SCRIPTCAST_ALS_TEMPLATE` (renamed from the in-project equivalent).

[0.1.0]: https://github.com/th30d0re/scriptCast/releases/tag/v0.1.0
