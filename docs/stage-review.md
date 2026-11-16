# Stage still review

Stage stills are rendered from a `stage_plan.json`. The native Stage column
shows **Approve** and **Reject** beside every still. Approval remains the
existing handoff decision; use Reject when a still needs actionable feedback.

Reject asks for a short reason and appends one JSON record to
`<episode>/stage/review/rejections.jsonl` (creating `review/` as needed):

```json
{"shot":"S01","still_path":"/absolute/path/S01.png","frame":42,"reason":"caption overlaps diagram","element_id":"caption","ts":"2026-11-06T12:00:00Z"}
```

`element_id` is optional. The UI currently leaves it out because the still
thumbnail does not expose an element picker. `still_path` is absolute, so
calibration can consume logs independent of its working directory.

For a review recorded outside the UI, use:

```sh
scriptcast-stage reject path/to/stage_plan.json --shot S01 --reason 'caption overlaps diagram'
```

The command derives the shot midpoint frame and assumes
`<plan-dir>/S01.png` when no still is supplied. Pass `--still PATH` and/or
`--frame N` to record the exact rendered still:

```sh
scriptcast-stage reject path/to/stage_plan.json --shot S01 --still /tmp/S01.png --frame 42 --reason 'caption overlaps diagram'
```

`experiments/r3_calibration/collect.py` reads this JSONL as documented
`label=bad` examples. It ignores malformed records and records whose still
image no longer exists, rather than inferring negatives.
