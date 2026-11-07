# Laya Router Implementation State

Format: Airlock `Planning/Airlock_Pipeline/STATE.md` (per-phase checklist, fixes, blockers).

## High Priority (waiting on the operator)

- [x] D1 — Stage tools are on origin/dev (v0.3-stage, v0.4-stage-loop fully merged). Local dev was 31 behind; synced 2026-11-06.
- [ ] D2 — v1.1.1 released 2026-11-06; fresh whole-book notebook `d3f54f50-61a5-465e-a94a-55d17326c0e2` (v1.1.1 PDF) replaces `58729638…`/`60b309ce…` as `book`. 36 chapter PDFs cut to the book repo's `chapters/v1.1.1/`; the book has outgrown the Ch 0–22 episode map, episode map set by operator (Ep 1 What Racism Actually Is = whole book; Ch N = Ep N+3); 27 episode notebooks built, ids in the book repo `chapters/v1.1.1/episode_map.json`. Old notebooks kept by operator choice.
- [x] D3 — M1 Max 64 GB first; design for it; move to M3 Ultra only on a measured limit (operator, 2026-11-06).

## Channel structure (operator, 2026-11-06)

Architecting the Operation is the chapter-by-chapter series; its Ep 0 premiere (What Racism Actually Is) is a whole-book preview. Specials (origins of the police, of gun control) stand outside it. Router rule for `context_source`: Ep 0 or special → `book` (whole-book notebook `d3f54f50…`); series episode → `chapter` (its notebook in the book repo's `chapters/v1.1.1/episode_map.json`). Canonical: TheOriginalPower `Architecting_the_operation/SERIES.md`.

## Planning — 2026-11-06

- Status: **plan written**; no code, no loop started.
- Inputs: four research notes; NotebookLM **Laya Decision Engine** (`79efb7c7-6f5d-4a63-8f42-736560a7d0db`); planning format from `Eocon/Partners/AEI_Speakers_Inc/Planning/Airlock_Pipeline/`.

### Measured during planning

| # | Item | Result |
|---|---|---|
| 1 | Host | Apple M1 Max, 64 GB |
| 2 | System `laya-mlx` | 0.1.0 — fails to load `aac6fef/laya-mlx` (`tokenizers 0.19.1`: untagged enum ModelWrapper) |
| 3 | `laya-mlx 0.6.0` (scratch venv) | Loads; 27–51 ms per two-question call; warns of clamped, uncalibrated temperature buckets |
| 4 | Zero-shot lane, 4 YT02-shaped tasks (`probe.py`) | 3/4 correct, confidence 0.21–0.58; TSX overlap fix → `audit` at 0.10 |
| 5 | Zero-shot follow-up `noul` | 0.09–0.20 on 4/4, including the obvious follow-up |
| 6 | `mlx-vlm` | Not installed |
| 7 | `nlm` | Authenticated; list and query work |
| 8 | Dispatch-log records | 23 |

## R0 — Prerequisites

- Status: mostly done (notebook ids in `rules.toml` still missing, blocked on D2)

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | Checkpoint commits (scriptCast, book repo) | done | dev commit `ddc8f88` |
| 2 | `.venv-router` with `laya-mlx==0.6.0`; probe latency | installed (uv, py3.11) | load 1.08 s; one forward+reversed call (3 questions) P50 323 ms, P95 431 ms over 33 cards (M1 Max, 2026-11-06) |
| 3 | `.venv-critic` with `mlx-vlm`; Qwen2.5-VL-7B 4-bit and MiniCPM-V loaded | done: Qwen load 2.5 s; MiniCPM-V 4.7 1B 3.6 s; 35B-A3B 11.5 s | M1 Max |
| 4 | `labels.jsonl` ≥ 40 labelled cards | 46 pre-filled, 33 reviewed | `python3 experiments/router_eval/review.py` |
| 5 | Notebook ids written to `rules.toml` | — | missing, blocked on D2 |

## E1 — Apple Vision OCR vs local VLM (experiments/e1_ocr_vlm)
- 30 frames, one per 25 s of `YT02_stage_0000-1232_1080p_20261105.mp4`.
- Vision (`stage-ocr --level accurate`): 9.8 s for 30 frames (~0.33 s/frame). Clean on prose and labels; misreads Π as Cyrillic in Eq. 2.8a frames (f16–f18); emits `å` on icons.
- Qwen2.5-VL-7B: 6.3 s/frame, recall 0.98, equations correct, reads dim panels. MiniCPM-V 4.7 1B: 2.1 s/frame, recall 0.92 (min 0.32), substitutes plausible wrong values. MiniCPM-V 4.7 35B-A3B: 4.8 s/frame, recall 0.98, equations correct, invents values on dim panels. Text read-back pick: Qwen. See `experiments/e1_ocr_vlm/FINDINGS-E1.md`.

## R1 — Router in shadow
- Status: **done (shadow)** (merged on dev, commit `263531d`, entrypoints `b2926cc`)
- Files: `scriptcast/router/{cue_card.py,questions.json,rules.toml,tiers.toml,laya_runner.py}`, `scriptcast/tools/stage_route.py`, `experiments/router_eval/eval.py` (output `eval-r1.json`), `tests/test_stage_route.py` (23 tests, stub laya backend).
- Laya runs as a subprocess in `.venv-router` (`laya_runner.py`, imports nothing from scriptcast) so the main venv needs no MLX; missing venv or load error → `laya: unavailable`, rules decide.
- Notebook keys resolve via `scriptcast/router/notebooks.toml` (R2) when present, else the book repo `chapters/v1.1.1/episode_map.json`. Context rule: Ep 0 premiere / special → `book`; series episode → `chapter`.
- Console entry points added: `scriptcast-stage-route` and `scriptcast-stage-context` in `pyproject.toml`.

### R1 eval — 33 reviewed labels (`python experiments/router_eval/eval.py`)

| Question | Rules acc | Laya fwd | Laya rev | Laya avg | Flip rate | Abstain | ECE | Laya acc when not abstaining |
|---|---|---|---|---|---|---|---|---|
| lane | 0.85 | 0.67 | 0.48 | 0.58 | 0.39 | 0.39 | 0.16 | 0.70 |
| tier | 1.00 | 0.64 | 0.82 | 0.76 | 0.18 | 0.18 | 0.14 | 0.78 |
| session_action | 0.94 | rules-only (D2) | | | | | | |
| context_source | 0.85 | 0.61 | 0.61 | 0.61 | 0.00 | 0.00 | 0.20 | 0.61 |

- Caveat: rules were written with these 33 labels in view, so rules accuracy is in-sample and optimistic (tier 1.00 especially). Measure again on R1 shadow-log overrides before trusting it.
- Labels are free-text cards (no owned files, defect category, live-session flag), so file-glob and session rules are barely exercised. Both session misses are `resume_builder` labels with no live-session signal in the card.
- Rules misses: lane 5 (audit vs tsx on briefs that mix both), context 5 (Stage-engine briefs labelled `chapter`).
- Laya zero-shot: lane flips under option reversal on 39% of cards (D5 abstention justified); no question near the R6 gates (acc ≥ 0.9, ECE ≤ 0.1, flip ≤ 0.05).

## R2 — Registry + JIT context
- Status: **built + step 17 run** (context arm 14% faster, equal acceptance, tokens proxy, true resume unmeasured; see `experiments/r2_measure/FINDINGS-R2.md` and `experiments/router_eval/r2_tokens.csv`)

| # | Step | Result | Notes |
|---|---|---|---|
| 13 | `stage_dispatch.py` registry | done | `<episode>/stage/.router/sessions.json`, key `<episode>/<scene>/<role>`; `start --episode --scene [--role]` registers, `resume` (session/agent/model from registry) increments, `close --summary` archives. flock + atomic write. Tokens reported, else brief-chars/4 marked `proxy`. Compaction text appended at ≥60%, final-summary request at ≥80% (status `close_due`). Context sizes from `tiers.toml` (`[context]` or `[models.X] context`) when R1 adds it, else default table in code. Every transition appends `event: registry_start/resume/close` to vault `AI Session Logs/workers/sessions.jsonl`. |
| 14 | `scriptcast-stage-context` | done | `nlm notebook query --json`; answer ≤ 1/3 budget, cited passages fill the rest of `--max-chars 1200`; source titles via `nlm notebook get`; cache `.router/context_cache/<sha256>.json`; auth error → `nlm login` hint, exit 2. |
| 14a | Notebook kinds | done | `scriptcast/router/notebooks.toml` (R2-owned; R1's `rules.toml` can reference its aliases): `book` = `d3f54f50…`, `ep1`–`ep27` from `episode_map.json` (all `book`), `persuasion` = `28147d00…` as `public_research` (web sources; fenced). Unknown ids are fenced. |
| 15 | `--context <notebook>:<query>` | done | start and resume; repeatable; fails closed (exit 2), no dispatch. |
| 16 | Tests | done | `tests/test_stage_dispatch_sessions.py` (17), `tests/test_stage_context.py` (10) with an `nlm` stub. Full suite: 503 passed, 1 skipped; 2 `test_long_speech` errors are a worktree artefact (cargo resolves the parent repo's workspace), unrelated. |
| — | Real query | ok | `--notebook book --query "How does the book define the material wage?"`: 31.6 s uncached, 1,343-char section, cached repeat instant. |
| 17 | Measurement | done | Three real YT02 fixes run from the commit their briefs were written at (Codex gpt-5.6-luna): context arm cuts wall time 8–21% (mean 14%), equal acceptance (yes/yes, n=3); tokens proxy; true resume unmeasured. Results in `experiments/router_eval/r2_tokens.csv` and `experiments/r2_measure/FINDINGS-R2.md`. |

Open: the `> 80%` path asks the worker for `.session-summary.md`; seeding the fresh session from it is a manual `close` + `start` until R4 automates it. `--no-context` (tech-plan failure-mode row) not added: dispatch without `--context` is the same thing.

## R3 — Tier 2 VLM critic
- Status: **built two-tier + element boxes, first calibration 7 bad/8 good** (merged on dev; see commits `b1fc9ad`, `682fb2d`, `12a5be6`, `cad2b93`, and `experiments/r3_calibration/FINDINGS-R3.md`, `docs/stage-marks.md`, `docs/stage-review.md`)
- First calibration results: S1 catches 6/7 at threshold 0.2; MiniCPM vetoes true flags (overturns most of S1's correct flags to pass, reducing two-tier recall); gate needs ≥20 bad labels to set authority. Element box extraction built via `StageMarks` Remotion composition (`stage boxes`, `docs/stage-marks.md`). Grounded defect reporting in `scriptcast/tools/stage_critic.py`.

## R4 — Repair
- Status: **stage_repair built report-only** (merged on dev, commit `aa903a6`; see `docs/stage-repair.md`, `scriptcast/tools/stage_repair.py`, `tests/test_stage_repair.py`)
- Reads grounded `critic.json`, produces one repair brief per failed marked element; default is report-only; optional `--dispatch` resumes builder registry session; logs before/after verdicts to `repairs.jsonl`.

## R5 — SVG offload
- Status: **condition B done, round 2 Muse rerun ranked, condition A blocked on weights** (merged on dev; see `experiments/svg_offload/FINDINGS-E3.md`)
- Condition B: inline SVG across 10 cues, 4 workers. Round 2 Muse rerun with stdout capture and author ranking in `author_rank_round2.csv`. Rule: `svg_asset` → Codex gpt-5.6-terra default, agy gemini-3.8-flash-low when cheap, Luna only behind a critic, Muse second-opinion only. Condition A (local generation via SVGen/OmniSVG) blocked on weights.

## R6 — Fine-tune and promote
- Status: **blocked (46 labels, need 150)**
- Requires ≥ 150 labelled cue cards with every lane ≥ 20 before fine-tuning and running promotion evaluation. Currently 46 pre-filled, 33 reviewed in `experiments/router_eval/labels.jsonl`.

## R7 — AI sidebar + stage view
- Status: **merged** (commits `31ab295`, `bd8c724`; see `ui/src/ai_sidebar.rs`, `ui/src/stage_view.rs`, `docs/stage-review.md`)
- Merged R7 AI sidebar and R8 stage scroll view into egui UI with two-way turn sync, Ctrl+R selection mode, and Reject button in stage view appending to `rejections.jsonl` (CLI `scriptcast-stage reject`).

---
Run records: vault `Orchestration/dispatch-log.jsonl`

## E2 — visual QA (2026-11-06)
- Done. Critic = MiniCPM-V 4.7 35B-A3B **with the approved still as reference**: recall 0.87, false-fail 0 (self-compare, optimistic), 3.7 s/frame. Qwen2.5-VL-7B passes everything; dropped as critic. Model regions unusable (0/30) → R3 grounds by element/OCR, not model boxes. Details: `experiments/e2_visual_qa/FINDINGS-E2.md`.

## E3 — inline SVG by worker (2026-11-06)
- Done (condition B). Author ranking: Terra 7 top placements, Gemini Flash 5, Luna 2, Muse 2 (from only 3 deliveries). Rule: `svg_asset` → codex gpt-5.6-terra by default, gemini-3.8-flash-low when cheap; Luna only behind a critic; Muse second-opinion only. Condition A blocked on weights. Critic schema failures 21/37 → R3 follow-up. `experiments/svg_offload/FINDINGS-E3.md`.

## E4 — System 1 visual judge (2026-11-06)
- Done, passes gate. Per-element features (SSIM, edge, palette, abs diff) + logistic regression, NumPy/PIL only (optional MLX embedding). Held-out: recall 0.958, false-fail 0.056 incl. benign re-render noise, precision 0.55, 3–7 ms/element. E2 frames: recall 0.80, localisation 0.77 (vs 0/30 for VLM regions). Plan: S1 on every frame, MiniCPM only on S1 flags. `experiments/e4_visual_s1/FINDINGS-E4.md`.
