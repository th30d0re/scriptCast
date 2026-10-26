#!/bin/sh
# Run by the orchestrator from the worktree root. No synthesis; every capture is muted.
# Usage: sh plans/Native_UI/evidence/capture-phaseM.sh [PROJECT_ROOT]
set -eu
root=$(pwd)
project=${1:-/tmp/scriptcast-k/proj}
fix="$root/ui/tests/fixtures/phase_m"
out="$root/plans/Native_UI/evidence"
bin="$root/ui/target/debug/scriptcast-ui"
export SCRIPTCAST_PROJECT="$project"
"$bin" --project-data clips --clip-maker --words-fixture "$fix/words.json" \
    --fresh --mute --screenshot "$out/phaseM-M1.png"
"$bin" --project-data voices --reference-maker --candidates-fixture "$fix/candidates.json" \
    --fresh --mute --screenshot "$out/phaseM-M2.png"
"$bin" --project-data voices --audition-fixture "$fix/audition.json" \
    --fresh --mute --screenshot "$out/phaseM-M3.png"
"$bin" --project-data pronunciations --calibrate-fixture "$fix/calibrate.json" \
    --outliers-fixture "$fix/outliers.json" \
    --fresh --mute --screenshot "$out/phaseM-M4.png"
