# Addendum 1 to export-fcpxml.md (orchestrator decision, 2026-10-03)

Your stop was correct, and your findings are accepted: `generate_fcpxml` quantizes every time to the nearest 1/30 s frame (`fcpxml_generator.py:15-40`). On EP03 that moves 353 of 392 clips by at most 16.67 ms.

**Decision: accept the existing 30 fps quantization. Do not change the generator's timing.** It is the documented existing behaviour, it is what the render's own `--fcpxml` already produces, and a half-frame error is inaudible in dialogue. Logic's import is the operator's check.

Changes to the original brief:
- **Acceptance:** each clip offset must equal the manifest `start_ms` rounded to the nearest 1/30 s, exactly as `_ms_to_time_str` does, so `|offset - start_ms| <= 1000/60 ms`. Assert that with `fractions.Fraction` in `tests/test_export_session.py`, plus one test that a frame-aligned start (for example 5000 ms) is exact.
- Everything else in `export-fcpxml.md` stands: the standalone command, the channel-count fix (your proposed optional `SegmentResult` field defaulting to 2, with the exporter supplying `soundfile.info(...).channels`), the console script, temp-file and `os.replace` writes, failures that write nothing, the EP03 run to `/tmp` with 392 assets and `audioChannels="1"`, EP03 unchanged, and the full-suite run report.
- Add a line to the module docstring and the `--help` text: "Clip times are rounded to the nearest 1/30 s frame (FCPXML timeline), at most 16.7 ms from the manifest."

Continue from where you stopped. Update your findings file.
