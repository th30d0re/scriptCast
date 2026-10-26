You are implementing **Phase 2 only** of `docs/briefs/v0.2-video-tooling.md` (scriptCast,
branch `v0.2-video-tooling`). Read the whole brief, then `docs/briefs/FINDINGS-v0.2.md`
(Phase 1 is done and committed as 21fe7bf; append a `## Phase 2` section, do not rewrite
Phase 1's). Do not start Phases 3 or 4.

Rules (same as Phase 1):

1. Do NOT run `git commit`, `git add`, `git checkout`, `git stash`, or any history
   command. Leave changes uncommitted; the orchestrator reviews and commits.
2. Do not modify `/Users/emmanuel/Documents/Theory/TheOriginalPower`. Read-only.
3. Test interpreter (has scriptcast's dependencies):
   `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`
   Baseline now: **124 passed**. It must stay green plus your new tests.
4. No network. No audio synthesis. Do not edit `README.md`; add to the `[0.2.0]` entry
   in `CHANGELOG.md`.
5. Test fixtures must be **neutral and synthetic** (this repo's history is
   "neutral test fixtures, generic docs"): invent a tiny fake show, do not copy
   manuscript or Chapter 135 content into `tests/`. Scratch output goes to `/tmp` or
   `./.scratch/`, never into tracked paths; do not commit or leave junk in the tree.

## Task: `scriptcast-shotspec`

New file `scriptcast/tools/shotspec.py`, entry `scriptcast-shotspec` in `pyproject.toml`,
tests in `tests/test_shotspec.py`, fixtures in `tests/fixtures/` (check existing
fixtures for conventions).

Reference implementation, read it fully first:
`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/shotspec.py`

Keep its behaviour for: parsing `## G-NN — title` shots and `- **Label:** value` fields
(with indented continuation lines); `Anchor` (`Speaker (MM:SS)`) resolved to
`turn_index`/`start_ms`/`end_ms`/`speaker_id` **through the script's turn order into the
manifest** (`--script`, `--manifest`), falling back to the manifest's own
`source_timestamp` when no script is given; `Hold` with `script_ms`; `described`,
`note`, `prompt_seed`, `suggested_track` (`archival` vs `vector`); `--out`, `--strict`;
the console summary line. The output JSON keeps `schema_version: 1`.

Changes:

1. **Tags.** Recognised tags become `book`, `data`, `design`, **`source`**.
2. **Citations, generic by default.** Without any flags, extract `citations` entries of
   the forms `{"url": "https://..."}` and `{"card": "Card 15"}` (any "Card N" /
   "Cards N and M" reference, one entry per number). A shot tagged `[source]` must have
   at least one citation, otherwise `error: tagged [source] with no URL or card
   reference`. Shots with no tag stay a `warn`, as before.
3. **Manuscript checks become opt-in.** Everything tied to the book (`BOOK` path,
   `_CITE` for `Paper/...` paths, bare `:149` line numbers, quoted-phrase resolution,
   `\label{}` checks, `[book]`/`[data]` validation, file-exists and line-in-range checks)
   only runs when `--book PATH` (manuscript file) and/or `--repo ROOT` are passed.
   Off by default. When on, behaviour must match the reference.
4. Keep `_manifest_lookup` semantics exactly (it took a real bug to get right: after a
   `scriptcast-retime`, manifest timestamps drift from the script's, so anchors resolve
   via turn order).

## Acceptance (run these)

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
```

Plus a **read-only real-data comparison** (do this, and paste the raw result into the
findings file). From `/Users/emmanuel/Documents/Theory/TheOriginalPower`:

```bash
python -m scriptcast.tools.shotspec \
  Architecting_the_operation/video/chapter135_rebuttal_shotlist.md \
  --script Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md \
  --manifest outputs/chapter135_reply/episode_manifest.json \
  --out /tmp/shotspec_new.json
```

Then compare against `Architecting_the_operation/video/specs/chapter135_rebuttal.json`
(the reference output). For every shot, `id`, `title`, `anchor` (including `start_ms`,
`end_ms`, `turn_index`), `hold`, `type`, `described`, `note`, `suggested_track` must be
**identical**. Differences are allowed only in `provenance.tags`,
`provenance.citations`, `validation`, and `prompt_seed` if you can explain them. Report
the differing field names, not just "matches".

Stop and report (do not paper over) if any anchor `start_ms` differs.

## Also required tests (synthetic)

anchor resolution through turn order when the manifest's own timestamps have drifted;
fallback resolution without `--script`; `[source]` shot with and without a citation;
`--book` off by default (a `Paper/x.tex:12` citation is ignored) and on when passed;
multi-line continued fields; `--strict` exit code.

## Findings and final message

Update `docs/briefs/FINDINGS-v0.2.md` early and as you go (brief defects, decisions,
what you could not verify). End your final message with: files changed, the raw
`pytest` result, the raw comparison result, and every unverified item, including whether
you smoke-tested the installed `scriptcast-shotspec` console script.
