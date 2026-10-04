# Phase H1 + H2 (Python) findings

Started: 2026-10-04. Worktree: `scriptCast-worktrees/phase-h` on `ui/phase-h`.
Completed live run the same day.

## Scope

Port videolab ASR merge + ensemble adapters into `scriptcast/asr/`, add
`scriptcast-clip-transcribe`, tests, and a live run on EP03 turn 220
(`atwater_1981`) using a `/tmp/scriptcast-h/` copy.

## Source map

| Videolab | scriptCast |
|---|---|
| `videolab/asr_merge.py` | `scriptcast/asr/merge.py` (byte-identical) |
| `videolab/asr.py` ensemble/engines | `scriptcast/asr/engines.py` (adapted API) |
| `videolab/tests/test_asr_merge.py` | `tests/test_asr_merge.py` |
| ensemble cases in `test_asr.py` | `tests/test_asr_engines.py` |
| (new) | `scriptcast/tools/clip_transcribe.py`, `tests/test_clip_transcribe.py` |
| — | `pyproject.toml` console script `scriptcast-clip-transcribe` |

## API adaptation (intentional)

Videolab's `transcribe_ensemble` writes job-dir artifacts and returns `AsrResult`.
scriptCast's `ensemble(audio_path)` returns a dict:

```
{words, segments, coverage_warnings, whisper_disagreement_pct, engines_used, text}
```

optional `coverage_check_error` when Phonon-1-Big fails (videolab's job detail key).

Merge order matches videolab: Whisper pass → Phonon primary + gap fill →
group → Whisper backfill (≥8 tokens) → drop overlapping Phonon words →
regroup + insert Whisper segments → Phonon-1-Big coverage check (warn only) →
Whisper-only fallback if Phonon fails.

`whisper_disagreement_pct` is not stored by videolab's job writer; it comes from
the measured rule in `videolab/docs/asr_phonon2_vs_whisper.md` (flag >15% for
manual listening). Computed as word-level edit distance over `normalize_tokens`
of merged vs Whisper text, as a percentage of `max(len(merged), len(whisper))`.

## Behaviours that could not be reproduced exactly

1. **Fermion location.** Videolab resolves `.venv-fermion` from its repo root.
   scriptCast has none. Resolution order:
   `SCRIPTCAST_FERMION` → `VIDEOLAB_FERMION` →
   `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-fermion/bin/fermion`
   when present → `PATH`.
2. **Job-dir / transcript.srt/vtt / job.json side effects.** Not ported; clip
   tool writes only `clip_transcripts.json`.
3. **Word `engine` tags.** Videolab keeps engine on segments; words in
   `transcript.json` are untagged Phonon leftovers. scriptCast tags each word
   (`phonon-2` / `whisper`) for clip read-along. Whisper-backfilled spans
   synthesize per-word times by splitting segment text evenly across the
   segment window (videolab never materializes those as words).
4. **Phonon-1-Big on this machine (turn 220).** Phonon-2 and mlx-whisper ran.
   `big_text` failed with `RemoteDisconnected: Remote end closed connection
   without response` (~5 s). Merge kept; `engines_used` omitted `phonon-1-big`;
   `coverage_warnings` stayed `[]`. Same class of failure videolab records as
   `coverage_check_error`.
5. **Interpreter split.** Brief asked for `.venv-fermion`'s python. That venv
   has `fermion` but not `mlx_whisper` (which lives in `.venv-voice`). Live run
   used fermion's python with
   `PYTHONPATH=<worktree>:<.venv-voice site-packages>` so mlx import works while
   fermion still shells out to `.venv-fermion/bin/fermion`.

## Pytest

```
python3 -m pytest tests/test_asr_merge.py tests/test_asr_engines.py \
  tests/test_clip_transcribe.py tests/test_word_timings.py -q
```

Result: **34 passed** (2026-10-04).

## Live run — EP03 turn 220 (`atwater_1981`)

Command (after `cp -cR` of the episode and a copy of the script under `/tmp/scriptcast-h/`):

```bash
PYTHONPATH=<worktree>:<Theory/.venv-voice/.../site-packages> \
SCRIPTCAST_FERMION=<Theory/.venv-fermion/bin/fermion> \
  <Theory>/.venv-fermion/bin/python -m scriptcast.tools.clip_transcribe \
  /tmp/scriptcast-h/outputs/ATO_EP03_five_levels \
  --script /tmp/scriptcast-h/script/ATO_EP03_redefining_racism_five_levels.md \
  --turns 220
```

| Field | Value |
|---|---|
| Wall clock | **25.75 s** real (user 16.92, sys 5.17) |
| Turn / clip | `99086561b2a46360` / `atwater_1981` |
| Word count | **159** |
| Engines used | `phonon-2`, `mlx-whisper` (all 159 words tagged `phonon-2`; no Whisper backfill spans) |
| Coverage warnings | `[]` |
| Big check | failed: `RemoteDisconnected` (see above) |
| `whisper_disagreement_pct` | **11.8** |
| `listen_manually` | **false** (threshold is >15) |
| Output | `/tmp/scriptcast-h/outputs/ATO_EP03_five_levels/clip_transcripts.json` (also copied to `/tmp/scriptcast-h/clip_transcripts_turn220.json`) |
| Source tree | untouched (no `clip_transcripts.json` under Theory/outputs) |

### First 30 words (times in ms, turn-relative)

Historical Lee Atwater archive audio; transcript is what the ensemble heard.

| # | text | start_ms | end_ms | engine |
|---|---|---|---|---|
| 1 | You | 480 | 560 | phonon-2 |
| 2 | start | 640 | 880 | phonon-2 |
| 3 | out | 880 | 1040 | phonon-2 |
| 4 | in | 1040 | 1200 | phonon-2 |
| 5 | nineteen | 1200 | 1440 | phonon-2 |
| 6 | fifty | 1600 | 1840 | phonon-2 |
| 7 | four | 1920 | 2160 | phonon-2 |
| 8 | by | 2160 | 2320 | phonon-2 |
| 9 | saying | 2320 | 2480 | phonon-2 |
| 10 | nigger, | 2640 | 3040 | phonon-2 |
| 11 | nigger, | 3040 | 3360 | phonon-2 |
| 12 | nigger. | 3440 | 4160 | phonon-2 |
| 13 | By | 4240 | 4400 | phonon-2 |
| 14 | nineteen | 4560 | 4800 | phonon-2 |
| 15 | sixty | 4960 | 5200 | phonon-2 |
| 16 | eight | 5280 | 5360 | phonon-2 |
| 17 | you | 5440 | 5520 | phonon-2 |
| 18 | can't | 5520 | 5760 | phonon-2 |
| 19 | say | 5760 | 5840 | phonon-2 |
| 20 | nigger, | 6000 | 6320 | phonon-2 |
| 21 | that | 6320 | 6480 | phonon-2 |
| 22 | hurts | 6560 | 6800 | phonon-2 |
| 23 | your | 6800 | 6880 | phonon-2 |
| 24 | backpower. | 6960 | 7600 | phonon-2 |
| 25 | So | 7600 | 7680 | phonon-2 |
| 26 | you | 7840 | 7920 | phonon-2 |
| 27 | say | 8000 | 8160 | phonon-2 |
| 28 | stuff | 8240 | 8480 | phonon-2 |
| 29 | like | 8480 | 8640 | phonon-2 |
| 30 | uh | 8800 | 8960 | phonon-2 |

Note on word 24: ensemble heard `backpower.` (likely "black power" misheard); disagreement
11.8% is consistent with wording drift without a coverage gap large enough for Whisper backfill.
