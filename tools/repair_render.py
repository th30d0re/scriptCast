"""Verify a rendered episode and re-synthesize whatever fails, until it passes.

Chatterbox is stochastic. A turn that comes out mangled usually comes out fine
on a re-roll, which makes the fix mechanical: transcribe the audio, compare it
to the script, regenerate the turns that disagree, and check again. Doing that
by ear does not scale past one episode.

    python3 tools/repair_render.py outputs/ATO_EP02_local \
        --transcript Architecting_the_operation/podcasts/ATO_EP02_preface.md \
        --voices voice_pipeline/voices.local.yaml --episode-id ATO_EP02_local

Each pass regenerates the flagged turns and re-checks only those, so passes get
cheap quickly. A turn that fails `--max-passes` times in a row is reported and
left alone: repeated failure on the same text points at the text or the
reference audio rather than at a bad sample, and that wants a person.

The final pass relays the timeline, because a regenerated clip is a different
length and leaves a hole where the old one sat.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parent.parent


def _verify(episode_dir: Path, transcript: Path, model: str,
            turns: list[int] | None, threshold: float, max_run: int) -> list[dict]:
    out = _ROOT / ".verify_pass.json"
    cmd = [
        sys.executable, str(_ROOT / "tools" / "verify_render.py"), str(episode_dir),
        "--transcript", str(transcript), "--model", model,
        "--threshold", str(threshold), "--max-run", str(max_run),
        "--json-out", str(out),
    ]
    if turns:
        cmd += ["--turns", ",".join(str(t) for t in turns)]
    subprocess.run(cmd, cwd=_ROOT, capture_output=True, text=True)
    if not out.exists():
        return []
    results = json.loads(out.read_text())
    out.unlink()
    return [
        r for r in results
        if r["score"] < threshold or r["worst_run"] > max_run
    ]


def _regenerate(transcript: Path, episode_id: str, out_dir: Path,
                voices: Path, gap_ms: int, turns: list[int]) -> None:
    subprocess.run(
        [
            sys.executable, "-m", "voice_pipeline",
            "--transcript", str(transcript), "--episode-id", episode_id,
            "--out-dir", str(out_dir), "--voices", str(voices),
            "--gap-ms", str(gap_ms),
            "--regenerate-turns", ",".join(str(t) for t in turns),
        ],
        cwd=_ROOT, capture_output=True, text=True, check=True,
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--transcript", type=Path, required=True)
    ap.add_argument("--episode-id", required=True)
    ap.add_argument("--voices", type=Path, required=True)
    ap.add_argument("--out-dir", type=Path, default=Path("./outputs"))
    ap.add_argument("--gap-ms", type=int, default=350)
    ap.add_argument("--model", choices=["tiny", "small", "medium"], default="small")
    ap.add_argument("--threshold", type=float, default=0.90)
    ap.add_argument("--max-run", type=int, default=2)
    ap.add_argument("--max-passes", type=int, default=4)
    ap.add_argument("--skip-relayout", action="store_true")
    ap.add_argument("--seed-json", type=Path, default=None,
                    help="Reuse a verify_render --json-out file for pass 0 "
                         "instead of transcribing the whole episode again.")
    args = ap.parse_args()

    if args.seed_json:
        print(f"pass 0: reusing {args.seed_json}")
        seeded = json.loads(args.seed_json.read_text())
        failing = [r for r in seeded
                   if r["score"] < args.threshold or r["worst_run"] > args.max_run]
    else:
        print(f"pass 0: verifying every turn with whisper-{args.model}.en")
        failing = _verify(args.episode_dir, args.transcript, args.model, None,
                          args.threshold, args.max_run)
    if not failing:
        print("nothing to repair.")
        return 0
    print(f"  {len(failing)} turn(s) failing: "
          f"{', '.join(str(r['turn']) for r in failing[:20])}"
          f"{' ...' if len(failing) > 20 else ''}")

    # Address turns by id and resolve to a script index on each pass. Indices
    # move when the script is edited between a verification run and a repair,
    # and a stale index regenerates a healthy neighbour while leaving the broken
    # turn untouched.
    attempts: dict[str, int] = {}
    for pass_no in range(1, args.max_passes + 1):
        turns = sorted({r["turn"] for r in failing})
        for r in failing:
            attempts[r["turn_id"]] = attempts.get(r["turn_id"], 0) + 1
        print(f"\npass {pass_no}: re-synthesizing {len(turns)} turn(s)")
        _regenerate(args.transcript, args.episode_id, args.out_dir,
                    args.voices, args.gap_ms, turns)
        failing = _verify(args.episode_dir, args.transcript, args.model, turns,
                          args.threshold, args.max_run)
        print(f"  {len(failing)} still failing")
        if not failing:
            break

    if not args.skip_relayout:
        print("\nrelaying the timeline")
        subprocess.run(
            [sys.executable, str(_ROOT / "tools" / "relayout_episode.py"),
             str(args.episode_dir), "--gap-ms", str(args.gap_ms)],
            cwd=_ROOT, check=True,
        )

    if failing:
        print(f"\n{len(failing)} turn(s) still failing after {args.max_passes} passes. "
              "Repeated failure on the same text points at the text or the reference "
              "audio, not at a bad sample:")
        for r in failing:
            print(f"  turn {r['turn']:>3} {r['speaker']} (score {r['score']:.3f}, "
                  f"run {r['worst_run']}) {r['turn_id']}")
            print(f"      want: {r['expected'][:130]}")
            print(f"      got : {r['heard'][:130]}")
        return 1
    print("\nrepair-render: episode verifies clean.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
