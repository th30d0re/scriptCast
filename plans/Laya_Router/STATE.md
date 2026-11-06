# Laya Router Implementation State

Format: Airlock `Planning/Airlock_Pipeline/STATE.md` (per-phase checklist, fixes, blockers).

## High Priority (waiting on the operator)

- [x] D1 — Stage tools are on origin/dev (v0.3-stage, v0.4-stage-loop fully merged). Local dev was 31 behind; synced 2026-11-06.
- [ ] D2 — Proposed: keep book `58729638…` (43 sources), Ep 2 `f9055233…`, Ep 3 `e1e23897…`; delete `60b309ce…` (1 source, older), `c1eb4028…`, `548ad34f…` (TEST, 0 sources), `3593a8cb…`. Duplicates carry the same chapter PDF. Awaiting delete approval.
- [x] D3 — M1 Max 64 GB first; design for it; move to M3 Ultra only on a measured limit (operator, 2026-11-06).

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

- Status: not started

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | Checkpoint commits (scriptCast, book repo) | — | |
| 2 | `.venv-router` with `laya-mlx==0.6.0`; probe latency | installed (uv, py3.11) | latency pending |
| 3 | `.venv-critic` with `mlx-vlm`; Qwen2.5-VL-7B 4-bit and MiniCPM-V loaded | done: Qwen load 2.5 s; MiniCPM-V 4.7 1B 3.6 s; 35B-A3B 11.5 s | M1 Max |
| 4 | `labels.jsonl` ≥ 40 labelled cards | 46 pre-filled, 0 reviewed | `python3 experiments/router_eval/review.py` |
| 5 | Notebook ids written to `rules.toml` | — | blocked on D2 |

## E1 — Apple Vision OCR vs local VLM (experiments/e1_ocr_vlm)
- 30 frames, one per 25 s of `YT02_stage_0000-1232_1080p_20261105.mp4`.
- Vision (`stage-ocr --level accurate`): 9.8 s for 30 frames (~0.33 s/frame). Clean on prose and labels; misreads Π as Cyrillic in Eq. 2.8a frames (f16–f18); emits `å` on icons.
- Qwen2.5-VL-7B: 6.3 s/frame, recall 0.98, equations correct, reads dim panels. MiniCPM-V 4.7 1B: 2.1 s/frame, recall 0.92 (min 0.32), substitutes plausible wrong values. MiniCPM-V 4.7 35B-A3B: 4.8 s/frame, recall 0.98, equations correct, invents values on dim panels. Text read-back pick: Qwen. See `experiments/e1_ocr_vlm/FINDINGS-E1.md`.

## R1 — Router in shadow
- Status: blocked on R0

## R2 — Registry + JIT context
- Status: blocked on R1

## R3 — Tier 2 VLM critic
- Status: blocked on R0

## R4 — Repair
- Status: blocked on R2, R3

## R5 — SVG offload
- Status: blocked on R3

## R6 — Fine-tune and promote
- Status: blocked on labels (≥ 150, each lane ≥ 20)

---
Run records: vault `Orchestration/dispatch-log.jsonl`
