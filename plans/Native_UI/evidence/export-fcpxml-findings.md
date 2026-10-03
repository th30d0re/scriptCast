# Standalone FCPXML export findings

Date: 2026-10-03. Model: GPT-6 (Codex).

## Status: implementation and EP03 acceptance complete; full suite pending

The orchestrator accepted existing 30 fps quantization on 2026-10-03.
Implementation is complete without changes to generator timing. Tests compare
rational offsets to rounded manifest starts and enforce the half-frame error
bound, including an exact 5000 ms case. The exporter supplies WAV-header channels
through `SegmentResult.audio_channels`, defaulting to 2 for old callers.
The module docstring and CLI help document rounding.

## Implementation under Addendum 1

- Added `scriptcast.tools.export_session` and the `scriptcast-export` console
  entry point. The required format selector currently accepts only `--fcpxml`;
  `--out` overrides the default `<episode_dir>/<episode_id>.fcpxml`.
- Validates manifest fields used by the export, including unique position keys,
  nonnegative integer times, and relative segment paths within the episode.
  Reads every referenced WAV header via `soundfile.info`, without decoding audio.
  Missing WAV errors list all missing paths before any output is created.
- Writes through a temporary file beside the destination and publishes using
  `os.replace`. Cleanup removes temporary output after generation or replacement
  failures. Existing exports survive failure. Output cannot overwrite an input.
  The destination parent must already exist; no extra directories are created.
- Changed only the channel attribute in the generator. Existing constructors
  and callers still default to stereo; the standalone exporter declares the
  channel count read from each WAV header. Generator timing remains unchanged.
- No UI, Logic exporter, engine, video, or Swift source changes. No git writes.

## Current validation

Targeted command (test scratch confined to the authorized directory):

```sh
PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp/scriptcast-export PYTHONPATH=$PWD \
  python3 -m pytest -q -p no:cacheprovider \
  --basetemp=/tmp/scriptcast-export/pytest-export-final \
  tests/test_export_session.py tests/test_fcpxml_generator.py tests/test_manifest.py
```

```text
............................                                             [100%]
28 passed in 0.26s
```

Coverage includes asset counts, mixed mono/stereo headers, rational rounded
offsets and the half-frame bound, a frame-aligned 5000 ms start, default and
custom outputs, missing WAVs, malformed/missing manifests, invalid fields,
duplicate position keys, unreadable WAV headers, required format and help text,
unchanged input inventories, input overwrite protection, and cleanup/preservation
after both partial generation and replacement failures. A fresh-process module
CLI test blocks imports containing `engine`, `mlx`, or `whisper` and succeeds.
The existing generator tests additionally check legacy stereo and explicit mono.

EP03 command:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=$PWD python3 -m scriptcast.tools.export_session \
  /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels \
  --fcpxml --out /tmp/scriptcast-export/EP03.fcpxml
```

```text
exit: 0
/tmp/scriptcast-export/EP03.fcpxml
fcpxml 1.10 392
audioChannels: ['1']
Rounded offsets: 353 max error ms: 50/3
After SHA-256: b17ae4104871ee0f46dd4b1a2db5b9b3a8cabf990af638d81b06cf7cf6463837
Recursive inventory, sizes, mtimes unchanged: True
```

Before and after manifest SHA-256 match the expected full digest. All 814
recursive entries retained their names, sizes, and modification times; no new
entry appeared. All 392 clip offsets match the generator's rounding rule, and
their exact rational error is at most 1000/60 ms. `--help` displays the required
rounding disclosure. `git diff --check` passes.

## Current remaining work

The full suite imports `scriptcast.engine` in existing tests such as
`tests/test_engine.py` and `tests/test_project.py`. A clarification is pending
on the full-suite requirement versus the blanket engine-import prohibition.
The exporter and targeted tests import no engine/ML modules.

Logic import remains the operator's check. Logic Pro and Final Cut Pro have not
been launched. The brief's AppleScript finding remains untouched.

## Original stop evidence (superseded by Addendum 1)

The brief requires clip offsets equal to manifest `start_ms` and explicitly
instructs the implementer to stop if `generate_fcpxml` cannot produce those
offsets. Inspection found that `_build_asset_clip` passes `start_ms` through
`_ms_to_time_str` and `_time_str`, which rounds all times to 30 fps. The
`position_map` is respected before this rounding, but exact millisecond
positions are lost. EP03 contains 392 segments, 353 with starts that are not
exact multiples of 1/30 second.

No implementation files have been changed. The direct generator reproduction
and baseline tests below confirm the blocker.

## Defects and channel-fix approach

- Timing: existing 30 fps quantization conflicts with the exact-offset
  acceptance requirement. Fixing this requires a generator change beyond the
  authorized channel-count fix, or an explicit acceptance of rounded offsets.
- Channels: `_build_asset` hardcodes `audioChannels="2"`. The intended fix is
  an optional `SegmentResult` channel-count field with a default of 2; the
  standalone exporter would supply `soundfile.info(wav_path).channels`.
  This preserves existing constructors and generator callers. Not implemented
  because the timing stop condition was reached first.
- The brief's separate Logic AppleScript finding is left untouched and was
  not independently rechecked.

## Validation

### Direct EP03 reproduction (existing generator)

Read the manifest, constructed all 392 `SegmentResult` objects, and supplied
every `(turn_id, chunk_index): start_ms` in `position_map`. Read WAV headers
using `soundfile.info`; no audio was synthesized or decoded. Called the
unmodified generator with output confined to:

`/tmp/scriptcast-export/EP03-existing-generator-probe.fcpxml`

The diagnostic exited 0 and produced:

```text
root/version/assets: fcpxml 1.10 392
WAV header channel counts: [1]
XML audioChannels: ['2']
offset mismatches: 353
first mismatch:
  turn_id: 0a8a2240770c2d72, chunk_index: 0
  manifest start_ms: 8620
  XML offset: 259/30s (8633.333333... ms)
  error: +13.333333... ms
maximum absolute offset error: 16.666666... ms
directory inventory, sizes, mtimes unchanged: True
forbidden imports: []
```

Offsets were compared using `fractions.Fraction`, so the mismatch count is
based on exact rational arithmetic. The existing generator does not
re-sequence mapped clips; it quantizes their positions during serialization.
The existing position-map test uses frame-aligned starts (0 and 5000 ms),
which do not expose this limitation.

Manifest SHA-256 before and after both equal:

`b17ae4104871ee0f46dd4b1a2db5b9b3a8cabf990af638d81b06cf7cf6463837`

The recursive episode directory inventory, file sizes, and modification
times were identical before and after. No new file appeared in EP03.
The diagnostic XML is evidence of existing behavior, not a completed export
acceptance artifact: it still declares stereo and has rounded offsets.

### Baseline tests

```sh
PYTHONDONTWRITEBYTECODE=1 TMPDIR=/tmp/scriptcast-export PYTHONPATH=$PWD \
  python3 -m pytest -q -p no:cacheprovider \
  --basetemp=/tmp/scriptcast-export/pytest-baseline \
  tests/test_fcpxml_generator.py tests/test_manifest.py
```

```text
....                                                                     [100%]
4 passed in 0.03s
```

Bytecode and pytest cache writes were disabled, with temporary test files
confined to the allowed scratch directory. The requested three-file test
command cannot run because implementation stopped before creating
`tests/test_export_session.py`.

## Original deferred work (now authorized)

The standalone command, console entry point, channel-count fix, new export
tests, requested CLI acceptance run, and full-suite run are deferred at the
explicit stop condition. Logic import remains the operator's check.

To resume, the orchestrator must resolve whether exact millisecond offsets
are required (authorizing the necessary timing change), or the existing
30 fps rounding is acceptable. No timing change was attempted.

The parent AGENTS.md requests an external Obsidian log and git checkpoints;
the brief forbids external writes and all git writes. This findings file is
the session record, and no git writes are performed.
