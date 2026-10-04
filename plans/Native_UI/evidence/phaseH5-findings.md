# Phase H5 Findings

Status: complete 2026-10-04. Worktree `phase-h5` on `ui/phase-h5`.
No git writes. Theory tree read-only; live work under `/tmp/scriptcast-h5/`.

## Scope

- Python: `scriptcast/review.py`, `scriptcast/tools/review_stats.py`,
  minimal hooks in `verify_render.py` and `word_timings.py`
- UI: Verification inspector "Reviewer decision" below Second opinion;
  Jobs → Review stats
- `project_dir` = parent directory of the configured `voices` file from
  `scriptcast.toml` (same folder as voices/pronunciations)

## What landed

| Piece | Location |
|---|---|
| Review log + allowlist + hints | `scriptcast/review.py` (`append_review`, `load_accepted_hearings`, `add_accepted_hearing`, `vocabulary_hints`, `scriptcast-review add`) |
| Stats CLI | `scriptcast/tools/review_stats.py` → `scriptcast-review-stats` |
| Verifier hooks | `verify_render.py`: accepted hearings in `_score` / run detection; `initial_prompt` when hints exist; `--no-feedback`; `feedback` key on stress entries only when applied |
| Word timings | `word_timings.py`: same hints via `transcribe_with_prompt`; `--no-feedback`; default path unchanged when no hints |
| UI logic | `ui/src/review.rs` — outcome labels, replace-pair accept rules, argv builder |
| Inspector | Verification panel → Reviewer decision (radios, correction, Save, accept checkbox / multi-pair list, calibrate hint) |
| Jobs | `Preset::ReviewStats` (read-only log); resolver includes `scriptcast-review` / `scriptcast-review-stats` |
| Tests | `tests/test_review.py`, `tests/test_review_stats.py`, `tests/test_verify_render_feedback.py`; Rust tests in `review.rs` |

## Acceptance

```text
python3 -m pytest tests/test_review.py tests/test_review_stats.py \
  tests/test_verify_render*.py tests/test_verify_ensemble.py -q
# 19 passed

python3 -m pytest -q 2>&1 | tail -1
# 11 failed, 283 passed  (known test_pronunciation only)

cd ui && cargo fmt && cargo build --release && cargo test \
  && cargo clippy --all-targets -- -D warnings
# ok

awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
# empty
```

Byte-identical path: with no `accepted_hearings.yaml` and no hint list,
`_score` / `_diff` and verify JSON omit any `feedback` key and do not pass
`initial_prompt` (covered in `test_verify_render_feedback.py`).

## Live run — `/tmp/scriptcast-h5`

Copy: `scriptcast.toml` + episode `ATO_EP03_five_levels` (rsync, no `_backups`);
`Architecting_the_operation/podcasts` and venvs symlinked into Theory (read-only).
`project_dir` = `/tmp/scriptcast-h5/Architecting_the_operation/scriptcast`.

### Injected failure (copy only)

Stress entry `turn_id` `e1b0c78e35ea0402` (joins manifest turn **115**; script
index in the stale stress file is 49). Expected shortened to a real EP03 opening
clause so one mishear fails scoring under default thresholds:

| Field | Value |
|---|---|
| Expected | `That split makes a prediction.` |
| Heard | `That split xylophone a prediction.` |
| `_score` before | `(0.800, 1)` → failed |
| Manifest join | turn **115** |

### CLI accept

```bash
PYTHONPATH=<worktree> SCRIPTCAST_PROJECT=/tmp/scriptcast-h5 \
  python3 -m scriptcast.review add \
  /tmp/scriptcast-h5/Architecting_the_operation/scriptcast \
  --turn-id e1b0c78e35ea0402 --episode ATO_EP03_five_levels \
  --start-token 2 --end-token 3 \
  --script-text makes --heard-whisper xylophone \
  --outcome recognizer_misheard \
  --timestamp 2026-10-04T16:20:00Z \
  --accept makes=xylophone
```

Printed: append path, "accepted 1 hearing(s)", and
"Saved locally. Used by the next verification; does not retrain any model."

### Re-score (no Whisper)

`_score_with_feedback` with loaded allowlist → `(1.000, 0, hits=1)` → **failure cleared**.

### `scriptcast-review-stats`

```text
False-failure rate … ATO_EP03_five_levels: 1/1 = 100.0%
makes ← [xylophone] added 2026-10-04T16:20:00Z: before 1 · after 0
```

## Evidence screenshots

### `phaseH5-reviewer-decision.png` (+ `-crop.png`, `-block.png`)

- Episode copy under `/tmp`; `--fresh --select 115 --inspector verification --mute`.
- Verification · Turn **115**: × Failed, Score **0.800 / 0.90**.
- Expected `That split makes a prediction.`; Heard with **xylophone** emphasised.
- Second opinion placeholder + Run… (no ensemble file on the copy).
- **Reviewer decision** below it:
  - Copy: "Saved locally. Used by the next verification; does not retrain any model."
  - Three radios with one-line explanations; **Whisper was wrong — recognition error** selected by default.
  - Optional correction field.
  - **Save decision** (primary; resolves `scriptcast-review` via the command resolver).
  - Checked: "Accept this hearing for **makes** in future verifications" (single differing word).
- Table footer: 1 failed. Theory tree untouched.

Honest note: with a short timeline / tall inspector the accept checkbox and Save
both fit in the scroll; depending on crop, one or the other sits nearer the
transport chrome. Full window PNG has the complete inspector.

## Notes / brief fidelity

- UI never writes JSON; only spawns `scriptcast-review add`.
- `--no-feedback` disables allowlist + hints on verify and word-timings.
- `feedback` on a stress entry appears only when `accepted > 0` or hints were passed.
- YAML review comments on accepted hearings are preserved across later appends.
- Selecting by stress `turn`/`manifest_turn` is unsafe on stale EP03 stress files;
  join by `turn_id` (here → manifest 115).
