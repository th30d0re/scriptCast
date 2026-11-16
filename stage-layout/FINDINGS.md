# stage-layout spike (A8)

Verdict: **adopt with changes: bundle IBM Plex as TTF (v6.4.2) because `load_font_bytes` accepts SFNT and the `@fontsource` files are WOFF; after every macOS link, run `stage-layout/pad_linkedit.py` on the cdylib (Xcode 27 `ld` writes `LC_SYMTAB.stroff` 4 bytes off an 8-byte boundary, and dyld refuses to load it); map CSS weight 650 to the SemiBold face; cover U+2261 (IBM Plex has no glyph, so FigCraft measures `.notdef` at 16.79px while Chrome paints a fallback).**

FigCraft v0.14.0 (`5fc2840d`) can place this scene and break its text. Stage keeps drawing. The crate depends on `figcraft-doc`, `figcraft-layout`, and `figcraft-text` only. `figcraft-engine` is not a dependency; `src/measure.rs` is the engine's `Measure` adapter copied in.

## WASM size

`wasm-pack 0.15.1 build stage-layout --target web` builds the **release** profile and runs `wasm-opt`.

| artifact | bytes |
|---|---|
| `stage-layout/pkg/stage_layout_bg.wasm` | 3,765,782 |
| same file, gzip -9 | 1,627,555 |

`layout(doc_json, fonts)` is exported from `stage-layout/pkg/stage_layout.js`. The module is large enough to load once behind `delayRender`. The five bundled TTFs are about 0.9 MB of that.

## Layout time

Release PyO3 module in `.venv-voice`, document `stage-layout/fixtures/apa_t21.json`, font list `[]` (bundled faces only).

| measurement | time |
|---|---|
| median of 100 warm calls (`time.perf_counter`) | 0.502 ms |
| `python -m timeit -n 100 -r 5` | 463 µs per loop, best of 5 |
| first call in a fresh process (font bytes loaded once) | 13.354 ms |

Warm min 0.444 ms, p95 0.689 ms, max 0.712 ms. A warm call is a small fraction of a 33 ms frame, so calling `layout` per frame is viable after the first call in the process.

## Line breaks vs Chrome

Same paragraph, width 574, IBM Plex Sans 400 at 28px, line-height 1.34. Headless Chrome (`Range` client rects after `document.fonts.ready`) on the shipped TTF and on `@fontsource` `ibm-plex-sans-latin-400-normal.woff`:

```json
{"ttf":["the tendency to favor one’s own group and its","members when dividing resources, judging","conduct, and explaining behavior"],"woff":["the tendency to favor one’s own group and its","members when dividing resources, judging","conduct, and explaining behavior"],"label":["EXPLAINING BEHAVIOR"]}
```

FigCraft returns those same three body lines (`fixtures/apa_t21.layout.json`, `definition-body`). `EXPLAINING BEHAVIOR` is one line at 288px, IBM Plex Mono 500, line-height 1.05, in both Chrome (fontsource mono WOFF) and FigCraft.

FigCraft rounds line height to whole pixels (`28 * 1.34 = 37.52` becomes 38). Body line tops are 288, 326, 364. Wrapping still matched Chrome for this string.

## The scene

`APAGroupDefinitions` turn 21 (definition card, field, derived-illustration tray) is `stage-layout/fixtures/apa_t21.json`. Auto-layout lands on the hand-typed boxes. The test allows 1px; the fixture values are exact:

| node | x, y | w × h |
|---|---|---|
| definition-card | 110, 190 | 650 × 400 |
| definition-body | 148, 288 | 574 × 114 |
| definition-source | 148, 512 | one line |
| definition-paraphrased | 148, 550 | one line |
| apa-field | 830, 130 | 960 × 520 |
| group-boundary | 985, 165 | 430 × 430 |
| allocation-tray | 830, 690 | 960 × 220 |
| allocation-panel-1/2/3 | 842 / 1156 / 1470, 734 | 304 × 164 |
| resource counts 8 and 4 | 864 and 1092, 866 | |
| behavior lines | 1492, 814 and 1492, 852 | `→ favorable`, `→ situational` |

`e239.jpg` is the turn-19 frame (ingroup, no tray). The turn-21 approved frame is `e264.jpg`. The Stage still is saved beside `e239.jpg`:

`/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov/video/stage_runs/ab_v2/apa_t21_stage_layout.jpg`

It draws the returned boxes and one `white-space: pre` span per returned line. Next to `e264.jpg`, the card, the three-line definition, the source line, the field circle, and the tray (heading, three panels, 8 and 4, two conduct frames, the two behavior lines) sit in the same places. Person marks, resource dots, the tendency arrow, and the ornate frame stay in Stage, which is what A8 keeps there.

## FigCraft API

`figcraft_text::load_font_bytes` exists (`FontDb::add_bytes`). It parses SFNT (TTF/OTF). `@fontsource` ships WOFF/WOFF2, which skrifa does not parse. The crate ships IBM Plex Sans Regular/Medium/SemiBold and IBM Plex Mono Regular/Medium, TTF v6.4.2, plus `fonts/OFL.txt`. `register` calls `opt_out_of_system_fonts()` before any database use, loads the bundle once, and passes extra bytes on every call. Chrome's breaks for this paragraph are the same on the TTF and the fontsource latin WOFF, so the workaround does not change wrapping here.

Other gaps:

- The font database is process-global. Faces cannot be scoped to one `layout` call.
- CSS weight 650 is a named style in FigCraft. Stage loads Sans 400/500/600, so 650 resolves to SemiBold. Style names in the name table are `Regular`, `Medium`, `SemiBold`.
- Default Stage `Label` font is IBM Plex Mono. Only the definition term and body use Sans.
- Node names are the JSON keys. A repeated name keeps the later node.
- Line `x`/`y` is the page-space top of the glyph run, in px, rounded to 2 decimals. `char_start`/`char_end` index Unicode scalars.
- U+2019 and U+2192 are in the bundled faces. U+2261 IDENTICAL TO is not. FigCraft's `≡` line is 16.79px wide (`.notdef`). The still still shows a triple-bar because Chrome substitutes a glyph.
- Duplicate or unnamed text nodes would collapse or disappear from the JSON. Every text node in the fixture has its own name.

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

The comparison still exists (82,496 bytes, jpeg):

`/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov/video/stage_runs/ab_v2/apa_t21_stage_layout.jpg`

Python install used for that import: `maturin 1.16.0 develop --release` with `VIRTUAL_ENV` and `PYO3_PYTHON` pointed at `.venv-voice` (CPython 3.11), then `pad_linkedit.py` on `site-packages/stage_layout/stage_layout.cpython-311-darwin.so`. `maturin develop` with no `VIRTUAL_ENV` picks `~/.venv` (CPython 3.9) and fails looking for pip. `-ld_classic` is ignored by `ld-27037.1`.
