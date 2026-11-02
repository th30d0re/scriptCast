# Loop L4 Findings — stage-verify

- **Loop:** L4 (spec C2; brief `docs/briefs/v0.4-loop.md`)
- **Checkpoint:** `a921ca4`
- **Worktree:** `/Users/emmanuel/Documents/Theory/scriptCast-v04-L4` (branch `v04-L4`)
- **Entry point:** `scriptcast-stage-verify` → `scriptcast.tools.stage_verify:main`
- **Files:** `scriptcast/tools/stage_verify.py`, `tests/test_stage_verify.py`, `pyproject.toml`, `FINDINGS-L4.md`

The tool reads a still back and scores it against what the scene declared. Same shape as the audio tools: content read-back, a specialised second check, then an outlier model. OCR is a `OcrBackend` (`TesseractBackend` now; Apple Vision can implement the same `read()` later). With declared boxes, overlap is geometry. Without them, the still is measured directly. Output is `verify.json` (`scene`, `cue`, `ms`, `pass`, `reasons`, element ids) and an annotated contact sheet.

## Calibrated thresholds

Set on `video/fixtures/verify` after normalising every still to 1920×1080. The ornamental frame sits in the 70 px band on every approved still, so "text-coloured ink in the band" is the ink the approved chrome does not explain.

| Constant | Value | What it separates |
| --- | --- | --- |
| `EMPTY_INK` (E) | 1.2% of a container, luminance ≥ 48 | Empty panels. The sparsest approved container the finder keeps is about 1.6%. |
| `CROWDED_INK` (C) | 45% | Declared containers only. The fullest approved declared panel is well under this. A detector that traces a diagram will call the diagram itself crowded (the sorting still is 81% ink inside one found panel), so crowded is not applied to detected boxes. |
| Mostly empty | ink < 6%, empty-row run ≥ 22% of height, height ≥ 480, area ≥ 80,000 | Tall cards with a citation at the top and the rest blank (`f_v1_e545`, `f_v1_e578`). Ink alone cannot separate those from an approved plot. |
| Lower half | ink < 0.75% in y 560–1000, x 140–1780 | `cd_v1_e424.5` (measured 0.6%). Approved minimum in that slice is about 1.0%. |
| Border thick | > 500 px surviving a 4×4 opening at luminance ≥ 175 | Question and cards on the frame. Approved ornament opens down to ~380 px. |
| Border outside chrome | > 1000 px at luminance > 110, or > 800 px at 42–120, outside a mask of pixels lit in ≥ 60% of the approved stills | Ghost band and notch text (`v4_e476`). |
| `DENSITY_Z` | 8.0 | Leave-one-out against the eight approved stills. The sorting still's ink z is **6.85**; the next largest \|z\| is text z 2.77. 8.0 sits above the approved set. |
| Filler | ≥ 70 OCR words (conf ≥ 50, length ≥ 3) and ≥ 12 sentence-case prose words | `v4_e434` is 83 words / 15 prose. Approved stills reach one of those counts, not both. |
| Line through text | score > 400 | Bucket lines through the term blocks (`ab_v1_e207` scores 518). Approved maximum is 221. |
| Curve inside a panel | > 8000 px of one round stroke | Tray over the field (`ab_v1_e264`, 12634 px) and `cd_v1_e392` (32044 px). Approved maximum is 4249 px. |

`TURN 63` is an internal label. The match is `turn` plus digits, so "Turner" on the approved Tajfel still is not flagged.

## How the four overlap frames are caught

Declared geometry (text on text, a line through a text box, a mark on text) is the check when the scene supplies boxes. The fixtures have no markup, so the still is read:

- `ab_v1_e161` — the subtitle is still on the pixels but tesseract's confidence collapses. "Astiangode" (conf 36) sits 7 px under "Discriminatory"; "PAERERNA" (conf 10) sits 12 px under "INSTITUTIONAL". No approved still has that pair.
- `ab_v1_e207` — a faint stroke touches the term and continues past it (line score 518, cutoff 400).
- `ab_v1_e264` — a round stroke of 12634 px inside the field panel at (830, 130). The definition card is also flagged empty (see precision).
- `cd_v1_e392` — border ink outside the chrome (the title on the frame edge) plus a round stroke of 32044 px inside the right-hand panel at (1010, 235). The review note's strips sit on the left-hand title; that panel's own stroke stays under the 8000 px cutoff, so the overlap box is the right-hand network, which is the stroke that separates this PNG from the approved networks. The border flag is the title on the frame edge.

## e405

The 70 px band on `cd_v1_e405` matches the approved chrome: no extra bright or faint pixels there. The equation row is a full-width empty box at y=105, 1680×154, 0% ink. That box is the empty flag. Because its top is above y=115 and it is full width, it is also the border flag. The gauge the review describes is not a separate blob of text-coloured ink in the band.

## Precision / recall

Every approved frame passes. Every labelled defect on a bad frame is flagged. `ghost` has no fixture.

| Defect | Precision | Recall | Notes |
| --- | --- | --- | --- |
| border | 1.000 | 1.000 | 5 / 5 |
| empty | 0.769 | 1.000 | 10 true, 3 extra |
| filler | 1.000 | 1.000 | |
| internal | 1.000 | 1.000 | |
| overlap | 1.000 | 1.000 | 4 / 4 |

The three extra empty flags, all on frames whose labelled defect is also caught:

- `cd_v1_e360` — a side panel at (1270, 280), 500×519, 1.2% ink, empty-row run 59% of its height. The label is the question on the top border.
- `ab_v1_e264` — the definition card at (110, 130), 650×789, 3.9% ink, empty-row run 65%. The label is the tray over the field.
- `h_v1_e751` — the dashed box at (1439, 381), 342×338, 0% ink. The label is `TURN 63`; the fixture note already calls the dashed box empty.

## Acceptance tails

```bash
cd /Users/emmanuel/Documents/Theory/scriptCast-v04-L4
PYTHONPATH=. /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests/test_stage_verify.py -q
```

```text
........                                                                 [100%]
8 passed in 38.89s
```

```bash
PYTHONPATH=. /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m scriptcast.tools.stage_verify --fixtures video/fixtures/verify --out /tmp/l4-verify --no-sheet
```

```text
thresholds  E=1.2%  C=45%  z=8
runtime     1.344s mean / 1.939s max per still
precision / recall
  border     precision 1.000  recall 1.000  (tp 5 fp 0 fn 0)
  empty      precision 0.769  recall 1.000  (tp 10 fp 3 fn 0)
  filler     precision 1.000  recall 1.000  (tp 1 fp 0 fn 0)
  internal   precision 1.000  recall 1.000  (tp 1 fp 0 fn 0)
  overlap    precision 1.000  recall 1.000  (tp 4 fp 0 fn 0)
good frames: all passed
bad frames: every labelled defect flagged
  PASS ab_v2_e161.jpg          1.41s  -
  PASS ab_v2_e207.jpg          1.32s  -
  PASS ab_v2_e239.jpg          1.20s  -
  PASS ab_v2_e275.jpg          1.22s  -
  PASS cd_v2_e294.jpg          1.39s  -
  PASS cd_v2_e382.jpg          1.47s  -
  PASS cd_v2_e405.jpg          1.19s  -
  PASS cd_v2_e424.5.jpg        1.30s  -
  FAIL cd_v1_e360.png          1.15s  empty,border
  FAIL cd_v1_e392.png          1.52s  border,overlap
  FAIL cd_v1_e405.png          1.10s  empty,border
  FAIL cd_v1_e424.5.png        1.30s  empty,border
  FAIL ab_v1_e161.png          1.44s  overlap
  FAIL ab_v1_e169.png          1.17s  empty
  FAIL ab_v1_e207.png          1.26s  overlap
  FAIL ab_v1_e264.png          1.35s  empty,overlap
  FAIL f_v1_e545.jpg           1.19s  empty
  FAIL f_v1_e578.jpg           1.14s  empty
  FAIL f_v1_e508.jpg           1.32s  empty
  FAIL e_v1_e441.jpg           1.29s  empty
  FAIL h_v1_e751.jpg           1.39s  internal,empty
  FAIL v4_e476.jpg             1.94s  border
  FAIL v4_e434.jpg             1.77s  filler
  FAIL v5_e508.jpg             1.29s  empty
  FAIL v5_e552.jpg             1.14s  empty
  FAIL v5_e586.jpg             1.69s  empty
```

Runtime is one tesseract pass (`psm 11`) plus the pixel checks, per still. The annotated sheet is `verify_sheet.png` next to `verify.json` when `--no-sheet` is left off. The same fixture assertion is `test_fixtures_flag_every_labelled_defect_and_pass_the_good_frames`; it writes under `/tmp/l4-verify-pytest`.
