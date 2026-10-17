# UI tests

`snapshots.rs` renders the episode browser with `egui_kittest` at
1440×900, scale 1.0, using the real fonts and theme. It does not launch
a window. Audio stays closed. The harness formats dates in UTC and turns
animations off, including the text caret.

Accepted images live in `tests/snapshots/`. Regenerate them with:

```bash
cd ui
UPDATE_SNAPSHOTS=1 cargo test --test snapshots
```

A mismatch writes `tests/snapshots/{name}.diff.png` next to
`{name}.new.png`. The command above replaces `{name}.png` and keeps the
previous file as `{name}.old.png`.
