# UI tests

`snapshots.rs` renders the episode browser with `egui_kittest` at
1440×900, scale 1.0, using the real fonts and theme. It does not launch
a window. Audio stays closed. The harness formats dates in UTC and turns
animations off, including the text caret. Every path it opens is under
`tests/fixtures` (cwd is the `ui` package). `phase_l/show.json` uses root
`tests/fixtures`, with voice and clip media at `audio/tone.m4a` and
`audio/tone.mp4`. Project tabs render that JSON. The harness does not
spawn `scriptcast-project`, a login shell, or PATH command discovery,
and it ignores `SCRIPTCAST_PROJECT`.

Accepted images live in `tests/snapshots/`. Regenerate them with:

```bash
cd ui
UPDATE_SNAPSHOTS=1 cargo test --test snapshots
```

A mismatch writes `tests/snapshots/{name}.diff.png` next to
`{name}.new.png`. The command above replaces `{name}.png` and keeps the
previous file as `{name}.old.png`.
