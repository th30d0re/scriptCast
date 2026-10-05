# Clip read-along — findings (2026-10-04)

## What changed

- `clip_transcribe.py` aligns ensemble words onto each clip turn’s script tokens,
  expanding plain integers for matching only (`spoken_number_tokens`), then
  `_assign_times` + `_fill_gaps`. Output is `clip_transcripts.json` **version 2**
  with per-turn `script_words` (`text`, `start_ms`, `end_ms`, `matched`).
- UI prefers `script_words` for in-row highlighting; v1 files without them keep
  today’s ensemble-only behaviour. Inspector keeps ensemble highlighting and shows
  the aligned-row note when `script_words` exist.
- Settings shows a selectable `scriptCast <version> (<git>)` line; `ui/build.rs`
  sets `SCRIPTCAST_GIT`.

## Acceptance commands

```
python3 -m pytest tests/test_clip_transcribe.py -q
→ 14 passed

cd ui && cargo fmt && cargo build --release
→ Finished release

cargo test
→ all crates ok (0 failed)

cargo clippy --all-targets -- -D warnings
→ Finished (no warnings)
```

## Live: EP03 turn 220

Copied `ATO_EP03_five_levels` and its script under `/tmp/scriptcast-clip/`.
Ran Theory `.venv-fermion` against that copy with `--turns 220`.

| Metric | Value |
| --- | --- |
| Ensemble words | 159 |
| Script words | 94 |
| Matched | **84 / 94** |
| Disagreement | 11.8% |

First 15 `script_words`:

| text | start_ms | end_ms | matched |
| --- | ---: | ---: | --- |
| You | 480 | 560 | true |
| start | 640 | 880 | true |
| out | 880 | 1040 | true |
| in | 1040 | 1200 | true |
| 1954 | 1200 | 2160 | true |
| by | 2160 | 2320 | true |
| saying, | 2320 | 2480 | true |
| nigger, | 2640 | 3040 | true |
| nigger, | 3040 | 3360 | true |
| nigger. | 3440 | 4160 | true |
| By | 4240 | 4400 | true |
| 1968 | 4560 | 5360 | true |
| you | 5440 | 5520 | true |
| can't | 5520 | 5760 | true |
| say | 5760 | 5840 | true |

Digit years `1954` and `1968` matched as spoken expansions and kept digit `text`.

## Screenshot

`plans/Native_UI/evidence/clip-readalong.png`  
(`--select 220 --play --at 6000`, muted via screenshot+play; `SCRIPTCAST_PROJECT`
pointed at `/tmp/scriptcast-clip/project`).

**Honest read of the capture:** Yes — a word in the table row for turn 220 is
highlighted (blue pill on a script word in the transcript column). Transport shows
~00:06.2 into the turn. The inspector Ensemble block shows the new note
(“Row highlighting is aligned to your transcript; unmatched words are approximate.”)
and still displays the ensemble wording (`nineteen fifty four`, etc.).
