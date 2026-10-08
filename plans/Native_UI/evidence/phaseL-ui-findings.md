# Phase L UI findings

## Status

Implementation started on 2026-10-08. No GUI launch, synthesis, Python edits, or git writes.

## Files changed

- `ui/tests/fixtures/phase_l/show.json`: captured once using the real `show --json` tool on
  `/tmp/scriptcast-k/proj`, with `/tmp/scriptcast-l/venv/bin/python` and this worktree on PYTHONPATH.
- This findings file tracks implementation, checks, deviations, and unverified behavior.

## Contract blockers discovered

- `config_edit.py::_merge_into` deletes every mapping key absent from `fields`. The requested
  changed-key-only speaker/clip operations therefore cannot preserve required fields. Asked whether full
  mappings are authorized or the Python contract should be fixed first. No Python changes made.
- `show` omits extra clip keys (`linger`, `speaker`, `transcript_source`), so even resending the show
  mapping is insufficient to preserve all clip data with the current replacement operation.
- `edit` supports TOML `set` only. There is no operation that creates a comment-header-only TOML file.
  Direct UI writes are prohibited. This requires a tool contract addition.

## Tests

Not run yet. Planned: fixture/schema, draft operations, stale/validation errors, engine validation,
project discovery/status, real-tool one-line edit, fmt, clippy, and complete cargo tests.

## Unverified

GUI behavior and capture comparison must be run by the orchestrator. No GUI will be launched here.

## Session log

Model: GPT-6 (Codex). The inherited session-log destination in the Obsidian vault is outside the
permitted write roots. This repository-local record serves as the session log for this run.

Requested: project picker and project data editors with tool-mediated, hash-guarded writes.
Work so far: inspected contract, command resolver, existing app/settings/capture/waveform code;
captured real fixture; identified replacement semantics and project creation contract gaps.

Next ideas (six): exercise stale saves; verify comment retention; compare all four mockups; test missing
media; test interrupted jobs; test missing ruamel.yaml reporting.
