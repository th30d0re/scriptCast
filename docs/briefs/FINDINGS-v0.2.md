# v0.2 findings

## Scope and brief defects

- This run implements Phase 1 only. The user's instructions override the brief's branch creation and commit instructions; no git mutations or history commands will be run.
- Phases 2–4 and their acceptance commands are out of scope.

## Phase 1

This file was created before any code change. Phase 1 implementation is present, but acceptance is blocked by the existing path-only configuration test described below.

## Not verified

- Real synthesis and real-episode rendering, DAW/FCPXML round trips, and Apple container service are not exercised; this run is offline and uses synthetic WAV data only.

### Inspection findings

- Additional tail-loss bug: `render_loop` calls `_load_completed_turn_segments` without forwarding `tail_ms`. Fixing only `_segment_from_wav` would still lose a caller's explicit tail override during resume.
- Configuration contract conflict: `tests/test_project.py::test_defaults` iterates every `DEFAULTS` entry and asserts `project.path(key) == tmp_path / default`. Required numeric defaults cannot satisfy this path-only contract. No existing test expectation has been edited. Asked whether the configuration test may be adapted while preserving existing numeric expectations.
- `Project.__init__` currently resolves every default as a path; numeric settings need separate storage and validation.
- Persistence ambiguity: reloading WAVs recalculates durations using CLI settings, not the state's stored `speech_duration_ms`. Updating state durations alone cannot preserve arbitrary refit overrides on a later render with different settings. The brief does not define settings precedence/persistence for this case.
- `--detect-changes` currently returns after reporting changes; it does not reload WAVs, contrary to the brief's description.

Read-only probe of the existing test's path expression with the required defaults:

```text
tail_ms=400: TypeError: unsupported operand type(s) for /: 'PosixPath' and 'int'
speech_threshold=0.03: TypeError: unsupported operand type(s) for /: 'PosixPath' and 'float'
```

### Implementation and decisions

- Forwarded `tail_ms` at both reload call sites; `__main__.py` signatures now use `project.DEFAULTS` as the sole source of numeric defaults.
- Added validated numeric project settings separately from path settings. CLI flags override project values.
- Added and registered `scriptcast-refit`: reads existing WAVs without synthesis, updates measured physical/speech durations and segment/turn ends, preserves starts unless `--gap-ms` is provided, and backs up the manifest to `.json.bak`.
- Refit updates matching state segments and stores `refit_settings` in state. Subsequent renderer precedence is explicit CLI flag > saved refit setting > project setting. This resolves the custom-tail persistence ambiguity without modifying a project's TOML. This precedence is an implementation decision extending the brief's simple project-default rule.
- With `--gap-ms`, refit regenerates the ALS using the existing generator. Without it, it does not regenerate exports. Existing MP3/FCPXML/Logic exports must be regenerated separately. WAVs are unchanged, so waveform caches are left intact.
- Missing WAVs fail before JSON writes. Legacy state is rejected with migration guidance. Missing state is reported rather than fabricated (source fingerprints cannot be recovered safely from the manifest).
- Added eight synthetic tests covering unchanged reload with two tails, duration growth/capping/idempotence, saved-settings reload, relayout/state/ALS dispatch, missing WAV safety, numeric configuration, and refit CLI overrides.
- Added the 0.2.0 Unreleased changelog entry. No existing tests or expectations were edited. No git commands, network calls, or real synthesis were used. The manuscript repository and README were not edited. No work on Phases 2–4.

### Acceptance blocker

`tests/test_project.py::test_defaults` fails at `project.path("tail_ms")` with `KeyError`. Its subsequent expression `tmp_path / 400` would also fail. This is a path-vs-number contract conflict, not an audio expected-number change. All other 115 existing tests pass, as do all eight new tests. No old audio expectation changed. A clarification request to adapt the configuration test remains unanswered; the test is untouched.

### Verification limits and known limitations

- Full-suite green acceptance is not achieved until the path-only configuration test is reconciled with numeric defaults.
- Full CLI `--precision-insert` after refit was not exercised end to end. Synthetic tests verify state deserialization and reloading with saved settings; the CLI precedence path was inspected.
- Actual regenerated ALS output from refit was not visually opened in Ableton; the new relayout test checks generator dispatch with a stub. Existing ALS generator tests pass.
- No installed console entry-point smoke test was run; registration is in pyproject.toml and refit `main()` is tested directly.
- Real audio perceptual quality, real-episode synthesis/rendering, DAW/FCPXML round trips, and Apple container service remain unverified, as instructed.
- Optional refit gap values update state positions/gaps for the current episode, but a later precision insert recomputes its timeline using that render's `--gap-ms`; custom gaps must be supplied again. Only speech-tail settings are persisted as renderer defaults.
- Without render_state.json, refit cannot preserve its custom settings in a later incremental render; it prints this limitation. Supply matching project settings or CLI flags.
- JSON replacements and ALS generation are not a transactional multi-file operation; interrupted writes were not tested.
- No Phase 2–4 acceptance or live API checks were performed (out of scope).

### Final pytest output (raw)

Command: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`

```text
...................................................................F.... [ 58%]
....................................................                     [100%]
=================================== FAILURES ===================================
________________________________ test_defaults _________________________________

tmp_path = PosixPath('/private/var/folders/95/ps3stnmx59j3ws19bprd4h9r0000gn/T/pytest-of-emmanuel/pytest-4/test_defaults0')

    def test_defaults(tmp_path) -> None:
        project = Project(tmp_path)
        for key, default in DEFAULTS.items():
>           assert project.path(key) == tmp_path / default
                   ^^^^^^^^^^^^^^^^^

tests/test_project.py:34: 
_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 

self = <scriptcast.project.Project object at 0x1114c5210>, key = 'tail_ms'

    def path(self, key: str) -> Path:
        if key not in self._paths:
>           raise KeyError(
                f"unknown project key {key!r}; known keys: {', '.join(sorted(self._paths))}"
            )
E           KeyError: "unknown project key 'tail_ms'; known keys: clip_sources, clips, omnivoice_python, outputs, pronunciations, references, scripts, speaker_rates, voices"

scriptcast/project.py:85: KeyError
=============================== warnings summary ===============================
tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/torch/jit/_script.py:1488: DeprecationWarning: `torch.jit.script` is deprecated. Please switch to `torch.compile` or `torch.export`.
    warnings.warn(

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyPacked has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyObject has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:143: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_gold.json") as r:

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:145: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_silver.json") as r:

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
=========================== short test summary info ============================
FAILED tests/test_project.py::test_defaults - KeyError: "unknown project key ...
1 failed, 123 passed, 5 warnings in 4.76s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

## Phase 2

### Scope and initial inspection

- This run implements Phase 2 only; the earlier Phase 1 record above is preserved verbatim. The user reports Phase 1 committed and a corrected baseline of 124 passing tests; its historical blocker text is not the current acceptance status.
- Read the full brief and reference shotspec implementation. User instructions override the brief's commit and later-phase instructions. No git commands, network, synthesis, README edits, or manuscript-repository writes are planned.
- Preserve `_manifest_lookup` unchanged, including first-match behavior and fallback when no script mapping is available.
- Manuscript checks will be enabled by either `--book` or `--repo`. `--repo` retains the reference's default manuscript location `Paper/The_Original_Power.tex`; `--book` selects a manuscript (relative to `--repo`, or the working directory when no root is supplied). Generic URLs/cards remain available in either mode.

### Implementation and comparison findings

- Added the tool, console registration, neutral Tiny Shapes fixtures, and synthetic tests. No Phase 3 or 4 work.
- Real-data comparison passed all required fields for all 16 shots, including complete anchors. Prompt seeds also match. Differences are limited to newly recognized source tags, generic URL/card citations, and validation. G-03, G-04, and G-16 have source tags without a URL/card reference and correctly report errors; non-strict execution still exits zero.
- Initial test run: 137 passed and two new parametrized tests failed because their test argv slice left a dangling `--book`; corrected the test harness. No existing expectations changed.
- Reference behavior retained: nonexistent manuscripts skip quote/label checks, explicit missing cited files error, and only lines past EOF are rejected (zero line numbers are not newly rejected).

### Final acceptance output (raw)

Command: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`

```text
........................................................................ [ 51%]
...................................................................      [100%]
=============================== warnings summary ===============================
tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/torch/jit/_script.py:1488: DeprecationWarning: `torch.jit.script` is deprecated. Please switch to `torch.compile` or `torch.export`.
    warnings.warn(

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyPacked has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyObject has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:143: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_gold.json") as r:

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:145: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_silver.json") as r:

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
139 passed, 5 warnings in 4.24s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

Real-data command ran from `/Users/emmanuel/Documents/Theory/TheOriginalPower`, using its `.venv-voice/bin/python`, with `PYTHONPATH=/Users/emmanuel/Documents/Theory/scriptCast` to select this implementation and `PYTHONDONTWRITEBYTECODE=1` to prevent writes there:

```text
python -m scriptcast.tools.shotspec Architecting_the_operation/video/chapter135_rebuttal_shotlist.md --script Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md --manifest outputs/chapter135_reply/episode_manifest.json --out /tmp/shotspec_new.json
```

```text
Skipping non-speaker preamble line 1 in Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md
chapter135_rebuttal_shotlist.md: 16 shots, 16 anchored to real time, 16 vector
  error G-03     tagged [source] with no URL or card reference
  error G-04     tagged [source] with no URL or card reference
  error G-16     tagged [source] with no URL or card reference
wrote /tmp/shotspec_new.json
Compared 16 shots.
anchor.start_ms differences: 0
Required-field differences: []
Identical fields: id, title, anchor, hold, type, described, note, suggested_track
Differing field provenance.citations: G-01, G-02, G-05, G-06, G-07, G-08, G-09, G-10, G-11, G-12, G-13, G-14, G-15
Differing field provenance.tags: G-01, G-02, G-03, G-04, G-05, G-06, G-07, G-08, G-09, G-10, G-11, G-12, G-13, G-14, G-15, G-16
Differing document field: validation
prompt_seed differences: 0
```

The comparison checked equal shot counts, every required field per shot, all provenance subfields, prompt seeds, and document-level fields. An AST comparison also verified `_manifest_lookup` is identical to the reference. All 124 existing tests and 15 new cases passed.

### Verification limits

- Installed `scriptcast-shotspec` console script was not smoke-tested: `command -v scriptcast-shotspec` exited 1. No installation was attempted in the read-only reference environment. Module execution and CLI `main()` were tested; pyproject registration is present.
- Opt-in manuscript validation was tested with synthetic files, not against the real manuscript. The required real-data comparison used default generic mode.
- No synthesis, audio/perceptual checks, DAW round trips, container checks, Phase 3/4 tooling, live APIs, or video rendering were performed; all are outside Phase 2 scope.
- No git commands ran, changes remain uncommitted, Phase 1 findings and README are untouched, and scratch output is under `/tmp`.

## Phase 3a

### Scope and decisions

- Read the whole brief and earlier findings. Implementing cards and `still` only; no Episode, render subcommand, or Phase 4. No git commands or manuscript writes.
- Reference HTML is full-bleed and puts QR above sources; this implementation follows the user's safe-zone and QR-below-sources requirements while preserving copy. Group maxima, rather than the reference's shared maximum, determine bar widths as requested.
- Exact percentage boundaries are top 268.8, bottom 1248, sides 64.8. A conservative rectangular usable box ends at y=1152 to avoid the deeper bottom-right overlay across its entire width.
- Markup verification will use TypeScript compilation and Node with react-dom/server, without installing tsx. Network is limited to the authorized npm packages and dependencies.

### Implementation and verification

- Added strict TypeScript workspace and four registered 1080x1920 / 30fps / one-frame compositions, shared palette/system typography, JSON props, sources followed by an outlined 120x120 QR placeholder. svgAsset is a reserved public-relative slot, validated for absolute paths/traversal but deliberately not loaded until Phase 4.
- G-10 and G-11 preserve all reference prose and numbers. Labels are outside narrow bars so small values remain readable. The reference's extra gray #8697b8 is replaced with the requested palette's mute color.
- Every card uses one bounded main container inside the conservative usable box. Internal content flows normally and overflow is clipped to prevent content entering excluded zones. Arbitrarily large JSON may be clipped; automatic text fitting is not implemented.
- Python still resolves paths against caller cwd, reads the component from JSON or override, validates supported compositions, chooses SCRIPTCAST_CHROME then installed macOS Chrome then Remotion default, and propagates the child exit code. Uses npx --no-install to prevent implicit package downloads. Requires the adjacent source workspace and installed Node dependencies; wheel-only installs do not bundle video.
- Nine neutral Python cases cover command construction, browser precedence/fallback, paths containing spaces, component override, malformed/missing component data, and missing dependencies. All 139 prior tests remain green.
- Markup check compiles with installed tsc and uses react-dom/server. Checks every example prose field plus required strings/numbers, all four layouts' shared safe-zone constants/containment, QR ordering/dimensions, and group-max/zero-value bar calculations. A narrow local declaration covers renderToStaticMarkup because @types/react-dom is outside the package allowlist.
- Initial tsc errors (createElement generic inference, ES2020 replaceAll, missing server declarations) were corrected using JSX Root, ES2021 target, and the narrow declaration. Final tsc and markup checks pass.
- npm default cache was unwritable (EPERM); retry succeeded with /tmp/scriptcast-npm-cache, disabling audit/funding/update checks. No license acknowledgement or paid step appeared.
- Installed versions: remotion 4.0.354; @remotion/cli 4.0.354; react 19.1.0; react-dom 19.1.0; typescript 5.9.3; @types/react 19.1.0. package-lock.json records dependencies.
- All four still attempts failed to launch Chrome in this sandbox (three explicitly reported SIGABRT). No PNG was produced. No license prompt appeared before browser failure.

### Unverified items

- PNG dimensions, visual readability, unclipped example text, comparison against HTML screenshots, and byte-identical repeated renders remain unverified because Chrome cannot launch here. Static markup proves text presence and constant-based containment, not browser geometry/visibility.
- Installed scriptcast-video console script was NOT smoke-tested: command -v scriptcast-video exited 1 with no output. No pip installation was attempted in the read-only reference environment. Tests exercise main(); pyproject.toml registers the entry point.
- Remotion browser-download fallback was checked only through mocked arguments, not invoked because downloads are outside the network allowlist.
- svgAsset rendering is reserved for Phase 4. Episode assembly, render subcommand, full episode/reel/audio rendering, real synthesis, DAW round trips, container service, and live APIs were not attempted; outside Phase 3a.
- Changes are uncommitted; no git commands, README edits, or manuscript repository edits were performed.

### Acceptance output (raw)

Initial npm install (default cache; failed)

```text
npm error code EPERM
npm error syscall open
npm error path /Users/emmanuel/.npm/_cacache/tmp/fd066eef
npm error errno EPERM
npm error
npm error Your cache folder contains root-owned files, due to a bug in
npm error previous versions of npm which has since been addressed.
npm error
npm error To permanently fix this problem, please run:
npm error   sudo chown -R 501:20 "/Users/emmanuel/.npm"
npm notice
npm notice New major version of npm available! 10.9.2 -> 12.1.0
npm notice Changelog: https://github.com/npm/cli/releases/tag/v12.1.0
npm notice To update run: npm install -g npm@12.1.0
npm notice
npm error Log files were not written due to an error writing to the directory: /Users/emmanuel/.npm/_logs
npm error You can rerun the command with `--loglevel=verbose` to see the logs in your terminal
```

npm install --cache /tmp/scriptcast-npm-cache --no-audit --no-fund --update-notifier=false (exit 0)

```text

added 179 packages in 10s
```

/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q (exit 0)

```text
........................................................................ [ 48%]
........................................................................ [ 97%]
....                                                                     [100%]
=============================== warnings summary ===============================
tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/torch/jit/_script.py:1488: DeprecationWarning: `torch.jit.script` is deprecated. Please switch to `torch.compile` or `torch.export`.
    warnings.warn(

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyPacked has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  <frozen importlib._bootstrap>:241: DeprecationWarning: builtin type SwigPyObject has no __module__ attribute

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:143: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_gold.json") as r:

tests/test_pronunciation.py::test_reading_follows_part_of_speech[It includes one specific historical record.-record-default]
  /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/lib/python3.11/site-packages/misaki/en.py:145: DeprecationWarning: open_text is deprecated. Use files() instead. Refer to https://importlib-resources.readthedocs.io/en/latest/using.html#migrating-from-legacy for migration advice.
    with importlib.resources.open_text(data, f"{'gb' if british else 'us'}_silver.json") as r:

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
148 passed, 5 warnings in 6.63s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

npx --no-install tsc --noEmit (exit 0; empty stdout/stderr)

```text
```

npm run check-copy (exit 0)

```text

> scriptcast-video@0.2.0 check-copy
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-copy.js

PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.
npm notice
npm notice New major version of npm available! 10.9.2 -> 12.1.0
npm notice Changelog: https://github.com/npm/cli/releases/tag/v12.1.0
npm notice To update run: npm install -g npm@12.1.0
npm notice
```

npx --no-install remotion still src/index.ts TimelineCard /tmp/g10.png --props=examples/g10.json --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" (exit 1)

```text
Bundling 6%
Bundling 24%
Bundling 38%
Bundling 45%
Bundling 65%
Bundling 71%
Bundling 76%
Bundling 81%
Bundling 87%
Bundling 92%
Bundling 98%
Bundling 100%
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (244kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (136kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (514kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (109kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (166kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (1381kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (109kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (166kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (1381kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (244kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (244kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (109kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (166kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (1381kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (109kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (166kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (1381kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (109kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (503kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (166kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (1381kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (136kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (136kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
<w> [webpack.cache.PackFileCacheStrategy] Serializing big strings (514kiB) impacts deserialization performance (consider using Buffer instead and decode when needed)
Getting compositions
⚡️ Cached bundle. Subsequent renders will be faster.

Error: Failed to launch the browser process!
Troubleshooting: https://remotion.dev/docs/troubleshooting/browser-launch
    at Socket.onClose (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261:20)
    at Socket.emit (node:events:519:35)
    at Pipe.<anonymous> (node:net:351:12)
```

npx --no-install remotion still src/index.ts StatBarsCard /tmp/g11.png --props=examples/g11.json --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" (exit 1)

```text
Bundling 6%
/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261
            reject(new Error([
                   ^

Error: Failed to launch the browser process!
Error: Closed with null signal: SIGABRT
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:32)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)
Troubleshooting: https://remotion.dev/docs/troubleshooting/browser-launch
    at onClose (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261:20)
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:24)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)

Node.js v23.7.0
```

Same TimelineCard command with /tmp/g10_b.png (exit 1)

```text
Bundling 6%
/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261
            reject(new Error([
                   ^

Error: Failed to launch the browser process!
Error: Closed with null signal: SIGABRT
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:32)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)
Troubleshooting: https://remotion.dev/docs/troubleshooting/browser-launch
    at onClose (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261:20)
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:24)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)

Node.js v23.7.0
```

Same StatBarsCard command with /tmp/g11_b.png (exit 1)

```text
Bundling 6%
/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261
            reject(new Error([
                   ^

Error: Failed to launch the browser process!
Error: Closed with null signal: SIGABRT
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:32)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)
Troubleshooting: https://remotion.dev/docs/troubleshooting/browser-launch
    at onClose (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:261:20)
    at ChildProcess.<anonymous> (/Users/emmanuel/Documents/Theory/scriptCast/video/node_modules/@remotion/renderer/dist/browser/BrowserRunner.js:252:24)
    at ChildProcess.emit (node:events:507:28)
    at ChildProcess._handle.onexit (node:internal/child_process:294:12)

Node.js v23.7.0
```

file /tmp/g10.png /tmp/g11.png

```text
/tmp/g10.png: cannot open `/tmp/g10.png' (No such file or directory)
/tmp/g11.png: cannot open `/tmp/g11.png' (No such file or directory)
```

shasum /tmp/g10.png /tmp/g10_b.png /tmp/g11.png /tmp/g11_b.png (exit 1)

```text
shasum: /tmp/g10.png: No such file or directory
shasum: /tmp/g10_b.png: No such file or directory
shasum: /tmp/g11.png: No such file or directory
shasum: /tmp/g11_b.png: No such file or directory
```

Module-only help smoke test (exit 0; this does not test an installed console script):

```text
usage: video.py [-h] {still} ...

Render JSON cards using the adjacent, separately installed Remotion workspace.

positional arguments:
  {still}
    still     Render one card to PNG

options:
  -h, --help  show this help message and exit
```
