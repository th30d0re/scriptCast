# Phase K (UI): turn structure editing, tag toolbar, clip picker, retime

Working directory: a git worktree of scriptCast on branch `ui/phase-k`, from `dev` (app 0.17.0). The Phase K Python half is already merged on this branch.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, never use `osascript`, and never launch Live or Logic.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures, or copies under `/tmp/scriptcast-k/`.
- **No Python changes**, and no synthesis.
- Lines under 120 characters; `cargo fmt`; delete no comments; no colour or size literals outside `theme.rs`.
- **Tooltips:** only for a disabled reason, or for information that is not visible. Disabled menu items show a short visible reason instead.
- Redirect cargo output: `cargo test > /tmp/k-ui.log 2>&1; tail -30 /tmp/k-ui.log`.
- **Mute** captures.
- **The UI never splices script structure itself.** It goes only through `scriptcast-export --edit-json`.

## Design

Match `ui/design/K1-turn-context-menu.png`, `K2-new-turn.png` and `K3-tag-toolbar.png` with the existing tokens and components.

**Ignore in the mock-ups:**
- the bottom annotation strip;
- K1's "Separate example · turn 001" box;
- the ▶ glyph timeline (keep real waveforms);
- the engine names shown, which are illustrative. Show the real ones from `--speakers-json`.

## Contract (Python, merged): `scriptcast-export`

`plans/Native_UI/briefs/phase-k-py.md`, section "Contract", is the authority. Read only that section. In short:
- `--edit-json OPS --expect-sha256 HEX [--script] [--voices]` applies `insert` / `delete` / `move` / `set_speaker` atomically and prints the turns JSON. Exit 3 means a stale hash; exit 1 means validation failed, with `op[i]: message` lines.
- `--tags-json` lists tag problems per turn.
- `--speakers-json` lists the speakers (id, name, engine, has_expressive) and the clips.

## Established facts (`file:line` on dev)

- `script.rs:90` `splice` and `:172` `save` handle body edits, with a hash check; `:328` `save_many` makes one batch checkpoint. Structural edits go through a new `script.rs::apply_edits(ops)`, which takes a checkpoint first and then calls the CLI.
- `editing.rs:289` (inline row) and `:367` (side panel) use `TextEdit::multiline` for the body. Inline (A) and side panel (B) are toggled in Settings.
- `runner.rs` resolves the commands. Reuse the export command resolution used for `--turns-json`.
- After any structural save, the existing Detect changes → Precision insert flow applies. Turn ids change for moved or inserted turns, so reload the turns JSON and keep the selection on the edited turn.

## Build

**K1. Turn context menu**
1. Right-click a table row (and a timeline block) to open a menu with:
   - Insert turn above (⌥⌘↑), Insert turn below (⌥⌘↓);
   - Move up (⌘↑), Move down (⌘↓);
   - Change speaker ▸, a submenu of `--speakers-json` speakers with a colour dot and the engine, with the current speaker checked;
   - Delete turn (danger, ⌫), which asks for confirmation when the turn has rendered audio.
2. Disabled items show their reason in the menu, for example "Already first turn".
3. Shortcuts work on the selected turn when no text field has focus.
4. Move, Delete and Change speaker apply immediately through `--edit-json`, after a checkpoint.
5. Then the row shows "Unsaved · structure changed" until Detect changes runs, and the side panel's next step shows "Detect changes → Precision insert" as in K1.

**K2. New turn**
1. Insert creates a local draft row (not yet in the file) in the editing state, both inline (A) and in the side panel (B). It has a speaker picker (colour dot plus engine), a timestamp field defaulting to "est.", which is left `null` so the CLI interpolates, and an empty body with the placeholder "Write what <Speaker> says…".
2. **Save (Cmd+Return)** sends one `insert` op; **Cancel** discards the draft.
3. If the chosen speaker's engine differs from both neighbours', show the muted note "<Speaker> renders with <Engine>."
4. **An archive speaker** replaces the free text with a "Choose archive clip…" picker; Save stays disabled with the visible reason "Clip required · no free text". The body becomes `[clip:id] ` followed by the transcript the user types after choosing.

**K3. Tag toolbar, tag rendering, raw toggle**
1. A toolbar above every body editor (A and B, including the existing body editing):
   - Pause, with an ms field defaulting to 800, inserts `[pause:800ms]`;
   - Beat inserts `[beat]`; Emphasis inserts `[emphasis]`;
   - Tone, with a text field, inserts `[tone:<text>]`;
   - Clip ▾ opens the clip picker.
   Everything inserts at the caret.
2. **Clip picker popover:**
   - a search field and the clips from `--speakers-json`: id, source file, duration (end minus start), and the citation in muted text;
   - a play button per row, which plays the clip's range from its source through the existing audio output. If the source cannot be decoded, show a visible "can't preview" note instead;
   - "Insert selected <id>", which inserts `[clip:id]` at the caret.
3. **Tags render as pills** inside the editor, using a `TextEdit` layouter: a tag span gets a pill background and the caption font; an invalid tag gets the danger pill. Brackets stay in the text, so caret positions map 1:1. Do **not** replace them with display text.
4. **Validation:**
   - while typing, check tags locally against the same rules as the CLI's `--tags-json`;
   - on Save, run `--tags-json` against the saved file as the authority;
   - a red line under the body, e.g. "[pause:abc] is not a number of milliseconds";
   - Save is disabled for malformed `pause` and `clip` tags (unknown bracket text is only a warning).
5. **Raw script toggle:** switches the editor to plain monospace text with no pills, with the same draft text.

**K4. Retime** (moved here from Phase J)
1. Add a Jobs button "Retime script" (`scriptcast-retime <episode_dir> --transcript <script>`). Check the tool's `--help` for a dry-run or diff mode and use it to show a before/after timestamp diff list.
2. If the tool has no dry-run mode, run it on a temporary copy of the script and diff that.
3. "Apply" writes, with a checkpoint and a hash check.

## Acceptance

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must all be green. The new tests cover:
   - building the ops JSON for each menu action;
   - parsing exit 3 and exit 1 errors into visible messages;
   - an end-to-end run with the **real** merged Python on a fixture copy, using the dev checkout's `scriptcast-export` with `PYTHONPATH`: move, delete, set_speaker and insert, each followed by a reload of the turns JSON;
   - the local tag validator agreeing with `--tags-json` on a fixture set;
   - tag-span detection for the layouter;
   - the archive-speaker Save gating;
   - parsing the retime diff.
2. Add capture flags so the orchestrator can screenshot without clicking:
   - `--context-menu <turn>` (with the speaker submenu open);
   - `--new-turn-after <turn> [--new-turn-speaker <id>]`;
   - `--tag-toolbar <turn> [--clip-picker]`;
   - `--raw-script`.
   Add `plans/Native_UI/evidence/capture-phaseK.sh`. You cannot launch the GUI, so the orchestrator runs it.
3. Bump the version to 0.18.0.

## Stop conditions

If an egui 0.36 `TextEdit` layouter cannot paint pill backgrounds without breaking caret hit-testing, stop and report. A fallback of coloured text with no background is acceptable; note it in the findings file.

## Findings file

Write `plans/Native_UI/evidence/phaseK-ui-findings.md` early: the files changed, the tests, deviations, and what is unverified.

## Later work (2026-10-18) — not part of the contract above

The Tone field already inserts `[tone:<text>]`. Leave that control as it is. Rendering the tag is the Rust engine in `briefs/phase-k-py.md` (later-work section), not a new toolbar button. Do not add Laugh or Sigh controls. Flow 0 in `plan.md` previews a tone edit by re-rendering the selected range through that engine.
