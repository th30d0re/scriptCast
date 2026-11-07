# R3 two-tier Stage critic calibration

## Data collection

| label | count | provenance |
|---|---:|---|
| good | _TBD_ | Author-approved `stage/handoff/png` references and filename-paired logged re-renders |
| bad | _TBD_ | Explicit rejected-still review labels only |

The initial repository scan found no machine-readable rejected-still labels in
the YT02 Stage tree. If collection still yields fewer than 20 bad rows, report
that limitation; do not create synthetic or filename-inferred negatives.

## Threshold sweep

| S1 threshold | mode | n | bad n | recall | false-fail | Tier-2 call rate | mean latency (ms) |
|---:|---|---:|---:|---:|---:|---:|---:|
| 0.1 | tier1 only | | | | | | |
| 0.1 | two-tier | | | | | | |
| … | … | | | | | | |
| 0.9 | tier1 only | | | | | | |
| 0.9 | two-tier | | | | | | |

## Decision

- Chosen S1 threshold: _TBD_.
- Reason: _TBD; select against recall, false-fail, Tier-2 call rate, and latency._
- Hardware/runtime: _TBD; record GPU/runtime, model, and command._

## First real calibration run (2026-11-07)

Labels: 7 bad stills from the author's E3 rankings (cells called last, horrible or unusable, recorded with `scriptcast-stage reject`), 8 good stills. Collector bug: rejected stills were paired with themselves as reference (self-compare, always pass); `manifest_fixed.csv` pairs each with its cue's reference. 4 of the 8 good rows are still self-compared, so false-fail is optimistic.

| S1 threshold | S1 alone recall | two-tier recall | false-fail | MiniCPM call rate |
|---|---|---|---|---|
| 0.1 | 6/7 | 1/7 | 0 | 0.40 |
| 0.2 | 6/7 | 2/7 | 0 | 0.40 |
| 0.3 | 5/7 | 2/7 | 0 | 0.33 |
| 0.4 | 1/7 | 1/7 | 0 | 0.07 |
| ≥0.5 | 0/7 | 0/7 | 0 | 0 |

S1 alone catches 6 of 7 at threshold 0.2. MiniCPM, as tier 2, overturns most of S1's correct flags to pass, so two-tier recall is far lower. Provisional: threshold 0.2, and MiniCPM should describe flagged frames rather than veto them. Too few labels (gate needs ≥20 bad) to set authority.
