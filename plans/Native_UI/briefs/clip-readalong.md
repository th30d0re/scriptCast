# Clip read-along inside the row

Branch `ui/clip-readalong`, this worktree. The operator approved this Python change on 2026-10-04 (it is part of the approved ensemble work).

## Rules

- No git writes.
- Never start other agent CLIs; never use `osascript`.
- `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only. Run its `.venv-fermion` interpreter only against `/tmp/scriptcast-clip/` copies.
- No synthesis. Mute captures.
- Lines ≤120 characters, `cargo fmt`, keep existing comments, no colour or size literals outside `theme.rs`.
- **Send build and test output to files** in `/tmp/scriptcast-clip/` and read only the tail.
- Read only the files named below; do not read `plans/`.

## Problem

Clip turns (`[clip:<id>]`) never highlight in their table row.

- `scriptcast/tools/clip_transcribe.py:64` (`build_clip_transcripts`) stores the ensemble's own words.
- The UI (`ui/src/table.rs:322`, `ui/src/read_along.rs`) highlights in the row only when those words match the row's script words one for one.
- For clips they essentially never match: the script has "1954", the recording says "nineteen fifty four", plus fillers and mishearings. So the highlight falls back to the inspector.

Synthesized turns avoid this because `scriptcast/tools/word_timings.py:58` (`build_timings`) aligns the recognised words to the **script** words with `_assign_times` and `_fill_gaps` from `scriptcast/tools/captions.py:40-80`. That gives every script word a time, and gaps are interpolated.

## Do

1. **In `clip_transcribe.py`,** after building the ensemble `words` for a clip turn, also produce `script_words`: one entry per word of that turn's script text after the tag, `{text, start_ms, end_ms, matched: bool}`. Use the same method as `word_timings.py` (`_assign_times`, then `_fill_gaps` within `(0, duration)`); `matched` is true when the word got a direct alignment. Note that `captions.script_words` skips clip turns via its `clip` flag, so build the word list for the clip turn yourself, with the same tokenisation and `normalize`.
2. **Numbers.** Add a small helper `spoken_number_tokens(token) -> list[str]` that expands plain integers for **matching only** (the stored `text` stays as written):
   - years 1100–2099 read in pairs ("1954" → "nineteen fifty four"; "2012" → "twenty twelve", and also try "two thousand twelve");
   - other integers 0–9999 in standard English.

   During alignment, a script number matches when its expansion appears in order in the recognised words. Give each expanded token the number's time span. `num2words` is **not** installed; write the helper in about 40 lines with tests.
3. **Output.** Add `script_words` to each turn in `clip_transcripts.json`. Bump the file to `"version": 2`, keeping every existing field.
4. **UI.**
   - `ui/src/clip_transcripts.rs` parses the optional `script_words`.
   - Where the table resolves row highlight words for a clip turn (around `table.rs:322`), prefer `script_words` when present, so the in-row highlight works like synthesized turns.
   - In the inspector's Ensemble transcript block, keep highlighting the ensemble words as today, and change the note to "Row highlighting is aligned to your transcript; unmatched words are approximate." when `script_words` exists.
   - A version-1 file without `script_words` keeps today's behaviour.
5. **Tests.**
   - Python: number expansion (1954, 1968, 2012, 21, 0, 100, 4000); alignment of a clip with a digit year, a filler word ("uh") and one misheard word; gap-filled words fall between their neighbours; `matched` flags; version-2 output includes `script_words` while the old fields are unchanged.
   - Rust: parsing of v1 and v2 files; the row lookup prefers `script_words`.

## Acceptance

```bash
python3 -m pytest tests/test_clip_transcribe.py -q > /tmp/scriptcast-clip/py.log 2>&1; tail -3 /tmp/scriptcast-clip/py.log
cd ui && cargo fmt && cargo build --release > /tmp/scriptcast-clip/build.log 2>&1; tail -2 /tmp/scriptcast-clip/build.log
cargo test > /tmp/scriptcast-clip/test.log 2>&1; grep "test result" /tmp/scriptcast-clip/test.log
cargo clippy --all-targets -- -D warnings > /tmp/scriptcast-clip/clippy.log 2>&1; tail -1 /tmp/scriptcast-clip/clippy.log
```

**Live:**
1. `cp -cR` EP03's output folder and copy its script to `/tmp/scriptcast-clip/`.
2. Run `PYTHONPATH=<worktree> /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-fermion/bin/python -m scriptcast.tools.clip_transcribe <copy> --script <copied script> --turns 220`.
3. Report how many script words matched out of the total, and the first 15 `script_words` with times.
4. Capture `--select 220 --play --at 6000` with `SCRIPTCAST_PROJECT` pointing at a copy of the project files, to `plans/Native_UI/evidence/clip-readalong.png`. Describe honestly whether a word is highlighted in the row.

Write `plans/Native_UI/evidence/clip-readalong-findings.md` with the results.

## 6. Version number in Settings (operator request, 2026-10-04)

- `ui/Cargo.toml` is now `version = "0.14.1"`. The orchestrator keeps it in step with the app bundle's version.
- Add `ui/build.rs`, which sets `SCRIPTCAST_GIT` to `git rev-parse --short HEAD`, with `-dirty` appended when `git status --porcelain` is non-empty, or `unknown` if git fails. Rerun it when `.git/HEAD` changes, if that is easy.
- At the bottom of the Settings window (`ui/src/settings.rs`), show a muted line: "scriptCast <CARGO_PKG_VERSION> (<SCRIPTCAST_GIT>)". Make it selectable so it can be copied.
- Add a test that the version string is non-empty and contains `CARGO_PKG_VERSION`.
