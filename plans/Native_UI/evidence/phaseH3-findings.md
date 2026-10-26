# Phase H3 Findings

Status: complete 2026-10-04. Worktree `phase-h3` on `ui/phase-h3`.
No git writes. Theory tree read-only; live work under `/tmp/scriptcast-h3/`.

## Scope

- CLI: `scriptcast-verify-ensemble` → `stress_ensemble.json` (advisory; never mutates `stress.json`)
- UI: Jobs preset + Verification inspector "Second opinion" block + table note

## What landed

| Piece | Location |
|---|---|
| CLI | `scriptcast/tools/verify_ensemble.py` + `scriptcast-verify-ensemble` in `pyproject.toml` |
| Tests | `tests/test_verify_ensemble.py` (injected Phonon; verdicts, offsets, default failed selection, atomic write) |
| Loader | `ui/src/stress_ensemble.rs` — stale when `manifest_sha256` or `stress_sha256` changes |
| Preset | `Preset::SecondOpinion` — "Second opinion (failed turns)"; checkpoints `stress_ensemble.json`; does not synthesize |
| Resolver | `cli.rs` reuses fermion/`mlx_whisper` probe for `scriptcast-verify-ensemble` |
| Inspector | Verification panel keeps original result first; Second opinion below for failed turns |
| Table | Failed + `likely_recognition` → muted "maybe misheard" after "Failed" |

## Acceptance

```text
python3 -m pytest tests/test_verify_ensemble.py tests/test_asr_engines.py tests/test_asr_merge.py -q
# 22 passed

cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
# ok

awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs   # empty
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs  # empty
```

## Live run — EP03 under `/tmp/scriptcast-h3`

Copy: episode directory + `scriptcast.toml`; `Architecting_the_operation` and
`.venv-fermion` / `.venv-voice` as symlinks into Theory (read-only). Theory
`stress.json` left with 0 failed turns.

Injected into the **copy only**:

| Script turn | Manifest turn | `turn_id` | Change |
|---|---|---|---|
| 53 | 119 | `424e0d9be600e5ef` | `failed: true`; `expected` Portugal → **Spain** |
| 74 | 163 | `a1dcdfaa2383bad8` | `failed: true`; expected/heard unchanged |

```bash
SCRIPTCAST_PROJECT=/tmp/scriptcast-h3 \
PYTHONPATH=<worktree> SCRIPTCAST_FERMION=<Theory>/.venv-fermion/bin/fermion \
  <Theory>/.venv-fermion/bin/python -m scriptcast.tools.verify_ensemble \
  /tmp/scriptcast-h3/outputs/ATO_EP03_five_levels \
  --script .../ATO_EP03_redefining_racism_five_levels.md
```

| Field | Value |
|---|---|
| Wall clock | **35.98 s** real (user 28.14, sys 4.79) |
| Turns written | **2** |
| `stress.json` SHA | unchanged (advisory write only) |

### Turn 119 (`424e0d9be600e5ef`) — altered expected

| Field | Value |
|---|---|
| Summary | `likely_audio` |
| Scores | Whisper **0.901**, Phonon **0.901** |
| Spans | one: expected `spain` / whisper `portugal` / phonon `portugal` / `likely_audio` |

Both engines hear the real audio ("Portugal"); the script side was deliberately wrong.

### Turn 163 (`a1dcdfaa2383bad8`) — failed flag only

| Field | Value |
|---|---|
| Summary | `mixed` |
| Scores | Whisper **1.000**, Phonon **1.000** |
| Spans | **none** (engines already match expected) |

Marking a clean turn failed produces no disagreeing spans; summary falls through to
`mixed`. Useful as a negative control, not as a recognition-error demo.

### Resolver (UI)

```
scriptcast-verify-ensemble · Dev checkout: .../.venv-fermion/bin/python
  (.../phase-h3; -m scriptcast.tools.verify_ensemble)
```

Same fermion probe path as clip-transcribe; other commands still prefer `.venv-voice`
when it has an installed scriptcast.

## Evidence

### `phaseH3-second-opinion-turn53.png`

- Episode `ATO_EP03_five_levels`; turn **119** selected; Verification inspector open.
- Original block first: **× Failed**, Score **0.700 / 0.90** (injected stress score),
  Expected "…Spain chapter.", Heard "…Portugal chapter.", stress `lives · verb · ok`.
- **Second opinion** below: summary *"Engines agree the audio differs from the script:
  likely an audio problem"*; Whisper **0.901** beside Phonon **0.901**; span label
  `audio`; Expected `spain` / Whisper `portugal` / Phonon `portugal` with differing
  tokens emphasised.
- Table row shows **× Failed** without "maybe misheard" (summary is `likely_audio`,
  not `likely_recognition`).
- Footer notes 2 failed; mute capture; Theory tree untouched.

## Notes / brief fidelity

- Default turn selection is `failed: true` entries matched by `turn_id`.
- Whisper text is taken from `stress.json` `heard`; Phonon-2 runs per segment WAV.
- Token alignment uses `difflib` over `_normalize` tokens from `verify_render`.
- Empty-span summaries are `mixed` (vacuous `all(likely_recognition)` is avoided by
  requiring a non-empty span list for that branch).
