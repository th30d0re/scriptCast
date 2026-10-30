# L2 Tone findings

## Status

Implemented 2026-10-28. Model: Codex (GPT-5).

The shipped L2 Voices form now includes `tone` in its existing engine switch. Tone uses exactly `name`, `engine`, `instruct`, and `character_profile`. `instruct` is required after trimming whitespace; `character_profile` remains a separate optional field. Tone does not expose reference audio, speed, language, a second reference clip, or Phonon fields.

## Files changed

- `ui/src/project.rs` — added the exact Tone field set and live required validation for `instruct`.
- `ui/src/phase_l.rs` — added the Tone combo option and selected label, Tone speaker chip, and the `Voice description · required` field label in the existing Voices form.
- `ui/tests/phase_l.rs` — added coverage for the exact Tone field set, whitespace-only rejection, and a valid non-empty instruct.
- `CHANGELOG.md` — added one line under `[0.2.0] - Unreleased` / Added for native Tone form support.
- `plans/Native_UI/evidence/l2-tone-findings.md` — recorded implementation scope and verification.

No Python, `tone/`, README, or `/Users/emmanuel/Documents/Theory/TheOriginalPower` files were edited. No second form, second reference clip, or Phonon engine was added.

## Tests run

- `cargo fmt --check` — passed.
- `cargo test --lib tone_uses_distinct_form_and_chip_labels` — passed (1 test). Verifies the required and optional labels plus the Tone combo/chip labels.
- `cargo test --test phase_l tone_engine_has_only_its_shipped_fields_and_requires_instruct` — passed (1 test). Verifies the exact four-field schema and trimmed non-empty requirement.
- `cargo test` — all non-snapshot tests passed. The run reached 138 passed / 2 ignored library tests and all subsequent integration suites through `phase_m`, `runner_presets`, and `script_edit`; the final 10 `tests/snapshots.rs` cases failed before rendering because this environment reported `CustomNativeAdapterSelectionError("No adapter found")` for WGPU.
- `cargo clippy --all-targets -- -D warnings` — blocked by two existing `clippy::too_many_arguments` findings in `ui/src/waveform.rs` (`paint_columns` and `columns_mesh`), outside the files changed for this task.

## Notes

The form remains schema-driven through `engine_fields`. Because Tone's list contains no `reference_audio`, `speed`, or `lang_code`, the existing renderer cannot show those rows for Tone. Engine changes continue to use the existing confirmation and removal flow.
