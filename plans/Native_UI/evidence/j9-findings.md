# J9 findings: Rust text chunker (2026-10-28)

## What changed

New crate `chunk/` (package and binary `scriptcast-chunk`, edition 2024,
GPL-3.0-or-later, `publish = false`). Only dependency: `hound`. No soundcraft
dependency. Python is not wired. No TTS engine is called. The caller decides
when audio is unstable and calls `halve`.

## CLI

```
scriptcast-chunk split --text TEXT --max-chars N --json
scriptcast-chunk halve --text TEXT --json
scriptcast-chunk crossfade --a A.wav --b B.wav --out OUT.wav [--fade-ms 50]
```

- `split` and `halve` print one line and nothing else:
  `{"pieces":[{"text":"...","extended_for_tag":false}]}`.
  `--json` is accepted; JSON is the only output format.
- `crossfade` prints nothing on success.
- Any error: message on stderr, empty stdout, exit 1.

Example:

```
$ scriptcast-chunk split --text "Mr. Smith went home. He sat." --max-chars 20 --json
{"pieces":[{"text":"Mr. Smith went home. ","extended_for_tag":false},{"text":"He sat.","extended_for_tag":false}]}
```

### split rules

Lengths count Unicode chars. A cut is made after the mark and the whitespace
that follows it, so the pieces always join back to the input. The mark must be
within N; the whitespace after it may go past N.

1. Last sentence end at or before the limit: `. ! ?` and `。！？`.
   An ASCII mark counts only before whitespace or at the end of text. This
   keeps `3.14`, `file.txt` and the inside of `e.g.` whole. CJK marks need no
   following space.
2. Last clause boundary: `;` `,` `:` (before whitespace) or `—`.
3. Last whitespace.
4. Hard cut at N.

A cut never lands strictly inside a `[...]` tag. Tags do not nest: the first
`]` closes the tag, and a stray `]` is plain text. An unclosed `[` is an error.
`extended_for_tag` is true only on a piece that grew past N to keep a tag whole.

### halve rules

One split at the whitespace nearest the midpoint, outside tags, with text on
both sides. If the midpoint is inside a tag, use the nearest whitespace before
the tag, else the first after it. With no usable whitespace, cut at the midpoint
(or at the tag edge); a text with no cut point comes back as one piece.

### crossfade rules

Mono PCM, 16-bit or 32-bit, same sample rate. Linear ramp; sample `i` of the
overlap uses weight `(i+1)/(fade+1)` for B. Output frames =
frames(a) + frames(b) - fade_frames, where fade_frames =
round(fade_ms * rate / 1000). Different rates, channel counts, or a file
shorter than the fade: non-zero exit, no output file (the file is written to
`OUT.wav.part` and renamed only when complete). No resampling.

## Abbreviation list

Case-insensitive, period after the whole word is not a sentence end:
`mr mrs ms dr prof sr jr st vs etc us uk`.

Written forms (case-insensitive, final period is not a sentence end):
`e.g.  i.e.  a.m.  p.m.  u.s.  u.k.`

## Decisions where the contract was open

- **Tag at the limit.** Rule d says to extend when the limit falls inside a
  tag. I try a sentence, clause or whitespace cut *before* the tag first, and
  extend only when no such cut exists (for example the tag starts the
  remainder). `"hello [pause:500ms] world"` at 8 gives `hello ` /
  `[pause:500ms] ` / `world`, with only the tag piece flagged. This keeps
  pieces within N whenever that is possible. If you want literal rule d (extend
  before trying clause and whitespace cuts), change `next_cut` in
  `chunk/src/split.rs`.
- **Hard cut before a tag.** If a hard cut at N would land inside a tag that
  starts after real text, the cut moves back to the tag start (no extension).
- **Mixed bit depth.** A 16-bit and a 32-bit input are joined as 32-bit; the
  16-bit samples are shifted left 16. Not resampling.
- **Digit grouping.** `1,000` and `10:30` are not clause boundaries.

## Files

- `chunk/Cargo.toml`, `chunk/Cargo.lock`, `chunk/.gitignore`
- `chunk/src/main.rs` (CLI), `chunk/src/lib.rs` (JSON output)
- `chunk/src/split.rs` (split and halve), `chunk/src/crossfade.rs`
- `chunk/tests/cli.rs` (binary tests, including wav files)
- `plans/Native_UI/evidence/j9-findings.md` (this file)

Nothing under `scriptcast/`, `ui/`, `media/`, `video/`, `pyproject.toml`,
`README.md` or `CHANGELOG.md` was changed.

## Test results

From `chunk/`: `cargo test` and `cargo clippy --all-targets -- -D warnings`.
Final run output is recorded at the end of this file.

Acceptance cases covered (unit tests in `split.rs`, `crossfade.rs`; binary
tests in `tests/cli.rs`):

- `Mr. Smith went home. He sat.` at 20: first piece has `home.`, not `He`; pieces join to the input.
- `3.14 is pi. Done.`: no split at `3.` for every limit from 4 up.
- `[pause:500ms] hello there friend` at 8: tag whole, `extended_for_tag` true on the first piece only.
- `你好。世界` at 3: `你好。` then `世界`.
- `alpha; beta gamma` at 8: first piece `alpha; `.
- `alpha beta gamma` at 8: whitespace splits.
- `halve` of `left [pause:10ms] right`: `left ` / `[pause:10ms] right`.
- crossfade 100 + 80 frames, 20-frame fade: 160 frames.
- crossfade refuses rate mismatch, channel mismatch, short input; no file written.
- Unclosed `[`: error, exit 1, empty stdout.

## Final run

```
$ cargo test            # exit 0
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo clippy --all-targets -- -D warnings   # exit 0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
```
