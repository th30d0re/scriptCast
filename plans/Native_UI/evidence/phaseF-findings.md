# Phase F findings

Status: finished (2026-10-04). Inherited Codex work audited, gaps closed, acceptance green, three required muted captures reviewed.

Constraints held: no git writes; no other agent CLIs / `osascript`; real EP03 read-only; no synthesis; captures muted; scratch under `/tmp/scriptcast-f/`.

## Audit vs Build items 1–6

| # | Item | Final | Notes |
|---|------|-------|-------|
| 1 | Full-text Transcript column | done | Header "Transcript"; body from ScriptTurns with `first_words` fallback; 2-line wrap + ellipsis; `TABLE_ROW_HEIGHT` / `TABLE_ROW_HEIGHT_2`; virtualised `RowOffsets` prefix sums for scroll/footer; other columns top-aligned to first text line. |
| 2 | A: edit in the row cell | done | Selected Inline draft replaces the Transcript cell with multiline `TextEdit`; Save draft / Cancel / Unsaved draft (when dirty) in-row; no under-row editor block. |
| 3 | Mode toggle in filter bar | done | Accent "Inline / Side panel" control after Failed only with active-mode pill; Settings labels Inline / Side panel; same `preferences.edit_mode`. |
| 4 | B: read-along inside playing row | done | Accent pill (radius 3, pad 4×2, bold, `ON_ACCENT`) painted inside the playing row's Transcript; under-row strip removed; unequal word counts → no highlight. |
| 5 | Playback bar "· Paused" | done | Current-audio line appends ` · Paused` when paused. |
| 6 | Inspector copy | done | Playing / Paused sentences match the design; long format strings wrapped under 120 columns. |

## Finish-pass code changes

1. Split inspector status sentences so no source line exceeds 120 characters.
2. Added `table::tests::prefix_sums_mix_one_and_two_line_heights_and_scroll_to`.
3. Added `editing::tests::mode_toggle_writes_the_persisted_edit_mode` (click flips `preferences.edit_mode`, serde round-trip).
4. Word-mapping tests were already present in `read_along.rs` (equal / unequal / punctuation+case).

No item needed a >150-line rewrite; inherited table/read-along/editing work was kept.

## Acceptance

```text
cargo fmt
cargo build --release          # ok
cargo test                     # 125 passed, 5 ignored (live/hardware/scratch)
cargo clippy --all-targets -- -D warnings   # ok
awk 'length>120…'              # no hits
grep colour literals outside theme.rs       # no hits
```

## Captures (muted)

Real EP03 (read-only):

```sh
ui/target/release/scriptcast-ui …/ATO_EP03_five_levels \
  --fresh --mute --select 12 \
  --screenshot plans/Native_UI/evidence/phaseF-rows.png
# 2996×2252, exit 0

ui/target/release/scriptcast-ui …/ATO_EP03_five_levels \
  --fresh --mute --select 12 --edit --edit-mode row \
  --screenshot plans/Native_UI/evidence/phaseF-inline-edit.png
# 2996×2252, exit 0
```

Read-along `/tmp` copy:

```sh
cp -cR outputs/ATO_EP03_five_levels → /tmp/scriptcast-f/project/outputs/
# script + voices/pronunciations/speaker_rates/clips.yaml; render_state.source_file rebased to relative
PYTHONPATH=<this worktree> SCRIPTCAST_PROJECT=/tmp/scriptcast-f/project \
  …/python -m scriptcast.tools.word_timings … --turns 5-12
# 8 turns / 187 words, exit 0, ~10 s (reused /tmp/scriptcast-e huggingface cache offline)

SCRIPTCAST_PROJECT=/tmp/scriptcast-f/project … \
  ui/target/release/scriptcast-ui /tmp/scriptcast-f/project/outputs/ATO_EP03_five_levels \
  --fresh --select 7 --play --at 3000 --mute \
  --screenshot plans/Native_UI/evidence/phaseF-readalong.png
# 2996×2252 at ~3211 ms, exit 0
```

Supplemental (not required; taken because 3000 ms falls in a natural gap with no word):

```sh
… --at 3400 --screenshot plans/Native_UI/evidence/phaseF-readalong-highlight.png
# ~3620 ms, in-row pill on "book", exit 0
```

No capture timed out; display was kept awake with `caffeinate` during runs.

Scratch-copy inspector shows red `misaki` / `--changes-json` errors (system Python on the copy). Word timings and playback still work. Original project untouched.

## Visual review vs 03e targets

### `phaseF-rows.png` vs `03-alt-b.png` / table portion of A

Matches:
- Column header is **Transcript**; cells show full (wrapped) turn text, not first-words stubs.
- Row heights vary; long turns use a taller band (~2-line), short turns stay compact.
- Filter bar has **Inline / Side panel** after Failed only (here active pill = Side panel).
- No separate under-row read-along strip.

Differs from the Figma frames (expected / out of scope):
- Live EP03 speakers/text (Toussaint/Aisha/…) vs design cast (Alex/Maya/Jordan/Sam).
- Verification is mostly "Not verified"; design shows Passed/Failed sample data.
- Window chrome includes Live/Logic/Settings; design crop is tighter.
- Mode defaults to Side panel when not forced; design B emphasises panel edit + playing highlight.

### `phaseF-inline-edit.png` vs `03-selected-turn.png` (A)

Matches:
- Mode pill shows **Inline**.
- Selected row Transcript cell is the editor; **Save draft** (accent) and **Cancel** sit under the text in the same row.
- Row grows for the editor chrome; no separate editor block below the row.
- Other columns stay top-aligned with the first text line.

Differs:
- Draft is clean on open → no **Unsaved draft** badge (design A shows a dirty draft).
- Transport idle → no "· Paused" on the playback line (item 5 only applies when paused).
- Inspector idle copy ("Turn 012 selected…") rather than the paused listening sentence.
- Turn 012 / Not verified vs design turn 004 / Failed.

### `phaseF-readalong.png` vs `03-alt-b.png` (B)

Matches:
- Playing turn 007 with ▶ in the Turn column; Pause active in the playback bar.
- Inspector: "Listening to turn 007. Transcript edits are local until saved; rendered audio remains unchanged."
- No under-row read-along expansion; following rows sit flush under the playing row.
- Mode toggle present (Side panel).

Differs / timing:
- Capture settled at **~3211 ms**, inside the known gap after "failure," (2860–3220). **No word pill** — correct for the timings, so this frame does not prove the highlight paint.
- Supplemental `phaseF-readalong-highlight.png` at **~3620 ms** shows an accent pill on **"book"** inside the Transcript cell (matches item 4 / design B behaviour).
- Scratch-copy red export/`misaki` notices in the inspector; design has a clean panel.
- Design B also shows side-panel Edit tab with an unsaved draft; this capture is Turn view while playing.

## Tests covering the brief

| Required | Location |
|----------|----------|
| Row-height prefix sums + `scroll_to` | `table::tests::prefix_sums_mix_one_and_two_line_heights_and_scroll_to` |
| Word mapping equal / unequal / punctuation+case | `read_along::tests::mapping_requires_equal_counts_and_ordered_normalized_words` |
| Mode toggle writes persisted setting | `editing::tests::mode_toggle_writes_the_persisted_edit_mode` |
