# Phase B2 UI findings

Model: GPT-6 (Codex). Started 2026-10-03. Scope: B2.4–B2.5.

## Requested work and constraints

Add Open in Live / Open in Logic, command resolution, persisted CLI folder,
background export/open tasks, and tests. No git writes, Python changes, real
episode writes, or DAW launches. Export tests use copies under
`/tmp/scriptcast-b2/`. This file also serves as the session log; the user's
write boundary excludes the usual external session-log location.

## Decisions and API notes

- Reuse `theme::secondary_button` (including 45% disabled opacity), no DAW icons.
- Reuse the existing egui 0.36 `Popup::menu(&response).show(...)` pattern.
- `resolve_export` accepts `Option<&Path>` for the persisted CLI folder. This
  small signature adjustment keeps preferences owned by `main.rs`, avoids
  mutable global settings, and allows isolated resolver tests with injected
  PATH and executable location.
- Native Rust/egui desktop app: self-capture is the verification surface; no
  simulator, Chromium, or mobile interaction is involved.
- Export discovery is cached until an episode loads, Reload is pressed, or the
  CLI folder changes. The tooltip identifies Folder, PATH, or DevCheckout and
  its actual path. A missing configured executable may fall through to PATH
  or the checkout, with the selected source shown explicitly.
- `render_state.als_path` is retained on `Episode`; an existing file there
  takes priority. Missing/absent paths fall back to the episode-named set.
- Tasks capture the selected episode paths and export command. Process launch,
  export, and waiting happen on worker threads; mpsc events request repaint.
  Freshness is checked again by the worker, including after successful export.
- Export cwd is the episode directory as specified in this implementer brief
  (the older plan's project-root wording is superseded). Stdin is null.
- `PYTHONDONTWRITEBYTECODE=1` prevents the source fallback from writing Python
  bytecode into the checkout. `PYTHONPATH` is set only on the child command.
- Failed commands retain the last six stderr lines. The status bar uses
  `STATUS_FAILED`, with full multiline details in its hover tooltip.

## Implementation and validation

Implemented `cli.rs`, `daw.rs`, header controls, CLI folder picker/clear popup,
persisted settings, and task status. The loader now retains optional
`render_state.als_path`. No dependencies or Python files changed.

Final acceptance on 2026-10-03:

```text
cargo build --release
    Finished `release` profile [optimized] target(s) in 27.65s
cargo test
    lib:          4 passed, 1 ignored
    bin:         31 passed
    daw_cli:     16 passed
    episode_load:15 passed, 1 ignored
    fonts:        1 passed
    total:       67 passed, 0 failed, 2 existing ignored
cargo clippy --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.68s
colour grep outside theme.rs
    no output (grep exits 1 for no matches)
git diff --check
    no output
```

The two pre-existing ignored tests are real EP03 loading and device-dependent
muted audio playback. All new tests run by default.

The real export integration test resolves DevCheckout at this worktree, invokes
`python3 -m scriptcast.tools.export_session` on a copied fixture with spaces in
its directory name under `/tmp/scriptcast-b2/`, verifies the generated FCPXML
and mono channel metadata, and receives `Writing episode_small.fcpxml…` then
`Opened in Logic`. Its injected opener is `/usr/bin/true`; Logic is never
launched. The system Python import worked; no installs were needed.

Additional coverage: exact DAW argv (construction only); source priority and
wrong-project rejection without touching process PATH; absent/older/equal/newer
FCPXML mtimes; disabled reasons; existing fresh XML with no exporter; export
failure stderr tail; successful exit without output; simulated missing app;
background execution; preferences round-trip/backward compatibility; UI
progress, completion and disconnected-worker recovery.

No egui API substitutions were needed. `Popup::menu` and
`Response::on_disabled_hover_text` compile and use the existing theme. Existing
headless layout checks also pass at default and minimum window sizes.

## Screenshot

- `/tmp/scriptcast-b2/hdr-ep03.png` — requested EP03 capture, exit 0,
  2880×1800. Visually inspected: Open, Reload, Open in Live, Open in Logic,
  CLI… appear in order; both DAW buttons are enabled in the real episode state.
- `/tmp/scriptcast-b2/hdr-empty.png` — `--fresh --screenshot` with no episode,
  exit 0, 2880×1800; both DAW controls visibly disabled.

Self-capture did not click either DAW button and did not export into EP03.

## Remaining work

Implementation and automated acceptance complete. No git writes, DAW launches,
Python source edits, or writes to real episodes were made. Operator alone
verifies Live set launch, Logic import, and click feel (including folder picker
interaction). No claim is made about these operator legs.

## Next ideas (outside this phase)

1. Reuse export command resolution in Phase C.
2. Extend the CLI settings popover for other tools.
3. Operator checks Live set launch.
4. Operator checks Logic FCPXML import.
5. Operator checks missing-application feedback.
6. Operator checks controls at their preferred window size.
