# E2 findings: visual QA, Qwen2.5-VL-7B vs MiniCPM-V 4.7 35B-A3B (2026-11-06, M1 Max)

Set: 30 approved YT02v2_ov frames (from E1) + 30 mutated copies, one defect each, rotating
overlap / clip / wrong_colour / remove / duplicate (`mutate.py`, seed 2, regions from Apple Vision boxes).
Each frame is shown with its expected on-screen text (Vision words, conf >= 50) as the scene-spec stand-in.
`--reference` also shows the approved still as image 1.

| Run | Recall (30 mutated) | False-fail (30 clean) | Localised | p50 s/frame |
|---|---|---|---|---|
| Qwen, no reference | 0.00 | 0.00 | 0 | 5.0 |
| Qwen, reference | 0.00 | 0.00 | 0 | 8.5 |
| MiniCPM 35B, no reference | 0.37 | 0.00 | 2 | 2.3 |
| **MiniCPM 35B, reference** | **0.87** | 0.00* | 0 | 3.7 |

Per type, MiniCPM with reference: clip 5/6, duplicate 6/6, overlap 6/6, remove 3/6, wrong_colour 6/6.

\* With a reference, each clean frame is compared to itself, so 0.00 false-fail is optimistic. A real
re-render differs slightly from its approved still (anti-aliasing, animation phase); the R3 calibration
must measure false-fail on genuine re-renders.

## Decisions
- Critic model: **MiniCPM-V 4.7 35B-A3B**. Qwen2.5-VL-7B answers "pass" to everything here and is dropped as critic (it stays the read-back OCR for equations/dim panels per E1).
- Always pass the approved still when one exists; without it recall drops from 0.87 to 0.37.
- Regions are not trustworthy (0/30 overlapped the true defect), so R3 grounding (step 20a) cannot rely on model regions alone: ground by element_id/OCR text, or crop-compare the approved vs new still per declared element box.
- Clears the plan's gate on recall (>= 0.8) for this synthetic set; the gate still needs real known-bad stills and real re-render false-fail before the critic blocks anything.
