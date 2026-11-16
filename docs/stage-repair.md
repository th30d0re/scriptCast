# Stage repair

`scriptcast-stage-repair` reads a grounded `critic.json` and produces one repair brief per failed marked element. Its default is report-only; it does not contact a worker.

```sh
scriptcast-stage-repair --episode video/yt02 --critic video/yt02/stage/review/critic.json \
  --stills video/yt02/stage/review/stills --plan video/yt02/stage/ov/stage_plan.json \
  --reference-dir video/yt02/stage/review/approved
```

Add `--dispatch` to resume the scene's `builder` registry session. If none exists, it starts the selected `--agent`/`--model`. Each dispatch is re-criticised using only that still, and before/after verdicts are appended to `<episode>/stage/review/repairs.jsonl`. An element is dispatched at most twice; later failures are recorded for the operator queue.
