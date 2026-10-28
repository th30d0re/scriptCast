# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Unreleased

### Added
- Tone TTS (`engine: tone`) through the local Qwen3 VoiceDesign checkpoint, with per-speech `[tone:…]` voice descriptions and a persistent `scriptcast-tone` worker.
- The renderer calls `scriptcast-chunk` when an engine sets `max_chars` and a speech job is longer than that, and when a take runs away (non-finite samples, or longer than 8 seconds and slower than 0.6 seconds per character). Pieces crossfade inside the turn. A missing binary fails that render. Engines do not set a cap yet, so a stable take is still one synthesis call.
- `scriptcast-youtube` cuts a 16:9 picture onto the episode clock: one shot per turn, frame `round(start_ms / 1000 × fps)`. The default is 3840×2160; 1920×1080 and 960×540 remain. `figcraft` draws one still per beat through `figcraft-cli` and holds it. `conform` fits an external video. The vertical 1080×1920 pipeline is unchanged.
- The macOS app shows that shot on the playhead and plays its silent run. Preview is 1080p or 720p; a 4K master is not decoded at 4K. Timeline and playback waveforms draw as one mesh at 60 Hz.
- Phonon TTS (`engine: phonon` in `voices.yaml`). `phonon_voice` is a Phonon catalog name or a `.safetensors` clone. `ptts` is a core dependency. `SCRIPTCAST_PHONON_SEED` draws a different take; otherwise Phonon is deterministic.
- Native macOS app `scriptcast-ui` (Rust). It opens a project, shows the episode timeline, plays turns, and edits voices, clips, and pronunciations. Apple Silicon builds are attached to GitHub releases.
- `scriptcast-craft cue` writes a 1080×1920, 30 fps cue sheet from the episode manifest clock. `scriptcast-craft timeline` places silent Craft picture on that clock in FCPXML 1.10 and writes SRT and WebVTT beside it from the captions JSON. Dialogue wavs and `scriptcast-captions` JSON are unchanged.
- Dynamic density scaling for cards, optimizing font sizes and layout for large text content while staying within the safe zone. Added "hero" art slot support across all cards. Added runtime visual overflow guards.
- Quiver SVG generation, animation, and model-list client with bounded retries, a manifest-backed asset cache, offline checks, and `scriptcast-video svg` dry runs. All four cards render public-relative `svgAsset` artwork inside the safe zone; a hand-written SVG example is included.
- Episode Remotion assembly and `scriptcast-video render`: script-order archive resolution, card windows, muted excerpts with last-frame holds, per-run media staging, `--plan-only`, and `--dry-run`.
- Standalone Remotion card workspace with safe-zone layouts, four JSON-driven cards, G-10/G-11 examples, and scriptcast-video still with optional Chrome selection.
- `scriptcast-shotspec` resolves Markdown shot lists through script turn order into manifest timing; extracts URL/card citations and validates `[source]`. Manuscript citations and checks are opt-in with `--book PATH` and/or `--repo ROOT`; supports JSON output and `--strict`.
- `scriptcast-refit` remeasures existing WAVs, backs up the manifest, updates render state, and optionally relays out clips with `--gap-ms`.
- Project settings `tail_ms` (400) and `speech_threshold` (0.03), shared by CLI defaults. Explicit CLI flags override saved refit settings, which override project settings on subsequent renders.

### Fixed
- Accept three-digit minutes in transcript headers and retiming headers, shot-list anchors, and hold times, preserving timestamps beyond 99:59.
- Prevented default prop leakage from unrelated examples in card compositions. Fails fast in Python if a card references missing SVG assets, with an --allow-missing-assets override.
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
