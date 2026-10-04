# Phase I: find words, find turns, and find & replace

Your working directory is a git worktree of scriptCast on branch `ui/phase-i`, from `dev` (app 0.12.0). A separate agent is building the Discrepancy review view (H4) in parallel in another worktree. Keep your changes to the filter bar, table, find and replace code, and avoid restructuring unrelated parts of `main.rs`.

## Prohibitions (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only**, including scripts. Every replace test runs on a `/tmp/scriptcast-i/` copy, with `SCRIPTCAST_PROJECT` pointing at the copy.
- **No Python changes. No synthesis.** Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Operator request (2026-10-04)

> "Right now it's just find turns, because it's not really as helpful. I want to find the words. I want to be able to do both, and then be able to find and replace as well."

## Target

- `ui/design/screens/03f-find-replace/03-A.png` (inline editing) and `03-B.png` (side panel). The filter bar becomes:
  - `Find · ⌘F [query]`;
  - the match count, "1 of N";
  - Previous and Next;
  - All speakers ▾, Failed only · n, and the Inline / Side panel toggle (A only);
  - an "Episode text" scope label;
  - an **Options ▸** menu and a **Replace ▸ ⌘⌥F** menu.
- Matched words are underlined in an accent-gold colour inside the Transcript text (add a token), and the current match is emphasised further.

## Build

1. **Find words (default).**
   - The query searches every turn's **full transcript text**, from `ScriptTurns` (`editing.rs`), falling back to first words. Matching is case-insensitive unless Options says otherwise.
   - All rows stay visible, as the design shows.
   - Every occurrence is underlined in place in the table, and the current one gets a stronger style: an accent outline or fill behind the word.
   - Typing updates the matches. **Return or Next** goes to the next match and **Shift-Return or Previous** to the one before; both scroll the table to that row and select it, and wrap around at the ends.
   - The count shows "i of N", or "No matches".
   - Search the text as displayed: the clip tag is stripped (`script::display_body`).
2. **Find turns (Options).**
   - **Mode: Words / Turns.** Turns mode keeps the existing behaviour: the table is filtered to matching turns and Previous / Next step through them.
   - **Match case, Whole word, Regular expression** (use the `regex` crate if needed and add it to `Cargo.toml`; show an invalid pattern as "Invalid pattern" in the danger colour).
   - **Include audio clips** (default on).
   - Persist the options in `Preferences`.
3. **Replace (Replace ▸ or ⌘⌥F).** Opens a second row under the filter bar with a Replace field, **Replace** (current match), **Replace all**, and a preview line: "Replace N occurrences in M turns".
   - Replacing changes the **script**, through a new batch function in `script.rs` (Rust): `save_many(episode, edits: &[(ScriptTurn, String)], loaded_sha256)`. It must:
     - hash-check once;
     - take **one** `script-edit` checkpoint for the whole batch;
     - splice every affected body in a single pass from bottom to top, so line numbers stay valid;
     - write once with temp file + rename;
     - keep every other byte.
   - Add tests in the style of `tests/script_edit.rs`: bytes preserved on LF and CRLF; one checkpoint for a multi-turn replace; a hash conflict refuses and writes nothing; restore gives the original bytes.
   - The replacement is applied to each affected turn's **body** text (not the displayed text), using the same matching rules.
   - **Clip turns:** the `[clip:<id>]` tag must survive. If a match falls inside the tag, skip that occurrence and count it as skipped in the preview. Clip edits remain transcript-only corrections (Phase G rules).
   - **Replace all** shows a confirmation, using the existing dialog style: "Replace N occurrences in M turns of <script name>? A checkpoint of the script is saved first; rendered audio is unchanged." It lists up to 10 turns with before → after snippets, then "+K more".
   - **Replace** (the current match) saves without a dialog, but still takes the checkpoint.
   - After saving: reload the script turns and changes, so the edited markers appear; keep the find query and recompute the matches; show "Replaced N in M turns · checkpoint vNN" in the status bar.
   - Any open inline draft blocks Replace, with "Save or cancel the open edit first".
4. **Keyboard:**
   - ⌘F focuses Find; ⌘⌥F opens Replace;
   - Esc in Find clears it, and a second Esc leaves the field;
   - ⌘G / ⇧⌘G go to the next and previous match;
   - all shortcuts are suppressed while the inline editor has focus, except Esc.

## Tests (required)

- Match finding: case, whole word, regex, an invalid regex, and clip-tag exclusion.
- Wrap-around navigation.
- Turns-mode filtering is unchanged.
- The replace planner: counts, skipped matches inside clip tags, and per-turn new bodies.
- The `save_many` byte tests listed above.

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

**Live, on a `/tmp/scriptcast-i` copy** of EP03 and its project files:

1. Capture find "racism" in Words mode (`plans/Native_UI/evidence/phaseI-find.png`; add capture flags `--find <q>` and `--replace <r>` if needed).
2. Run a replace-all of one word across several turns through a test or debug path.
3. Check that the copy's script diff touches only those words, that `--changes-json` lists those turns, and that restoring the checkpoint brings the original bytes back.

Record the hashes. The real EP03 script's sha256 must be unchanged at the end.

Write `plans/Native_UI/evidence/phaseI-findings.md` early and keep it current.
