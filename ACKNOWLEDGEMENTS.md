# Acknowledgements

scriptCast is built on other people's models, libraries, and research. This page lists what it runs, links, or ports. Each project's own license governs its code and weights. Where code was ported, the port's NOTICE says so.

## Ported code

- **[HyperFrames](https://github.com/heygen-com/hyperframes)** (HeyGen, Apache-2.0). `scriptcast-media` re-implements in Rust its dialogue ducking (`duck.mjs`, `audio-duck.mjs`) and spectral carve (`audioCarve.ts`, `carve.mjs`). See [media/NOTICE](media/NOTICE).

## Speech synthesis models

| Engine | Model | Through |
|---|---|---|
| Kokoro | Kokoro-82M (hexgrad) | [mlx-audio](https://github.com/Blaizzy/mlx-audio), [misaki](https://github.com/hexgrad/misaki) G2P |
| Chatterbox | `mlx-community/chatterbox-multilingual-v3` (Resemble AI) | mlx-audio |
| OmniVoice | `k2-fsa/OmniVoice` (k2-fsa) | local worker |
| Phonon | `gradium/phonon-31m-en` (Gradium) | `ptts` |
| Tone | `mlx-community/Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16` (Qwen) | local worker |
| ElevenLabs | ElevenLabs API (optional, cloud) | `elevenlabs` SDK |

## Recognition and verification models

- **Whisper** (OpenAI), MLX ports from mlx-community (`whisper-tiny.en`, `small.en`, `medium.en`, `small`, `large-v3-turbo`), through [mlx-whisper](https://github.com/ml-explore/mlx-examples).
- **Phonon-2** (Gradium), the second recognizer in the ensemble, run in its own `fermion` environment.
- **`facebook/wav2vec2-lv-60-espeak-cv-ft`** (Meta), the phoneme recognizer behind `scriptcast-stress`, with eSpeak phonemes.
- **Tesseract OCR**, which reads rendered video stills back.
- **SigLIP** (`siglip-so400m-patch14-384`, Google) and **MiniCPM-V 4.7** (OpenBMB, 4-bit MLX) for still review.
- **Laya** (`aac6fef/laya-mlx`), in the Laya Router work.

## Platform

- **[MLX](https://github.com/ml-explore/mlx)** (Apple), the runtime for every local model on Apple Silicon.
- **[FFmpeg](https://ffmpeg.org)** for media conversion, cutting, and stitching.

## Python libraries

numpy, scipy, soundfile, PyYAML, ruamel.yaml, Pillow, and, for the optional `cinetic` checks, OpenCV, librosa, and pyloudnorm.

## Rust crates

- **[egui / eframe](https://github.com/emilk/egui)** and egui_extras: the native app's UI.
- **[soundcraft](https://github.com/storytold/soundcraft)** (`soundcraft-audio-io`, `soundcraft-dsp`): audio I/O and DSP for the app, `scriptcast-fx`, and `scriptcast-media`.
- rodio, hound, rfd, image, serde / serde_json, toml, regex, sha2, chrono, and libc.

## Video and web

- **[Remotion](https://www.remotion.dev)** and React: programmatic video scenes.
- Vite and TypeScript.
- d3-geo, d3-interpolate, topojson-client, and world-atlas for maps. KaTeX for math. qrcode-generator.
- **[Quiver AI](https://quiver.ai)** SDK for SVG artwork (`scriptcast-video svg`).
- The Craft CLIs (`figcraft-cli`, `filmcraft-cli`, `effectcraft-cli`, `vectorcraft-cli`, and `designcraft-cli`) produce the plates that `scriptcast-craft` places.

## Exports

These are file-format targets, not code dependencies: Ableton Live (`.als`), Logic Pro, and Final Cut Pro (FCPXML 1.10).

## Fonts

- **OpenDyslexic** and OpenDyslexic Mono (SIL OFL), bundled in the app.
- **IBM Plex Sans** and **IBM Plex Mono** (SIL OFL), through Fontsource, in the video and site.
