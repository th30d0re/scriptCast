# Phase H1 + H2 (Python): ASR ensemble in scriptCast, and clip transcripts

Working directory: a git worktree of scriptCast on branch `ui/phase-h`, from `dev`. **Python only in this run;** the UI parts of H2 come later.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **`/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** That includes `videolab/` (read it, copy from it, never edit it), the `.venv*` folders, and episode outputs.
  - Live runs use a `cp -cR` copy of EP03's output folder under `/tmp/scriptcast-h/`.
  - You may *run* interpreters from `.venv-fermion` and `.venv-voice`.
- Python changes are allowed (the operator approved the ensemble port on 2026-10-04) only in:
  - new `scriptcast/asr/` (`__init__.py`, `merge.py`, `engines.py`);
  - new `scriptcast/tools/clip_transcribe.py`;
  - new tests under `tests/`;
  - `pyproject.toml` (console script).
- Do not change existing scriptCast modules.
- Put scratch files in `/tmp/scriptcast-h/`. No synthesis.

## Read

1. `plans/Native_UI/plan.md`, the "Phase H" section: H1, H2 and the background.
2. `TheOriginalPower/videolab/src/videolab/asr_merge.py` (218 lines) and `videolab/tests/test_asr_merge.py`.
3. `TheOriginalPower/videolab/src/videolab/asr.py`: `_phonon_words` (~351), `_big_text` (~384), `_run_fermion_server` (~244), `_post_audio`, `_wait_for_health`, `_fermion_executable`, the Whisper pass (`_whisper_pass` ~186), and the ensemble entry (~489–540).
4. `scriptcast/tools/word_timings.py` (Phase E): how scriptCast reads the manifest, maps segments to turns with offsets, aligns to script words, and writes JSON atomically.
5. `scriptcast/parser.py`: clip turns are those whose text contains `[clip:<id>]`.

## Build

1. **`scriptcast/asr/merge.py`:** a faithful copy of `asr_merge.py`. Keep the behaviour identical, and keep its docstrings and comments. Port `test_asr_merge.py` to `tests/test_asr_merge.py`; it must pass unchanged in substance.
2. **`scriptcast/asr/engines.py`:** the engine adapters ported from `asr.py`:
   - Phonon-2 words with 15 s windows at offsets 0, 5 and 10 s;
   - Phonon-1-Big text;
   - Whisper (mlx);
   - `ensemble(audio_path) -> {words: [{text, start, end, engine}], segments, coverage_warnings, whisper_disagreement_pct, engines_used}`, following videolab's order: primary, gap fill, Whisper backfill (≥ 8 words), dedup, Big check (warn only), and a Whisper-only fallback.
   - Make the engine calls injectable, as videolab does, so tests need no models.
   - Port the relevant `test_asr.py` cases.
3. **`scriptcast/tools/clip_transcribe.py`**, registered as `scriptcast-clip-transcribe <episode_dir> [--script PATH] [--turns SPEC] [--out PATH]`:
   - for each clip turn, run `ensemble` on its processed WAV segments, offset by segment position as `word_timings.py` does;
   - write `<episode_dir>/clip_transcripts.json` atomically:
     `{manifest_sha256, turns: {"<turn_id>": {clip_id, words: [{text, start_ms, end_ms, engine}], coverage_warnings, whisper_disagreement_pct, listen_manually: pct > 15}}}`;
   - non-clip turns are ignored.
   - Tests with injected engines: offsets, `listen_manually`, an atomic write, `--turns`, clip-only selection.

## Acceptance

```bash
python3 -m pytest tests/test_asr_merge.py tests/test_asr_engines.py tests/test_clip_transcribe.py tests/test_word_timings.py -q
```

Then a live run on the `/tmp` copy, using `.venv-fermion`'s python:

```bash
PYTHONPATH=<this worktree> /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-fermion/bin/python -m scriptcast.tools.clip_transcribe /tmp/scriptcast-h/outputs/ATO_EP03_five_levels --script <copy of the EP03 script> --turns 220
```

Turn 220 is the `atwater_1981` clip. Record:
- the time taken;
- the word count and the engines used;
- the coverage warnings;
- the disagreement percentage;
- the first 30 words with their times.

If the fermion server or a model cannot run (missing executable, model download, GPU), record the exact error and keep everything else working. A well-documented "cannot run here" is acceptable.

Write `plans/Native_UI/evidence/phaseH1-findings.md` early and keep it current. Include which videolab behaviours you could not reproduce exactly, and why.
