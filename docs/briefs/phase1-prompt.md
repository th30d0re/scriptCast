You are implementing **Phase 1 only** of the brief at
`docs/briefs/v0.2-video-tooling.md` in this repository (scriptCast, branch
`v0.2-video-tooling`). Read that whole file first. Do not start Phases 2, 3, or 4.

Rules for this run:

1. Create `docs/briefs/FINDINGS-v0.2.md` FIRST, before any code change, and update it as
   you go. Report brief defects, contract bugs, and anything you could not verify there.
2. Do NOT run `git commit`, `git add`, `git checkout`, `git stash`, or any history
   command. Leave all changes uncommitted in the working tree; the orchestrator reviews
   and commits. Do not delete `docs/`.
3. Do not touch `/Users/emmanuel/Documents/Theory/TheOriginalPower`. You may read it.
4. Use this interpreter for tests, it has scriptcast's dependencies:
   `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`
   The baseline before your changes is **116 passed**.
5. No network access. Do not synthesize audio (no OmniVoice, Kokoro, or ElevenLabs).
   All tests use synthetic wav data.
6. Do not edit `README.md`. Add a `0.2.0` (Unreleased) entry to `CHANGELOG.md`.

Phase 1 deliverables are listed under "Phase 1 deliverables" in the brief: pass `tail_ms`
in `_segment_from_wav`; add `tail_ms` (400) and `speech_threshold` (0.03) project keys
with one source of truth for the defaults across `__main__.py`; add the `scriptcast-refit`
tool with tests for reload-keeps-tail, refit-never-exceeds-duration, and idempotence.

Read the existing tools `scriptcast/tools/relayout_episode.py` and
`scriptcast/tools/stitch_episode.py` and follow their conventions (manifest `.json.bak`,
`render_state.json` updates, output style). Reference workaround (read-only):
`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/refit_tails.py`.

Stop and report in the findings file (do not edit test expectations) if the new defaults
change any existing test's expected numbers.

When finished, end your final message with: files changed, the raw output of the final
`pytest` run, and every item you could not verify.
