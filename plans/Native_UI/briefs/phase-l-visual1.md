Phase L visual round 1 (same rules: no git writes, never start other agent CLIs, no osascript, no Python changes). Captured on SCRIPTCAST_PROJECT=/tmp/scriptcast-k/proj (a copy of the real project; it has .venv-l with scriptcast-project) and compared to ui/design/L1-L4. Fix exactly these:

L1 project picker
1. The theme is not applied: grey default egui background, default buttons, no app header. Use the app theme, the standard header (logo, project name + root path, "Project settings" secondary and "New episode…" primary on the right), the two-column layout with bordered panels from L1, the recent-project cards (selected card highlighted), and status chips (Rendered green, Interrupted render red, Not rendered muted) in the table.
2. "Last rendered" shows a raw ISO timestamp; format as "05 Oct 2026 10:32" (two lines in the cell as in L1 is fine).
3. A phantom row "ATO_EP04_what_power_is · Not rendered" appears although that script IS rendered as ATO_EP04. Match scripts to outputs by each manifest's `source_file` (resolve relative to the project root), not by file stem.
4. Add the selected-episode summary card under the table and the "Double-click an episode row to open it." helper, as in L1.

L2–L4 project data
5. It is drawn as a small collapsible egui Window (collapse arrow, title bar) overlapping the episode. Per L2–L4 it is a full-window view: app header (project name + root, "Project settings"), a tab bar (Voices · Clips · Pronunciations · Accepted hearings · Speaker rates) with the active tab as an accent pill, left list panel + right detail panel, and a footer bar with the status/reason text on the left and Revert + primary Save "<file name>" on the right. Close returns to the episode (Esc or a Close button in the header).
6. Voices: field labels are raw keys ("reference audio", "character profile"); use the mock's labels ("Reference audio", "Reference text · required", "Character profile · optional", "Speed"). Engine chip next to each speaker in the list (OmniVoice accent chip, archive muted chip) plus turn count right-aligned, header "Speakers  N · M invalid". Speed slider full width with "0.5 … 1.5" under it and "1.0 · default" right. Reference audio row: path field + Choose… + Play with duration, on one row.
7. Clips: the source-file column wraps into very tall rows. Single-line, truncated with … (full path in the detail panel). Rows ~44 px like L3. Search + "+ Register clip…" on one row.
8. Clips detail: "Cannot decode source audio … isomp4: packet out-of-bounds" for .mp4 sources (the registry's real sources are .mp4/.m4a). Decode clip sources through ffmpeg (`ffmpeg -v error -ss <start-pad> -t <len> -i <src> -f wav -ac 1 -ar 48000 -` into memory) when the native decoder fails; use that for both the waveform and Play (and for the Phase K clip picker's Play). If ffmpeg is missing, show the visible note.
9. Clips detail: "Used in:" is empty for jalachantelle_2026a although EP04 turns 019 and 021 contain [clip:jalachantelle_2026a]. Fix the lookup (use the loaded episode's turn bodies / clip ids) and render it as the bordered list from L3 ("019  ATO_EP04 · Jala Chantelle").
10. Accepted hearings: table with headers Script word / Heard as / Review date / Review risk; date formatted "05 Oct 2026"; Delete as a danger-outline button right-aligned.
11. Settings → Project: each path field with its Choose… button on the same row (field fills, button right), as in L4.

Then cargo fmt/clippy/test green (output to a file), and add "Visual round 1" to phaseL-ui-findings.md.
