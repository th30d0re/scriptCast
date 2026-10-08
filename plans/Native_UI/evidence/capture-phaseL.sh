#!/bin/sh
# Run by the orchestrator from the worktree root. No synthesis; every capture is muted.
# Usage: sh plans/Native_UI/evidence/capture-phaseL.sh [PROJECT_ROOT] [EPISODE_DIR]
set -eu
root=$(pwd)
project=${1:-/tmp/scriptcast-k/proj}
out="$root/plans/Native_UI/evidence"
bin="$root/ui/target/debug/scriptcast-ui"
export SCRIPTCAST_PROJECT="$project"
"$bin" --project-picker "$project" --fresh --mute --screenshot "$out/phaseL-L1.png"
if [ "${2:-}" ]; then
    set -- "$2"
else
    set --
fi
"$bin" "$@" --project-data voices --select aisha --fresh --mute --screenshot "$out/phaseL-L2.png"
"$bin" "$@" --project-data clips --fresh --mute --screenshot "$out/phaseL-L3.png"
"$bin" "$@" --project-data accepted-hearings --fresh --mute --screenshot "$out/phaseL-L4-hearings.png"
"$bin" "$@" --project-data pronunciations --fresh --mute --screenshot "$out/phaseL-L4-pronunciations.png"
"$bin" "$@" --project-data speaker-rates --fresh --mute --screenshot "$out/phaseL-L4-rates.png"
"$bin" "$@" --settings-project --fresh --mute --screenshot "$out/phaseL-L4-settings.png"
