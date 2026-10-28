# K4 tone engine

Pinned 2026-10-28 from the weights on this machine and from `mlx_audio` in the scriptCast virtualenv. Do not download another checkpoint. Do not vendor Voicebox.

## Weights

CustomVoice is not on this machine. The checkpoint that is here is VoiceDesign. Its config says `tts_model_type` is `voice_design` and `spk_id` is empty, so there is no speaker name to pass.

| | |
|---|---|
| Hub id | `mlx-community/Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16` |
| Revision | `7d3824abff87e49756bb0f83fb5411de75d160c4` |
| Snapshot | `/Users/emmanuel/.cache/huggingface/hub/models--mlx-community--Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16/snapshots/7d3824abff87e49756bb0f83fb5411de75d160c4` |
| Talker weights | `model.safetensors`, 3833402589 bytes |
| Tokenizer weights | `speech_tokenizer/model.safetensors`, 682293092 bytes |

Load that snapshot directory with `mlx_audio.tts.utils.load`. If the directory is missing, exit 1 and name the path. Do not call the Hub.

## What the model accepts

`Model.generate` routes `voice_design` to `generate_voice_design`. That call requires a non-empty `instruct`. The library's own error is that VoiceDesign needs an instruct string. `[tone:…]` is that string. `speed` is not applied by this model (`generate` documents it as not supported). Do not add a speed flag.

Sampling stays in `generation_config.json` next to the weights: temperature 0.9, top_k 50, top_p 1.0, repetition_penalty 1.05, max_new_tokens 8192. Do not pass those arguments, and do not add CLI flags for them. Output is 24000 Hz. If the result reports another rate, fail and write nothing.

`--lang` defaults to `english`. The codec id for english in this config is 2050. Allowed values are the keys of `codec_language_id` plus `auto`.

## Binary

Crate `tone/`, binary `scriptcast-tone`, same argv style as `chunk/src/main.rs`. The Rust process does not implement the network. It starts `scriptcast/tone_worker.py`.

```
scriptcast-tone speak --text TEXT --instruct TEXT --out OUT.wav [--lang english]
scriptcast-tone worker
```

`SCRIPTCAST_TONE_PYTHON` is the interpreter. The Python engine sets it to `sys.executable`. The worker loads the snapshot above unless `SCRIPTCAST_TONE_MODEL` names another local directory.

`speak` is one worker request, then exit. Missing `--text`, `--instruct`, or `--out`, or a blank instruct, exits 1 and writes no file. That check happens before Python starts.

`worker` prints one JSON line `{"ready": true}` after the model is loaded, then reads one JSON object per stdin line:

```
{"text": "...", "instruct": "...", "out": "out.wav", "lang": "english"}
```

and answers:

```
{"ok": true, "out": "out.wav", "samples": 123, "sample_rate": 24000}
{"ok": false, "error": "..."}
```

A failed request leaves the worker up and does not leave `out` behind.

## Renderer

Engine id `tone`. `VoiceConfig.instruct` is the speaker's voice description. `load_voices` requires a non-empty `instruct` when `engine` is `tone`, and lists `tone` among the supported engines. `character_profile` is not a fallback.

`tokenize_markup` keeps a tone annotation as `tag == "tone"` and stores the trimmed value on `MarkupChunk.text`. `[tone:soft] hello` stays an annotation followed by speech. The existing assertion `tag == "tone"` stays true.

For each speech job, the instruct is the text of the latest `tone` annotation before that speech chunk. If that annotation has no text, or there is none, use `voice.instruct`. If that is also empty, raise, and name the speaker. Pass it through `speak(..., instruct=...)` into `synthesize_chunk`. Other engines ignore the argument.

`ToneEngine` follows `OmniVoiceEngine`: one worker for the render, `sample_rate` 24000, `SCRIPTCAST_TONE` overrides the binary the way `SCRIPTCAST_CHUNK` does. A missing binary fails only when a `tone` speaker is rendered.

## Tests

- CLI: missing instruct exits 1 and writes no wav, without starting Python.
- Markup: `[tone:soft] hello` still has `tag == "tone"`, and the value `soft` is on that chunk.
- Voices: `engine: tone` without `instruct` is an error. A kokoro speaker still loads.
- Render: a fake `SCRIPTCAST_TONE` that writes a wav is enough. Do not call the real model from pytest.
- One smoke, once: `scriptcast-tone speak --text "Hello." --instruct "A calm adult voice, speaking quietly." --out /tmp/k4-smoke.wav --lang english`. The wav is 24000 Hz and not empty. If load fails, stop and record the stderr in the findings file. Do not try another checkpoint.

## Out of scope

No L2 voices form. No second reference clip. No `[laugh]` or `[sigh]`. No `max_chars` on this engine. No Stitch or Mix changes. No edits under `/Users/emmanuel/Documents/Theory/TheOriginalPower`. No README rewrite. One changelog line under `[0.2.0] Unreleased` / Added.

Write `plans/Native_UI/evidence/k4-findings.md` before the smoke test.
