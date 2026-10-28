"""A 16:9 picture locked to the episode manifest. The default cut is 4K.

One shot is one turn. A run is a primary shot plus the holds that name it.
Shot ``i`` occupies frames ``[F(i), F(i+1))``, where ``F(t) = round(t × fps)``
and ``t`` is the turn's ``start_ms``. A cut taken that way cannot drift against
the voiceover. FigCraft draws one still per beat. This module does not render
a manim scene and it does not call Figma.
"""
from __future__ import annotations

import json
import math
import os
import re
import shlex
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

SIZES = {
    "960x540": (960, 540),
    "1920x1080": (1920, 1080),
    "3840x2160": (3840, 2160),
}
DEFAULT_RES = "3840x2160"
FRAME_RATES = frozenset({15, 30})
REVEAL_S = 0.25
CARD_FILES = frozenset({"figcraft", "figma-card"})
BG = "0F1B33"
INK = "F4EAD2"
DIM = "9FB0CF"

_SHOT = re.compile(
    r"(?ms)^## (S(\d\d)-\d\d[a-z]?)(?: — (.*?))?\n(.*?)(?=^## |\Z)"
)


@dataclass(frozen=True)
class Shot:
    id: str
    seg: str
    index: int
    kind: str
    file: str
    scene: str
    holds: str | None
    title: str
    note: str

    @property
    def primary(self) -> str:
        return self.holds or self.id


def frame_index(t_seconds: float, fps: int) -> int:
    """The frame ``t`` falls on. Matches ``edit16x9.frame``: round half away from zero via floor(x+0.5)."""
    if fps not in FRAME_RATES:
        raise ValueError(f"fps must be one of {sorted(FRAME_RATES)}")
    return math.floor(t_seconds * fps + 0.5)


def parse_size(res: str) -> tuple[int, int]:
    if res not in SIZES:
        raise ValueError(f"res must be one of {', '.join(SIZES)}")
    return SIZES[res]


def load_shots(path: Path) -> list[Shot]:
    text = path.read_text(encoding="utf-8")
    shots: list[Shot] = []
    for match in _SHOT.finditer(text):
        sid, seg, title, body = match.groups()

        def field(name: str, body: str = body) -> str:
            hit = re.search(rf"\*\*{name}:\*\* (.*)", body)
            return hit.group(1).strip() if hit else ""

        render = field("Render").strip("`")
        file, _, scene = render.partition(":")
        holds = field("Holds").strip("`") or None
        shots.append(Shot(
            sid, seg, len(shots), field("Type"), file, scene, holds,
            (title or "").strip(), field("Note"),
        ))
    if not shots:
        raise ValueError(f"{path}: no shots")
    return shots


def runs_of(shots: list[Shot]) -> dict[str, list[Shot]]:
    grouped: dict[str, list[Shot]] = {}
    known = {shot.id for shot in shots}
    for shot in shots:
        if shot.holds and shot.holds not in known:
            raise ValueError(f"{shot.id} holds {shot.holds}, which is not a shot")
        grouped.setdefault(shot.primary, []).append(shot)
    return grouped


def load_turns(episode_dir: Path) -> tuple[str, list[dict]]:
    path = episode_dir / "episode_manifest.json"
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"Cannot read {path}: {exc}") from exc
    if not isinstance(manifest, dict):
        raise ValueError(f"{path}: expected an object")
    turns = manifest.get("turns")
    if not isinstance(turns, list) or not turns:
        raise ValueError(f"{path}: no turns")
    episode_id = manifest.get("episode_id")
    if not isinstance(episode_id, str) or not episode_id:
        episode_id = episode_dir.name
    windows = []
    for i, turn in enumerate(turns):
        if not isinstance(turn, dict):
            raise ValueError(f"turns[{i}] must be an object")
        start = turn.get("start_ms")
        end = turn.get("end_ms")
        if isinstance(start, bool) or not isinstance(start, int) or start < 0:
            raise ValueError(f"turns[{i}] start_ms must be a nonnegative integer")
        if isinstance(end, bool) or not isinstance(end, int) or end <= start:
            raise ValueError(f"turns[{i}] end_ms must be after start_ms")
        windows.append({"start_ms": start, "end_ms": end})
    return episode_id, windows


def require_alignment(shots: list[Shot], turns: list[dict]) -> None:
    if len(shots) != len(turns):
        raise ValueError(
            f"manifest has {len(turns)} turns; shot list has {len(shots)}"
        )


def frame_marks(turns: list[dict], fps: int) -> list[int]:
    """``F[i]`` is the first frame of turn ``i``. The last mark is one second past the episode end."""
    starts = [turn["start_ms"] / 1000 for turn in turns]
    end = turns[-1]["end_ms"] / 1000
    return [frame_index(t, fps) for t in starts] + [frame_index(end + 1.0, fps)]


def run_bounds(run: list[Shot], turns: list[dict]) -> tuple[float, float]:
    t0 = turns[run[0].index]["start_ms"] / 1000
    after = run[-1].index + 1
    if after < len(turns):
        t1 = turns[after]["start_ms"] / 1000
    else:
        t1 = turns[-1]["end_ms"] / 1000
    return t0, t1


def beat_sheet(run: list[Shot], turns: list[dict]) -> dict:
    """Reveal times relative to the run, a quarter second after each turn starts."""
    t0, t1 = run_bounds(run, turns)
    beats = []
    for shot in run:
        beats.append({
            "shot": shot.id,
            "t": round(turns[shot.index]["start_ms"] / 1000 - t0 + REVEAL_S, 2),
        })
    return {
        "primary": run[0].primary,
        "duration": round(t1 - t0, 2),
        "offset_s": REVEAL_S,
        "beats": beats,
    }


def write_beats(shots: list[Shot], turns: list[dict], dest: Path) -> Path:
    require_alignment(shots, turns)
    payload = {
        "runs": [beat_sheet(run, turns) for run in runs_of(shots).values()],
    }
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return dest


def resolve_scene(scene: str, shots_path: Path, root: Path | None) -> Path:
    candidate = Path(scene)
    if candidate.is_absolute():
        return candidate
    for base in (shots_path.parent, root):
        if base is None:
            continue
        found = base / candidate
        if found.exists():
            return found
    return (root or shots_path.parent) / candidate


def check_shots(shots: list[Shot], shots_path: Path, root: Path | None) -> list[str]:
    """Return one problem per shot whose picture source is missing. A manim scene is not checked."""
    problems = []
    for shot in shots:
        if shot.file in CARD_FILES or shot.file == "ext":
            path = resolve_scene(shot.scene, shots_path, root)
            if not path.is_file():
                problems.append(f"{shot.id}: missing {path}")
        elif not shot.file:
            problems.append(f"{shot.id}: no Render target")
    return problems


def _ffmpeg(args: list[str]) -> None:
    binary = shutil.which("ffmpeg")
    if binary is None:
        raise RuntimeError("ffmpeg is not on PATH")
    try:
        subprocess.run([binary, *args], check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as exc:
        detail = (exc.stderr or exc.stdout or "").strip()
        raise RuntimeError(detail or f"ffmpeg failed ({exc.returncode})") from exc


def conform(src: Path, out: Path, res: str, fps: int, n_frames: int) -> Path:
    """Fit ``src`` inside the frame, then pad or trim to exactly ``n_frames``. Silent."""
    if n_frames < 1:
        raise ValueError("n_frames must be positive")
    width, height = parse_size(res)
    frame_index(0, fps)
    out.parent.mkdir(parents=True, exist_ok=True)
    # stop_duration is seconds. Ask for more tail than the run, then keep n_frames,
    # so a short source ends on a held last frame instead of looping.
    pad_s = n_frames / fps + 1
    vf = (
        f"fps={fps},scale={width}:{height}:force_original_aspect_ratio=decrease,"
        f"pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=0x{BG},setsar=1,"
        f"tpad=stop_mode=clone:stop_duration={pad_s}"
    )
    _ffmpeg([
        "-v", "error", "-y", "-i", str(src),
        "-vf", vf, "-frames:v", str(n_frames), "-an",
        "-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-pix_fmt", "yuv420p",
        str(out),
    ])
    return out


def _placeholder(out: Path, n_frames: int, res: str, fps: int) -> None:
    width, height = parse_size(res)
    out.parent.mkdir(parents=True, exist_ok=True)
    _ffmpeg([
        "-v", "error", "-y", "-f", "lavfi",
        "-i", f"color=c=0x{BG}:s={width}x{height}:r={fps}",
        "-frames:v", str(n_frames), "-an",
        "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p",
        str(out),
    ])


def run_path(episode_dir: Path, primary: str, res: str, fps: int) -> Path:
    return episode_dir / "video" / "runs" / f"{res}@{fps}" / f"{primary}.mp4"


def _cut_window(src: Path, dest: Path, start_frame: int, n_frames: int, fps: int) -> None:
    vf = (
        f"fps={fps},tpad=stop_mode=clone:stop={n_frames},"
        f"trim=start_frame={start_frame}:end_frame={start_frame + n_frames},setpts=PTS-STARTPTS"
    )
    _ffmpeg([
        "-v", "error", "-y", "-i", str(src),
        "-vf", vf, "-frames:v", str(n_frames), "-an",
        "-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-pix_fmt", "yuv420p",
        str(dest),
    ])


def cut_segment(
    episode_dir: Path,
    shots: list[Shot],
    turns: list[dict],
    seg: str,
    res: str,
    fps: int,
    audio: Path | None,
) -> Path:
    """Cut one segment. A missing run becomes a placeholder of the right length."""
    require_alignment(shots, turns)
    parse_size(res)
    marks = frame_marks(turns, fps)
    wanted = [shot for shot in shots if shot.seg == seg]
    if not wanted:
        raise ValueError(f"no shots in segment {seg}")
    runs = runs_of(shots)
    first_of = {primary: run[0].index for primary, run in runs.items()}
    work = episode_dir / "video" / "preview" / res / "_clips"
    work.mkdir(parents=True, exist_ok=True)
    listing: list[str] = []
    for shot in wanted:
        n_frames = marks[shot.index + 1] - marks[shot.index]
        if n_frames < 1:
            raise ValueError(f"{shot.id} is shorter than one frame")
        clip = work / f"{shot.id}.mp4"
        src = run_path(episode_dir, shot.primary, res, fps)
        if not src.is_file():
            _placeholder(clip, n_frames, res, fps)
        else:
            start = marks[shot.index] - marks[first_of[shot.primary]]
            _cut_window(src, clip, start, n_frames, fps)
        listing.append(f"file '{clip}'\nduration {n_frames / fps:.6f}")
    concat = work / f"SEG{seg}.txt"
    concat.write_text("\n".join(listing) + "\n", encoding="utf-8")
    out = episode_dir / "video" / "preview" / res / f"SEG{seg}.mp4"
    a0 = marks[wanted[0].index] / fps
    a1 = marks[wanted[-1].index + 1] / fps
    command = [
        "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(concat),
    ]
    if audio is not None:
        command += ["-ss", f"{a0:.6f}", "-t", f"{a1 - a0:.6f}", "-i", str(audio)]
        command += ["-map", "0:v", "-map", "1:a", "-c:a", "aac", "-b:a", "160k"]
    command += [
        "-vf", f"fps={fps}", "-c:v", "libx264", "-preset", "veryfast",
        "-crf", "18", "-pix_fmt", "yuv420p", str(out),
    ]
    _ffmpeg(command)
    shutil.rmtree(work, ignore_errors=True)
    return out


def hold_frames(sheet: dict, fps: int) -> list[int]:
    """Frame count of each beat's still, on the same rounding as the cut."""
    beats = sheet["beats"]
    if not beats:
        raise ValueError(f"{sheet['primary']} has no beats")
    times = [0.0]
    times.extend(float(beat["t"]) for beat in beats[1:])
    times.append(float(sheet["duration"]))
    marks = [frame_index(t, fps) for t in times]
    counts = [marks[i + 1] - marks[i] for i in range(len(beats))]
    if any(count < 1 for count in counts):
        raise ValueError(f"{sheet['primary']} has a beat at or after the run ends")
    return counts


def stills_to_run(pngs: list[tuple[Path, int]], out: Path, res: str, fps: int) -> Path:
    """Hold each PNG for a whole number of frames and write a silent run."""
    if not pngs:
        raise ValueError("no stills")
    width, height = parse_size(res)
    work = out.parent / f".{out.stem}-stills"
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    clips = []
    for index, (png, n_frames) in enumerate(pngs):
        if n_frames < 1:
            raise ValueError(f"{png} has no frames")
        clip = work / f"{index}.mp4"
        _ffmpeg([
            "-v", "error", "-y", "-loop", "1", "-i", str(png),
            "-frames:v", str(n_frames), "-r", str(fps),
            "-vf", f"scale={width}:{height},setsar=1",
            "-an", "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p",
            str(clip),
        ])
        clips.append(clip)
    listing = work / "stills.txt"
    listing.write_text("".join(f"file '{clip.resolve()}'\n" for clip in clips), encoding="utf-8")
    out.parent.mkdir(parents=True, exist_ok=True)
    _ffmpeg([
        "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(listing),
        "-c", "copy", str(out),
    ])
    shutil.rmtree(work, ignore_errors=True)
    return out


def _item_text(item: object) -> tuple[str, str]:
    if isinstance(item, str):
        return item, ""
    if not isinstance(item, dict):
        return str(item), ""
    title = item.get("title") or item.get("quote") or item.get("label") or ""
    detail = item.get("detail") or item.get("attribution") or item.get("gloss") or ""
    if not title and item.get("latex"):
        title = str(item["latex"])
    return str(title), str(detail)


def card_commands(
    card: dict, beats: list[dict], directory: Path, res: str = "1920x1080",
) -> tuple[list[dict], list[dict]]:
    """One frame per beat, at ``res``. Item ``i`` is drawn from beat ``i`` onward.

    Positions are the 1920×1080 layout, scaled to ``res``.

    The first list is FigCraft ``node.create`` commands, in order. A ``$ref``
    names the id of an earlier command marked ``as``. The second list is one
    PNG export per frame. A title, a quote, or a list of items is set in the
    house colours.
    A ``latex`` field is left as that string; math stays a manim scene.
    """
    if not isinstance(card, dict):
        raise ValueError("card must be an object")
    items = card.get("items") or []
    if not isinstance(items, list):
        raise ValueError("card items must be a list")
    headline = str(card.get("headline") or card.get("quote") or "")
    sources = str(card.get("sources") or "")
    width, height = parse_size(res)

    def at(px: int) -> int:
        return round(px * width / 1920)

    commands: list[dict] = []
    exports: list[dict] = []
    for beat_index, _beat in enumerate(beats):
        visible = items[: beat_index + 1]
        frame = f"beat-{beat_index}"
        commands.append({
            "command": "node.create",
            "as": frame,
            "params": {
                "type": "frame",
                "name": frame,
                "x": beat_index * at(2000),
                "y": 0,
                "width": width,
                "height": height,
                "props": {"fill": f"#{BG}"},
            },
        })
        commands.append(_text(
            frame, "headline", at(80), at(64), at(1760), at(88), headline, at(48), INK,
        ))
        y = at(180)
        for item_index, item in enumerate(visible):
            title, detail = _item_text(item)
            commands.append(_text(
                frame, f"item-{item_index}", at(80), y, at(1760), at(48), title, at(32), INK,
            ))
            if detail:
                commands.append(_text(
                    frame, f"item-{item_index}-detail", at(80), y + at(48), at(1760), at(64),
                    detail, at(24), DIM,
                ))
            y += at(140)
        if sources:
            commands.append(_text(
                frame, "sources", at(80), at(980), at(1760), at(40), sources, at(22), DIM,
            ))
        exports.append({
            "frame": frame,
            "path": str(directory / f"beat-{beat_index:02d}.png"),
        })
    return commands, exports


def _text(frame: str, name: str, x: int, y: int, width: int, height: int,
          text: str, size: int, fill: str) -> dict:
    return {
        "command": "node.create",
        "params": {
            "type": "text",
            "parent": {"$ref": frame},
            "name": name,
            "x": x,
            "y": y,
            "width": width,
            "height": height,
            "text": text,
            "props": {"fill": f"#{fill}", "fontSize": size},
        },
    }


def hold_durations(sheet: dict) -> list[float]:
    """How long each beat's still stays up. The first still also covers the lead-in before its reveal."""
    beats = sheet["beats"]
    if not beats:
        raise ValueError(f"{sheet['primary']} has no beats")
    duration = float(sheet["duration"])
    boundaries = [0.0]
    for beat in beats[1:]:
        boundaries.append(float(beat["t"]))
    boundaries.append(duration)
    holds = [round(boundaries[i + 1] - boundaries[i], 6) for i in range(len(beats))]
    if any(hold <= 0 for hold in holds):
        raise ValueError(f"{sheet['primary']} has a beat at or after the run ends")
    return holds


def render_hook(command: str, episode_dir: Path, shots_path: Path, primary: str) -> None:
    if not command.strip():
        raise ValueError("no render command; set youtube.render in scriptcast.toml")
    env = os.environ.copy()
    env["SCRIPTCAST_EPISODE"] = str(episode_dir)
    env["SCRIPTCAST_SHOTS"] = str(shots_path)
    env["SCRIPTCAST_PRIMARY"] = primary
    try:
        subprocess.run(shlex.split(command), check=True, env=env)
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"render command failed ({exc.returncode})") from exc
