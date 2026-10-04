# Phase E findings

Status: implemented; acceptance checks passed; required muted screenshot saved and visually reviewed.

- Work is confined to the phase-e worktree and `/tmp/scriptcast-e`; no git writes.
- Native Rust/egui desktop app, not a mobile or Chromium app. No other apps or agent CLIs used.
- Phase D table virtualizes explicit row rectangles, with editor height added before later rows.
  Read-along shares this accounting, including when editor and playback target the same row.
- CLI transcribes processed segment WAVs, offset relative to manifest turn start, and aligns
  script words independently for each turn. Archive clips are excluded.
- Existing resolver discovers console entries from pyproject.toml for the checkout fallback.
- Jobs already confirm overwrites and checkpoint before launch; read-along uses that path.
- Read-along uses 13 px primary text, 4 px gaps, 3 px accent-pill radius and 4×2 padding.
  The app embeds OpenDyslexic regular/bold only; emphasis uses its existing bold font token.
  It truncates after two lines with an ellipsis and requests no additional continuous repaints.
- Playback concatenates WAVs. The word lookup maps that cursor through manifest segment durations
  and start offsets, so multi-segment gaps do not accumulate read-along drift.

## Validation

Python acceptance tests: **53 passed** (fake ASR; no model needed).
Rust tests: **125 passed, 5 existing live/hardware tests ignored**.
`cargo fmt --check`, `cargo build --release`, `cargo clippy --all-targets -- -D warnings`,
`git diff --check`, and the requested `awk` Rust line-length audit all pass.
Python source/test line lengths also pass.
Existing Rust tests had hardcoded scratch roots from Phase D. Active test scratch roots now use
`/tmp/scriptcast-e` so validation respects this phase's write boundary.

## Live evidence

Cloned `outputs/ATO_EP03_five_levels` with `cp -cR` to
`/tmp/scriptcast-e/project/outputs/ATO_EP03_five_levels` (392 turns).
Copied the source script, scriptcast.toml and configured voices YAML with relative paths preserved.
Cloned the cached whisper-small model under `/tmp/scriptcast-e/huggingface` and ran offline with
HF_HOME, XDG_CACHE_HOME, TMPDIR and PYTHONDONTWRITEBYTECODE redirected/set; original project is read-only.

Exact module CLI (default model `mlx-community/whisper-small-mlx`):

```sh
PYTHONPATH=/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-e \
SCRIPTCAST_PROJECT=/tmp/scriptcast-e/project \
HF_HOME=/tmp/scriptcast-e/huggingface HF_HUB_OFFLINE=1 \
XDG_CACHE_HOME=/tmp/scriptcast-e/cache TMPDIR=/tmp/scriptcast-e PYTHONDONTWRITEBYTECODE=1 \
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python \
  -m scriptcast.tools.word_timings \
  /tmp/scriptcast-e/project/outputs/ATO_EP03_five_levels --turns 5-12
```

Wall time: **11.716 seconds**, exit 0; **8 turns / 187 words** written.
An initial direct-main smoke run took 10.288 seconds.

Turn 7 (`1497a46e70d59fc1`), 45 words: processed WAV header gives **15,870 ms**;
manifest speech window is **15,830 ms**; final word ends at **15,660 ms**.
Every word was checked to satisfy `0 ≤ start ≤ end ≤ WAV length`.
At 3000 ms there is a natural gap (2860–3220); no word should be highlighted in that gap.

Muted capture succeeded, exit 0, at **2996×1800** pixels:
`phaseE-readalong.png`. Capture was run with `--fresh --select 7 --play --at 3000 --mute`.
The final required capture is at **3211 ms**, within the natural gap, and correctly has no highlight.
An earlier capture was visually checked at **3327 ms** with **and** highlighted (3220–3340 ms).
That earlier image was replaced by the final capture after fixing scratch-project setup.
A supplemental `--at 3400` capture timed out after 20 seconds with **0 frames painted**, consistent
with the display being asleep. No supplemental image was produced; stopped captures as instructed.
The read-along strip is directly below turn 7, wraps onto two lines, and following rows do not overlap.

The cloned render state contained an absolute path to the original script. Rebased **only the copied
render_state.json** to the manifest's relative script path so UI script reads use the copy.
Also copied the configured pronunciation, speaker-rate and archive-clip metadata (no source media)
to satisfy the existing inspector's edit-marker reader. The copy's system Python lacked `misaki`;
scratch `.venv-voice/bin` wrappers delegate to the explicitly authorized original venv interpreter
with this checkout's PYTHONPATH and bytecode writes disabled. Final capture resolves project-venv
console wrappers directly, avoiding module-probe startup delays. Only read-only export commands ran;
no synthesis, Verify or Repair jobs were launched.
No installed venv or original project files were modified.

The manifest and all eight transcribed WAVs were hash-checked against the original and remain identical.

Final required capture command (from the worktree):

```sh
SCRIPTCAST_PROJECT=/tmp/scriptcast-e/project TMPDIR=/tmp/scriptcast-e \
XDG_CACHE_HOME=/tmp/scriptcast-e/cache PYTHONDONTWRITEBYTECODE=1 \
ui/target/release/scriptcast-ui /tmp/scriptcast-e/project/outputs/ATO_EP03_five_levels \
  --fresh --select 7 --play --at 3000 --mute \
  --screenshot plans/Native_UI/evidence/phaseE-readalong.png
```

| Word | Start ms | End ms |
| --- | ---: | ---: |
| In | 0 | 140 |
| engineering, | 140 | 680 |
| that | 1040 | 1280 |
| is | 1280 | 1460 |
| a | 1460 | 1560 |
| specification | 1560 | 2140 |
| failure, | 2140 | 2860 |
| and | 3220 | 3340 |
| one | 3340 | 3600 |
| book | 3600 | 3900 |
| treats | 3900 | 4240 |
| it | 4240 | 4460 |
| as | 4460 | 4680 |
| exactly | 4680 | 5200 |
| that | 5200 | 5580 |
| and | 5580 | 5880 |
| writes | 5880 | 6140 |
| the | 6140 | 6360 |
| spec. | 6360 | 6600 |
| Racism | 7180 | 7760 |
| is | 7760 | 7940 |
| a | 7940 | 8100 |
| system | 8100 | 8380 |
| built | 8380 | 8680 |
| to | 8680 | 8920 |
| move | 8920 | 9160 |
| wealth | 9160 | 9520 |
| upward, | 9520 | 9920 |
| and | 10280 | 10460 |
| the | 10460 | 10620 |
| prejudice | 10620 | 11020 |
| most | 11020 | 11500 |
| people | 11500 | 11900 |
| mean | 11900 | 12200 |
| by | 12200 | 12420 |
| the | 12420 | 12560 |
| word | 12560 | 12820 |
| is | 12820 | 13100 |
| something | 13100 | 13400 |
| that | 13400 | 13700 |
| system | 13700 | 14040 |
| manufactures | 14040 | 14880 |
| to | 14880 | 15020 |
| protect | 15020 | 15360 |
| itself. | 15360 | 15660 |
