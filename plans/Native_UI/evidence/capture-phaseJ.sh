#!/bin/sh
# Run from the worktree root in a session that can launch macOS GUI applications.
# These captures only read fixtures; they never invoke the Python commands.
set -eu
root=$(pwd)
fixture="$root/ui/tests/fixtures"
scratch=/tmp/scriptcast-j/capture
out="$root/plans/Native_UI/evidence"
bin="$root/ui/target/debug/scriptcast-ui"
mkdir -p "$scratch"
cp -R "$fixture/episode_small" "$scratch/"
cp "$fixture/script.md" "$fixture/scriptcast.toml" "$scratch/"
cp "$fixture/script.md" "$scratch/episode_small.md"
printf 'speakers: {}\n' > /tmp/scriptcast-j/voices.yaml
# Only turn 1 has rendered in the handwritten progress fixture.
rm -f "$scratch/episode_small/Samples/Processed/turn_2_0.wav"
rm -f "$scratch/episode_small/Samples/Processed/turn_2_1.wav"
rm -f "$scratch/episode_small/Samples/Processed/turn_3_0.wav"
"$bin" "$scratch/episode_small" --fresh --mute --new-episode "$scratch/episode_small.md" \
    --dry-run-json "$fixture/phase_j/dry-run.json" --screenshot "$out/phaseJ-J1.png"
"$bin" "$scratch/episode_small" --fresh --mute --select 1 \
    --progress-fixture "$fixture/phase_j/progress.jsonl" --screenshot "$out/phaseJ-J2.png"
"$bin" "$scratch/episode_small" --fresh --mute --select 1 --quit-alert \
    --progress-fixture "$fixture/phase_j/progress.jsonl" --screenshot "$out/phaseJ-J3.png"
"$bin" "$scratch/episode_small" --fresh --mute \
    --pipeline-fixture "$fixture/phase_j/pipeline-running.json" --screenshot "$out/phaseJ-J4-running.png"
"$bin" "$scratch/episode_small" --fresh --mute \
    --pipeline-fixture "$fixture/phase_j/pipeline.json" --screenshot "$out/phaseJ-J4-stopped.png"
