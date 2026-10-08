# Phase L (Python): `scriptcast-project`, a read/validate/edit tool for project config

Working directory: a git worktree of scriptCast on branch `py/phase-l`, from `dev` (app 0.18.0).
A UI agent builds against the **Contract** below, so follow it exactly. Report any deviation in the findings file.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures, or copies under `/tmp/scriptcast-l/`.
- **No synthesis.**
- **Python changes are approved** (operator, 2026-10-07):
  - a new `scriptcast/tools/project_config.py`, with its console script `scriptcast-project` in `pyproject.toml`;
  - a new `scriptcast/config_edit.py`;
  - tests;
  - one README "Commands" row.
  - **New dependency:** `ruamel.yaml`, added to `dependencies` in `pyproject.toml`. It is needed so that edits keep the comments in the user's YAML files.
- Existing loaders (`voices.py`, `archive.py`, `project.py`, `pronunciation.py`, `review.py`) are **read-only**: call them, don't change them. Validation must agree with them. If a file loads in the loader, it validates; if the loader raises, validation reports that message.
- Run pytest with output redirected: `PYTHONPATH=. pytest -q > /tmp/l-py.log 2>&1; tail -30 /tmp/l-py.log`. If `ruamel.yaml` is missing from the interpreter, `pip install ruamel.yaml` into a venv under `/tmp/scriptcast-l/venv` and use that; never install into a project venv.

## Established facts

- The project root is found by `project.py:37` `find_root` (`$SCRIPTCAST_PROJECT` or the nearest `scriptcast.toml`). `Project` (:50) resolves the keys `voices`, `pronunciations`, `speaker_rates`, `clips`, `clip_sources`, `references`, `scripts` and `outputs` (see the README "Configuration" table). It rejects unknown keys.
- **Voices:** `voices.py:11` `load_voices`. Engines are `kokoro`/`mlx_kokoro`, `elevenlabs`, `mlx_chatterbox`, `omnivoice` and `archive`. The required fields per engine are in the README "Voices" section and are enforced by `load_voices`.
- **Clips:** `archive.py:55` `load_registry`. Each entry needs `source`, `start` and `end`; `citation`, `origin_url` and `content_note` are optional.
- **Pronunciations:** `pronunciation.py:173` `load_respellings`.
- **Accepted hearings:** `review.py`, with `load_accepted_hearings` and `add_accepted_hearing`. Each entry carries a review-timestamp comment, and the file has 4 comment lines.
- **Comments matter:** the real voices file has 23 comment lines in 122; pronunciations has 20 in 35. `make_clip.py:89` and `calibrate_pronunciation.py:163` use `yaml.safe_dump`, which drops comments. Do not copy that.

## Contract

### 1. `scriptcast-project show --json [--root PATH]`

```json
{"schema": 1, "root": "/abs", "toml": {"path": "...", "values": {"voices": "...", ...}, "unknown_keys": []},
 "voices": {"path": "...", "speakers": [{"id": "aisha", "name": "Aisha", "engine": "omnivoice",
            "fields": {...all keys as in the file...}, "problems": ["..."]}], "problems": ["file-level"]},
 "clips": {"path": "...", "clips": [{"id": "...", "source": "...", "start": 1.0, "end": 2.0, "citation": null,
            "origin_url": null, "content_note": null, "source_exists": true, "problems": []}], "problems": []},
 "pronunciations": {"path": "...", "entries": [{"word": "...", "readings": {...}, "problems": []}], "problems": []},
 "accepted_hearings": {"path": "...", "entries": [{"word": "bonded", "heard": ["about"],
                        "review_ts": "2026-10-05T01:33:56Z" | null, "risky": false}], "problems": []},
 "speaker_rates": {"path": "...", "rates": {"aisha": 2.9}, "problems": []}}
```

- A missing file is `{"path": ..., "missing": true}` with empty lists, not an error.
- `risky` for accepted hearings mirrors the UI rule: the word or the hearing is ≤ 2 characters, or is in the UI's common-word list. Copy that list from `ui/src/review.rs` `COMMON_WORDS`, with a comment linking it.

### 2. `scriptcast-project edit --ops OPS.json --expect-sha256 FILE=HEX [...] [--root PATH]`

- Ops are applied in order, **atomically per file** (temp file then rename); if any op fails, no file changes.
- Each op names its target:
  - `{"file": "voices", "op": "set_speaker", "id": "aisha", "fields": {...}}`: creates or replaces the speaker's mapping, keeping comments elsewhere and the key order of untouched keys;
  - `{"file": "voices", "op": "delete_speaker", "id": "..."}`;
  - `{"file": "voices", "op": "rename_speaker", "id": "old", "to": "new"}`;
  - `{"file": "clips", "op": "set_clip", "id": "...", "fields": {...}}` and `{"file": "clips", "op": "delete_clip", "id": "..."}`;
  - `{"file": "pronunciations", "op": "set_entry", "word": "...", "readings": {...}}` and `{"file": "pronunciations", "op": "delete_entry", "word": "..."}`;
  - `{"file": "accepted_hearings", "op": "delete", "word": "...", "heard": "..."}`: removes one hearing, and the word too when it has none left;
  - `{"file": "toml", "op": "set", "key": "...", "value": ...}`: only known keys. Use plain line editing that keeps comments; no new TOML dependency.
- After the ops, **re-validate with the real loaders**. If validation fails, write nothing; exit 1 with `op[i]: message` lines, or `file: message` lines for loader errors.
- `--expect-sha256 voices=HEX` (one per touched file): a mismatch exits 3 and writes nothing.
- On success, print the `show --json` output.

### 3. `scriptcast-project validate --json`

Same shape as `show`, but it exits 1 if any problem exists.

## Acceptance

1. `pytest -q` must be green. The new tests use fixture copies that **include comments**, and check that:
   - every op works, and comments plus untouched-key order survive (byte-compare the regions outside the edit);
   - an invalid engine field (for example, `omnivoice` without `reference_text`) is rejected with the loader's message and the file is unchanged;
   - a stale hash returns exit 3;
   - several files in one ops batch are atomic;
   - a TOML set keeps comments, and an unknown TOML key is rejected;
   - `show` on a missing file, and `risky` flags, behave as specified;
   - existing loaders still read every edited file.
2. Run `show --json` on a `/tmp` copy of the real project config (copy only `scriptcast.toml` and the YAML/JSON files). Paste the summary counts into the findings file.

## Findings file

Write `plans/Native_UI/evidence/phaseL-py-findings.md` early, and update it as you go.
