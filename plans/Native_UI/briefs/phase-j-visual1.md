Visual round 1 for Phase J. Same rules as before (no git writes, no other agent CLIs, no osascript, no Python changes, mute). Branch ui/phase-j is now synced with dev. The orchestrator captured your fixture screens; compare them to ui/design/J1..J4 and fix these, nothing else:

J1 New episode:
- Header row: title left, "N turns · N speakers · 48 kHz" summary right (muted).
- Field labels sit directly above their own field; two-column grid (Script file | Episode id, Voices file | Output folder) with equal column alignment, picker "Choose…" inside the field's right end as in the mock.
- Warnings: each in its own bordered row, full width; blocking = danger border/tint with action link right-aligned; non-blocking = neutral with "Review line N" link.
- Parsed turns and Speaker summary each in their own bordered panel with a header ("Parsed turns" + "First N of M" right; "Speaker summary" + "N voices" right). The turns panel scrolls inside itself; nothing is clipped mid-row. Speaker summary has a Total row; unknown speaker row tinted danger.
- Footer: blocking reason helper text left (danger), Cancel + primary Render right-aligned.

J2: status strip gets a thin progress bar under it and a Cancel button at the right (as mock).

J3 alert: shield icon before the title; body; "Stop render" (danger outline) on its own left; Cancel and Keep rendering right-aligned on the bottom row. Fixed width ~440 logical px.

J4 pipeline:
- Every step is a full-width row of equal width (not shrink-wrapped), separated by dividers; a switch toggle (not a checkbox) left of the title; status text right-aligned in the row (✓ green complete, red counts, accent "Running · …", muted Queued).
- Header right: status dot + "Running"/"Stopped".
- Relayout row: slider fills the row width, "0 ms … 1000 ms" under it, "new length mm:ss" accent right.
- Footer: helper text left ("Original audio kept in a checkpoint…"), "Cancel run" right while running; when idle "Run pipeline" primary + Close.

Re-run your capture script into plans/Native_UI/evidence/ only if your session can launch the GUI; otherwise just make cargo fmt/clippy/test green and update phaseJ-ui-findings.md with a "Visual round 1" section. Redirect cargo output to a file.


File anchors: phase_j.rs path_field :92, new_episode_sheet :616, pipeline_sheet :878, quit_sheet :1070; main.rs status :1733; theme tokens at theme.rs :671-675. You cannot launch the GUI; the orchestrator captures. Never start other agent CLIs.
