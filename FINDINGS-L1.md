# L1 findings — stage-layout

Worktree `/Users/emmanuel/Documents/Theory/scriptCast-v04-L1`, branch `v04-L1`. No git commands that change state. Measurements and the same acceptance tails are in `stage-layout/FINDINGS.md`.

Verdict: **adopt with changes: bundle IBM Plex as TTF (v6.4.2) because `load_font_bytes` accepts SFNT and the `@fontsource` files are WOFF; after every macOS link, run `stage-layout/pad_linkedit.py` on the cdylib (Xcode 27 `ld` writes `LC_SYMTAB.stroff` 4 bytes off an 8-byte boundary, and dyld refuses to load it); map CSS weight 650 to the SemiBold face; cover U+2261 (IBM Plex has no glyph, so FigCraft measures `.notdef` at 16.79px while Chrome paints a fallback).**

## What was built

`stage-layout` is a workspace member. It depends on `figcraft-doc`, `figcraft-layout`, and `figcraft-text` by git tag `v0.14.0` (lock resolves to `5fc2840d9089e0b21056b57c4dff9f795e38a4e3`). The lock also has the transitive crates `figcraft-geom` and `figcraft-color`. There is no `figcraft-engine` dependency. `src/measure.rs` is the engine's `Measure` adapter, copied.

`layout(doc_json, fonts) -> String` returns `{ element_id: {x, y, w, h, lines?: [{text, x, y, w}]} }` keyed by FigCraft node name. Page origin is top-left, y down. Line `x`/`y` is the top of the glyph run.

Font API: `figcraft_text::load_font_bytes` exists. It accepts TTF/OTF. The `@fontsource` IBM Plex packages under `scriptCast-stage` are WOFF/WOFF2, which skrifa does not parse. The crate bundles IBM Plex Sans Regular, Medium, and SemiBold, and IBM Plex Mono Regular and Medium (TTF v6.4.2, OFL, `stage-layout/fonts/`). `register` calls `opt_out_of_system_fonts()`, loads that bundle once, and adds caller bytes on every call. An empty font list still shapes.

Front ends:

- WASM: `wasm-pack build stage-layout --target web` (wasm-pack 0.15.1). This build uses the release profile and `wasm-opt`. `stage_layout_bg.wasm` is 3,765,782 bytes (1,627,555 gzip -9). `layout` is exported.
- Python: PyO3 0.24, feature `python` (includes `pyo3/extension-module`), installed into `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice` with `maturin 1.16.0 develop --release`. On this OS the fresh cdylib does not `dlopen` until `stage-layout/pad_linkedit.py` inserts 4 bytes in front of the string pool and 4 more in front of the code signature, then `codesign`s. `ld -ld_classic` is ignored by ld-27037.1.

## The scene

Turn 21 of `APAGroupDefinitions` (definition card, field panel, derived-illustration tray) is the FigCraft document `stage-layout/fixtures/apa_t21.json`. Laid-out boxes match the hand-typed scene (card 110,190 650×400; field 830,130 960×520; tray 830,690 960×220; circle 985,165 430×430; panels at y 734). The body wraps to the same three lines Chrome draws at width 574, Sans 400, 28px, line-height 1.34, for both the shipped TTF and the fontsource latin WOFF:

```text
the tendency to favor one’s own group and its
members when dividing resources, judging
conduct, and explaining behavior
```

`e239.jpg` is episode second 239, which is turn 19 (ingroup, no tray). The turn-21 approved frame is `e264.jpg`. The comparison still is beside `e239.jpg`:

`/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov/video/stage_runs/ab_v2/apa_t21_stage_layout.jpg`

Rendered with the existing Remotion still command (`StageStill`, frame 0, `STAGE_EPISODE_DIR=stage-layout/episode`). The episode paints plain boxes and one `white-space: pre` span per returned line. Next to `e264.jpg`, the card, the three-line definition, the source and `(paraphrased)`, the field circle, and the tray agree. Person marks, resource dots, the tendency arrow, and the ornate frame are Stage drawings and are absent from this still.

Warm layout of this document is 0.502 ms median of 100 calls (timeit: 463 µs per loop, best of 5). The first call in a process is 13.354 ms while the faces load.

## Acceptance tails

`cargo test -p stage-layout`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (target/debug/deps/stage_layout-03fe9a990ad14f28)

running 4 tests
test tests::fixture_file_matches_the_builder ... ok
test tests::bundled_fonts_register_plex_families ... ok
test tests::garbage_font_bytes_do_not_replace_the_bundle ... ok
test tests::apa_names_are_unique_and_regions_match_the_scene ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running unittests src/bin/dump_apa.rs (target/debug/deps/dump_apa-ccd957691fa8ec37)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests stage_layout

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`wasm-pack build stage-layout --target web`

```text
[INFO]: 🎯  Checking for the Wasm target...
[INFO]: 🌀  Compiling to Wasm...
    Finished `release` profile [optimized] target(s) in 0.11s
[INFO]: ⬇️  Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: Optional field missing from Cargo.toml: 'repository'. This is not necessary, but recommended
[INFO]: License key is set in Cargo.toml but no LICENSE file(s) were found; Please add the LICENSE file(s) to your project directory
[INFO]: ✨   Done in 5.55s
[INFO]: 📦   Your wasm pkg is ready to publish at stage-layout/pkg.
```

```text
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -c "import stage_layout, json; print(len(json.loads(stage_layout.layout(open('stage-layout/fixtures/apa_t21.json').read(), [])) ))"
40
```

The rendered comparison still exists at the path above (jpeg, 82 KB).

Stopped. L2 and the rest of the loop were not started.
