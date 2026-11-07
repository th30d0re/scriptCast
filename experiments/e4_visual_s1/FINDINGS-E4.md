# E4 findings — System 1 visual judge

`make_pairs.py` produced 2,002 deterministic line-crop pairs from the 30 E1
approved frames.  It includes E2-style defects and explicit benign one-pixel
translations, JPEG quality-90 round trips, and small gamma shifts.  The split
is by source frame, not by crop: 1,636 train / 366 held out.

| Held-out pair metric | Result |
|---|---:|
| Precision | 0.548 |
| Recall | 0.958 |
| False-fail | 0.056 |
| Feature extraction time | 3.248 ms/element |

On the independent E2 frame set, frame recall was **0.800**, false-fail was
**0.000**, and top-element localisation (IoU > 0.05 with the recorded defect
region) was **0.767**.  End-to-end E2 timing, including opening both PNGs and
all element crops, was 6.683 ms/element.  The requested gate passes: recall is
at least 0.8 and false-fail is at most 0.1.

## E3 author-ranking comparison

This is diagnostic only: E3 has no defect ground truth.  `eval.py` prints a
full per-worker table beside `author_rank.csv`; the S1 whole-frame defect
probabilities were generally low (0.044–0.136).  The largest were
`doctrine-chain/codex-gpt-luna` (0.136), `pulse-network/codex-gpt-luna`
(0.123), and `doctrine-chain/codex-gpt-terra` (0.121).

The default path uses NumPy/Pillow exclusively.  An embedding feature can be
provided as a callable to `features`; MLX weights remain optional and can be
acquired explicitly with `python -m scriptcast.critic.s1 --download`.
