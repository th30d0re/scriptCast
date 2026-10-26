# scriptcast-export: write an FCPXML from an existing render (implementer brief)

Your working directory is a git worktree of the scriptCast repo on branch `py/export-fcpxml`, created from `dev`. The Python package is `scriptcast/`, and the tests are pytest files in `tests/`.

## Prohibitions (read first)

- **No git writes** (commit, push, branch, checkout, reset, restore, clean, stash, rebase, merge). The orchestrator reviews and commits.
- **Write only inside this worktree**, plus scratch under `/tmp/scriptcast-export/`.
- **Real episodes are read-only:** never write under `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. To try the tool on EP03, pass `--out /tmp/scriptcast-export/EP03.fcpxml`.
- **Do not touch** `ui/**`, `plans/**` (except the findings file below), `video/`, `swift/`, or any module outside the files you own.
- **Never synthesize audio, and never import MLX, Whisper or an engine module.** This tool only reads the manifest and WAV headers.
- **Do not run Logic Pro or Final Cut Pro.** Opening the file in Logic is the operator's check.

## Goal

The native UI will get an "Open in Logic" button. Logic Pro 12.4 opens `.fcpxml` files directly: its Info.plist declares the document type "Final Cut Pro XML" with `com.apple.finalcutpro.xml` as a Viewer. But scriptCast only writes FCPXML at the end of a full render (`scriptcast/__main__.py:1039-1042`, the `--fcpxml` flag). Add a standalone command that writes the same FCPXML for an episode that is **already rendered**, with no synthesis.

## Established facts (measured 2026-10-03)

- `generate_fcpxml(segment_results, output_path, position_map=None, episode_id="episode") -> Path` is in `scriptcast/fcpxml_generator.py:104`.
  - `position_map` maps `(turn_id, chunk_index)` to `start_ms` (line 146).
  - It reads each `SegmentResult`'s `wav_path`, `duration_ms` and `sample_rate`.
  - **Defect:** `_build_asset` hardcodes `"audioChannels": "2"` (line 65). Every scriptCast WAV checked is mono: EP03 `Samples/Processed/toussaint/eda1cfb57abfc51b_chunk_0000.wav` is 1 channel, 48 kHz, Int16 (afinfo). A mono file declared as stereo can import wrongly. Fix it by giving `SegmentResult` an optional channel count, or by reading the WAV header in the generator. Keep the existing behaviour for callers that pass nothing (default "2"), and keep `tests/test_fcpxml_generator.py` green.
- `SegmentResult` (`scriptcast/models.py:37-49`) has these fields: `turn_index, turn_id, chunk_index, speaker_id, wav_path: Path, duration_ms, speech_duration_ms, sample_rate, gap_after_ms, checksum`.
- `episode_manifest.json` (schema in `scriptcast/models.py:73-118`, written by `scriptcast/manifest.py`) gives everything needed:
  - top level: `episode_id`, `sample_rate`;
  - per turn: `turn_index`, `turn_id`, `speaker_id`, and `segments[]` with `chunk_index`, `segment_wav` (relative to the episode dir), `duration_ms`, `speech_duration_ms`, `start_ms`, `gap_after_ms`, `checksum`.
- The existing render path writes `<episode_dir>/<episode_id>.fcpxml` (`__main__.py:1039`).
- `soundfile` 0.13.1 is installed and can read WAV channel counts (`soundfile.info(path).channels`).
- Python tests run with `PYTHONPATH=$PWD python3 -m pytest -q tests/...` (Python 3.11). This already passes for `tests/test_fcpxml_generator.py` and `tests/test_manifest.py`.
- **Separate finding (do not fix):** `scriptcast/logic_exporter.py` generates AppleScript using `make new project`, `make new audio track` and `import audio file`. Logic Pro 12.4's scripting dictionary (`sdef "/Applications/Logic Pro.app"`) has only the Standard Suite (document, window, open, save, close, make…). It has no project, track or import terms, so `--logic` cannot work. Leave that module untouched; this FCPXML route replaces it for the UI.

## Files you own

- `scriptcast/tools/export_session.py` (new)
- `scriptcast/fcpxml_generator.py` (only the channel-count fix)
- `scriptcast/models.py` (only if you add an optional field to `SegmentResult`, with a default so every existing constructor still works)
- `pyproject.toml` (add the console script `scriptcast-export = "scriptcast.tools.export_session:main"` under `[project.scripts]`)
- `tests/test_export_session.py` (new), and `tests/test_fcpxml_generator.py` (add a mono-channel test)
- the findings file `plans/Native_UI/evidence/export-fcpxml-findings.md`

## Behaviour

```
scriptcast-export <episode_dir> --fcpxml [--out PATH]
```

- Read `<episode_dir>/episode_manifest.json` and fail with a clear message if it is missing or invalid.
- Build one `SegmentResult` per manifest segment, with `wav_path = episode_dir / segment_wav` and the channel count from the WAV header (`soundfile.info`). Build `position_map[(turn_id, chunk_index)] = segment.start_ms`.
- **If any WAV is missing:** exit 1, list the missing paths, and write nothing.
- Default output is `<episode_dir>/<episode_id>.fcpxml`; `--out` overrides it. Write through a temp file in the same directory, then `os.replace`, so a failure never leaves a half-written file.
- Print the output path on success. Exit 0.
- `--fcpxml` is required for now. Design the CLI so other formats can be added later (`--aaf`), but do not add them.
- No other files are created or modified, and no engine or ML imports.

## Acceptance (run these)

```bash
PYTHONPATH=$PWD python3 -m pytest -q tests/test_export_session.py tests/test_fcpxml_generator.py tests/test_manifest.py
PYTHONPATH=$PWD python3 -m scriptcast.tools.export_session /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels --fcpxml --out /tmp/scriptcast-export/EP03.fcpxml
python3 -c "import xml.etree.ElementTree as E; r=E.parse('/tmp/scriptcast-export/EP03.fcpxml').getroot(); print(r.tag, r.get('version'), len(r.findall('.//asset')))"
```

The EP03 run must:
- exit 0;
- produce 392 `<asset>` elements, one per segment;
- declare `audioChannels="1"` throughout;
- leave the EP03 directory unchanged: `shasum -a 256 episode_manifest.json` before and after must match `b17ae4104871ee0f`…, and no new file may appear in that directory.

`tests/test_export_session.py` must cover, on a temp copy of a small manifest with tiny generated WAVs:
- asset count;
- clip offsets equal to the manifest `start_ms`;
- `audioChannels` matching the WAV;
- default output path;
- `--out`;
- a missing WAV, which gives exit 1 and writes nothing;
- an invalid manifest;
- no extra files written.

Also run the whole suite once (`PYTHONPATH=$PWD python3 -m pytest -q`) and report any failures. Pre-existing failures unrelated to your files are reported, not fixed.

## Not yours to claim

Whether Logic imports the file correctly. The operator opens it in Logic.

## Stop conditions

If `generate_fcpxml` cannot produce correct offsets from the manifest's `start_ms` (for example, it re-sequences clips despite `position_map`), stop. Record the evidence in the findings file and do not rewrite the generator beyond the channel fix.

## Findings file

Write `plans/Native_UI/evidence/export-fcpxml-findings.md` early. Record:
- defects found;
- the channel-fix approach;
- test output;
- the EP03 asset count and checks;
- anything left undone.
