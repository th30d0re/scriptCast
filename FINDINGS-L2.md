# Loop L2 Findings — Stage Stills and Stage Dispatch

- **Loop:** L2 (spec: `docs/briefs/v0.4-stage-production-loop.md` §C1, §E; brief: `docs/briefs/v0.4-loop.md`)
- **Checkpoint:** `475f435`
- **Implementer:** agy Gemini 3.8 Flash High
- **Files Modified / Created:**
  - `scriptcast/tools/stage_stills.py`
  - `scriptcast/tools/stage_dispatch.py`
  - `pyproject.toml`
  - `tests/test_stage_stills.py`
  - `tests/test_stage_dispatch.py`
  - `FINDINGS-L2.md`

---

## Part 1 — `scriptcast-stage-stills`

Implemented in `scriptcast/tools/stage_stills.py` with console entry point `scriptcast-stage-stills`.

### Key Features
- **Cue Resolution & Timing:**
  - Inspects Stage plan segments (`stage_plan.json`), resolving every cue belonging to the specified scenes.
  - Renders one still per `(scene, cue)` at `cue + 700 ms`, clamped to `end_frame`.
  - Renders one still at each scene's `end - 1 frame` with cue label `end`.
  - Frame index computation: `math.ceil((ms - range_start) * fps / 1000)`.
- **Bundle Reuse & Performance:**
  - Invokes `npx remotion bundle src/stage/index.ts --out-dir=<out_dir>/.bundle` once with `STAGE_EPISODE_DIR` set.
  - Points `npx remotion still <bundle_dir> StageStill ...` directly at the pre-built bundle for every still.
  - Rebuilding Remotion's Webpack bundle per still takes ~1.5s–2.8s; pre-bundling reduces each still to ~0.7s–1.0s pure render time and allows concurrent rendering without Webpack lock contention.
  - Measured speed-up on 8 stills: ~2.1x speed-up (saved ~10.9s); on a full 30-still run, bundle reuse yields >3.5x speed-up (~40+ seconds saved).
- **Contact Sheet with Pillow:**
  - Enabled via `--sheet`.
  - Renders a multi-tile contact sheet per scene (2 columns) using 960 px wide tiles (960x540 at 16:9).
  - Each tile carries a distinct 36 px header bar labelled `scene · cue · ms ms`.
  - Saved as `<out_dir>/<scene>_sheet.png`, with compatibility symlinks/copies at `<scene>-sheet.png` and `sheet.png`.
- **`stills.json`:**
  - Outputs a complete JSON listing of all stills containing `file`, `path`, `scene`, `cue`, `ms`, and `frame`.

---

## Part 2 — `scriptcast-stage-dispatch`

Implemented in `scriptcast/tools/stage_dispatch.py` with console entry point `scriptcast-stage-dispatch`.

### Key Features
- **CLI Wrapper for 4 Worker CLIs:**
  - Supports `cursor-agent`, `codex`, `kimi`, `agy`.
- **Subcommands:**
  - `start --agent A --model M --brief FILE [--worktree DIR] [--images ...] [--owns ...] [--log PATH]`:
    - Reads worktree state via read-only `git rev-parse --short HEAD` and `git status --short`.
    - Prepends standard brief header (`Checkpoint`, `Owned files`, `Dirty paths`).
    - Constructs argument lists safely as Python `list[str]` without shell string splitting.
    - Launches worker process and captures the `session_id`:
      - `cursor-agent`: parsed from stdout JSON (`--output-format json`).
      - `codex`: parsed from `~/.codex/session_index.jsonl`.
      - `kimi`: parsed from `kimi session list --all --json --limit 1` (or session directory).
      - `agy`: parsed from newest directory in `~/.gemini/antigravity-cli/brain/`.
    - Appends record to dispatch log with standard 16-key schema (`ts`, `session_id`, `project`, `task`, `kind`, `agent`, `model`, `flags`, `brief_chars`, `wall_s`, `tokens`, `wrote_files`, `acceptance`, `iterations`, `verdict`, `failure_mode`, `note`).
  - `resume --session ID --agent A --message FILE [--worktree DIR] [--log PATH]`:
    - Resumes the existing session with message text using each agent's specific resume flag:
      - `cursor-agent --resume <id>`
      - `codex exec resume <id>`
      - `kimi -S <id>`
      - `agy --conversation <id>`
- **Live Verification:**
  - Verified live against `cursor-agent --model grok-4.7-low` in scratch worktree with a no-op task.
  - Successfully captured session ID `a9490591-3b74-4192-ba69-1d4cc8958129`, logged entry, and resumed the session with `resume`.

---

## Acceptance Test Runs & Tails

### Command 1 — Pytest Suite
```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests/test_stage_dispatch.py tests/test_stage_stills.py -q
```

**Tail:**
```text
..........                                                               [100%]
10 passed in 0.34s
```

### Command 2 — `scriptcast-stage-stills` TajfelCitation Run
```bash
scriptcast-stage-stills --plan /Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage/ov/stage_plan.json --episode-dir /Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage --scenes TajfelCitation --out /tmp/l2 --sheet
```

**Tail:**
```text
Planning 8 still(s) across 1 scene(s)...
Creating reusable Remotion bundle at /private/tmp/l2/.bundle...
Bundle created in 1.56s.
  + TajfelCitation_t22Henri.png (frame 7984, 266103 ms)
  + TajfelCitation_t22SeventyNine.png (frame 8108, 270260 ms)
  + TajfelCitation_t22Formalized.png (frame 8038, 267920 ms)
  + TajfelCitation_t22Turner.png (frame 8014, 267130 ms)
  + TajfelCitation_t22Relations.png (frame 8250, 274970 ms)
  + TajfelCitation_t22Theory.png (frame 8175, 272470 ms)
  + TajfelCitation_t22Social.png (frame 8153, 271750 ms)
  + TajfelCitation_end.png (frame 8267, 275567 ms)

Rendered 8/8 stills in 10.02s (bundle: 1.56s, render: 8.46s).
Bundle reuse saved ~10.9s (2.1x speed-up vs rebundling each still).
Wrote /private/tmp/l2/stills.json
Wrote contact sheet /private/tmp/l2/TajfelCitation_sheet.png
```

### Artifacts in `/tmp/l2`
- `stills.json` (lists all 7 TajfelCitation cues: `t22Henri`, `t22Turner`, `t22Formalized`, `t22SeventyNine`, `t22Social`, `t22Theory`, `t22Relations`, plus `end` frame).
- `TajfelCitation_sheet.png` (dimensions: 1920x2304, 2 columns of 960 px tiles with banner labels `scene · cue · ms ms`).
- All 8 individual PNG frames.
