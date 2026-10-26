#!/bin/sh
# Run from the worktree root in a session that can launch macOS GUI applications.
# These captures only read the episode; they never invoke synthesis. Every launch is muted.
# $1 is EPISODE_DIR. When omitted, a copy of ui/tests/fixtures/episode_small is used.
set -eu
root=$(pwd)
fixture="$root/ui/tests/fixtures"
scratch=/tmp/scriptcast-k/capture
out="$root/plans/Native_UI/evidence"
bin="$root/ui/target/debug/scriptcast-ui"
if [ "${1:-}" ]; then
    episode=$1
else
    mkdir -p "$scratch"
    rm -rf "$scratch/episode_small"
    cp -R "$fixture/episode_small" "$scratch/"
    cp "$fixture/script.md" "$fixture/scriptcast.toml" "$scratch/"
    cp "$fixture/script.md" "$scratch/episode_small.md"
    episode=$scratch/episode_small
fi
"$bin" "$episode" --fresh --mute --context-menu 1 \
    --screenshot "$out/phaseK-K1.png"
"$bin" "$episode" --fresh --mute --new-turn-after 1 \
    --screenshot "$out/phaseK-K2.png"
"$bin" "$episode" --fresh --mute --new-turn-after 1 --new-turn-speaker archive \
    --screenshot "$out/phaseK-K2-archive.png"
"$bin" "$episode" --fresh --mute --tag-toolbar 1 --edit-mode panel \
    --screenshot "$out/phaseK-K3.png"
"$bin" "$episode" --fresh --mute --tag-toolbar 1 --edit-mode row \
    --screenshot "$out/phaseK-K3-row.png"
"$bin" "$episode" --fresh --mute --tag-toolbar 1 --clip-picker \
    --screenshot "$out/phaseK-K3-clips.png"
"$bin" "$episode" --fresh --mute --tag-toolbar 1 --raw-script \
    --screenshot "$out/phaseK-K3-raw.png"
