# E1 — Apple Vision OCR vs local VLMs (YT02v2_ov)

Host: M1 Max 64 GB. 30 frames, one per 25 s of `YT02_stage_0000-1232_1080p_20261105.mp4`. VLMs see the frame at 1280 px long side, temperature 0.
Reference: Vision words at conf >= 50 (silver, not hand-labelled). "Extra" = VLM words Vision never produced; checked by eye on f02.

| Engine | s/frame (median, max) | Load | Recall vs Vision (mean, min) | Equations (f16-f18) | Dim panel text (f02) | Error style |
|---|---|---|---|---|---|---|
| Apple Vision (`stage-ocr` accurate) | 0.33 | — | reference | Π read as Cyrillic | missed | stray `å` on icons |
| Qwen2.5-VL-7B 4-bit | 6.3, 8.4 | 2.5 s | 0.98, 0.85 | correct (`P_i(T \| X, Π_race) ∝ …`) | read correctly | split hyphenations, rare misspellings |
| MiniCPM-V 4.7 1B 4-bit | 2.1, 6.4 | 3.6 s | 0.92, 0.32 | wrong subscripts (P₁, P₂, I_race) | read, with substituted values (HIGH→RIGHT, STAGNANT→STABANAME, 1.0→3.1) | plausible wrong words and numbers |
| MiniCPM-V 4.7 35B-A3B 4-bit | 4.8, 10.0 | 11.5 s | 0.98, 0.74 | correct | invents values (Capital→CAPTAIN, Compiling Version 1.0→Compelling Version 3.8, made-up coordinates) | confident substitutions on low-contrast text |

## Reading so far
- Vision stays the Tier 1 pass on every frame: 19x faster than Qwen and clean on prose.
- Qwen earns its cost only on known Vision weak spots: equation frames and low-contrast panel text.
- MiniCPM 1B is unsafe as a checker: it substitutes confident, plausible wrong values, which a verifier would pass or flag wrongly.
- MiniCPM 35B-A3B matches Qwen on prose and equations and is ~25% faster, but substitutes values on dim panels; Qwen reads them correctly.
- Pick for text read-back: **Qwen2.5-VL-7B**. E2 (layout QA) is still open between Qwen and MiniCPM 35B-A3B: speed favours MiniCPM, text fidelity favours Qwen.
- Draft routing rule: frame has an equation or a dim-panel region -> Vision + Qwen; else Vision only.
