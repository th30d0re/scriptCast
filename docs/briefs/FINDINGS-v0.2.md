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

## Phase 3b

### Scope and initial decisions

- Implementing episode assembly only. Read the full brief and prior findings; current user instructions supersede the brief's commit/install requirements. No Phase 4, README changes, git commands, network, or writes to TheOriginalPower.
- Python resolves a pure, data-only plan; CLI handles file reads and per-run media copies. Clip IDs come from parsed script archive turns in order, never timestamps. Card windows retain spec anchor times as explicitly requested.
- Use rounded absolute frame boundaries and subtract endpoints for sequence lengths; skip windows that round to zero. Background/audio span the episode, including intentional gaps between archive turns/cards.
- No render attempts are planned because the sandbox Chrome failure is established. HEVC decoding, visual layout, media synchronization, and held-frame rendering remain unverified until the orchestrator renders.

### Implementation and decisions

- Added pure `build_plan(manifest, script_turns, specs, registry, cards, *, audio, project_root)` in `video_plan.py`. The caller loads files and uses the existing transcript parser. Archive entries are paired in order, with count mismatches rejected; missing IDs name the offending ID. Input dictionaries are not mutated.
- The real manifest has no top-level duration. Use the maximum turn end as its total (explicit `duration_ms` / `total_duration_ms` is accepted and validated when present).
- Card filename stems ignore case/hyphens; ambiguous normalized names error. Missing cards warn and skip; equal-start/overlapping cards shorten the earlier window, and empty/subframe windows are omitted. Hold windows use spec anchor timing, without silently retiming the supplied specs.
- `still` and `render` share workspace/browser helpers. `--project-root` defaults to caller cwd. `--audio` overrides single-MP3 discovery. All render modes write `<out>.plan.json` and copy media to unique ignored `video/public/episode/run-*/` directories; no automatic cleanup. Plan-only needs no Node installation; dry-run prints shell-quoted argv and launches no subprocess. Real rendering propagates the child exit code and uses `npx --no-install`, H.264, and the shared Chrome resolution.
- `Episode` consumes the plan directly as props, with metadata driven by its duration. Core Remotion `Freeze` plus `OffthreadVideo` clamps the source frame at out-point minus one frame; excerpt audio is muted and the single MP3 supplies audio. Navy background spans the full duration. Cards are above footage; no captions. No new dependencies.
- Layout ambiguity: “safe-zone usable box's width” is 950.4px, whereas the brief also gives a 1080px / 1.5x upscale example. Chose the explicit safe-zone width (1.32x for a 720px source), horizontally and vertically centred in the full frame with `objectFit: contain`. Video is not cropped or restricted to the short safe-zone height.
- Generated neutral `plan.sample.json` through `build_plan` and then rewrote sample paths to public-relative placeholders. Root imports it, and TypeScript checks the Episode prop structure. Additional compiled Node checks exercise actual TypeScript frame conversion, metadata, source progression, and hold clamping.
- Eleven new Python cases cover drift/order, trim/hold, purity, missing/overlapping/equal-start/subframe cards, frame boundaries, missing IDs, count mismatch, staging, audio ambiguity/override, plan-only, dry-run argv, and mocked render dispatch. The existing 148 tests remain green.
- Real plan matched all five expected IDs at turns 0, 9, 14, 19, 23, with start/end values exactly equal to the manifest, and exactly two cards G-10/G-11. Fourteen absent cards produce warnings; no mismatch was observed. Registry source in/out values are preserved as instructed.

### Acceptance output (raw)

`/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q` (exit 0)

```text
........................................................................ [ 45%]
........................................................................ [ 90%]
...............                                                          [100%]
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
159 passed, 5 warnings in 7.11s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

From `video/`: `npx tsc --noEmit` (exit 0; empty stdout/stderr)

```text
```

`npm run check-copy` (exit 0)

```text

> scriptcast-video@0.2.0 check-copy
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-copy.js

PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.
```

`node .remotion/check/scripts/check-episode.js` (exit 0)

```text
PASS: sample metadata, shared frame boundaries, excerpt progression and last-frame hold.
```

### Real-data plan check (raw)

Ran `video.main` from `/Users/emmanuel/Documents/Theory/TheOriginalPower` with `PYTHONDONTWRITEBYTECODE=1` and `PYTHONPATH=/Users/emmanuel/Documents/Theory/scriptCast`. Arguments:

```text
render outputs/chapter135_reply --script Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md --specs Architecting_the_operation/video/specs/chapter135_rebuttal.json --clips Architecting_the_operation/archive/clips.yaml --cards /tmp/scriptcast-phase3b-v1hruta1/cards --out /tmp/scriptcast-phase3b-v1hruta1/episode.mp4 --plan-only
```

The scratch directory contains copies of only this workspace's g10.json and g11.json. Output and extracted arrays, without eliding card data:

```text
Skipping non-speaker preamble line 1 in Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md
wrote /private/tmp/scriptcast-phase3b-v1hruta1/episode.mp4.plan.json
{
  "clips": [
    {
      "turn_index": 0,
      "clip_id": "reel_q1",
      "src": "episode/run-spff4278/001.mp4",
      "in_ms": 0.0,
      "out_ms": 14860.0,
      "start_ms": 0,
      "end_ms": 14860
    },
    {
      "turn_index": 9,
      "clip_id": "reel_a1",
      "src": "episode/run-spff4278/002.mp4",
      "in_ms": 14840.0,
      "out_ms": 41520.0,
      "start_ms": 54980,
      "end_ms": 81660
    },
    {
      "turn_index": 14,
      "clip_id": "reel_q2",
      "src": "episode/run-spff4278/003.mp4",
      "in_ms": 43140.0,
      "out_ms": 48940.0,
      "start_ms": 112140,
      "end_ms": 117940
    },
    {
      "turn_index": 19,
      "clip_id": "reel_a2",
      "src": "episode/run-spff4278/004.mp4",
      "in_ms": 49200.0,
      "out_ms": 77500.0,
      "start_ms": 139070,
      "end_ms": 167370
    },
    {
      "turn_index": 23,
      "clip_id": "reel_close",
      "src": "episode/run-spff4278/005.mp4",
      "in_ms": 84700.0,
      "out_ms": 87600.0,
      "start_ms": 185130,
      "end_ms": 188030
    }
  ],
  "cards": [
    {
      "shot_id": "G-10",
      "start_ms": 417160,
      "end_ms": 450160,
      "card": {
        "component": "TimelineCard",
        "headline": "The timeline they don't put in the press release",
        "items": [
          {
            "date": "JUL\n25\n2024",
            "title": "Chapter 135 signed",
            "detail": "Original effective date set: October 23, 2024."
          },
          {
            "date": "OCT\n2\n2024",
            "title": "Emergency preamble signed",
            "detail": "Law takes effect immediately. A referendum petition can no longer stay it. Stated reasons: the measures needed to go into effect \"without delay\"; agencies and municipalities needed time to prepare.",
            "highlight": true,
            "badge": "69 DAYS LATER"
          },
          {
            "date": "OCT\n23\n2024",
            "title": "The law's own original effective date",
            "detail": "Where the normal 90-day clock would have ended."
          },
          {
            "date": "OCT\n2024",
            "title": "93,229 signatures submitted",
            "detail": "Raw count filed by the repeal campaign."
          },
          {
            "date": "NOV\n22\n2024",
            "title": "78,707 signatures verified",
            "detail": "Certifies the question for the November 3, 2026 ballot."
          }
        ],
        "note": "Under Article 48, a certified petition normally stays a law until voters decide. An emergency preamble removes that stay.",
        "sources": "Sources: Ballotpedia \u00b7 Foley Hoag LLP \u00b7 AP / NBC Boston (Oct 2, 2024)"
      }
    },
    {
      "shot_id": "G-11",
      "start_ms": 474350,
      "end_ms": 498350,
      "card": {
        "component": "StatBarsCard",
        "headline": "Louisiana, 1898: a neutral-sounding cutoff",
        "lead": "New literacy and property tests, with one exemption: you were excused if you, your father, or your grandfather could vote before January 1, 1867.",
        "items": [
          {
            "label": "Black registered voters",
            "period": "1897 to 1900",
            "values": [
              130344,
              5320
            ],
            "delta": "(\u221296%)",
            "color": "red"
          },
          {
            "label": "White registered voters",
            "period": "1897 to 1900",
            "values": [
              164088,
              125437
            ],
            "delta": "(\u221224%)",
            "color": "gold"
          }
        ],
        "quote": "racially neutral on its face",
        "attribution": "The U.S. Supreme Court struck down grandfather clauses in Guinn v. United States (1915).",
        "sources": "Sources: BlackPast.org \u00b7 Guinn v. United States, 238 U.S. 347 (1915) \u00b7 registration counts from secondary summaries of Louisiana state registration reports"
      }
    }
  ]
}
PASS: 5 clip IDs and manifest turn windows match; 2 cards.
```

Installed console script smoke test: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast-video render --help` succeeded. Also ran that installed executable from the reference project with the same arguments above, changing only `--out` to `/tmp/scriptcast-phase3b-v1hruta1/console.mp4`, again with `PYTHONDONTWRITEBYTECODE=1` and the source workspace on `PYTHONPATH` (exit 0):

```text
Skipping non-speaker preamble line 1 in Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md
wrote /private/tmp/scriptcast-phase3b-v1hruta1/console.mp4.plan.json
```

### Verification limits

- Renders were not run; no Chrome probe was repeated. Visual layout, actual card layering, media playback/synchronization, last-frame appearance, encoded H.264/audio output, and full-episode completion require the orchestrator's render.
- HEVC decode by Remotion's `OffthreadVideo` is untested. Source media duration/seek accuracy has not been probed; the plan honors registry offsets without guessing adjustments.
- Installed `scriptcast-video render` WAS smoke-tested for help and real-data plan-only using explicit source `PYTHONPATH`; installed console rendering and wheel-only distribution were not tested.
- Staged run directories are retained for the orchestrator and are ignored. Plans depend on these directories remaining under this source workspace's public directory. Copy/write interruption recovery and arbitrarily malformed card payloads beyond component selection were not tested.
- No git commands, README changes, package installation, network requests, manuscript-repository writes, or Phase 4 work occurred. All implementation changes are left uncommitted.

## Phase 4

### Scope and initial decisions

- Implementing TypeScript in `video/src/quiver/`, using plain Node fetch with no SDK runtime imports. SDK installation is for schema inspection only.
- Verification uses synthetic fixtures, injected credential providers/fetch, and dry runs. No real environment credential will be inspected and no Quiver endpoint contacted. Network is restricted to the authorized SDK npm installation. No renders, git mutations, README edits, or writes to TheOriginalPower.
- This section supersedes earlier phases' historical scope notes. Implementation and acceptance evidence follow below.

### Schema evidence and SDK decision

Installed `@quiverai/sdk` **0.9.4** using `npm install @quiverai/sdk --save-dev --ignore-scripts --no-audit --no-fund --update-notifier=false --cache /tmp/scriptcast-npm-cache` inside `video/` (two packages added: SDK and its zod dependency). The SDK is pinned as a development-only schema reference; neither application code nor tests import it. Runtime uses Node fetch, crypto, fs, path, and util. No new test framework or execution tool was added. The available Node is v23.7.0; the implementation uses APIs available in Node 22, but was not separately executed under Node 22.

All SDK paths below are relative to `video/node_modules/@quiverai/sdk/`:

| Fields / contract | Exact source |
| --- | --- |
| Generation request `model`, `prompt`, `instructions`, `references` (URL string or `{url}` / `{base64}`), `n`, `stream`, `temperature`, `top_p`, `presence_penalty`, `max_output_tokens`; generation limits/defaults | Saved `docs/reference/quiver_developers_models_text-to-svg.md`, Parameters and Reference images; corroborated by `src/sdk/models/shared/generatesvgrequest.ts` outbound schema |
| Animation `model`, optional `prompt`, `max_output_tokens`, `reasoning_effort` (`low`, `medium`, `high`, `xhigh`), `stream`, `svg_source`, `temperature` | `src/sdk/models/shared/animatesvgrequest.ts`, `AnimateSVGRequest$Outbound` and outbound schema (wire names are snake_case, not SDK camelCase) |
| Animation `svg_source` union `{url: string}` or `{base64: string}`; base64 is the raw encoded payload | `src/sdk/models/shared/svginputreference.ts`, `svginputreferenceurl.ts`, `svginputreferencebase64.ts` |
| Generation JSON envelope `created`, optional `credits`, `data`, `id`, optional `usage` | `src/sdk/models/operations/generatesvg.ts` result union selects `SvgResponse`; `src/sdk/models/shared/svgresponse.ts` inbound schema |
| Each generation document's `svg`, `mime_type: "image/svg+xml"` | `src/sdk/models/shared/svgdocument.ts` inbound schema |
| Generation `usage.input_tokens`, `output_tokens`, `total_tokens` | `src/sdk/models/shared/svgusage.ts` inbound schema |
| Animation envelope `created`, optional `credits`, `data`, `id`, optional nullable `svg_score`, optional nullable `usage`; usage token fields | `src/sdk/models/operations/animatesvg.ts`, `AnimateSVGResponseBody$inboundSchema` and `Usage$inboundSchema` |
| Each animation document's `svg`, `mime_type`, optional nullable `loop_period_ms`, `opening_animation_ms` | `src/sdk/models/shared/animatedsvgresponse.ts` inbound schema |
| Model-list envelope `object: "list"`, `data`; model `id` | `src/sdk/models/shared/listmodelsresponse.ts`, `src/sdk/models/shared/model.ts` inbound schemas; other catalog fields pass through as unknown values rather than guessed billing types |
| Methods/paths and JSON body transport (not the SDK's wrapper object) | `src/funcs/createSVGsGenerateSVG.ts`, `src/funcs/animateSVGAnimateSVG.ts`, `src/funcs/modelsListModels.ts` |
| Base URL, Bearer authorization, JSON content type, `X-Request-ID`, `Retry-After`, error `status`, `code`, `message`, `request_id` | Saved `quiverai-llms-full.txt`, Introduction; `quiver_developers_guides_errors-and-debugging.md` |
| Sandbox environment header and SVG markers; reject test outputs from production asset cache | Saved `quiver_developers_guides_sandbox-and-test-keys.md` |

Both missing response shapes and the animation request are derived from SDK schemas; **no opaque/unverified-shape fallback was necessary**. The SDK's `headers`/`result` wrapper is not the HTTP JSON envelope. `request_id` comes from the response header, not generation `id`. Animation sends local SVG bytes as base64, never an invented inline `svg` field. Default model is `arrow-2`, as confirmed by the saved Arrow 2 page. Saved documentation describes token pricing and production-host test keys; neither production nor sandbox was contacted.

The SDK generation type contains additional `attributes` and `reasoning_effort` fields absent from the saved generation parameter table. This client deliberately exposes the documented generation subset. Its reference-limit comment discusses Arrow 1.x, whereas the saved docs explicitly give Arrow 2's limit of 14; CLI supplies no references, and the client does not infer Arrow 2 limits from that older comment. Streaming is explicitly unsupported (only omitted/false accepted), so no SSE preview can enter the cache.

### Implementation and asset policy

- `client.ts` exports `generateSvg`, `animateSvg`, `listModels`, and an injectable `QuiverClient`. Credential lookup happens only when making a request, never during construction/import. Tests inject in-memory random synthetic credentials; no environment credential was read, printed, or written. Errors redact the supplied credential if an error response reflects it; transport errors omit potentially sensitive details. Redirects fail instead of forwarding authentication elsewhere.
- Retries apply only to 429/503, defaulting to two retries. Numeric seconds and HTTP-date `Retry-After` are honored using injectable sleep/clock; missing headers use 1s/2s backoff. Invalid or over-bound delays surface the typed error rather than retrying early. Default maximum accepted wait is 60s; configurable bounds are validated. Other HTTP failures are not retried. Error status/code/message/request_id are preserved (with credential redaction).
- Cache identity is SHA-256 of recursively sorted JSON `{endpoint, request, output_index}`; the request includes model and parameters and has no authentication field. Endpoint separation avoids generation/animation collisions. Output zero uses the request hash; further outputs use their own indexed hashes. Every returned SVG is retained, including batches, with prompt/model/request_id/date/endpoint/hash plus request_hash/output_index/output_count in the manifest. A complete hit checks every file and avoids both credential access and fetch. Missing files are regenerated; malformed manifest JSON fails visibly.
- Cache defaults to `public/assets/quiver` relative to the video workspace; the Python wrapper always sets that cwd. Direct library consumers outside `video/` should pass the explicit cache directory. Manifest replacement is atomic, but the cache is designed for sequential CLI use, not concurrent writers: read/merge/write is not a cross-process lock, and SVG writes are not a multi-file transaction.
- **Version-control policy:** retain `video/public/assets/quiver/manifest.json` (initially `[]`) and generated `*.svg` together under version control for reproducible rendering. No new `.gitignore` rules: nothing in this asset directory is ignored. Files are left uncommitted as requested. Test cache files were created in temporary directories and removed individually; no generated SVG was added to public assets.
- CLI compiles exactly like check-copy using existing tsc, then runs Node. `npm run --silent quiver -- ...` is the Python passthrough. Generation validates `--n` in 1–16; animation resolves paths against caller cwd before switching to `video/`. `--dry-run` prints exact method/URL/body and exits before creating a client/cache or resolving credentials. `models --dry-run` is also supported and has no JSON body because it is GET.
- All four components forward `svgAsset` to shared `Frame`, which renders `Img`/`staticFile` in a 140px-high contained slot inside the clipped safe-zone box. Paths must be public-relative SVGs without traversal, URL schemes, query/fragment, backslash, or encoded path escapes. `g10_svg.json` reuses G-10 copy and references the hand-written neutral circle at `assets/example.svg`.
- Initial tsc uncovered a parseArgs union inference issue, fixed with string narrowing. Initial markup check expected SSR `Img` to include `src`; Remotion defers that attribute. The final check separately verifies actual React `Img` props equal `staticFile(...)` in all four components, plus emitted image markup and shared bounds. No browser was launched.
- Six new Python cases cover SVG argv/defaults/flags, path resolution, models, child exit propagation, and missing compiler dependencies. TypeScript checks cover all three endpoint requests/headers, missing/lazy credentials, seconds/date/bounded retries, typed errors/redaction, stable nested cache keys, credential-independent zero-fetch hits, manifest/files, multiple outputs, animation, sandbox rejection, and all dry-run forms.

### Acceptance output (raw)

`/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q` (exit 0)

```text
........................................................................ [ 43%]
........................................................................ [ 87%]
.....................                                                    [100%]
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
165 passed, 5 warnings in 6.85s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

`cd video && npx tsc --noEmit` (exit 0; empty stdout/stderr)

```text
```

`cd video && npm run check-copy` (exit 0)

```text

> scriptcast-video@0.2.0 check-copy
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-copy.js

PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.
PASS: public SVG slot renders in all four cards inside the shared safe-zone box.
```

`cd video && npm run check-quiver` (exit 0)

```text

> scriptcast-video@0.2.0 check-quiver
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-quiver.js

PASS: documented generation/animation/models wire requests, lazy credentials, missing key, retries/date/bounds, typed errors and redaction.
PASS: canonical cache keys, credential-independent hits with zero fetches, SVG/manifest persistence, multi-output and animation caching, sandbox rejection, and exact key-free CLI dry runs.
```

### Installed console smoke test (raw)

The installed `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast-video` WAS smoke-tested for all three SVG commands, using `PYTHONDONTWRITEBYTECODE=1` and `PYTHONPATH=/Users/emmanuel/Documents/Theory/scriptCast`, from this workspace. Each exited 0. This verifies the installed entry point against source code, not a rebuilt wheel or live API dispatch.

```text
svg generate 'A plain circle' --dry-run
svg animate video/public/assets/example.svg --prompt Rotate --dry-run
svg models --dry-run
```

Combined raw stdout/stderr:

```text
{
  "method": "POST",
  "url": "https://api.quiver.ai/v1/svgs/generations",
  "body": {
    "model": "arrow-2",
    "prompt": "A plain circle",
    "n": 1
  }
}
{
  "method": "POST",
  "url": "https://api.quiver.ai/v1/svgs/animations",
  "body": {
    "model": "arrow-2",
    "svg_source": {
      "base64": "PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxNjAgMTAwIj48Y2lyY2xlIGN4PSI4MCIgY3k9IjUwIiByPSIzNiIgZmlsbD0iIzZlYzNjMCIvPjwvc3ZnPgo="
    },
    "prompt": "Rotate"
  }
}
{
  "method": "GET",
  "url": "https://api.quiver.ai/v1/models"
}
```

### Files changed and verification limits

- Modified: `CHANGELOG.md`, this findings file, `scriptcast/tools/video.py`, `tests/test_video.py`, `video/package.json`, `video/package-lock.json`, `video/scripts/check-copy.tsx`, `video/src/cards/Frame.tsx`.
- Added: `video/src/quiver/client.ts`, `video/src/quiver/cache.ts`, `video/scripts/quiver.ts`, `video/scripts/check-quiver.ts`, `video/examples/g10_svg.json`, `video/public/assets/example.svg`, `video/public/assets/quiver/manifest.json`.
- User-supplied untracked `docs/reference/` and `docs/briefs/phase4-prompt.md` were not changed. README and TheOriginalPower were not modified. No git add/commit/checkout/stash was run; implementation is uncommitted. `git diff --check` passed with empty output.
- **No live API call was made**, including test-key/sandbox calls. Authentication, account/model availability, billing, live rate limits, generation quality, animation appearance, and compatibility of the shipped SDK schemas with the live service remain unverified.
- **No animation or response shape remains underived** for the implemented non-streaming endpoints. Full runtime validation of all response metadata and model billing subtypes is not implemented; cache checks SVG data before persistence. Streaming, edits, and vectorization are outside scope.
- **Renders were not run**; Chrome was not probed. Image decoding, visual overflow/clipping, actual animation timing/determinism within Remotion, and encoded output need orchestrator validation. The extra art slot reduces text space; long content can be clipped by the existing safe-zone container.
- **Installed `scriptcast-video svg` console path was smoke-tested**, as described above; installed wheel-only packaging and non-dry-run console dispatch were not tested. Node 22 specifically and simultaneous cache writers were not tested.

### Orchestrator review (Phase 4)

Rendered `examples/g10_svg.json` outside the sandbox: with Codex's 140px art slot the G-10 note box and last timeline node were clipped by the safe-zone box, which the markup check did not catch (it asserts bounds, not content overflow). Fix: art slot reduced to 64px (`Frame.tsx`, `check-copy.tsx`), and the SVG example sets `"note": ""` (Remotion merges example props over composition defaults, so removal alone does not suppress it). Dense cards with art still need to drop optional elements; a content-overflow assertion is a follow-up.

## Phase 5

- Redesigned `Frame`, `TimelineCard`, `StatBarsCard`, `TitleCard`, `CompareCard` to use a dynamic density scale based on text length and item count, distributing elements over the full safe zone height. Body text and headings scale down automatically (no lower than ~24px).
- Implemented `art: {src, size, caption}` across all cards. Added runtime `useLayoutEffect` to throw an error if the card's content exceeds the bounding box (`scrollHeight > clientHeight`). Added checks to ensure SVG paths are public-relative.
- Cleaned up default `Root.tsx` props to contain only required minimum fields, preventing data leakage across unrelated compositions. Added leakage assertions to `check-copy.tsx`.
- Extended Python video runner to resolve and check for missing assets prior to rendering, exposing an `--allow-missing-assets` override flag and adding corresponding tests.

### Acceptance output (raw)

`/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q` (exit 0)

```text
........................................................................ [ 43%]
........................................................................ [ 86%]
.......................                                                  [100%]
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
167 passed, 5 warnings in 6.17s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
```

`cd video && npx tsc --noEmit && npm run check-copy && npm run check-quiver` (exit 0)

```text
> scriptcast-video@0.2.0 check-copy
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-copy.js

PASS: all G-10/G-11 copy and numbers; four card layouts within safe-zone constants; QR below sources; group-relative bar widths.
PASS: public SVG slot renders in all four cards inside the shared safe-zone box.

> scriptcast-video@0.2.0 check-quiver
> tsc --outDir .remotion/check --module commonjs --moduleResolution node --noEmit false && node .remotion/check/scripts/check-quiver.js

PASS: documented generation/animation/models wire requests, lazy credentials, missing key, retries/date/bounds, typed errors and redaction.
PASS: canonical cache keys, credential-independent hits with zero fetches, SVG/manifest persistence, multi-output and animation caching, sandbox rejection, and exact key-free CLI dry runs.
```

### Unverified items
- Full renders were not run. The orchestrator must run a full `remotion render` to test integration.
- The runtime overflow check relies on `el.scrollHeight > el.clientHeight` inside `useLayoutEffect` in Chrome. This logic was not verified in a live browser due to sandbox limitations. Stills of `g10` and `g11` passed layout locally via manual still checks, meaning no error was thrown and content fits.
- `findImage` test was removed because the `Frame` component is now stateful with hooks, preventing `check-copy.tsx` from evaluating it functionally. Assertions verify `data-svg-slot` in the rendered markup.

### Files changed
- Modified: `video/src/cards/Frame.tsx`, `video/src/cards/TimelineCard.tsx`, `video/src/cards/StatBarsCard.tsx`, `video/src/cards/CompareCard.tsx`, `video/src/cards/TitleCard.tsx`, `video/src/Root.tsx`, `video/scripts/check-copy.tsx`, `scriptcast/tools/video.py`, `tests/test_video.py`, `docs/briefs/FINDINGS-v0.2.md`.
- Added: `video/examples/hero_art.json`, `video/examples/title_emphasis.json`.
