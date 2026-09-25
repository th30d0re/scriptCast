"""Pure episode timeline resolution. File IO and media staging belong to video.py."""
from __future__ import annotations

from copy import deepcopy
import math
from pathlib import Path
import re

COMPONENTS = {"TimelineCard", "StatBarsCard", "TitleCard", "CompareCard", "FreezeCallout"}


def frame_at(ms: float, fps: int = 30) -> int:
    """Match JavaScript Math.round for nonnegative timeline positions."""
    return math.floor(ms * fps / 1000 + 0.5)


def card_key(value: str) -> str:
    return value.lower().replace("-", "")


def _number(value, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
        raise ValueError(f"{label} must be a finite nonnegative number")
    return value


def build_plan(manifest: dict, script_turns: list, specs: dict, registry: dict,
               cards: dict[str, dict], *, audio: str, project_root: str | Path,
               persist_cards: bool = False, captions: list[dict] | None = None) -> dict:
    """Resolve already-loaded inputs without reading/writing files or mutating inputs.

    script_turns is parse_transcript's result. cards maps filename stems to JSON.
    Relative media sources resolve against an explicit absolute project_root.
    """
    root = Path(project_root)
    if not root.is_absolute() or not Path(audio).is_absolute():
        raise ValueError("audio and project_root must be absolute paths")
    turns = sorted(manifest["turns"], key=lambda t: t["turn_index"])
    if not turns:
        raise ValueError("Manifest has no turns")
    by_index = {t["turn_index"]: t for t in turns}
    if len(by_index) != len(turns):
        raise ValueError("Duplicate manifest turn_index")
    for t in turns:
        if _number(t["end_ms"], "turn end") <= _number(t["start_ms"], "turn start"):
            raise ValueError(f"Invalid window for turn {t['turn_index']}")
    duration = manifest.get("duration_ms", manifest.get("total_duration_ms", max(t["end_ms"] for t in turns)))
    if _number(duration, "duration_ms") < max(t["end_ms"] for t in turns):
        raise ValueError("Manifest duration ends before its turns")
    warnings: list[str] = []
    clip_turns = []
    for turn in script_turns:
        ids = re.findall(r"\[clip:([^\]\s]+)\]", turn.clean_text)
        if len(ids) > 1:
            raise ValueError(f"Multiple clip IDs in script turn {turn.turn_index}")
        if ids:
            clip_turns.append((turn, ids[0]))
    archive_speakers = {s["speaker_id"] for s in manifest.get("speakers", []) if s.get("engine") == "archive"}
    archive_speakers.update(t.speaker_id for t, _ in clip_turns)
    archive_speakers.add("the_reel")
    archive = [t for t in turns if t["speaker_id"] in archive_speakers]
    if len(archive) != len(clip_turns):
        raise ValueError(f"Archive turn count mismatch: script {len(clip_turns)}, manifest {len(archive)}")
    clips = []
    for (_, clip_id), turn in zip(clip_turns, archive):
        if clip_id not in registry["clips"]:
            raise ValueError(f"Missing clip ID in clips.yaml: {clip_id}")
        entry = registry["clips"][clip_id]
        in_ms = _number(entry["start"], f"{clip_id} start") * 1000
        out_ms = _number(entry["end"], f"{clip_id} end") * 1000
        if out_ms <= in_ms or frame_at(out_ms) <= frame_at(in_ms):
            raise ValueError(f"Invalid clip range: {clip_id}")
        length = turn["end_ms"] - turn["start_ms"]
        if length > out_ms - in_ms:
            warnings.append(f"{clip_id}: hold last frame for {length - (out_ms - in_ms):g} ms")
        out_ms = min(out_ms, in_ms + length)
        clips.append(dict(turn_index=turn["turn_index"], clip_id=clip_id,
                          src=str(root / entry["source"]), in_ms=in_ms, out_ms=out_ms,
                          start_ms=turn["start_ms"], end_ms=turn["end_ms"], linger=entry.get("linger") == "next"))
    for previous, current in zip(clips, clips[1:]):
        if previous["end_ms"] > current["start_ms"]:
            raise ValueError("Archive clip turns overlap")
    for index, clip in enumerate(clips):
        # A clip registered with linger: next keeps its last frame on screen until the next
        # archive clip begins (or the episode ends), for example across a spoken reaction.
        if clip.pop("linger"):
            following = clips[index + 1]["start_ms"] if index + 1 < len(clips) else duration
            clip["end_ms"] = max(clip["end_ms"], following)
    normalized = {}
    for key, card in cards.items():
        key = card_key(key)
        if key in normalized:
            raise ValueError(f"Ambiguous card filename: {key}")
        normalized[key] = card
    windows = []
    for shot in specs["shots"]:
        sid = shot["id"]
        card = normalized.get(card_key(sid))
        if card is None:
            warnings.append(f"{sid}: no card file; skipped")
            continue
        if not isinstance(card, dict) or card.get("component") not in COMPONENTS:
            raise ValueError(f"{sid}: unsupported card component")
        anchor = shot["anchor"]
        start = _number(anchor["start_ms"], f"{sid} start")
        hold = shot.get("hold") or {}
        end = start + _number(hold["script_ms"], f"{sid} hold") if "script_ms" in hold else by_index[anchor["turn_index"]]["end_ms"]
        end = min(end, duration)
        if frame_at(end) <= frame_at(start):
            warnings.append(f"{sid}: empty card window; skipped")
            continue
        windows.append(dict(shot_id=sid, start_ms=start, end_ms=end, card=deepcopy(card)))
    windows.sort(key=lambda c: c["start_ms"])
    for earlier, later in zip(windows, windows[1:]):
        if earlier["end_ms"] > later["start_ms"]:
            earlier["end_ms"] = later["start_ms"]
            warnings.append(f"{earlier['shot_id']}: overlap shortened to {later['shot_id']} start")
    windows = [c for c in windows if frame_at(c["end_ms"]) > frame_at(c["start_ms"])]
    if persist_cards:
        # Each card stays up until the next card starts, or until the next archive clip
        # begins (cards are drawn above clips and must not cover them). The last card
        # runs to the next clip or the end of the episode.
        for i, card_window in enumerate(windows):
            limits = [duration]
            if i + 1 < len(windows):
                limits.append(windows[i + 1]["start_ms"])
            limits += [c["start_ms"] for c in clips if c["start_ms"] >= card_window["start_ms"]]
            card_window["end_ms"] = max(card_window["end_ms"], min(limits))
    caption_list = []
    for index, item in enumerate(captions or []):
        text = item.get("text") if isinstance(item, dict) else None
        if not isinstance(text, str) or not text.strip():
            raise ValueError(f"caption {index} has no text")
        start = _number(item.get("start_ms"), f"caption {index} start")
        end = min(_number(item.get("end_ms"), f"caption {index} end"), duration)
        if frame_at(end) > frame_at(start):
            caption_list.append(dict(start_ms=start, end_ms=end, text=text.strip()))
    caption_list.sort(key=lambda c: c["start_ms"])
    return dict(fps=30, width=1080, height=1920, duration_ms=duration, audio=audio,
                clips=clips, cards=windows, captions=caption_list, warnings=warnings)
