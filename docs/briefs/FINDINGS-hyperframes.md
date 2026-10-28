# Findings: HyperFrames vs ScriptCast Comparison

**Date:** 2026-10-26  
**Repositories Analyzed:**  
- **ScriptCast:** `/Users/emmanuel/Dev/Tools/scriptCast` (`v0.1.0`, commit history through Phase M native UI & video tooling)  
- **HyperFrames:** `/Users/emmanuel/Dev/Tools/hyperframes` (`heygen-com/hyperframes`, `v0.8.205`)

---

## 1. What Each Project Is

### Summary Table

| Dimension | ScriptCast (`/Users/emmanuel/Dev/Tools/scriptCast`) | HyperFrames (`/Users/emmanuel/Dev/Tools/hyperframes`) |
|---|---|---|
| **Primary Purpose** | Audio-first multi-speaker podcast episode synthesizer, ASR/phoneme verifier, DAW session exporter, and 9:16 portrait video assembler on Apple Silicon (`README.md:7,24,40`). | General-purpose HTML/CSS/GSAP-to-video rendering engine, component registry, browser visual studio, and AI agent skill suite (`README.md:33,205-240`, `DESIGN.md:1-14`). |
| **Core Unit of Composition** | Spoken dialogue or archival turn (`turn_NNN` / content-hashed `turn_id`) indexed on a millisecond episode clock in `episode_manifest.json` (`scriptcast/manifest.py`, `swift/EpisodeManifest.swift:26-101`). | HTML DOM element (`class="clip"`, `data-composition-id`, `data-start`, `data-duration`, `data-track-index`) coupled to a deterministic `window.__hf.seek(t)` function (`packages/core/src/runtime/README.md`, `packages/parsers/src/htmlParser.ts`). |
| **Architecture** | Python 3.11+ engine & 29 CLI tools (`pyproject.toml:45-74`, `scriptcast/`), adjacent Remotion 4.0.354 React workspace (`video/`), native Rust `eframe`/`egui` desktop app (`ui/`), and Swift manifest types (`swift/EpisodeManifest.swift`). | Bun/TypeScript 15-package monorepo (`package.json:7-10`, `packages/*`), static HTML/CSS/JS component registry (`registry/`), and 7 agent skills with helper scripts (`skills/*`). |
| **Render Pipeline** | Script parse → per-speaker TTS / archival ffmpeg cut → 48 kHz WAV post-process → ASR & phoneme stress verification/repair → manifest & DAW export (`.als`, `.fcpxml`, Logic) → shot-spec & script-aligned caption generation → Remotion chunk render or Craft FCPXML export (`scriptcast/__main__.py`, `scriptcast/tools/video.py`, `scriptcast/tools/craft_export.py`). | Static lint & headless Chrome check → Producer Stage 1 compile (sub-compositions, local fonts) → Stage 1b browser probe → Stage 2 `<video>` frame pre-extraction to `<img>` → Stage 3 Web Audio / FFmpeg mix → Stage 4 parallel Chrome `BeginFrame`/`drawElement` capture with `FrameReorderBuffer` → Stage 5/6 `image2pipe` FFmpeg encode & mux (`packages/producer/src/services/renderOrchestrator.ts:1-120`). |
| **Input Format** | Markdown dialogue transcript (`Speaker (MM:SS)` + inline `[pause:Nms]`, `[beat]`, `[clip:id]`, `[emphasis]`, `[tone:...]`), `scriptcast.toml`, `voices.yaml`, `pronunciations.yaml`, `archive/clips.yaml`, `mix.yaml`, Markdown shot lists, and JSON card props (`README.md:148-246`, `docs/briefs/v0.2-video-tooling.md`). | Single-file or multi-composition HTML (`index.html`, `compositions/*.html`) with `data-*` timing attributes, paused GSAP timelines (`window.__timelines`), CSS/SVG/Canvas/WebGL/Three.js/Lottie, and `audio_request.json` for skill-driven audio (`skills/hyperframes/SKILL.md`, `skills/media-use/references/audio.md`). |
| **Tech Stack** | Python 3.11 (`mlx-audio`, `mlx-whisper`, `misaki[en]`, `soundfile`, `scipy`, `elevenlabs`, `ptts`, `opencv-python`), Node/TS (`remotion` 4.0.354, `react` 19.1.0, `@quiverai/sdk` 0.9.4), Rust 2024 (`eframe` 0.36.2, `rodio` 0.22.2, `soundcraft-dsp`), Swift, FFmpeg (`pyproject.toml`, `video/package.json`, `ui/Cargo.toml`). | Node >=22, Bun 1.4.4, TypeScript 5, `puppeteer-core` 25.10, `linkedom`, `acorn`, `gsap` 3.14, `@chenglou/pretext`, `hono`, `react` 19 (`packages/studio`), `kokoro-onnx` / `onnxruntime-node`, `parakeet-mlx` / `whisper.cpp`, FFmpeg (`package.json`, `packages/cli/package.json`). |
| **License** | `GPL-3.0-or-later` (`LICENSE`, `pyproject.toml:10`, `README.md:339`). | `Apache-2.0` (`LICENSE`, `packages/cli/package.json:21`, `README.md:353`). |

---

### 1.1 ScriptCast Deep Dive (`/Users/emmanuel/Dev/Tools/scriptCast`)

- **Purpose:** ScriptCast synthesizes multi-speaker podcast episodes and vertical video explainers from a human-readable Markdown script (`README.md:24-36`). Every dialogue turn is rendered to an isolated 48 kHz mono WAV file (`Samples/Processed/<speaker>/<turn_id>_chunk_NNNN.wav`), measured for speech duration, placed on an episode timeline (`episode_manifest.json`), checked against ASR and phoneme models for mispronunciations or synthesis artifacts, and exported to DAWs or video renderers (`README.md:65-80`, `CHANGELOG.md:10-48`).
- **Architecture:**
  1. **Python Core (`scriptcast/`):**
     - `parser.py` & `markup.py`: Parses `Display Name (MM:SS)` turn headers, inline tags (`[pause:Nms]`, `[beat]`, `[clip:id]`, `[emphasis]`, `[tone:...]`, `[pronounce:word=phonemes]`), and archival clips (`README.md:219-246`).
     - `engine.py`: Dispatches synthesis across 6 operational engines registered in `ENGINE_REGISTRY` (`scriptcast/engine.py:667-675`): `mlx_kokoro` (local MLX Kokoro-82M), `mlx_chatterbox` (local MLX voice cloning), `omnivoice` (subprocess worker via `scriptcast/omnivoice_worker.py` in `.venv-omnivoice` supporting `instruct` and `emphasis_reference_wav`), `elevenlabs` (cloud API), `phonon` (on-device Gradium `.safetensors` voice clones via `ptts`), and `archive` (ffmpeg excerpt extraction via `scriptcast/archive.py`), plus stub `mlx_dia`.
     - `manifest.py` & `render_state.py`: Emits schema 2.0 `episode_manifest.json` and `render_state.json` with SHA-256 content hashes (`text_hash`, `voice_hash`, `speech_text_hash`) for incremental turn regeneration (`--detect-changes`, `--precision-insert`, `--regenerate-turns`).
     - DAW Exporters: `als_generator.py` (Ableton Live 12 `.als` gzipped XML at 120 BPM with `.asd` cache invalidation), `fcpxml_generator.py` (Final Cut Pro FCPXML 1.10), and `logic_exporter.py` (Logic Pro AppleScript automation).
     - CLI Tooling (`scriptcast/tools/*.py`): 29 console entry points in `pyproject.toml:45-74` for verification, ensemble ASR, phoneme stress checking, outlier detection, audio mixing, tape-stop DSP, shot-spec resolution, captions, video rendering, and Craft interchange.
  2. **Video Workspace (`video/`):**
     - Remotion 4.0.354 + React 19 TypeScript workspace (`video/package.json`).
     - Renders 9:16 portrait frames (`1080×1920`, 30 fps) constrained to a Meta Reels safe-zone `USABLE_BOX` (`x=64.8, y=268.8, w=950.4, h=883.2` in `video/src/safeZone.ts:15-26`) with reserved caption space at `CAPTION_TOP = 1162` (`video/src/safeZone.ts:32`).
     - Provides `Episode.tsx` (full episode compositor with `<Audio>`, video `<Sequence>` slots, `<CardSlot>`, and synchronized word-highlighted `<CaptionBar>`, `video/src/Episode.tsx:108-262`) and 6 deterministic JSON-driven data cards (`TimelineCard`, `StatBarsCard`, `TitleCard`, `CompareCard`, `FreezeCallout`, `PartsCard` under `video/src/cards/`) wrapped in `Frame.tsx` (`video/src/cards/Frame.tsx`).
     - Integrates Quiver AI (`@quiverai/sdk` `arrow-2`) for content-addressed SVG generation and animated SVG rendering (`video/src/quiver/client.ts`, `video/src/quiver/cache.ts`).
  3. **Native Desktop UI (`ui/`):**
     - Rust 2024 edition `eframe`/`egui` 0.36.2 desktop application (`ui/Cargo.toml`, `ui/SPEC.md:7`) with `rodio` 0.22.2 playback, `soundcraft-audio-io`, and `soundcraft-dsp`. Designed for `<300 ms` cold start, `<80 MB` idle RSS, zero embedded webviews, direct reading of episode files, and subprocess orchestration of the `scriptcast*` CLI binaries (`ui/SPEC.md:7-28`, `plans/Native_UI/parity-brief.md`).

### 1.2 HyperFrames Deep Dive (`/Users/emmanuel/Dev/Tools/hyperframes`)

- **Purpose:** HyperFrames renders standard web compositions (HTML, CSS, SVG, Canvas, WebGL, GSAP) to frame-accurate video files (`README.md:33-45`). Rather than requiring React component wrappers around every animation primitive, HyperFrames treats HTML files with `data-*` attributes and a deterministic `window.__hf.seek(t)` callback as the source of truth (`docs/guides/hyperframes-vs-remotion.mdx:12-40`).
- **Architecture:**
  1. **Core & Parsers (`packages/core`, `packages/parsers`, `packages/lint`):**
     - `@hyperframes/core`: Defines the browser runtime (`window.__hf`, GSAP ticker synchronization in `packages/core/src/adapters/gsap.ts`), canvas text layout (`packages/core/src/text/pretext.ts`, `fitTextFontSize.ts`), spectral audio carving (`packages/core/src/audioCarve.ts`), ITU-R BS.1770 loudness leveling (`packages/core/src/audioLeveller.ts`), and LUT/color-grading primitives (`packages/core/src/colorGrading.ts`).
     - `@hyperframes/parsers`: Parses HTML compositions via `linkedom` (`packages/parsers/src/htmlParser.ts`) and parses/mutates GSAP timelines in `<script>` tags via `acorn` AST transforms (`packages/parsers/src/gsapParserAcorn.ts`, `gsapWriterAcorn.ts`).
     - `@hyperframes/lint`: Runs 11 static lint rule groups covering HTML contract attributes, GSAP syntax, CSS anti-patterns, shaders, captions, media file existence, and composition nesting (`packages/lint/src/hyperframeLinter.ts:36-51`).
  2. **Engine & Producer (`packages/engine`, `packages/producer`):**
     - `@hyperframes/engine`: Drives headless Chrome via Puppeteer/CDP (`packages/engine/src/services/frameCapture.ts`), pre-extracts `<video>` frames via FFmpeg (`videoFrameExtractor.ts`), mixes multi-track audio with automation and effects (`audioMixer.ts`), and streams raw RGBA/JPEG/PNG frames to FFmpeg via `image2pipe` with out-of-order worker sorting (`FrameReorderBuffer` in `streamingEncoder.ts:69-125`).
     - `@hyperframes/producer`: Orchestrates the 6-stage render pipeline (`packages/producer/src/services/renderOrchestrator.ts:24-112`) and supports frozen render plans (`freezePlan.ts`) for distributed chunk execution on AWS Lambda (`packages/aws-lambda`) and GCP Cloud Run (`packages/gcp-cloud-run`).
  3. **CLI, Studio, Registry & Skills (`packages/cli`, `packages/studio`, `registry/`, `skills/`):**
     - `@hyperframes/cli`: 40+ commands (`packages/cli/src/cli.ts`) for scaffolding, linting, browser QA checks, snapshots, timeline editing, local TTS/ASR, previewing, and rendering.
     - `@hyperframes/studio` & `@hyperframes/studio-server`: React 19 + Hono visual DOM/timeline editor (`packages/studio/AGENTS.md`).
     - `registry/`: 210 motion blocks (`registry/blocks/`), 239 components (`registry/components/`), and 8 starter templates (`registry/examples/`) indexed in `registry/registry.json`.
     - `skills/`: 7 structured Claude/Codex agent skills (`skills/hyperframes`, `hyperframes-cli`, `hyperframes-registry`, `media-use`, `figma`, `slideshow`, `remotion-to-hyperframes`).

---

## 2. Overlap Map

| Capability | ScriptCast (`/Users/emmanuel/Dev/Tools/scriptCast`) | HyperFrames (`/Users/emmanuel/Dev/Tools/hyperframes`) | Comparison Summary |
|---|---|---|---|
| **1. Script & Narration Parsing** | `scriptcast/parser.py` (Markdown speaker turns & timestamps)<br>`scriptcast/markup.py` (`[pause]`, `[beat]`, `[clip]`, `[emphasis]`, `[tone]`, `[pronounce]`)<br>`scriptcast/script_edit.py` (turn insertion/deletion)<br>`scriptcast/tools/retime_script.py` (writes measured timestamps back to Markdown)<br>`scriptcast/tools/shotspec.py` (maps `## G-NN` shots via turn anchors) | `skills/hyperframes/references/story.md` (agent narrative guidelines)<br>`skills/hyperframes/references/spec.md` (creative brief template)<br>`skills/media-use/references/audio.md` (`audio_request.json` schema for agent narration)<br>`skills/media-use/audio/scripts/audio.mjs` (processes `segments[]` text array) | ScriptCast has a dedicated Markdown dialogue DSL with inline prosody/clip tags and bidirectional script retiming. HyperFrames has no script parser in its engine; narration is authored as raw JSON segments or CLI strings by an LLM agent. |
| **2. TTS & Voice Management** | `scriptcast/engine.py` (`ENGINE_REGISTRY`: `mlx_kokoro`, `mlx_chatterbox`, `omnivoice`, `elevenlabs`, `phonon`, `archive`)<br>`scriptcast/omnivoice_worker.py` (isolated worker, `emphasis_reference_wav`)<br>`scriptcast/voices.py` (`voices.yaml` loader)<br>`scriptcast/pronunciation.py` (`pronunciations.yaml` + Misaki)<br>`scriptcast/tools/make_reference.py` (`scriptcast-reference`)<br>`scriptcast/tools/audition_voice.py` (`scriptcast-audition`)<br>`scriptcast/tools/calibrate_pronunciation.py` (`scriptcast-calibrate`) | `packages/cli/src/commands/tts.ts` (`hyperframes tts`)<br>`packages/cli/src/tts/synthesize.ts` (`kokoro-onnx` `Kokoro-82M-v1.0-ONNX` with 510-token midpoint split)<br>`packages/cli/src/tts/manager.ts` (model cache)<br>`skills/media-use/audio/scripts/heygen-tts.mjs` & `heygen-voice.mjs` (HeyGen Starfish cloud TTS & voice design)<br>`skills/media-use/audio/scripts/audio.mjs` (HeyGen, Gemini 2.5 Flash TTS, ElevenLabs) | ScriptCast is an Apple-Silicon-optimized multi-engine voice studio supporting per-speaker local voice cloning (Chatterbox, OmniVoice, Phonon), pronunciation dictionaries, and reference auditioning. HyperFrames bundles `kokoro-onnx` in its CLI and relies on cloud API scripts (HeyGen, Gemini, ElevenLabs) in `skills/media-use`. |
| **3. Composition & Timeline** | `scriptcast/manifest.py` (`episode_manifest.json` v2.0)<br>`scriptcast/render_state.py` (`render_state.json` SHA-256 turn state)<br>`scriptcast/tools/relayout_episode.py` (re-spaces turns without re-synthesis)<br>`scriptcast/tools/refit_render.py` (fits manifest to re-edited audio)<br>`scriptcast/tools/video_plan.py` (`.plan.json` resolver)<br>`video/src/Episode.tsx` (Remotion `<Sequence>` timeline)<br>`swift/EpisodeManifest.swift` | `packages/parsers/src/htmlParser.ts` (`data-start`, `data-duration`, `data-track-index`)<br>`packages/parsers/src/gsapParserAcorn.ts` & `gsapWriterAcorn.ts`<br>`packages/cli/src/commands/timeline.ts` (`hyperframes timeline`)<br>`packages/cli/src/timeline/a2Commands.ts` & `a2Mutations.ts` (`trim`, `slip`, `slide`, `roll`, `ripple-delete`, `split`, `magnetic-snap`)<br>`packages/sdk/src/session.ts` | ScriptCast's timeline is a speech-driven turn clock (`episode_manifest.json`) that exports to Ableton, FCPXML, and Remotion `.plan.json`. HyperFrames's timeline is an HTML DOM + GSAP AST graph with NLE-style CLI mutation commands (`ripple-delete`, `slip`, `roll`). |
| **4. Rendering (Audio & Video)** | **Audio:** `scriptcast/__main__.py`, `scriptcast/post_processor.py`, `scriptcast/tools/stitch_episode.py`, `scriptcast/tools/mix.py`, `scriptcast/tools/tapestop.py`, `scriptcast/tools/tapefx.py`<br>**DAW:** `scriptcast/als_generator.py`, `scriptcast/fcpxml_generator.py`, `scriptcast/logic_exporter.py`, `scriptcast/tools/craft_export.py`<br>**Video:** `scriptcast/tools/video.py` (spawns `npx remotion render` with `--chunks` parallel worker concat and `-vf framerate=30` clip staging) | `packages/producer/src/services/renderOrchestrator.ts` (6-stage pipeline)<br>`packages/producer/src/services/render/stages/*.ts` (`compileStage`, `probeStage`, `extractVideosStage`, `audioStage`, `captureStage`, `captureStreamingStage`, `encodeStage`, `assembleStage`, `freezePlan`)<br>`packages/engine/src/services/frameCapture.ts`, `drawElementService.ts`, `videoFrameExtractor.ts`, `streamingEncoder.ts`, `audioMixer.ts`, `motionBlur.ts`<br>`packages/aws-lambda/src/handler.ts`, `packages/gcp-cloud-run/` | ScriptCast renders audio natively in Python/MLX, exports DAW project files, and delegates video frame rasterization to Remotion (or Craft CLI). HyperFrames owns its entire Chrome CDP + FFmpeg `image2pipe` video rasterizer, `<video>` frame pre-extractor, Web Audio mixer, and cloud chunk runners. |
| **5. Templates, Cards & Registry** | `video/src/safeZone.ts` (Meta Reels 9:16 safe-zone constants)<br>`video/src/theme.ts` (palette, typography)<br>`video/src/cards/Frame.tsx` (shared card chrome & auto-scale loop)<br>`video/src/cards/TimelineCard.tsx`, `StatBarsCard.tsx`, `TitleCard.tsx`, `CompareCard.tsx`, `FreezeCallout.tsx`, `PartsCard.tsx`, `QrCode.tsx`<br>`video/src/quiver/client.ts` & `cache.ts` (Quiver AI SVG generator) | `registry/registry.json` (catalog index)<br>`registry/blocks/*` (210 self-contained HTML motion blocks)<br>`registry/components/*` (239 visual components: kinetic type, charts, counters, badges, backgrounds, transitions)<br>`registry/examples/*` (8 full starter compositions)<br>`packages/cli/src/commands/catalog.ts` & `add.ts`<br>`packages/shader-transitions/` (GLSL transition shaders) | ScriptCast has 6 purpose-built, citation-backed React data cards tailored to a single documentary aesthetic plus Quiver AI SVG generation. HyperFrames provides a searchable catalog of 449 HTML/GSAP/GLSL blocks and components (`hyperframes catalog`, `hyperframes add`). |
| **6. Agent & Skill Integration** | `scriptcast/tool_json.py` (standardized `--json` CLI envelopes for agent & UI consumption)<br>`docs/briefs/*.md` & `plans/Native_UI/parity-brief.md` (orchestration specs) | `skills/hyperframes/SKILL.md` (10-phase video director workflow)<br>`skills/hyperframes-cli/SKILL.md`, `skills/hyperframes-registry/SKILL.md`, `skills/media-use/SKILL.md`, `skills/remotion-to-hyperframes/SKILL.md`, `skills/figma/SKILL.md`, `skills/slideshow/SKILL.md`<br>`packages/cli/src/commands/skills.ts` (`hyperframes skills install`)<br>`packages/studio/src/webmcp/` (WebMCP studio bridge) | ScriptCast exposes deterministic `--json` CLI envelopes across its 29 tools. HyperFrames ships 7 installable agent skills with reference docs, lint rules, and helper scripts (`skills/media-use/scripts/*.mjs`). |
| **7. CLI Surface** | `pyproject.toml:45-74` (29 `scriptcast-*` entry points: `scriptcast`, `verify`, `verify-ensemble`, `repair`, `refit`, `relayout`, `stitch`, `retime`, `turn-index`, `outliers`, `stress`, `clip`, `clip-transcribe`, `reference`, `audition`, `calibrate`, `project`, `export`, `review`, `review-stats`, `word-timings`, `captions`, `shotspec`, `video`, `craft`, `tapestop`, `tapefx`, `mix`) | `packages/cli/src/cli.ts` & `packages/cli/src/commands/*.ts` (40+ subcommands under `hyperframes`: `init`, `lint`, `check`, `snapshot`, `compare`, `keyframes`, `timeline`, `catalog`, `add`, `preview`, `play`, `render`, `tts`, `transcribe`, `beats`, `normalize-audio`, `media-treatment`, `remove-background`, `capture`, `history`, `doctor`, `cloud`, `lambda`, `cloudrun`) | Both projects are CLI-first and emit machine-readable JSON (`--json`). ScriptCast's CLI is specialized around turn-based audio synthesis, verification, and episode assembly; HyperFrames's CLI is specialized around HTML composition manipulation, media prep, and browser rendering. |
| **8. Preview & UI** | `ui/Cargo.toml`, `ui/SPEC.md`<br>`ui/src/main.rs`, `ui/src/episode.rs`, `ui/src/audio.rs`, `ui/src/runner.rs` (native macOS Rust `eframe`/`egui` desktop app with 3-pane Turn Rail, Inspector/Diff, Bottom Transport, checkpoint/restore, and `<300 ms` startup) | `packages/studio/src/App.tsx` (React 19 browser visual editor: canvas stage, timeline tracks, property inspector, curve editor, caption generator)<br>`packages/studio-server/src/createStudioApi.ts` (Hono API server)<br>`packages/player/src/` (`<hyperframes-player>` web component)<br>`packages/cli/src/commands/preview.ts` & `play.ts` | ScriptCast builds a native, zero-webview Rust `egui` desktop app focused on fast turn auditioning, ASR diff inspection, and checkpointed CLI dispatch. HyperFrames builds a browser-based DOM/GSAP visual studio and embeddable web player. |
| **9. QA, Verification & Linting** | **Audio QA:** `scriptcast/tools/verify_render.py` (`scriptcast-verify`), `scriptcast/tools/verify_ensemble.py` (Phonon-2 + Phonon-1-Big + Whisper), `scriptcast/tools/stress_check.py` (wav2vec2 phoneme heteronym check), `scriptcast/tools/check_render_outliers.py`, `scriptcast/tools/repair_render.py`, `scriptcast/review.py`<br>**Video/Citation QA:** `scriptcast/tools/shotspec.py` (manuscript/quote/data verifier), `scriptcast/tools/cinetic_qa.py` (`scriptcast-video qa`: ffprobe, blackdetect, freezedetect, loudnorm, signalstats, contact sheet), `video/scripts/check-copy.tsx` (word-budget gate), `video/scripts/check-quiver.ts` | **Static Lint:** `packages/lint/src/hyperframeLinter.ts` (11 rule groups)<br>**Browser QA (`hyperframes check`):** `packages/cli/src/commands/check.ts`, `packages/cli/src/utils/checkPipeline.ts`, `packages/cli/src/commands/layout-audit.browser.js` (offscreen, overlap, clipping), `packages/cli/src/commands/contrast-audit.browser.js` & `contrast-bg.ts` (WCAG contrast over gradients/video), `packages/cli/src/commands/motion-signature.browser.js`, `packages/cli/src/utils/craftAudit.ts` (reading speed, hold floors, safe zones)<br>**Visual Diff:** `packages/cli/src/commands/snapshot.ts`, `compare.ts`, `skills/remotion-to-hyperframes/scripts/render_diff.sh`, `frame_strip.sh`, `lint_source.py` | ScriptCast has deep audio/speech QA (multi-ASR verification, phoneme stress checks, auto-repair) and academic citation verification, while its video QA (`cinetic_qa.py`) checks rendered MP4 streams with FFmpeg filters. HyperFrames has deep pre-render DOM/visual QA (headless browser layout bounding-box audits, WCAG contrast over composited backgrounds, reading-speed craft checks, and SSIM frame diffing). |

---

## 3. What ScriptCast Could Adopt from HyperFrames (Ranked)

### Rank 1: Headless Browser Layout, WCAG Contrast, and Reading-Speed Craft Audit (`check`)
- **What it is:** A pre-render headless browser inspection pass that seeks a composition to representative timestamps and runs injected DOM geometry and contrast audits inside the live page:
  - **Layout audit:** Detects offscreen text/elements (`OFFSCREEN`), unexpected sibling bounding-box overlap (`OVERLAP`), and parent overflow clipping (`CLIPPED`) via `getBoundingClientRect()` and computed styles (`packages/cli/src/commands/layout-audit.browser.js:1-215`).
  - **Contrast audit:** Computes effective foreground-to-background luminance contrast ratios across ancestors, CSS gradients, semi-transparent scrims, and sampled `<video>`/`<img>` canvases (`packages/cli/src/commands/contrast-audit.browser.js:1-190`, `packages/cli/src/commands/contrast-bg.ts`).
  - **Craft audit:** Enforces minimum text hold duration (`0.5s + 0.3s * wordCount`), maximum words per card, font-size hierarchy ratios, and platform safe zones (`packages/cli/src/utils/craftAudit.ts:1-260`, orchestrated by `packages/cli/src/utils/checkPipeline.ts:48-135`).
- **File paths in `hyperframes`:**
  - `packages/cli/src/commands/layout-audit.browser.js`
  - `packages/cli/src/commands/contrast-audit.browser.js`
  - `packages/cli/src/commands/contrast-bg.ts`
  - `packages/cli/src/utils/craftAudit.ts`
  - `packages/cli/src/utils/checkPipeline.ts`
- **Effort:** **M** (3–4 days to adapt as a Remotion still/frame audit in `video/scripts/check-layout.ts` and wire into `scriptcast/tools/cinetic_qa.py`).
- **Why it fits ScriptCast's workflow:** `docs/briefs/FINDINGS-cinetic-qa.md:36-40` documented concrete visual defects in rendered ScriptCast reels (such as captions pushed down or overlapping when card heights shift, and `cinetic_qa.py` only inspecting post-render FFmpeg signal statistics via `scriptcast/tools/cinetic_qa.py:120-260`). Currently, `video/scripts/check-copy.tsx` only validates JSON character/word budgets (`video/src/cards/types.ts:75`) without rendering the DOM. Injecting HyperFrames's `layout-audit.browser.js` and `contrast-audit.browser.js` checks against `USABLE_BOX` (`video/src/safeZone.ts:21-26`) and `CAPTION_TOP` (`video/src/safeZone.ts:32`) catches card overflow, caption collision, and low-contrast text over B-roll before running a full Remotion render.

### Rank 2: Canvas-Based Reflow-Free Text Fitting (`@chenglou/pretext` / `fitTextFontSize`)
- **What it is:** Pure-canvas text measurement and binary-search font/box fitting using `@chenglou/pretext` (`prepareWithSegments`, `layoutWithLines`, `walkLineRanges`) with an automatic `OffscreenCanvas` 2D context fallback (`packages/core/src/text/pretext.ts:1-145`, `packages/core/src/text/fitTextFontSize.ts:1-142`). It computes exact line breaks, line counts, and fitted font sizes in microseconds without triggering browser DOM layout or reflow.
- **File paths in `hyperframes`:**
  - `packages/core/src/text/pretext.ts`
  - `packages/core/src/text/fitTextFontSize.ts`
- **Effort:** **S** (1–2 days).
- **Why it fits ScriptCast's workflow:** In ScriptCast's `video/src/cards/Frame.tsx:67-86`, every card mount uses `delayRender("Frame-fit")` inside a `useLayoutEffect([scale])` state loop that measures `innerRef.current.scrollHeight > SAFE_HEIGHT` and decrements `scale` by `0.05` down to `0.65`, forcing up to 8–10 synchronous React re-renders and DOM reflows per card sequence. Replacing this loop with `fitTextFontSize` / `@chenglou/pretext` makes card scaling deterministic on the first render pass and removes `delayRender` stalls during parallel Remotion chunk rendering (`scriptcast/tools/video.py`).

### Rank 3: Transcript-Driven Video Cut List & PCM-Intermediate Concatenation (`transcript-cut.mjs`)
- **What it is:** A CLI workflow that takes a word-level transcript JSON and source video, builds a keep/remove cut list (filtering filler words, silences `>= minSilence`, or explicit word index ranges), checks keyframe spacing (`gopMaxSec`), cuts segments to intermediate `.mov` files with uncompressed `pcm_s16le` audio, optionally applies micro-crossfades across cuts, and encodes to AAC only once on the final concatenated stream (`skills/media-use/scripts/transcript-cut.mjs:1-210`, `skills/media-use/scripts/lib/cutlist.mjs`, `skills/media-use/scripts/lib/transcriptCutFade.mjs`).
- **File paths in `hyperframes`:**
  - `skills/media-use/scripts/transcript-cut.mjs`
  - `skills/media-use/scripts/lib/cutlist.mjs`
  - `skills/media-use/scripts/lib/transcriptCutFade.mjs`
  - `skills/media-use/scripts/lib/ffmpeg.mjs`
- **Effort:** **S** (1–2 days to port into `scriptcast/archive.py` and `scriptcast/tools/make_clip.py`).
- **Why it fits ScriptCast's workflow:** ScriptCast extracts archival clips (`scriptcast-clip` in `scriptcast/tools/make_clip.py` and `scriptcast-clip-transcribe` in `scriptcast/tools/transcribe_clip.py`) and video excerpts (`_cut_clip` in `scriptcast/tools/video_plan.py:42-95`). Cutting and concatenating AAC-encoded sub-clips directly introduces ~25–35 ms AAC encoder priming silence and click artifacts at splice boundaries (`skills/media-use/references/audio.md`). Adopting PCM-in-`.mov` intermediate segment cutting and word-index cutlists directly improves archival clip surgery in `scriptcast-clip` and `scriptcast-video`.

### Rank 4: Intelligibility-Weighted Spectral Voiceover Carve & Auto-Ducking (`audioCarve.ts` & `audio-duck.mjs`)
- **What it is:** Two complementary mixing techniques:
  1. **Spectral carve (`audioCarve.ts`, `carve.mjs`):** Analyzes voiceover RMS energy in 50 ms windows and applies a 9-band third-octave peaking EQ dip (`160 Hz` to `6000 Hz`, weighted by speech intelligibility weights peaking at `1.0` for `1.25–2.5 kHz`) to background music/beds only when the speaker is active, preserving music bass and air (`packages/core/src/audioCarve.ts:15-165`, `skills/media-use/audio/scripts/carve.mjs:1-140`).
  2. **Envelope auto-ducking (`audio-duck.mjs`, `duck.mjs`):** Generates smooth FFmpeg `volume` / `anequalizer` automation curves from speech segment timestamps with configurable `targetDb`, `attackMs` (150 ms), `releaseMs` (400 ms), and `mergeGapMs` (300 ms) (`skills/media-use/scripts/audio-duck.mjs`, `skills/media-use/scripts/lib/duck.mjs`).
- **File paths in `hyperframes`:**
  - `packages/core/src/audioCarve.ts`
  - `skills/media-use/audio/scripts/carve.mjs`
  - `skills/media-use/scripts/audio-duck.mjs`
  - `skills/media-use/scripts/lib/duck.mjs`
- **Effort:** **M** (3–4 days to integrate into `scriptcast/tools/mix.py` or Phase N's `soundcraft-dsp` `scriptcast-fx` pipeline).
- **Why it fits ScriptCast's workflow:** `scriptcast-mix` (`scriptcast/tools/mix.py:1-195`) currently supports static per-turn `turn_gains` and one-shot word-anchored sound effect overlays (`mix.yaml`), with no continuous background bed ducking or spectral carving against `episode_manifest.json` turn windows. Adding spectral carving and envelope ducking directly complements `scriptcast-mix` and aligns with the planned Phase N `soundcraft-dsp` integration (`plans/Native_UI/parity-brief.md:96-99`).

### Rank 5: Parakeet-TDT (`parakeet-mlx`) Fast Local ASR Engine
- **What it is:** Integration of `mlx-community/parakeet-tdt-0.6b-v3` via the `parakeet-mlx` CLI/library for local Apple Silicon transcription with word-level aligned timestamps (`packages/cli/src/whisper/parakeet.ts:1-155`). Benchmarks documented in `skills/media-use/references/audio.md` show `parakeet-mlx` running 5–10× faster than Whisper on Apple Silicon with lower word error rate (6.05% vs 7.44%) and reduced repetition hallucinations on music-backed archival clips.
- **File paths in `hyperframes`:**
  - `packages/cli/src/whisper/parakeet.ts`
  - `skills/media-use/references/audio.md`
- **Effort:** **S** (1 day to add a `ParakeetEngine` class to `scriptcast/asr/engines.py`).
- **Why it fits ScriptCast's workflow:** ScriptCast already abstracts ASR engines behind `ASREngine` in `scriptcast/asr/engines.py:30-210` (`WhisperMLXEngine`, `PhononEngine` for `kyutai/stt-1b-en_fr-mlx` and `gradium/ugh7p484863638`). Adding `parakeet-tdt-0.6b-v3` provides a fourth independent model family for `scriptcast-verify-ensemble` (`scriptcast/tools/verify_ensemble.py`) and faster word-timestamp extraction for `scriptcast-captions` (`scriptcast/tools/captions.py`) and `scriptcast-clip-transcribe` (`scriptcast/tools/transcribe_clip.py`).

### Rank 6: Single-Pass FFmpeg SSIM Regression Diff & Side-by-Side Frame Strips
- **What it is:** Lightweight shell and TypeScript utilities that compare two video renders (`render_diff.sh`, `compare.ts`) using FFmpeg `ssim` filters to flag the exact timestamps where SSIM drops below `0.95`, and `frame_strip.sh` which extracts `N` evenly-spaced frames in a single FFmpeg pass and stacks reference vs. candidate frames into a single comparison PNG (`skills/remotion-to-hyperframes/scripts/render_diff.sh:1-110`, `skills/remotion-to-hyperframes/scripts/frame_strip.sh:1-95`, `packages/cli/src/commands/compare.ts:1-175`).
- **File paths in `hyperframes`:**
  - `skills/remotion-to-hyperframes/scripts/render_diff.sh`
  - `skills/remotion-to-hyperframes/scripts/frame_strip.sh`
  - `packages/cli/src/commands/compare.ts`
- **Effort:** **S** (1 day to add `scriptcast-video diff` or extend `scriptcast/tools/cinetic_qa.py`).
- **Why it fits ScriptCast's workflow:** `cinetic_qa.py` already generates a single-video contact sheet (`scriptcast/tools/cinetic_qa.py:212-245`), and `docs/briefs/FINDINGS-cinetic-qa.md` noted the need to verify visual regressions after fps conformation (`24 -> 30 fps`) or card layout edits. Adding A/B SSIM diffing and stacked reference/candidate strips gives `scriptcast-video qa` a deterministic visual regression gate.

### Rank 7: Kokoro Token-Overflow Midpoint Split Fallback
- **What it is:** A defensive synthesis wrapper that catches Kokoro's 510-token context window overflow (`index_select(): Index is supposed to be a vector` / sequence length error), splits long text at the nearest sentence or clause boundary near the midpoint (`[20%, 80%]`), synthesizes both halves recursively, and concatenates the float32 PCM buffers (`packages/cli/src/tts/synthesize.ts:44-75`).
- **File paths in `hyperframes`:**
  - `packages/cli/src/tts/synthesize.ts`
- **Effort:** **S** (<1 day).
- **Why it fits ScriptCast's workflow:** ScriptCast's parity brief explicitly tracks long-turn chunking and breath pauses under Phase J9 (`plans/Native_UI/parity-brief.md:49`). While ScriptCast already chunks turns in `engine.py`, adding a recursive midpoint fallback on token-limit exceptions prevents runtime synthesis crashes on unpunctuated or dense technical sentences.

---

## 4. What ScriptCast Has That HyperFrames Lacks

1. **Closed-Loop TTS Verification, Multi-Model Ensemble Adjudication, and Automated Repair (`scriptcast-verify`, `scriptcast-verify-ensemble`, `scriptcast-repair`):**
   - ScriptCast transcribes every synthesized turn with `mlx-whisper` (`scriptcast/tools/verify_render.py`), compares normalized tokens against expected script text (expanding currency, years, decades, symbols, and custom pronunciations via `scriptcast/pronunciation.py`), adjudicates ambiguous turns across 3 independent ASR models (`Phonon-2`, `Phonon-1-Big`, `mlx-whisper` in `scriptcast/asr/engines.py` and `scriptcast/tools/verify_ensemble.py`), supports interactive human review (`scriptcast/review.py`), and automatically re-synthesizes failing turns (`scriptcast/tools/repair_render.py`).
   - HyperFrames has no TTS verification loop; `hyperframes tts` (`packages/cli/src/commands/tts.ts`) writes raw synthesis output directly to disk without ASR round-trip checking.

2. **Phoneme-Level Heteronym Stress Verification & Pronunciation Calibration (`scriptcast-stress`, `scriptcast-calibrate`):**
   - ScriptCast detects English heteronyms (`record`, `present`, `conduct`, `project`, `produce`, etc.) in the script, runs `facebook/wav2vec2-lv-60-espeak-cv-ft` phoneme recognition on the rendered audio (`scriptcast/tools/stress_check.py:1-215`), scores the emitted IPA against Misaki's expected vs. unexpected part-of-speech stress patterns, and provides `scriptcast-calibrate` (`scriptcast/tools/calibrate_pronunciation.py`) to search alternative phonetic respellings automatically.
   - HyperFrames has no phoneme recognition, heteronym detection, or pronunciation calibration tooling.

3. **Multi-Speaker Conversational Turn Engine & Local Voice Cloning Workflow (`scriptcast/engine.py`, `scriptcast-reference`, `scriptcast-audition`):**
   - ScriptCast routes each character in a multi-speaker script to a distinct engine and voice profile (`voices.yaml` → `mlx_kokoro`, `mlx_chatterbox`, `omnivoice`, `phonon`, `elevenlabs`, `archive` in `scriptcast/engine.py:667-675`), supports expressive `[emphasis]` reference switching in OmniVoice (`scriptcast/omnivoice_worker.py`), extracts clean reference WAVs from archival recordings (`scriptcast/tools/make_reference.py`), and benchmarks clone stability across test prompts (`scriptcast/tools/audition_voice.py`).
   - HyperFrames's built-in CLI TTS (`packages/cli/src/tts/synthesize.ts`) only runs single-voice `kokoro-onnx` without voice cloning or multi-speaker turn parsing.

4. **Content-Addressed Incremental Turn State & Zero-Synthesis Relayout (`render_state.py`, `scriptcast-relayout`, `scriptcast-refit`, `scriptcast-retime`):**
   - ScriptCast tracks SHA-256 fingerprints (`text_hash`, `voice_hash`, `speech_text_hash`) per `turn_id` in `render_state.json` (`scriptcast/render_state.py`), enabling `--detect-changes` (regenerate only edited turns), `--precision-insert` (splice newly added script turns into an existing render without touching other turns), `scriptcast-relayout` (recompute all episode timestamps when pause durations change without running TTS), `scriptcast-refit` (reconcile manifest timings after manual WAV edits), and `scriptcast-retime` (write true cumulative durations back into the Markdown script headers).
   - HyperFrames has no turn-level incremental audio synthesis state or Markdown timestamp back-propagation.

5. **Professional DAW & NLE Timeline Exports (`als_generator.py`, `fcpxml_generator.py`, `logic_exporter.py`, `craft_export.py`):**
   - ScriptCast generates native Ableton Live `.als` project sets with per-speaker tracks and clip envelopes (`scriptcast/als_generator.py`), Final Cut Pro `.fcpxml` v1.10 multi-lane dialogue and picture timelines (`scriptcast/fcpxml_generator.py`, `scriptcast/tools/craft_export.py`), and Logic Pro sessions via AppleScript (`scriptcast/logic_exporter.py`).
   - HyperFrames only renders flat media containers (`mp4`, `mov`, `webm`, `gif`, `png-sequence`, `hls` in `docs/guides/rendering.mdx:50-57`) and has no DAW or NLE project export.

6. **Academic Manuscript & Primary-Source Provenance Verification (`scriptcast-shotspec`):**
   - `scriptcast/tools/shotspec.py:113-297` parses shot lists (`## G-NN`) and validates every `[book]`, `[data]`, and `[source]` claim against a LaTeX manuscript (`Paper/The_Original_Power.tex`), checking `\label{...}` existence, fuzzy-matching quoted phrases against stripped TeX prose (`resolve_quotes`), verifying local CSV/JSON data files under `Paper/data/`, and checking external URLs.
   - HyperFrames has no citation, manuscript, or factual provenance verification system.

7. **Script-Aligned Caption Generation (`scriptcast-captions`):**
   - `scriptcast/tools/captions.py:40-79` aligns ASR word timestamps to the authoritative script tokens using `difflib.SequenceMatcher(autojunk=False)` and linearly interpolates any unmatched words within each turn's `[start_ms, end_ms]` window, guaranteeing that on-screen captions display exact script spelling, capitalization, and punctuation rather than raw ASR hallucinations.
   - HyperFrames's caption workflow (`skills/hyperframes/references/captions.md`, `skills/media-use/audio/references/tts-to-captions.md`) renders word timestamps directly from `hyperframes transcribe` output without aligning against an authoritative script.

8. **Native Zero-Webview Rust Desktop Application (`ui/`) & Physical Tape-Machine DSP (`tapestop.py`, `tapefx.py`):**
   - ScriptCast provides a native Rust `eframe`/`egui` desktop app (`ui/src/main.rs`) with `<300 ms` startup, turn-rail diffing, and snapshot checkpoint/restore (`ui/SPEC.md`), plus coupled audio-video tape-stop and rewind-replay DSP transitions (`scriptcast/tools/tapestop.py`, `scriptcast/tools/tapefx.py`).

---

## 5. Strategic Decision

### Recommendation: **Option (C) — Cherry-Pick Ideas and Self-Contained Algorithms/Scripts Only**

ScriptCast should **not** be integrated into HyperFrames as a plugin/frontend **(reject Option A)**, and HyperFrames should **not** be adopted as ScriptCast's video render backend or npm dependency **(reject Option B)**. Instead, ScriptCast should **cherry-pick specific, high-leverage algorithms and QA patterns from HyperFrames (Option C)** while keeping its existing Python/Rust core and lightweight Remotion `video/` workspace.

---

### Evaluation of the Three Options

#### Why Option (A) Fails (Integrate ScriptCast into HyperFrames as a plugin/skill)
1. **License incompatibility for upstream/combined Apache-2.0 distribution:**
   - ScriptCast is licensed under `GPL-3.0-or-later` (`/Users/emmanuel/Dev/Tools/scriptCast/LICENSE`, `pyproject.toml:10`).
   - HyperFrames is licensed under `Apache-2.0` (`/Users/emmanuel/Dev/Tools/hyperframes/LICENSE`).
   - Patent and copyleft compatibility between Apache-2.0 and GPLv3 is **one-way**: Apache-2.0 code may be included inside a GPLv3 work, whereas GPLv3 code cannot be distributed inside an Apache-2.0 package without either relicensing ScriptCast or infecting the combined HyperFrames distribution with GPLv3 copyleft obligations.
2. **Inverted domain model:**
   - ScriptCast's primary artifact is the spoken episode (`episode_manifest.json`, 48 kHz turn WAVs, `.als`/`.fcpxml` DAW projects, and multi-engine local Apple Silicon TTS/ASR in Python + Rust `egui` UI per `README.md:24-36` and `ui/SPEC.md:7`). Video (`video/`) is one downstream consumer of the episode manifest (`scriptcast/tools/video.py`).
   - Turning ScriptCast into a plugin or skill inside HyperFrames's Node/Bun browser-studio ecosystem would subordinate the audio/DAW production pipeline and native Rust UI to an HTML/GSAP video tool that has no concept of dialogue turns, phoneme stress checks, or DAW session files.

#### Why Option (B) Fails (Integrate HyperFrames into ScriptCast as a render backend/dependency)
1. **Extreme upstream churn and pre-1.0 instability:**
   - Inspection of `/Users/emmanuel/Dev/Tools/hyperframes/docs/changelog.mdx` shows **362 releases between `v0.6.52` (2026-05-27) and `v0.8.205` (2026-10-26)** — an average of **2.38 releases per day** over 152 days, including 5 releases on 2026-10-26 alone (`0.8.201` through `0.8.205`, up to PR `#6449`).
   - HyperFrames is pre-1.0 (`0.8.x`), couples 15 internal workspace packages (`@hyperframes/core`, `@hyperframes/parsers`, `@hyperframes/engine`, `@hyperframes/producer`, `@hyperframes/lint`, etc.) via exact version locks (`packages/cli/package.json:58-65`), and requires a Bun 1.4.4 monorepo build pipeline (`package.json:18-45`, `AGENTS.md:37-44`). Taking `@hyperframes/producer` or `@hyperframes/cli` as a runtime dependency in `scriptCast/video/` exposes ScriptCast to daily upstream breaking changes.
2. **Direct incompatibility with ScriptCast's existing `video/` React components:**
   - HyperFrames's own migration linter (`skills/remotion-to-hyperframes/scripts/lint_source.py:21-98`) flags React `useState` (`r2hf/use-state`, error) and non-empty `useEffect`/`useLayoutEffect` dependency arrays (`r2hf/use-effect-deps`, error) as **hard blockers** because HyperFrames compositions are static HTML/GSAP trees driven by `window.__hf.seek(t)` rather than frame-reactive React components (`skills/remotion-to-hyperframes/references/escape-hatch.md`).
   - ScriptCast's `video/src/cards/Frame.tsx` relies on `useState` (`scale`, `markup` at lines 39, 40, 67, 69), `useLayoutEffect([svgId])` (line 43), `useLayoutEffect([scale])` (line 75), and `delayRender`/`continueRender` (lines 38, 41, 66). Migrating `video/` to HyperFrames would require rewriting `Episode.tsx` (`video/src/Episode.tsx`), `Frame.tsx`, and all 6 data cards (`TimelineCard`, `StatBarsCard`, `TitleCard`, `CompareCard`, `FreezeCallout`, `PartsCard`) from typed React TSX into imperative HTML/GSAP templates.
3. **Redundant media-staging work:**
   - HyperFrames's main rendering advantage over naive Remotion setups is Stage 2 `<video>` frame pre-extraction (`packages/engine/src/services/videoFrameExtractor.ts`), which avoids Chromium `<video>` seek jitter. However, ScriptCast's `scriptcast/tools/video_plan.py:42-95` (`_cut_clip`) already pre-cuts and conforms archival video excerpts to 30 fps H.264 (`-vf framerate=30`) before Remotion renders, and `scriptcast/tools/video.py` already parallelizes rendering across `--chunks` workers (`docs/briefs/FINDINGS-cinetic-qa.md:18-34`).

#### Why Option (C) Succeeds (Cherry-pick ideas and self-contained modules)
1. **One-way license compatibility works in ScriptCast's favor:**
   - Because Apache-2.0 is compatible with `GPL-3.0-or-later`, ScriptCast (`GPL-3.0-or-later`) can legally copy, adapt, or vendor specific Apache-2.0 files or algorithms from HyperFrames (such as `layout-audit.browser.js`, `contrast-audit.browser.js`, `pretext.ts`/`fitTextFontSize.ts`, `audioCarve.ts`, `transcript-cut.mjs`, and `render_diff.sh`) with proper Apache-2.0 attribution notices, without pulling in HyperFrames's 15-package Bun monorepo.
2. **Zero upstream churn exposure:**
   - Vendoring or porting ~500–800 lines of self-contained layout-audit, canvas text measurement, and DSP curve-generation logic insulates ScriptCast completely from HyperFrames's ~2.4-releases/day churn.
3. **What ScriptCast's existing `video/` pipeline becomes:**
   - `video/` remains ScriptCast's focused, 4-dependency Remotion 4.0.354 + React 19 workspace (`video/package.json`).
   - It keeps its typed JSON card schemas (`video/src/cards/types.ts`), Meta Reels `USABLE_BOX` geometry (`video/src/safeZone.ts`), Quiver SVG integration (`video/src/quiver/`), and `scriptcast-video` Python orchestrator (`scriptcast/tools/video.py`), while gaining HyperFrames's two strongest video engineering techniques: **reflow-free canvas text fitting** inside `Frame.tsx` and **pre-render headless DOM/contrast/SSIM QA gates** inside `cinetic_qa.py`.
   - Meanwhile, the Craft interchange path (`scriptcast/tools/craft_export.py`) remains parked as an optional FCPXML export until `filmcraft-cli` fixes its upstream `NaN` motion-vector bug (`docs/briefs/FINDINGS-craft-video.md:142-163`).

---

## 6. Concrete First Step (revised 2026-10-26 by owner direction)

Owner direction: every port lands in Rust, not Python or TypeScript. Parakeet (Rank 5) is dropped:
ScriptCast already runs Phonon-2 windowed passes, a Phonon-1-Big coverage check, and mlx-whisper
(`scriptcast/asr/engines.py:3-36`). Correction to Rank 5 above: the Phonon models are
`FermionResearch/Phonon-2` and `FermionResearch/Phonon-1-Big`, not kyutai/gradium. A Parakeet engine
stays an optional, unscheduled add-on.

Home: a new workspace crate `scriptcast-media` beside `ui/` (which already depends on
`soundcraft-dsp` and `soundcraft-audio-io`), exposed as a CLI and linked by `scriptcast-ui`.
Port from HyperFrames' Apache-2.0 sources with attribution in a NOTICE file.

| Order | Item | Rust shape | Notes |
|---|---|---|---|
| 1 | Speech-envelope ducking + 9-band spectral carve (Rank 4) | **done:** `media/` crate `scriptcast-media`, `scriptcast-media carve` | manifest turn windows, soundcraft-dsp peaking biquads, 150/400 ms ramps. Default bands are 1, 1.6 and 2.5 kHz (2 kHz is not a candidate centre). Not yet called from `scriptcast-ui`. |
| 2 | Transcript cut list + PCM-intermediate concat (Rank 3) | cut-list logic in Rust, ffmpeg as subprocess | replaces AAC-splice clicks in `scriptcast-clip` |
| 3 | SSIM diff + frame strips (Rank 6) | ffmpeg subprocess + `image` crate stacking | feeds `scriptcast-video qa` |
| 4 | Card text fitting (Rank 2) | precompute fitted scale per card in Rust (`cosmic-text` or `swash` with the card fonts), write it into the card JSON | Remotion then renders once; removes the `Frame.tsx` shrink loop |
| 5 | Layout + contrast audit (Rank 1) | Rust drives headless Chrome (`chromiumoxide`); the injected audit stays JS because it must run in the page | only part that cannot be all-Rust |
| - | Kokoro midpoint split (Rank 7) | port when/if TTS moves to Rust | skip for now |

Week 1: items 1-2. Week 2: items 3-4. Item 5 after, once fitted scales make overflow rare.
Each item's acceptance check: a Rust unit test on fixture audio/JSON plus an end-to-end run on one
existing episode, compared against the current Python output.
