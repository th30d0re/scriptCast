# Phase G: operator fixes (2026-10-04)

Working directory: this git worktree (branch `ui/phase-g`, from `dev`, app 0.8.0).

## Rules

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. Save tests run on `/tmp/scriptcast-g/` copies.
- **No Python changes. No synthesis.** Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Fix these (operator report, quoted)

1. **Unlimited transcript lines.** "I didn't mean only two and then truncate with ellipses. It should be however many lines need to be displayed to show the full text."
   - Remove the 2-line cap and the ellipsis in the Transcript column; rows grow to the full wrapped text.
   - Keep the measured-height cache and prefix sums.
2. **Collapsible side panel.** "We should be able to minimize the side panel. There's already a button for it, but it does nothing" (the list icon `list-15-text-muted` at the right of the filter bar, about `main.rs:1312`).
   - Make that icon toggle the inspector: hidden means the table takes the full width.
   - Persist the state in `Preferences` (`inspector_hidden`, `#[serde(default)]`).
   - Give it a tooltip: "Hide side panel" / "Show side panel".
   - Selecting a turn while hidden does **not** reopen the panel, but the Selected / Verification / Jobs buttons do.
3. **Read-along says why it is missing.** "Word highlighting is still not there, even though it's enabled."
   - Measured: EP03 has no `word_timings.json`, because the job was never run, so there is nothing to highlight.
   - When read-along is on and a turn plays without timings, show one line in the inspector's Turn view: "No read-along timings yet. Run Jobs → Read-along timings." With a "Run…" button that opens the job's confirmation.
   - When the timings are stale, show "Read-along timings are out of date (episode re-rendered). Run Jobs → Read-along timings."
4. **Settings window.** "There's not a way to scroll anymore in settings, and I can't get out of the settings menu … I have to quit the app. And I don't have the option to select colors for each speaker."
   - Likely cause: the window in `settings.rs` is anchored `CENTER_CENTER` and not resizable, with no `ScrollArea`. When its content outgrows the window, the title bar (with its close ×) and the lower rows (the speaker colours) end up off-screen.
   - Fix:
     - wrap the content in `egui::ScrollArea::vertical()` with `max_height` set to 80% of the screen height;
     - make sure the title bar and close × are always on-screen;
     - add an explicit **Done** button at the bottom;
     - **Esc** closes the window (verify that it works);
     - so does clicking outside it, using a scrim like the confirmation dialog, or `egui::Modal` if that suits.
   - Verify the speaker colour pickers are visible and usable: one per speaker in the open episode, labelled with the speaker's name. EP03 has 4 speakers; if an episode has more than 4, cycle the palette but still list each speaker.
5. **Audio clips (real recordings).** "I shouldn't be able to edit a clip that's not generated with AI … actually we should be able to edit what it says to make sure it says correctly, but that shouldn't cause any regeneration. We should note that and display that."
   - A turn is an audio clip when its script text contains a `[clip:<id>]` tag. EP03's turn 220 is `[clip:atwater_1981]`.
   - Follow the new Figma frames, saved locally:
     - `ui/design/screens/09-audio-clips/imported-audio-turn.png`;
     - the "Audio clip badge", Figma `48:139`: fill `#2c3037`, 1 px border-default, radius 4, padding 7, gap 6, the 13 px `file-audio` icon (already saved as `ui/design/assets/icons/file-audio-13-text.svg`; register it in `index.json` and `theme::icon`), and "Audio clip" in 11 px text `#c4cbd5`. Add those colours as tokens.
   - **In the Transcript cell:** the badge, then the clip id in mono muted text, above the transcript text.
   - **Playback bar**, while a clip plays: "Original recording · Not synthesized" at the right of the waveform area.
   - **Editing a clip turn is allowed** (transcript correction). The editor shows the note: "Audio clip: this corrects the transcript only. The original recording is not regenerated."
   - **The clip tag itself must be preserved:** if the edited body no longer contains the original `[clip:…]` tag, refuse the save with a clear message.
   - **Exclude clip turns from "Re-render edited turns":**
     - the confirmation lists them separately as "Transcript corrections (no synthesis)";
     - if **only** clip turns changed, the re-render button is disabled, with the tooltip "Only audio-clip transcript corrections changed; nothing to synthesize."
     - Note in the findings: a later Precision insert still sees those turns as changed, because the turn identity includes the text. Re-cutting a clip from the original recording is not synthesis. The orchestrator will raise the engine change with the operator.

## Tests (required)

- Row heights with 1, 3 and 6-line texts.
- Inspector toggle persistence.
- Settings: a content height larger than the viewport still lays out with the close control on screen. Test the layout helper, not the pixels.
- Clip detection from body text.
- Save refused when the clip tag is removed.
- Re-render scope split (synthesis turns vs clip corrections), including the clip-only disabled case.

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

Captures on real EP03 (read-only), muted, into `plans/Native_UI/evidence/phaseG-*.png`:

- the table around turn 017 (long text, many lines);
- `--select 220` (the audio clip);
- `--settings` at the default window size;
- the inspector hidden.

Look at each capture and describe it honestly.

Write `plans/Native_UI/evidence/phaseG-findings.md` early and keep it current.
