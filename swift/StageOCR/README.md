# StageOCR

Apple Vision OCR for Stage stills, the `vision` backend of `scriptcast-stage-verify`.

## Build

```bash
cd swift/StageOCR
swift build -c release
```

The binary lands at `.build/release/stage-ocr`.

## Usage

```bash
stage-ocr <image-path>... [--min-conf N] [--level fast|accurate]
```

- `--level` selects `VNRecognizeTextRequest.recognitionLevel` (default `accurate`).
- `--min-conf N` drops words below confidence N (0–100 scale; default 0, keep everything).
- Language correction is off: the stills carry names and codes a spellchecker would "fix".
- Errors (an unreadable image, a Vision failure) go to stderr and exit non-zero. Stdout carries only JSON.

`scriptcast-stage-verify --ocr vision` finds the binary via `STAGE_OCR_BIN`, then this package's release build.

## Output

One JSON object per image, one per line, nothing else on stdout. Boxes are pixels from the image's top-left (Vision's normalized bottom-left boxes, converted); `conf` is Vision's 0–1 confidence times 100.

```json
{"path": "still.png", "width": 1920, "height": 1080, "words": [{"text": "TURN", "conf": 100, "x": 1440, "y": 292, "w": 60, "h": 24}]}
```

Vision returns whole text lines; each candidate is split into words with `boundingBox(for: range)`, so every word carries its own box.
