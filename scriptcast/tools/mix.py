"""Post-mix an episode: a music bed, quiet overlays (an aside under a line) and per-turn gain.

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
    bed:
      file: music/bed.wav                   # looped to the episode length
      gain_db: -8                           # bed level under the dialogue
      treatment: duck_carve                 # duck_carve (default), duck, carve or none
      target_db: -12                        # duck depth during speech
      merge_gap_ms: 600                     # turns closer than this duck as one span

The bed is ducked and carved by the scriptcast-media binary (media/ crate). It is found
via $SCRIPTCAST_MEDIA, then PATH, then media/target/release in this repo.

Usage: scriptcast-mix EPISODE_DIR --script S.md --config mix.yaml --asr captions.json.asr.json --out mixed.mp3
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
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
    return {"overlays": overlays, "gains": gains, "bed": plan_bed(config.get("bed"), manifest, root)}


BED_TREATMENTS = ("duck_carve", "duck", "carve", "none")


def plan_bed(rule: dict | None, manifest: dict, root: Path) -> dict | None:
    if not rule:
        return None
    treatment = rule.get("treatment", "duck_carve")
    if treatment not in BED_TREATMENTS:
        raise ValueError(f"bed treatment must be one of {BED_TREATMENTS}: {treatment!r}")
    return {"file": str((root / rule["file"]).resolve()),
            "gain_db": float(rule.get("gain_db", -8)),
            "treatment": treatment,
            "target_db": float(rule.get("target_db", -12)),
            "merge_gap_ms": float(rule.get("merge_gap_ms", 600)),
            "spans": [[t["start_ms"], t["end_ms"]] for t in manifest["turns"]]}


def media_binary() -> str:
    candidates = [os.environ.get("SCRIPTCAST_MEDIA"), shutil.which("scriptcast-media"),
                  str(Path(__file__).resolve().parents[2] / "media/target/release/scriptcast-media")]
    for c in candidates:
        if c and Path(c).is_file():
            return c
    raise FileNotFoundError("scriptcast-media not found: build it with `cargo build --release` in media/, or set SCRIPTCAST_MEDIA")


def render_bed(audio: Path, bed: dict, work: Path) -> Path:
    """Loop the bed to the episode length, then duck and/or carve it against the dialogue."""
    looped = work / "bed.wav"
    duration = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(audio)],
                              check=True, capture_output=True, text=True).stdout.strip()
    subprocess.run(["ffmpeg", "-y", "-stream_loop", "-1", "-i", bed["file"], "-t", duration,
                    "-ar", "48000", str(looped)], check=True, capture_output=True)
    current = looped
    if bed["treatment"] == "none":
        return current
    binary = media_binary()
    if bed["treatment"] in ("duck_carve", "duck"):
        spans = work / "spans.json"
        spans.write_text(json.dumps(bed["spans"]))
        ducked = work / "bed_ducked.wav"
        subprocess.run([binary, "duck", "--bed", str(current), "--spans", str(spans), "--out", str(ducked),
                        "--target-db", str(bed["target_db"]), "--merge-gap-ms", str(bed["merge_gap_ms"])],
                       check=True, capture_output=True)
        current = ducked
    if bed["treatment"] in ("duck_carve", "carve"):
        voice = work / "voice.wav"
        subprocess.run(["ffmpeg", "-y", "-i", str(audio), "-ac", "1", "-ar", "48000", str(voice)], check=True, capture_output=True)
        carved = work / "bed_carved.wav"
        subprocess.run([binary, "carve", "--bed", str(current), "--voice", str(voice), "--out", str(carved),
                        "--merge-gap-ms", str(bed["merge_gap_ms"])], check=True, capture_output=True)
        current = carved
    return current


def build_command(audio: Path, plan: dict, out: Path, bitrate: str = "128k", bed_wav: Path | None = None) -> list[str]:
    cmd = ["ffmpeg", "-y", "-i", str(audio)]
    for o in plan["overlays"]:
        cmd += ["-i", o["file"]]
    if bed_wav is not None:
        cmd += ["-i", str(bed_wav)]
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
    if bed_wav is not None:
        i = 1 + len(plan["overlays"])
        filters.append(f"[{i}:a]volume={plan['bed']['gain_db']}dB[bed]")
        labels.append("[bed]")
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
    with tempfile.TemporaryDirectory(prefix=".scriptcast-bed-", dir=args.out.parent) as temporary:
        bed_wav = render_bed(audio, plan["bed"], Path(temporary)) if plan["bed"] else None
        subprocess.run(build_command(audio, plan, args.out, bed_wav=bed_wav), check=True, capture_output=True)
    bed_note = f", bed {plan['bed']['treatment']}" if plan["bed"] else ""
    print(f"wrote {args.out}: {len(plan['overlays'])} overlays, {len(plan['gains'])} gains{bed_note}")
    for o in plan["overlays"]:
        print(f"  overlay {Path(o['file']).name} at {o['at_ms'] / 1000:.2f}s ({o['gain_db']} dB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
