"""Write TURN_INDEX.csv for a rendered episode.

An editor's lookup table: where each turn lands in the stitched audio, what the
script header says, who speaks it, which clip file carries it, and enough words
to recognize it. Regenerate after any render or retime.

    python3 tools/turn_index.py outputs/ATO_EP01_local \
        --transcript Architecting_the_operation/podcasts/ATO_EP01_authors_preface.md
"""
from __future__ import annotations

import argparse
import csv
import json
import logging
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from voice_pipeline.parser import parse_transcript


def _mmss(ms: int) -> str:
    total = ms // 1000
    return f"{total // 60:02d}:{total % 60:02d}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--transcript", type=Path, required=True)
    ap.add_argument("--words", type=int, default=12)
    args = ap.parse_args()

    logging.disable(logging.WARNING)
    manifest = json.loads((args.episode_dir / "episode_manifest.json").read_text())
    by_id = {t.turn_id: t for t in parse_transcript(args.transcript)}

    out = args.episode_dir / "TURN_INDEX.csv"
    with out.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(
            ["turn_index", "audio_time", "script_time", "speaker",
             "clip_name", "first_words"]
        )
        for turn in manifest["turns"]:
            source = by_id.get(turn["turn_id"])
            clip = Path(turn["segments"][0]["segment_wav"]).stem if turn["segments"] else ""
            words = " ".join((source.clean_text if source else "").split()[: args.words])
            writer.writerow(
                [
                    turn["turn_index"],
                    _mmss(turn["start_ms"]),
                    source.timestamp_mmss if source else "",
                    turn["speaker_id"],
                    clip,
                    words,
                ]
            )
    print(f"wrote {out} ({len(manifest['turns'])} turns)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
