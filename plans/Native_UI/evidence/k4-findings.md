# K4 tone findings

## Contract status before smoke

- The pinned VoiceDesign snapshot and both named weight files are present at the contract path.
- No contract substitution was needed. The worker loads that local snapshot with `mlx_audio.tts.utils.load` and never passes a Hub id.
- The Rust CLI, persistent worker protocol, renderer integration, tone markup, voice validation, and changelog entry are implemented.
- Focused Python tests pass: 47 passed.
- The full suite passes in the supplied scriptCast virtualenv: 422 passed, 13 existing warnings.
- Rust tests pass offline: 2 passed.
- The system Python cannot collect the full suite because it lacks `ruamel.yaml`; no dependency was installed because the supplied virtualenv runs the suite successfully.

## Smoke

Codex ran the smoke once inside its sandbox. Model load failed because that sandbox has no Metal device. Stderr was:

```text
scriptcast-tone worker: [metal::load_device] No Metal device available. This typically occurs in headless, sandboxed, or virtualized macOS sessions where the GPU is not accessible.
scriptcast-tone: worker sent an invalid greeting: ""
```

No wav was produced, and no other checkpoint was tried.

The same binary was then run once on this machine, outside that sandbox, with the pinned snapshot:

```text
SCRIPTCAST_TONE_PYTHON=/Users/emmanuel/Dev/Tools/scriptCast/.venv/bin/python tone/target/debug/scriptcast-tone speak --text 'Hello.' --instruct 'A calm adult voice, speaking quietly.' --out /tmp/k4-smoke.wav --lang english
```

Exit 0. The wav is 24000 Hz, 19200 samples, peak 0.241851806640625. A transformers warning printed that a `qwen3_tts` checkpoint was instantiated as type ``; generation still completed.
