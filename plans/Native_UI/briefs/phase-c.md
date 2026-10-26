# Phase C — Job runner, checkpoints, verify overlay (implementer brief)

Your working directory is a git worktree of scriptCast on branch `ui/phase-c`, created from `dev`. Phases A (browser), B (playback) and B2 (Open in Live / Open in Logic) are merged and verified. The operator has listened to EP03 in the app and clicked both DAW buttons.

## Prohibitions (read first)

- **No git writes** (commit, push, branch, checkout, switch, reset, restore, clean, stash, rebase, merge). The orchestrator reviews and commits. `git status` and `git diff` are fine.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/`, and scratch under `/tmp/scriptcast-c/` (create it).
- **Real episodes are read-only.** Never write anything under `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. That includes its `outputs/`, its scripts, its `.venv*` folders and its `scriptcast.toml`. Running the project's installed commands from there is allowed (Detect changes and Verify, against the clone only); Python bytecode caches they write incidentally are fine. You must not create, edit or delete files there yourself. Live checks run on a clone made with `cp -cR` under `/tmp/scriptcast-c/` (details under "Live checks").
- **Never run a synthesizing job:** no Precision insert, Regenerate selected or Repair against anything, including the scratch clone. They run OmniVoice for minutes on the GPU and are the operator's to trigger. Unit-test their argv instead.
- **No Python changes:** `scriptcast/`, `tests/`, `pyproject.toml`, `video/`, `swift/`.
- **Do not modify** `ui/design/**` or `ui/assets/**`. Under `plans/**`, edit only the findings file named at the end.
- **No new colour or size literals outside `ui/src/theme.rs`.** Add new tokens there and name them for what they are.
- **Captures are muted.** Never call Figma. Never launch Live or Logic.

## Goal

Implement plan steps C0 and 11–17 of `plans/Native_UI/plan.md` Section 4, building to design screens 04, 05 and 06. The done state:

- a "Jobs" button and ⌘J open the Job runner;
- five presets build their exact commands from the episode's metadata;
- anything that synthesizes or overwrites asks for confirmation, showing the exact command;
- a checkpoint is saved before such a job starts;
- the log streams live;
- Cancel works;
- the episode reloads when the job ends;
- checkpoints can be listed and restored;
- the Verification inspector shows the real verifier output for the selected turn.

If part of the plan is not viable as written, say so in the findings file with evidence. A well-supported "this does not work" is an acceptable result.

## Read these, in order

1. `plans/Native_UI/plan.md`: Flows 3, 3b, 4 and 5; steps C0 and 11–17; Flow 6 and steps D1–D6. Phase D is **not yours**, but your checkpoint and runner APIs must not block it (see "Leave room for Phase D").
2. `plans/Native_UI/tech-plan.md`: decisions, failure modes, and the Runner and Checkpoints components.
3. `ui/design/DESIGN.md`: the Inspector, Confirmation dialog, Checkpoints, Keyboard and sample-data sections, and the font override.
4. `ui/design/screens/04-verification-review/`, `05-job-runner-active/`, `06-confirm-audio-replacement/`. Each `screenshot.png` is the target; `design-context.tsx` gives exact sizes and copy as a spec, not code to port. Also `ui/design/components/workspace-state/`, for the CLI unavailable card.
5. `plans/Native_UI/evidence/phaseA-findings.md`, `phaseB-findings.md`, `phaseB2-findings.md`. They record the egui 0.36 API quirks already learned: `App::ui` replaces `update`, `Ui::put` advances the cursor, `Popup::menu(&response)`, headless test texture cleanup, measured-width helpers, the rodio 0.22 notes.
6. The existing code: `ui/src/{main,lib,theme,episode,table,timeline,view,capture,audio,inspector,cli,daw}.rs` and `ui/tests/`.

## Established facts (measured 2026-10-03 by the orchestrator; raw output quoted)

**1. No scriptCast command is on any PATH.**

```
$ for c in scriptcast scriptcast-verify scriptcast-repair scriptcast-export; do command -v $c || echo "$c (not on PATH)"; done
scriptcast (not on PATH)   scriptcast-verify (not on PATH)   scriptcast-repair (not on PATH)   scriptcast-export (not on PATH)
```

**2. The operator's real install is a project virtualenv**, an editable install of `/Users/emmanuel/Documents/Theory/scriptCast`.

```
$ ls /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin | grep scriptcast
scriptcast  scriptcast-repair  scriptcast-verify  scriptcast-mix  ... (20 entries; no scriptcast-export)
$ cat .venv-voice/lib/python3*/site-packages/scriptcast-0.1.0.dist-info/direct_url.json
{"dir_info": {"editable": true}, "url": "file:///Users/emmanuel/Documents/Theory/scriptCast"}
$ head -1 .venv-voice/bin/scriptcast
#!/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python3.11
```

The same project also has `.venv`, `.venv-fermion` and `.venv-omnivoice`, and **none of them contains scriptcast commands**. Rule C0.2 ("first `.venv*` folder that contains the command") therefore picks `.venv-voice`. Do not pick "the first venv".

**3. Only `.venv-voice` has the verifier's model stack.**

```
$ /Library/Frameworks/Python.framework/Versions/3.11/bin/python3 -c "import mlx_whisper"
ModuleNotFoundError: No module named 'mlx_whisper'
```

`.venv-voice/lib/.../site-packages/mlx_whisper` exists.

**4. An app opened from Finder or the Dock gets launchd's minimal PATH**, so `python3` resolves to Xcode's 3.9, which lacks `soundfile`.

```
$ launchctl getenv PATH
(empty)
$ /usr/bin/python3 --version
Python 3.9.6
$ PYTHONPATH=<checkout> /usr/bin/python3 -m scriptcast.tools.export_session --help
ModuleNotFoundError: No module named 'soundfile'
```

The operator's current instance works only because it was started with `open` from a terminal and inherited the shell `PATH` (checked with `ps eww`). Step C0 fixes this for every command, including B2's export.

**5. The CLI flags in plan step 11 match the real parsers** (`scriptcast/__main__.py`, `tools/verify_render.py`, `tools/repair_render.py`):

```
scriptcast:        --transcript --episode-id --out-dir ... --overwrite ... --regenerate-turns --detect-changes --precision-insert ... --voices
scriptcast-verify: episode_dir --transcript --model --threshold --max-run --turns --json-out --no-stress
scriptcast-repair: episode_dir --transcript --episode-id --voices (required) --out-dir --gap-ms --model --threshold --max-run --max-passes --skip-relayout --seed-json --turns
```

- A bare `scriptcast` render prompts with `input()` when output exists. `--overwrite` deletes the episode. The UI must never emit either. All jobs run with stdin null.
- `--detect-changes` returns before any synthesis and before `_prepare_episode_output` (`scriptcast/__main__.py:907-913`).

**6. EP03's script is out of sync with its render.** It was modified at 12:53 on 2026-10-03, and the app's status bar shows "Script changed since render". So `script_in_sync == Some(false)` on EP03, and Regenerate selected and Repair-with-turns must be **disabled** there with the plan's tooltip. That is correct behaviour, not a bug. Use the fixture, where the script is in sync, to show the enabled state.

**7. `stress.json` in EP03 is stale.** Its verifier run predates the 2026-09-18 re-render, so 5 of its turn_ids no longer exist. The loader already reports this as a header warning. Expected counts: 13 Passed / 0 Failed / 379 Not verified / 0 Missing audio.

**8. Existing settings.** `cli.rs` already resolves `scriptcast-export` (Folder → PATH → dev checkout). The persisted `preferences.cli_dir` is set from the header's settings gear popover; this is the plan's "bin-dir" setting. Generalise it (C0); do not add a second folder setting.

**9. Existing delete call.** The only `remove_file` in `ui/src` is in `capture.rs`'s test module (line ~385). Non-test code must stay free of `remove_file` and `remove_dir*`.

## What to build (deltas from plan.md; the plan steps are the spec)

- **C0. `cli.rs`.**
  - Implement `resolve(command, &Episode-or-project-root, cli_dir) -> Result<Resolved, String>` with the four-step order in plan step C0, plus a `LoginPath` loaded once at startup on a background thread (`$SHELL -lc 'printf %s "$PATH"'`, 2 s timeout, falling back to the process `PATH`). Every child process gets that `PATH` in its environment.
  - Keep `resolve_export` working through the new resolver (B2's tests must still pass).
  - The module for the dev-checkout fallback comes from `[project.scripts]` in the checkout's `pyproject.toml`. The interpreter is the first project `.venv*/bin/python` that **has** the module's dependencies, or else `python3` on the login `PATH`. Probing a venv's interpreter with a short `-c "import scriptcast"` is acceptable if you need it; record what you chose.
  - Tooltips and the Job runner show which source each command resolved to.
  - The settings popover's caption becomes "Folder with scriptCast commands".
- **11. `runner.rs`.** As in plan step 11. `JobSpec::from_preset` takes the resolved commands, not a raw `bin_dir`. The working directory is the project root. stdin is null. The login `PATH` is set.
- **12. `checkpoint.rs`.** As in plan step 12. Time a Precision-insert-scope `create` on the scratch clone of EP03 (call it from a test or a small debug binary, never through the CLI) and record the time in the findings.
- **13. `episode.rs`.** As in plan step 13.
- **14. `main.rs` and modules.** As in plan step 14. The Jobs button sits at the right of the filter bar; ⌘J also opens it. Multi-select uses Shift-click and ⌘-click. The CLI unavailable state's "Configure CLI" opens the settings popover.
- **15–17.** Tests and `SPEC.md` as in the plan.
- **Self-capture.** Add flags so the orchestrator can check screens 04–06 without controlling the UI:
  - `--inspector <selected|verification|jobs>`;
  - `--confirm <preset>`, which opens the confirmation dialog and never confirms;
  - `--run <detect-changes|verify>`, which confirms and runs, then captures after the job exits. `--run` must **refuse** every synthesizing preset.
  - Captures stay muted.

### Leave room for Phase D

- `checkpoint::create` must be driven by an explicit file list: `plan_files` produces the list, and `create` takes it. Phase D will add `create_external` for files outside the episode, under `external/`, so do not hard-code "relative to episode" deep inside the copy loop.
- The PrecisionInsert confirmation must accept an optional, caller-supplied list of affected turns, because Phase D will pass `--changes-json` output. For Phase C, the scope line reads "Whole episode: only changed turns are synthesized".

## Acceptance (run all; paste the outputs into the findings)

```bash
cd ui
cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs        # must print nothing
grep -rn '"--overwrite"' src                                                                # must print nothing
grep -rn 'remove_file\|remove_dir' src                                                       # only capture.rs's test module
```

Tests must cover everything in plan steps 15 and 16, plus:

- the C0 resolution order, using temp dirs with fake commands and a fake project with `.venv-a/` (no command) and `.venv-b/bin/<command>`, which must pick `.venv-b`. Never touch the real `PATH`. Inject the login `PATH`.
- the login-PATH reader with a fake `$SHELL` script, including a timeout case;
- `--run` refusing synthesizing presets.

## Live checks (scratch clone only)

```bash
mkdir -p /tmp/scriptcast-c/outputs
cp -cR /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels /tmp/scriptcast-c/outputs/
shasum -a 256 /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels/{episode_manifest.json,render_state.json,stress.json} > /tmp/scriptcast-c/real-before.sha
```

- Point the clone at the real project with `SCRIPTCAST_PROJECT=/Users/emmanuel/Documents/Theory/TheOriginalPower`. The loader already honours it. The job's `--out-dir` is the clone's parent, so the CLI writes only into `/tmp/scriptcast-c/outputs/`.
- **Detect changes** on the clone. It streams, exits, and needs no confirmation or checkpoint. Capture `phaseC-05-detect.png`.
- **Verify** on the clone, through `--run verify`. Check that:
  1. a checkpoint folder with `checkpoint.json` and the old `stress.json` exists before the CLI's first output line (log timestamps or a test hook);
  2. the Verification column updates after the reload.

  Capture `phaseC-05-verify.png` while it runs and `phaseC-04.png` after it, with a failed turn selected if any.

  If Verify cannot run from your environment (GPU, model download, sandbox), record the exact error and stop that leg. The orchestrator will run it.
- **Restore** the Verify checkpoint and check that `stress.json` is back byte for byte and that a `pre-restore` checkpoint exists.
- Capture `phaseC-06.png` with `--confirm verify`. Also show the PrecisionInsert confirmation on the fixture (where it is enabled), captured with `--confirm precision-insert`. Never confirm it.
- At the end, re-hash the real files and diff against `real-before.sha`. It must be identical. Also run `git -C /Users/emmanuel/Documents/Theory/TheOriginalPower status --porcelain` before and after; the output must be identical. It already shows ` M ...ATO_EP03_redefining_racism_five_levels.md`, the operator's own edit.

Save captures to `plans/Native_UI/evidence/phaseC-*.png`.

## Not yours to claim

These are the operator's or orchestrator's legs:

- running Precision insert, Regenerate selected or Repair for real;
- how the dialogs feel to click;
- launching the bundled `.app` from Finder to confirm the login-PATH fix.

## Stop conditions

- If the login-shell PATH read hangs or prints shell noise (for example an `.zshrc` that echoes), keep the timeout, take only the last line, and record what you saw.
- If a preset's argv cannot be built from the files present (for example a missing `voices` key in `scriptcast.toml`), disable that preset with a tooltip naming the file and key. Never guess a path.
- If an egui 0.36 API differs from what the plan assumes, use what exists and note it.
- If checkpoint creation of the Precision-insert scope on the EP03 clone takes longer than 5 s, record the timing and the volume type (`diskutil info /tmp | grep -i 'file system'`) and continue.

## Findings file

Write `plans/Native_UI/evidence/phaseC-findings.md` early and keep it current. Record:

- decisions and API notes;
- the command-resolution results on this machine (which source each of the four commands resolved to);
- the checkpoint timing;
- the acceptance output;
- the live-check results, with hashes;
- the capture paths;
- anything left undone.
