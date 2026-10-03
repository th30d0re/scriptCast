# Phase B2 (UI) — Open in Live / Open in Logic (implementer brief)

Your working directory is a git worktree of scriptCast on branch `ui/phase-b2`, created from `dev`. Phases A and B (browser and playback) are merged and verified, and so is the Python `scriptcast-export --fcpxml` command.

## Prohibitions (read first)

- **No git writes** (commit, push, branch, checkout, reset, restore, clean, stash, rebase, merge). The orchestrator commits.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/`, and scratch under `/tmp/scriptcast-b2/`.
- **The real episodes are read-only:** never write under `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. Exercise export on a copy of the test fixture in `/tmp/scriptcast-b2/`.
- **Never launch Ableton Live or Logic Pro.** Do not run `open -b com.ableton.live …` or `open -b com.apple.logic10 …` yourself. Unit-test the argv instead; the operator clicks the real buttons.
- **No Python changes.** `scriptcast/**` and `tests/**` are done for this phase.
- **Do not modify** `ui/design/**` (read `DESIGN.md`'s section "Open in Live / Open in Logic"), `ui/assets/**`, or `plans/**` except the findings file.
- No new colour or size literals outside `ui/src/theme.rs`.

## Goal

Add plan steps B2.4 and B2.5 (`plans/Native_UI/plan.md`, section "Phase B2"):
- two header buttons, "Open in Live" and "Open in Logic";
- a small module that resolves and runs the commands behind them.

The app itself writes nothing. The `.fcpxml` is written by the export CLI.

## Established facts (measured 2026-10-03)

- Ableton Live 12 Suite is installed as `/Applications/Ableton Live 12 Suite.app`, bundle id `com.ableton.live`. Logic Pro 12.4 is at `/Applications/Logic Pro.app`, bundle id `com.apple.logic10`. Launch by bundle id with `/usr/bin/open -b <id> <file>`, which exits non-zero with "Unable to find application" if the app is missing.
- Every rendered episode has `<episode_dir>/<episode_id>.als`. `render_state.json` may also carry an absolute `als_path` (EP03: `.../ATO_EP03_five_levels/ATO_EP03_five_levels.als`). Prefer `render_state.als_path` when that file exists, else `<dir>/<episode_id>.als`. If neither exists, the Live button is disabled with the tooltip "No Ableton set (.als) in this episode".
- Logic Pro 12.4 opens `.fcpxml` directly. Its Info.plist declares `com.apple.finalcutpro.xml` with role Viewer, so a new Logic project is created on import.
- `scriptcast-export <episode_dir> --fcpxml [--out PATH]` (`scriptcast/tools/export_session.py`, merged):
  - writes `<episode_dir>/<episode_id>.fcpxml` atomically;
  - prints the output path on stdout;
  - exits 1 with a message on stderr on failure.
- **The command is not installed anywhere yet.** `which scriptcast-export` finds nothing, and the operator's installed scriptCast (`/Users/emmanuel/Documents/Theory/scriptCast`) predates it. The system `python3` (3.11) has `soundfile`, `numpy` and `yaml`, so `PYTHONPATH=<this checkout> python3 -m scriptcast.tools.export_session …` works from a source checkout.
- **The "Open in Logic" freshness rule:** the `.fcpxml` is fresh when it exists **and** its mtime is ≥ the mtime of `episode_manifest.json`. A fresh file is opened directly; otherwise the app regenerates it first.
- The app already has:
  - a persisted-preferences struct (`main.rs`);
  - a status bar;
  - secondary-button helpers in `theme.rs` (`secondary_button(ui, label, icon, enabled)`);
  - the header layout with Open and Reload;
  - the self-capture flags in `capture.rs`.
  Phase B's findings (`plans/Native_UI/evidence/phaseB-findings.md`) and Phase A's findings list the egui 0.36 API quirks.

## What to build

1. **`ui/src/cli.rs`** (shared with Phase C later), `resolve_export() -> Result<ExportCommand, String>`, resolving in order:
   1. `scriptcast-export` in the persisted **CLI folder** setting (a new preferences field `cli_dir: Option<PathBuf>`; empty means unset);
   2. `scriptcast-export` on `PATH`;
   3. **dev-checkout fallback:** walk up from `std::env::current_exe()` to the first directory containing a `pyproject.toml` whose `[project] name` is `scriptcast`, then run `python3 -m scriptcast.tools.export_session` with `PYTHONPATH=<that dir>`.

   `ExportCommand` records which source was used, `Folder | Path | DevCheckout(<dir>)`, for display. If none resolves, return `Err` with a message naming all three places checked. **No silent fallback:** the chosen source is shown in the Logic button's tooltip.
2. **`ui/src/daw.rs`**:
   - `live_target(&Episode) -> Option<PathBuf>`;
   - `live_command(&Path) -> Command` (`/usr/bin/open -b com.ableton.live <als>`);
   - `fcpxml_path(&Episode) -> PathBuf`;
   - `fcpxml_is_fresh(&Episode) -> bool`;
   - `logic_command(&Path) -> Command` (`/usr/bin/open -b com.apple.logic10 <fcpxml>`);
   - a `DawTask` that runs on a background thread. For Logic: if the file is not fresh, run the export command (stdin null, cwd = episode dir, capture stderr), then open; if fresh, open. Results go back over `std::sync::mpsc` with `ctx.request_repaint()`. The task never blocks the UI thread.
3. **Header buttons:** "Open in Live" and "Open in Logic" as secondary buttons after Reload, styled as DESIGN.md says.
   - Disabled at 45% with a tooltip when unavailable: no `.als`, or no export command **and** no fresh `.fcpxml`. A fresh `.fcpxml` still opens even with no export command.
   - While the Logic task runs, the label is "Preparing for Logic…" and the button is disabled.
   - Status bar: "Opening <file> in Live", "Writing <episode_id>.fcpxml…", "Opened in Logic", or the error in `status/failed` with the last stderr lines.
4. **Settings:** add a minimal "CLI folder" control to set `cli_dir`: a small settings popover from a gear or "CLI…" item, using an rfd folder picker, persisted. Phase C will extend it.
5. **Self-capture:** no new flags are needed. The buttons must appear in `--screenshot` captures, enabled or disabled, matching the real state.

## Acceptance (run all)

```bash
cd ui
cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs   # must print nothing
./target/release/scriptcast-ui --screenshot /tmp/scriptcast-b2/hdr-ep03.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Unit tests must cover:
- the Live target choice: `render_state.als_path` present, missing, or neither;
- the Live and Logic argv;
- the freshness rule (missing, older than the manifest, newer);
- `resolve_export` order, using temp dirs with a fake `scriptcast-export` script and a fake checkout `pyproject.toml`; never touch the real PATH;
- the dev-checkout detection, which must reject a `pyproject.toml` for another project;
- the disabled-state reasons.

**An integration test that runs the real export on a temp copy of the fixture episode.** It uses the dev-checkout fallback with this worktree as the checkout, and asserts that the `.fcpxml` is created in the copy and that the task reports success **without** launching Logic. Inject the opener as a parameter so the test can substitute a no-op.

## Not yours to claim

These are the operator's legs:
- that Live opens the set;
- that Logic imports the FCPXML;
- how the buttons feel to click.

## Stop conditions

- If the dev-checkout fallback cannot import `scriptcast` with the system `python3`, record the exact error and keep options 1–2 working. Do not add installs.
- If an egui 0.36 popover or menu API differs from what you expect, use what exists and note it.

## Findings file

Write `plans/Native_UI/evidence/phaseB2-findings.md` early and keep it current. Record:
- decisions and API notes;
- test output;
- the screenshot path;
- anything left undone.
