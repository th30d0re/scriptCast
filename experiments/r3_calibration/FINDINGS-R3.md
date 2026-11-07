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
