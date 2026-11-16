# Brief: write the scriptCast competition submission

Paste everything below the line into the model. Fill the four `[FILL]` fields first.

---

## Your task

Write a technical submission for an AI product competition. The product is **scriptCast**. The judges have said they do not want chat wrappers: thin UIs over a hosted LLM, prompt templates with a logo, or "GPT for X". Your job is to show, with specifics, that scriptCast is not that. It is a compiler and verification system for audio and video, and its AI models are components inside a deterministic pipeline, not the product's interface.

Competition: [FILL: name]
Track or category: [FILL]
Length limit: [FILL: words or pages]
Required sections, if any: [FILL]. If none are given, use the structure below.

## Rules for the writing

- Technical register. Write for engineers who will read the repo. No marketing adjectives ("revolutionary", "seamless", "powerful", "cutting-edge"). Prefer a number, a file name, or a command to a claim.
- Use only the facts in this brief. Don't invent benchmarks, user counts, latency numbers, or features. Where a number would help but isn't here, write `[NUMBER NEEDED: what to measure]` so the author can fill it.
- Say "on-device" and "offline" only where the brief says so. ElevenLabs is the one cloud engine and is optional.
- Address the "not a wrapper" question head-on in its own section. Don't be defensive. Show it.
- Name things exactly as written here: `scriptcast-ui`, `episode_manifest.json`, `turn_id`, and so on.

## What scriptCast is (one paragraph to build from)

scriptCast compiles a Markdown script into per-turn speech audio, frame-accurate video timelines, and native editing sessions for Ableton Live, Logic Pro, and Final Cut Pro. Each speaker turn is synthesized by that speaker's configured engine, checked against the script by on-device speech recognition, re-synthesized automatically if it fails, and placed on one canonical clock (`episode_manifest.json`). That clock drives the audio mix, the video cut, the captions, and the DAW exports, so audio and picture can't drift apart through edits. It runs locally on Apple Silicon.

## Core idea: production as compilation, not generation

The design thesis (from `docs/biological-diffusion-layer.md`): diffusion-style generators treat each output as a fresh lottery, so changing one line means regenerating the scene. scriptCast treats production as incremental compilation:

- **Source:** a Markdown script. Turns have speakers and inline directives: `[tone:…]`, `[pause:…]`, and `[clip:…]` to splice registered archival audio. A shot list (`shotspec`) specifies the visuals.
- **Content addressing:** every turn's identity is `turn_id`, a hash of speaker and text, tracked in `render_state.json`. Editing one sentence recompiles one turn. Re-timing the episode (`--gap-ms`, `scriptcast-relayout`) re-synthesizes nothing. Approved audio is byte-preserved.
- **The human as the denoiser:** the operator iterates passes over pacing, emphasis, and visual timing. Neural voices are fast prototypes, and a live microphone take can replace any single turn in the app.

## Architecture

Pipeline: parse script → route each turn to its speaker's engine → post-process (Rust DSP) → timeline and `render_state.json` → `episode_manifest.json` → verify → repair the failures → relayout → stitch and mix → turn index and captions. The manifest also feeds the DAW/NLE exports and the video tools.

**Speech engines (per speaker, set in `voices.yaml`):**
- MLX Kokoro: local, preset voices.
- MLX Chatterbox: local voice cloning from a reference clip.
- OmniVoice: local cloning, run as a worker process.
- Phonon: local.
- Qwen3 Tone: local, run as a worker.
- ElevenLabs: optional cloud engine.
- `archive`: plays a registered real-world clip as a turn (clips registry).

**Verification and repair (the closed loop):**
- `scriptcast-verify` transcribes every segment with on-device Whisper (mlx-whisper) and scores it against the script text.
- An ensemble pass adds Phonon-2 as a second, independent recognizer.
- `scriptcast-stress` uses a phoneme recognizer to check heteronym stress, catching errors Whisper can't hear (e.g. "read" vs "read", "live" vs "live").
- `scriptcast-repair` re-synthesizes only the failed turns and re-checks only those, up to `--max-passes`. The final pass relays the timeline.
- The principle: generated output is never trusted. It's checked against ground truth (the script) by models independent of the generator.

**Video ("Stage"), locked to the same clock:**
- Every visual cue resolves to `turns[].start_ms` / `end_ms` in the manifest. Frame boundaries are `round(start_ms / 1000 × fps)`.
- `scriptcast-shotspec` resolves Markdown shot lists and citations into manifest timestamps.
- `scriptcast-video` emits a typed JSON timeline (schema: `video/src/timeline/timeline.schema.json`) and renders data-driven Remotion scenes at 1080×1920 (Shorts), 1920×1080, or 3840×2160, with runtime safe-zone overflow guards.
- `scriptcast-youtube` cuts a 16:9 picture track onto the episode clock, 4K by default.
- `scriptcast-craft` and `scriptcast-captions` produce FCPXML 1.10 timelines and script-aligned JSON, SRT, and WebVTT captions.
- Video verification mirrors audio verification (`docs/STAGE_PRODUCTION_PARADIGM.md`): the scene spec is the ground truth, OCR plus pixel measurement reads rendered stills back, a vision model reviews only the flagged ones, and repair goes back to the session that built the scene.
- In-progress work, labeled as such: the **Laya Router**, a local model router that sends each stage task to the right local model and context source. It runs on an M1 Max with 64 GB.

**Native app (`scriptcast-ui`):** a Rust/egui macOS app. A 60 Hz timeline and waveform, per-turn verification state, find and replace across the episode, a transcript editor with tag toolbar, voices/clips/pronunciations editors, reference-clip maker and audition, live take recording into a turn, a Mix editor with A/B playback, a preview of the video shot under the playhead at 1080p, and one-click open in Live or Logic. Released as `scriptCast for macOS 0.22.0` (ad-hoc signed, Apple Silicon).

**Rust components:** latency-critical and DSP work runs as standalone Rust binaries: `scriptcast-ui`, `scriptcast-chunk` (text chunking), `scriptcast-fx` (effects chains, loudness), `scriptcast-media` (spectral dialogue carving and ducking music under speech), `scriptcast-tone`.

## Scale and evidence (verified from the repo, 2026-11-15)

- About 160,000 lines of Rust and about 20,500 lines of Python.
- 454 commits. 171 test and fixture files, including snapshot tests of the UI.
- Used in production for a long-form YouTube series. Episodes have run to 99 turns and about 17.5 minutes of single-narrator audio (the "iPhone Duo" essay).
- Open source on GitHub (`th30d0re/scriptCast`).

## Why this isn't a wrapper (make these points, with the evidence above)

1. The AI is plural, local, and specialized. Six speech engines, two independent recognizers, a phoneme model, OCR, and a VLM, each doing one checked job. None of them is the UI.
2. Verification is built into the architecture. Every generated artifact is measured against the source of truth by a model other than the one that made it, and failures are repaired automatically and in isolation.
3. Determinism and incrementality. The content-addressed turns and the single clock give compiler properties (reproducible output, minimal rebuilds) to a domain where generative tools usually give neither.
4. Interop with professional tools, not lock-in. The output is real Ableton, Logic, and FCPXML sessions and standard caption files.
5. On-device by default. It runs offline on Apple Silicon, with no per-minute API cost except the optional ElevenLabs engine.

## Built on (credit these in an Acknowledgements section)

Be specific and generous. Judges respect a clear account of what's original. The original work is the compiler design, the content-addressed turns, the single clock, the cross-model verify/repair loop, the Stage verification approach, the Rust DSP binaries, and the native app. The models and libraries below are what it composes.

- Ported: HyperFrames (HeyGen, Apache-2.0). Dialogue ducking and spectral carve, re-implemented in Rust in `scriptcast-media`.
- Speech models: Kokoro-82M (hexgrad) through mlx-audio and misaki; Chatterbox (Resemble AI); OmniVoice (k2-fsa); Phonon (Gradium); Qwen3-TTS VoiceDesign (Qwen); ElevenLabs (optional).
- Recognition and vision: Whisper (OpenAI) through mlx-whisper; Phonon-2 (Gradium); wav2vec2 phoneme model (Meta) with eSpeak; Tesseract OCR; SigLIP (Google); MiniCPM-V (OpenBMB); Laya.
- Platform and libraries: Apple MLX, FFmpeg, egui/eframe, soundcraft, Remotion, React, d3, KaTeX, Quiver AI.
- Full list: `ACKNOWLEDGEMENTS.md` in the repo.

## Suggested structure

1. Summary (3–4 sentences).
2. The problem: long-form narrated audio and video with AI voices today is either a cloud lottery or manual stitching, and nothing checks the output against the script.
3. The approach: production as compilation.
4. Architecture: pipeline, engines, verify/repair loop, single clock, video.
5. Why it isn't a wrapper.
6. Engineering evidence: scale, tests, the shipped app, real episodes.
7. Limitations and roadmap, stated plainly. Apple Silicon only. The app is ad-hoc signed and not notarized. The Python tools install from the repo, and the app isn't a single self-contained package yet. The Laya Router is in progress.
8. Acknowledgements: from "Built on" above.
9. Links: repo, release, demo video `[FILL: URL]`.

Output in Markdown.
