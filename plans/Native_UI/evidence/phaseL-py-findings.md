# Phase L Python findings

## Scope and status

- Started 2026-10-08 on `py/phase-l`.
- Approved implementation scope: new `scriptcast/tools/project_config.py` (`scriptcast-project`), new `scriptcast/config_edit.py`, tests, one README Commands row, and `ruamel.yaml` in `pyproject.toml` dependencies.
- Existing loaders (`voices.py`, `archive.py`, `project.py`, `pronunciation.py`, `review.py`) were not modified.
- No synthesis, no git writes, no agent CLIs, no AppleScript. `/Users/emmanuel/Documents/Theory/TheOriginalPower/` was only read. The show run used a copy under `/tmp/scriptcast-l/real`.
- Implementation is complete.

## Contract observations

- `find_root` uses `$SCRIPTCAST_PROJECT` or the nearest `scriptcast.toml`. `scriptcast-project --root` selects that directory directly and does not walk parents.
- Accepted hearings live beside the voices file (`review.project_dir_from_voices` + `accepted_hearings.yaml`), not necessarily at the project root. The real project’s voices path is `Architecting_the_operation/scriptcast/voices.omnivoice.yaml`, so the hearings file is in that directory.
- `load_registry` accepts a clip mapping without checking `source`, `start`, or `end`. `clip_entry` is stricter. Show and edit validation follow `load_registry`: a file that loads has empty clip `problems`. Per-clip `problems` stay `[]` in that case. `source_exists` is separate and is not a problem.
- `load_voices` stops at the first speaker error. File-level `problems` carry that loader message. When the message names `Speaker '…'`, the same string is also stored on that speaker.
- A missing file is `{"path", "missing": true, …}` with empty collections. It is not a validation failure. `validate` still exits 1 when any section or item `problems` list is non-empty, or when `toml.unknown_keys` is non-empty.
- Speaker-rate JSON in the real project is a per-speaker object (`ms_per_word`, `ms_per_mark`). The contract example shows a bare number. `show` emits the JSON values as stored. The fixture test covers a bare number (`aisha: 2.9`) and an object.
- Each clip in `show` keeps the top-level convenience keys and also has `fields` with every key from the file (`linger`, `speaker`, `transcript_source`, and the rest), in file types. Speaker `fields` and pronunciation `fields` likewise hold every key. Pronunciation entries still include `readings` with the same object.
- `set_speaker`, `set_clip`, and `set_entry` take the complete mapping. A key absent from the op is removed. Unchanged keys stay byte-identical. This is stated in `scriptcast-project --help`, `scriptcast-project edit --help`, the README Configuration section, and here.
- `scriptcast-project init --root PATH` writes `scriptcast.toml` containing only `# scriptCast project settings. Every key is optional; see the README Configuration table.` when that file is absent, then prints `show --json`. If the file exists, init exits 2 and does not change it.
- `risky` copies `COMMON_WORDS` from `ui/src/review.rs` and uses the same rule on the raw YAML strings: length ≤ 2, or membership in that list (case-sensitive, matching the Rust slice).
- `review_ts` is the hearing comment when a word has one hearing, or the shared timestamp when every hearing under that word has the same comment. Differing timestamps yield `null` (the `pati` fixture).
- Ops errors are `op[i]: …` with `i` starting at 0. Loader failures after a structurally valid op are `file: message` and write nothing. `--expect-sha256` is required for every touched file. A missing hash exits 1. A mismatch exits 3. A missing file hashes as the empty byte string.
- YAML edits load with `ruamel.yaml` `preserve_quotes` and then edit per key. Unchanged keys stay byte-identical inside the block, including double quotes and inline comments. A changed scalar keeps the key line’s prefix and comment; the new value is written in the quote style already used by that mapping (`"` when the block uses double quotes). Added keys are appended. A full-block or full-document dump is not used for an existing mapping, because that dump drops quotes (`"Aisha"` becomes `Aisha`). TOML `set` replaces the value token and keeps the inline comment and the rest of the line.
- Checked on a copy of `voices.omnivoice.yaml`: `set_speaker` for `aisha` with the same fields plus `speed: 1.04` inserts only `    speed: 1.04`. `name`, `engine`, `reference_audio`, `reference_text`, and `character_profile` keep their double quotes.

## Deviations

- The `toml` object includes `problems` (an empty list when `Project` accepts the file). The contract snippet lists `path`, `values`, and `unknown_keys` only. `problems` is where `Project()` errors are reported, including unknown keys and a bad `tail_ms` / `speech_threshold`. `unknown_keys` is still populated. A missing `scriptcast.toml` is `missing: true` with empty `values`, `unknown_keys`, and `problems`.

## Verification

- `PYTHONPATH=. /tmp/scriptcast-l/venv/bin/python -m pytest -q > /tmp/l-py.log 2>&1` (the `/tmp/scriptcast-l/venv` interpreter, because the system interpreter has no `ruamel.yaml`): **363 passed**, 4 warnings. The log is `/tmp/l-py.log`.
- `test_added_key_keeps_double_quoted_siblings` covers `set_speaker`, `set_clip`, and `set_entry`: a double-quoted block gains one line (`speed`, `content_note`, or `noun`) and every other byte stays put.
- `misaki` is not installed on the system interpreter. It was installed into `/tmp/scriptcast-l/venv` only, so the existing pronunciation tests could run. System `click` stayed 8.1.7. The first full run before that install was 349 passed and the same 11 `ModuleNotFoundError: misaki` failures already reported for phase K; the 8 new tests were in the passing set.
- New tests cover every op, byte-identical regions outside the edit, the omnivoice `reference_text` loader message with an unchanged file, exit 3 on a stale hash, a three-file batch that rolls back when voices fail, TOML comment retention, an unknown TOML key, missing-file `show`, `risky` flags, and the real loaders reading every edited file.
- `validate --json` on the `/tmp` copy exits 0.

## Real-project show summary

Copy: `scriptcast.toml`, `voices.omnivoice.yaml`, `pronunciations.yaml`, `speaker_rates.json`, `accepted_hearings.yaml`, and `archive/clips.yaml` under `/tmp/scriptcast-l/real`. Media was not copied, so every clip reports `source_exists: false`.

| Section | Count | Problems |
|---|---|---|
| TOML keys | 10 (`voices`, `pronunciations`, `speaker_rates`, `clips`, `clip_sources`, `references`, `scripts`, `outputs`, `omnivoice_python`, `tail_ms`) | none; `unknown_keys` empty |
| Voices | 20 speakers (3 `omnivoice`: `emmanuel_theodore`, `toussaint`, `aisha`; 17 `archive`) | none |
| Clips | 41 | none |
| Pronunciations | 5 entries | none |
| Accepted hearings | 2 (`bonded`/`about` and `trauma`/`traumatizing`, both `review_ts` `2026-10-05T01:33:56Z`, both `risky: false`) | none |
| Speaker rates | 17 speakers | none |

Comment counts on that copy:

- `voices.omnivoice.yaml`: 24 comment-only lines in 133 lines. The brief’s “23 comment lines in 122” does not match this file; the file is longer than that figure.
- `pronunciations.yaml`: 20 lines contain `#` in 35 lines (19 full-line comments plus the inline `# the noun, REC-ord`). That matches the brief’s “20 in 35”.
- `accepted_hearings.yaml`: 4 comment lines in 10 lines (2 header comments and one `# review` comment on each of the 2 hearings).
