# Phase A — scriptcast-ui episode browser (implementer brief)

You are implementing Phase A of a native Rust/egui desktop app for scriptCast. Your working directory is a git worktree of the scriptCast repo on branch `ui/phase-a`. Everything you need is in this worktree; read the files named below before writing code.

## Prohibitions (read first)

- **No git writes.** Do not run `git commit`, `push`, `branch`, `checkout`, `switch`, `reset`, `restore`, `clean`, `stash`, `rebase` or `merge`. The orchestrator reviews your diff and commits it. `git status` and `git diff` are fine.
- **Write only inside this worktree**, plus Cargo's own cache under `~/.cargo` and the build output in `ui/target/`. Never write, rename or delete anything anywhere else.
- **The real episode is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/` and everything in it. Read files there; never create, modify or delete anything there.
- **No Python changes.** Do not touch `scriptcast/`, `tests/` (the Python tests), `pyproject.toml`, `video/` or `swift/`.
- **Do not modify** `ui/design/**` or `ui/assets/fonts/**` (one exception in Stop conditions), or any file in `plans/` except the findings file named below.
- **No network** other than Cargo fetching crates from crates.io. Never call Figma or any other service.
- **Do not launch the GUI to take screenshots.** Visual validation is the orchestrator's job (see "Not yours to claim").

## Goal

Implement plan steps 1–8 of `plans/Native_UI/plan.md` Section 4 so that `cargo run --release -- <episode dir>` opens a read-only episode browser that matches design screens 01 and 02 using real episode data, with the Empty, Loading and Error workspace states. "Done" means the acceptance commands below pass and the code follows the plan and design. If something in the plan cannot work as written, say so in the findings file with evidence; a well-argued "this part is not viable as specified" is an acceptable outcome for that part.

## Read these, in this order

1. `ui/design/DESIGN.md`: the visual spec, the font override, window layout, components, keyboard, and the sample-data rules. **Authoritative for anything visual.**
2. `ui/design/tokens.json`: every colour, type role, radius and size. **The only allowed source of colour and size values.**
3. `ui/design/screens/01-no-episode-open/` and `02-episode-browser-idle/`: `screenshot.png` is the target, and `design-context.tsx` is Figma's React rendering. Read sizes and copy from its class names (`h-[28px]`, `gap-[12px]`, `text-[#a4abb6]`). **It is a spec, not code to port.**
4. `ui/design/components/workspace-state/`: the Empty, Loading and Error cards (ignore the Agent offline state).
5. `plans/Native_UI/plan.md`: Observations, plus Implementation Steps 1–8. **Authoritative for behaviour and data.**
6. `plans/Native_UI/tech-plan.md`: Key Architectural Decisions, Data Model.

## Established facts (measured by the orchestrator on 2026-10-02)

- The crate exists at `ui/` (`cargo new --name scriptcast-ui`, edition 2024). Current `ui/Cargo.toml` dependencies: `eframe = "0.36.2"`, `rfd = "0.17.2"`, `rodio = "0.22.2"`, `serde = { version = "1.0.229", features = ["derive"] }`, `serde_json = "1.0.151"`. `src/main.rs` is the stub. `cargo 1.96.0`. The registry has `egui_extras 0.36.2`, with features including `svg` (resvg) and `image`.
- **The eframe/egui 0.36 API has not been compiled against yet.** Check every API you use against the source in `~/.cargo/registry/src/*/egui-0.36*`, `eframe-0.36*` and `egui_extras-0.36*`. Do not trust memory of older egui versions.
- Fonts in `ui/assets/fonts/`: `OpenDyslexic-Regular.otf` and `OpenDyslexic-Bold.otf` (v0.990, OFL), and `OpenDyslexicMono-Regular.otf` (v1.001). **All three are CFF-outline OTFs.** Metrics: ascent 1300, descent -520 (units per em 1000, so the line box is about 1.82 em); average advance 815 and 844 for Regular and Bold, 726 for Mono. The text will be about 45% wider than the design's Inter. The design's weight 500 maps to Regular and 600 to Bold.
- Real test episode: `/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels/`
  - `episode_manifest.json`: schema 1.0.0, `engine: "archive, omnivoice"`, `sample_rate: 48000`, `source_file: "Architecting_the_operation/podcasts/ATO_EP03_redefining_racism_five_levels.md"` (relative to the project root), 392 turns, last `end_ms` 3861350 (64:21).
    - Speakers in order: `toussaint`/"Toussaint" (omnivoice, 124 turns), `aisha`/"Aisha" (omnivoice, 144), `emmanuel_theodore`/"Emmanuel Theodore" (omnivoice, 123), `lee_atwater`/"Lee Atwater" (archive, 1).
    - No turn has more than one segment.
    - Segment paths look like `Samples/Processed/toussaint/eda1cfb57abfc51b_chunk_0000.wav`, are relative to the episode dir, and all exist.
  - `render_state.json`: `schema_version "2.0"`, absolute `source_file` (exists), `source_hash` = SHA-256 of the script bytes, `rendered_at "2026-09-18T14:26:45.706211Z"`. **The script has changed since the render, so `script_in_sync` must come out `Some(false)` for EP03.**
  - `TURN_INDEX.csv`: header `turn_index,audio_time,script_time,speaker,clip_name,first_words`, 392 rows; `first_words` is quoted when it contains a comma, for example `1,00:08,00:08,aisha,0a8a2240770c2d72_chunk_0000,"Design a bridge, and the equations give you the load those cables"`.
  - `stress.json`: a list of 18 entries with keys `turn, repetition, manifest_turn, speaker, turn_id, score, worst_run, stress, failed, expected, heard`. **All 18 have `failed: false`**, so EP03 shows 18 Passed, 374 Not verified and 0 Failed, and the Failed-only toggle shows 0 and is disabled. Example `stress[]` item: `{"word": "produce", "reading": "default", "expected": "pɹədˈus", "contrast": "stress", "at_s": 9.66, "heard": "...", "verdict": "ok", "d_expected": 0.0, "d_other": 0.8}`.
  - The project root is `/Users/emmanuel/Documents/Theory/TheOriginalPower/` (it contains `scriptcast.toml`).
- Python dataclasses that define the manifest schema: `scriptcast/models.py:73-118`. Render state: `scriptcast/render_state.py:15-84`. Project-root rule: `scriptcast/project.py:1-40`.

## Files you own

You may create or modify only these files:
- `.gitignore`
- `ui/Cargo.toml`, `ui/Cargo.lock`
- `ui/src/main.rs`, `ui/src/lib.rs`, `ui/src/theme.rs`, `ui/src/episode.rs`, plus extra modules under `ui/src/` if they help (for example `ui/src/timeline.rs` or `ui/src/table.rs`)
- `ui/tests/**`
- the findings file `plans/Native_UI/evidence/phaseA-findings.md`

## Contract (later phases build on this)

- `ui/src/lib.rs` exports `pub mod episode; pub mod theme;`. Phases B and C will add `audio`, `runner` and `checkpoint`.
- `episode::Episode::load(dir: &Path) -> Result<Episode, LoadError>`, with the fields and the verification enum (`Passed`, `Failed`, `NotVerified`, `MissingAudio`) from plan step 4. Paths are absolute.
- `theme.rs` is the **only** place with colour literals or design sizes, as named constants that mirror `tokens.json` names. All other code uses those constants. `theme::apply(ctx)`, `theme::speaker_colour(i)`, `theme::FONT_SCALE`, and button and field-label helpers.
- Icons come from `ui/design/assets/icons/` via `egui::include_image!` with the `egui_extras` image loaders installed.

## Acceptance (run these; all must pass)

```bash
cd ui
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
```

Plus:
- `tests/episode_load.rs` covers every case in plan step 6. The fixture (plan step 5) has 3 turns and 2 speakers: turn 1 has `stress.json` passed, turn 2 has **two segments** and `stress.json` failed, turn 3 has no stress entry. It also includes one quoted `first_words` with a comma, and `render_state.source_hash` equal to the SHA-256 of `tests/fixtures/script.md`. Generate the three to four tiny WAVs (0.2 s, 48 kHz, mono, 16-bit) with a small script or test helper, and check them in.
- **A font test** proves the embedded fonts load and render: build an `egui::Context` (or epaint `Fonts`), apply the theme, lay out "Toussaint 00:19.740" in the proportional and monospace families, and assert both have positive width and that the glyphs are not the replacement glyph.
- **An ignored test** `ep03_loads` reads the env var `SCRIPTCAST_EP03` (skip if unset). It loads EP03 and asserts 392 turns, 4 speakers, `script_in_sync == Some(false)`, 18 Passed / 0 Failed / 374 NotVerified / 0 MissingAudio, and `total_ms == 3861350`, and prints the load time. Run it:

  ```bash
  SCRIPTCAST_EP03=/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels cargo test --release -- --ignored --nocapture
  ```

  Report the printed load time; the target is under 50 ms.
- No colour literal outside `ui/src/theme.rs`. The orchestrator will run `grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' ui/src | grep -v theme.rs`, which must print nothing.
- `git status` shows changes only in the files you own.

## Not yours to claim

- **Visual fidelity** against the design screenshots, live clicking and scrolling, idle CPU, and the window's look. The orchestrator launches the app and judges these. Do not claim them, and do not try to screenshot the app.
- Anything about audio playback or jobs (Phases B and C).

## Stop conditions

- **If the OpenDyslexic OTFs do not load or render in egui 0.36 (CFF outlines):** stop work on fonts, and record the exact error and the egui and epaint versions in the findings file. Then convert the three OTFs to TTF with `python3 -c "from fontTools.ttLib import TTFont; ..."` (fontTools is installed on this machine), add the `.ttf` files next to the OTFs in `ui/assets/fonts/` (this is the one permitted change there; never delete the OTFs), use the TTFs, and note it. **Never fall back silently to egui's default fonts.**
- **If an eframe 0.36 API the plan names does not exist** (persistence, `run_native`, `ViewportBuilder`, table builders): use the real API, and note the substitution in the findings file. Do not downgrade eframe.
- **If OpenDyslexic text clips the design's fixed row heights** (28 px rows, 30 px buttons): lower `theme::FONT_SCALE` in steps of 0.05, record the final value and why, and keep every layout height from `tokens.json` unchanged.

## Findings file

Create `plans/Native_UI/evidence/phaseA-findings.md` **before** you start step 8, and keep it updated. Record:
- every place the plan, DESIGN.md or this brief was wrong, ambiguous or contradictory, and what you did;
- API substitutions;
- the FONT_SCALE value;
- the EP03 load time;
- the full output summary of the acceptance commands;
- anything you left undone.

If you run low on budget, make sure this file is current first.
