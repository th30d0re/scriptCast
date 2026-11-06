# Loop L5 Findings — Apple Vision OCR backend for stage-verify

- **Loop:** L5 (brief `L5-BRIEF.md`)
- **Checkpoint:** `855019f`
- **Worktree:** `/Users/emmanuel/Documents/Theory/scriptCast-v04-L5` (branch `v04-L5`)
- **Files:** `swift/StageOCR/Package.swift`, `swift/StageOCR/Sources/stage-ocr/main.swift`, `swift/StageOCR/README.md`, `scriptcast/tools/stage_verify.py` (the `VisionBackend` class and its registration only), `tests/test_stage_ocr_vision.py`, `FINDINGS-L5.md`

`stage-ocr` is a small Swift executable over `VNRecognizeTextRequest` (accurate by default, language correction off because the stills carry names and codes). It prints one JSON object per image: word boxes in top-left pixels (Vision's normalized bottom-left boxes converted via `boundingBox(for:)` on each word range), confidence on the 0–100 scale. Errors go to stderr with a non-zero exit; stdout carries JSON only.

`VisionBackend` writes the still to a temp PNG, runs the binary as an argument list (no shell), and parses the JSON into `OcrWord` with the same filter `_parse_tsv` applies (`conf >= min_conf`, `len(text) >= 2`). Lookup order is `STAGE_OCR_BIN`, then `swift/StageOCR/.build/release/stage-ocr`; a missing binary raises a RuntimeError naming `swift build -c release`.

One deliberate shape note: `VisionBackend` subclasses `TesseractBackend` and defaults `min_conf` to 0. `evaluate_fixtures` (L4, unedited) replaces any backend that is not a TesseractBackend at `min_conf` 0 with `TesseractBackend(min_conf=0)`, so subclassing is the only way `--ocr vision` reaches the fixtures at all — and both backends then compete at the same `min_conf` 0. No L4 threshold or check was touched.

## Acceptance tails

```bash
cd swift/StageOCR && swift build -c release
```

```text
[89 / 90] stage-ocr-product
Build complete! (8.84 sec)
```

```bash
PYTHONPATH=. /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests/test_stage_ocr_vision.py tests/test_stage_verify.py -q
```

```text
................                                                         [100%]
16 passed in 42.97s
```

```bash
PYTHONPATH=. /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m scriptcast.tools.stage_verify --fixtures video/fixtures/verify --out /tmp/l5-vision --no-sheet --ocr vision
```

```text
thresholds  E=1.2%  C=45%  z=8
runtime     1.696s mean / 2.419s max per still
precision / recall
  border     precision 1.000  recall 1.000  (tp 5 fp 0 fn 0)
  empty      precision 0.769  recall 1.000  (tp 10 fp 3 fn 0)
  filler     precision 0.500  recall 1.000  (tp 1 fp 1 fn 0)
  internal   precision 1.000  recall 1.000  (tp 1 fp 0 fn 0)
  overlap    precision 1.000  recall 1.000  (tp 4 fp 0 fn 0)
good frames: all passed
bad frames: every labelled defect flagged
  PASS ab_v2_e161.jpg          2.12s  -
  PASS ab_v2_e207.jpg          2.42s  -
  PASS ab_v2_e239.jpg          1.55s  -
  PASS ab_v2_e275.jpg          1.58s  -
  PASS cd_v2_e294.jpg          1.18s  -
  PASS cd_v2_e382.jpg          1.89s  -
  PASS cd_v2_e405.jpg          1.52s  -
  PASS cd_v2_e424.5.jpg        1.65s  -
  FAIL cd_v1_e360.png          1.49s  empty,border
  FAIL cd_v1_e392.png          1.94s  border,overlap
  FAIL cd_v1_e405.png          1.42s  empty,border
  FAIL cd_v1_e424.5.png        1.63s  empty,border
  FAIL ab_v1_e161.png          1.74s  overlap
  FAIL ab_v1_e169.png          1.48s  empty
  FAIL ab_v1_e207.png          1.67s  overlap
  FAIL ab_v1_e264.png          1.73s  empty,overlap
  FAIL f_v1_e545.jpg           1.50s  empty
  FAIL f_v1_e578.jpg           1.44s  empty
  FAIL f_v1_e508.jpg           1.67s  empty
  FAIL e_v1_e441.jpg           1.64s  empty
  FAIL h_v1_e751.jpg           1.74s  internal,empty
  FAIL v4_e476.jpg             2.24s  filler,border
  FAIL v4_e434.jpg             2.21s  filler
  FAIL v5_e508.jpg             1.71s  empty
  FAIL v5_e552.jpg             1.43s  empty
  FAIL v5_e586.jpg             1.51s  empty
```

```bash
PYTHONPATH=. /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m scriptcast.tools.stage_verify --fixtures video/fixtures/verify --out /tmp/l5-tesseract --no-sheet
```

```text
thresholds  E=1.2%  C=45%  z=8
runtime     1.560s mean / 2.903s max per still
precision / recall
  border     precision 1.000  recall 1.000  (tp 5 fp 0 fn 0)
  empty      precision 0.769  recall 1.000  (tp 10 fp 3 fn 0)
  filler     precision 1.000  recall 1.000  (tp 1 fp 0 fn 0)
  internal   precision 1.000  recall 1.000  (tp 1 fp 0 fn 0)
  overlap    precision 1.000  recall 1.000  (tp 4 fp 0 fn 0)
good frames: all passed
bad frames: every labelled defect flagged
  PASS ab_v2_e161.jpg          1.49s  -
  PASS ab_v2_e207.jpg          1.56s  -
  PASS ab_v2_e239.jpg          1.25s  -
  PASS ab_v2_e275.jpg          1.24s  -
  PASS cd_v2_e294.jpg          1.42s  -
  PASS cd_v2_e382.jpg          1.57s  -
  PASS cd_v2_e405.jpg          1.20s  -
  PASS cd_v2_e424.5.jpg        1.32s  -
  FAIL cd_v1_e360.png          1.23s  empty,border
  FAIL cd_v1_e392.png          1.58s  border,overlap
  FAIL cd_v1_e405.png          1.35s  empty,border
  FAIL cd_v1_e424.5.png        2.32s  empty,border
  FAIL ab_v1_e161.png          1.41s  overlap
  FAIL ab_v1_e169.png          1.18s  empty
  FAIL ab_v1_e207.png          1.43s  overlap
  FAIL ab_v1_e264.png          2.90s  empty,overlap
  FAIL f_v1_e545.jpg           1.39s  empty
  FAIL f_v1_e578.jpg           1.20s  empty
  FAIL f_v1_e508.jpg           1.52s  empty
  FAIL e_v1_e441.jpg           1.43s  empty
  FAIL h_v1_e751.jpg           2.31s  internal,empty
  FAIL v4_e476.jpg             2.25s  border
  FAIL v4_e434.jpg             1.94s  filler
  FAIL v5_e508.jpg             1.35s  empty
  FAIL v5_e552.jpg             1.13s  empty
  FAIL v5_e586.jpg             1.58s  empty
```

## Vision beside tesseract

| Defect | Vision precision / recall | Tesseract precision / recall |
| --- | --- | --- |
| border | 1.000 / 1.000 (tp 5 fp 0 fn 0) | 1.000 / 1.000 (tp 5 fp 0 fn 0) |
| empty | 0.769 / 1.000 (tp 10 fp 3 fn 0) | 0.769 / 1.000 (tp 10 fp 3 fn 0) |
| filler | **0.500** / 1.000 (tp 1 fp 1 fn 0) | 1.000 / 1.000 (tp 1 fp 0 fn 0) |
| internal | 1.000 / 1.000 (tp 1 fp 0 fn 0) | 1.000 / 1.000 (tp 1 fp 0 fn 0) |
| overlap | 1.000 / 1.000 (tp 4 fp 0 fn 0) | 1.000 / 1.000 (tp 4 fp 0 fn 0) |

Mean seconds per still: vision **1.696**, tesseract **1.560**. Vision's read itself is ~0.3 s (process spawn + one accurate pass); the vision run stays slower because L4's `check_pixel_overlap` re-reads any still whose words are all conf ≥ 45 with tesseract — vision reports nearly everything at conf 100 (only 1 of 26 stills, the approved sorting diagram, returns anything under 45, as misread glyphs like `AAAAA` at conf 30), so the vision run pays for both engines on 25 of 26 stills. Every good frame passes under vision; every labelled defect is flagged under both backends.

## Verdict differences

Exactly one fixture's check set differs: **`v4_e476.jpg`** (labelled `border`) is `border` under tesseract and `filler,border` under vision. The labelled defect is caught either way; the extra flag is the uppercase-caption filler rule. The words that explain it:

- Vision reads four panel titles as caption lines' worth of caps words, including `MENTAL HALF INSTALLED PRIORS` at y=516 — MENTAL (x 140, w 84), HALF (226, 60), INSTALLED (310, 118), PRIORS (430, 118), with the HALF→INSTALLED gap of 24 px in the middot range — plus `CIRCUIT PARAMETERS FIVE TERM CHIPS` at y=204 (FIVE→TERM gap 3 px, PARAMETERS→FIVE 22 px). Two caption lines ≥ `MICROCOPY_LINES`.
- Tesseract reads the same `CIRCUIT PARAMETERS FIVE TERM CHIPS` line (one caption line is a title, not filler) but **drops `MENTAL` entirely** — at `min_conf=0` its y≈517 row is only `HALF (93), INSTALLED (93), PRIORS (92)`, three words, under `MICROCOPY_RUN` 4. The second line never forms.

So vision's filler false positive is a tesseract *miss*: the still does show two title rows, and the L4 rule calibrated on tesseract's blindness counts them under vision.

## What vision misses outright

`ab_v1_e161` (crushed subtitle). Vision reads the buried line at full confidence — `Aftianfode` (conf 100) under `Discriminatory`, `PATTERNE` (conf 100) under `INSTITUTIONAL` — where tesseract's confidence collapses (`Astiangode` 36, `PAERERNA` 10). `_crushed_subtitles` needs a word under conf 42, so vision's words alone flag nothing there. The fixture still passes as `overlap` in the vision run only because L4's `check_pixel_overlap` re-reads with tesseract when no word is under conf 45 — which is always, for vision on these stills. The identical reason strings in both runs (`"Astiangode" (conf 36) is crushed under "Discriminatory"`) are tesseract's words. A vision-only pipeline (no tesseract reread) would ship overlap recall 0.75 here.

## Retuning, if vision were the only backend

Not done — L4's thresholds stand. Hiding the differences would need:

- **filler on v4_e476:** `MICROCOPY_LINES` 2→3 re-hides this frame (vision sees 2 lines) and still catches the ImplicitTrace turn-42 captions (4 lines), but it erodes the check for genuine two-line caption stacks; `MICROCOPY_RUN` 4→5 does not work — it would drop the 4-word `EXPLICIT CHANNEL · CONSCIOUS COMMITMENT` lines the rule exists to catch.
- **crushed subtitles:** a vision-native signal, since confidence never collapses — e.g. treat a word whose box sits within ~14 px under another and shares ≥ 50% x-overlap as crushed regardless of confidence, or drop the conf < 42 gate for the vision backend. That is a check change, not a threshold, and belongs to a later loop with fixtures that separate the two engines (a crushed-subtitle still where the tesseract reread is disabled).
- **runtime:** skip the tesseract reread when the backend is vision (it exists to recover tesseract's low-confidence words). That removes ~1 tesseract pass per still from the vision run but makes the crushed-subtitle gap above immediate.
