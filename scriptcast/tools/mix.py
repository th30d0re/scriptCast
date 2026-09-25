"""Post-mix an episode: quiet overlays (an aside under a line) and per-turn gain.

The mix is written to a new file. The stitched MP3 stays as it is, so re-stitching
and re-mixing never compound. Placement follows the script text, not timestamps,
so it survives re-timing.

mix.yaml:

    overlays:
      - file: archive/sources/nice.wav      # relative to the config file's project root
        turn_contains: "sixty-nine days after she'd already signed"
        after_word: "^(69|sixtynine)$"      # normalized recognised word; overlay starts after it
        delay_ms: 120
        gain_db: -22
    gains:
      - turn_equals: "Interesting."         # or turn_contains
        gain_db: 5

Usage: scriptcast-mix EPISODE_DIR --script S.md --config mix.yaml --asr captions.json.asr.json --out mixed.mp3
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

import yaml

from scriptcast.parser import parse_transcript
from scriptcast.tools.captions import normalize


def _find_turn(script_turns, rule: dict):
    if "turn_equals" in rule:
        wanted = rule["turn_equals"].strip().lower()
        hits = [t for t in script_turns if t.clean_text.strip().lower() == wanted]
    elif "turn_contains" in rule:
        wanted = rule["turn_contains"].lower()
        hits = [t for t in script_turns if wanted in t.clean_text.lower()]
    else:
        raise ValueError(f"mix rule needs turn_equals or turn_contains: {rule}")
    if len(hits) != 1:
        raise ValueError(f"mix rule matched {len(hits)} turns, expected 1: {rule}")
    return hits[0]


def plan_mix(config: dict, script_turns, manifest: dict, asr_words: list[dict], root: Path) -> dict:
    turns = {t["turn_index"]: t for t in manifest["turns"]}
    overlays, gains = [], []
    for rule in config.get("overlays", []):
        turn = _find_turn(script_turns, rule)
        entry = turns[turn.turn_index]
        start_s, end_s = entry["start_ms"] / 1000, entry["end_ms"] / 1000
        pattern = re.compile(rule["after_word"])
        hit = next((w for w in asr_words
                    if w["start"] >= start_s - 0.05 and w["end"] <= end_s + 0.3
                    and pattern.search(normalize(w["word"]))), None)
        if hit is None:
            raise ValueError(f"no recognised word matches {rule['after_word']!r} in turn {turn.turn_index}")
        at_ms = int(round(hit["end"] * 1000)) + int(rule.get("delay_ms", 0))
        overlays.append({"file": str((root / rule["file"]).resolve()), "at_ms": at_ms, "gain_db": float(rule["gain_db"])})
    for rule in config.get("gains", []):
        turn = _find_turn(script_turns, rule)
        entry = turns[turn.turn_index]
        gains.append({"start_s": entry["start_ms"] / 1000, "end_s": entry["end_ms"] / 1000, "gain_db": float(rule["gain_db"])})
    return {"overlays": overlays, "gains": gains}


def build_command(audio: Path, plan: dict, out: Path, bitrate: str = "128k") -> list[str]:
    cmd = ["ffmpeg", "-y", "-i", str(audio)]
    for o in plan["overlays"]:
        cmd += ["-i", o["file"]]
    chain = "[0:a]"
    filters = []
    for i, g in enumerate(plan["gains"]):
        label = f"g{i}"
        filters.append(f"{chain}volume={g['gain_db']}dB:enable='between(t,{g['start_s']:.3f},{g['end_s']:.3f})'[{label}]")
        chain = f"[{label}]"
    labels = []
    for i, o in enumerate(plan["overlays"], start=1):
        filters.append(f"[{i}:a]aformat=channel_layouts=mono,volume={o['gain_db']}dB,adelay={o['at_ms']}|{o['at_ms']}[o{i}]")
        labels.append(f"[o{i}]")
    if labels:
        filters.append(f"{chain}{''.join(labels)}amix=inputs={1 + len(labels)}:normalize=0:duration=first[mix]")
        chain = "[mix]"
    filters.append(f"{chain}alimiter=limit=0.95[out]")
    cmd += ["-filter_complex", ";".join(filters), "-map", "[out]", "-c:a", "libmp3lame", "-b:a", bitrate, str(out)]
    return cmd


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--script", type=Path, required=True)
    ap.add_argument("--config", type=Path, required=True)
    ap.add_argument("--asr", type=Path, required=True, help="recognised words JSON written by scriptcast-captions")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--audio", type=Path, help="defaults to the single .mp3 in episode_dir")
    ap.add_argument("--root", type=Path, default=Path("."), help="base for relative overlay files")
    args = ap.parse_args(argv)
    audio = args.audio
    if audio is None:
        found = sorted(args.episode_dir.glob("*.mp3"))
        if len(found) != 1:
            ap.error("Expected exactly one .mp3 in the episode directory, or pass --audio")
        audio = found[0]
    manifest = json.loads((args.episode_dir / "episode_manifest.json").read_text())
    config = yaml.safe_load(args.config.read_text()) or {}
    plan = plan_mix(config, parse_transcript(args.script), manifest, json.loads(args.asr.read_text()), args.root)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(build_command(audio, plan, args.out), check=True, capture_output=True)
    print(f"wrote {args.out}: {len(plan['overlays'])} overlays, {len(plan['gains'])} gains")
    for o in plan["overlays"]:
        print(f"  overlay {Path(o['file']).name} at {o['at_ms'] / 1000:.2f}s ({o['gain_db']} dB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
