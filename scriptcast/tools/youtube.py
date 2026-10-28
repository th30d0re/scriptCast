"""Cut a 16:9 picture onto the episode clock. The default is 4K.

    scriptcast-youtube beats EPISODE --shots shotlist.md
    scriptcast-youtube check EPISODE --shots shotlist.md
    scriptcast-youtube conform --src clip.mp4 --out run.mp4 --frames 30 --res 3840x2160 --fps 30
    scriptcast-youtube figcraft EPISODE --shots shotlist.md --primary S00-01
    scriptcast-youtube cut EPISODE --shots shotlist.md --seg 00
    scriptcast-youtube sheet EPISODE --shots shotlist.md --seg 00
    scriptcast-youtube render EPISODE --shots shotlist.md --primary S00-01

``figcraft`` needs ``figcraft-cli`` on PATH, or ``--figcraft``. It draws one
still per beat and holds it for that beat. It does not call Figma.
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

from scriptcast.figcraft_mcp import FigCraftError, export_beats, start
from scriptcast.project import current
from scriptcast.youtube import (
    CARD_FILES,
    DEFAULT_RES,
    _ffmpeg,
    beat_sheet,
    card_commands,
    check_shots,
    conform,
    cut_segment,
    frame_marks,
    hold_frames,
    load_shots,
    load_turns,
    parse_size,
    render_hook,
    require_alignment,
    resolve_scene,
    run_path,
    runs_of,
    stills_to_run,
    write_beats,
)


def _shots(episode: Path, given: Path | None) -> tuple[list, Path]:
    project = current(episode)
    path = given or project.youtube_shots_for(episode.name)
    if path is None:
        raise ValueError("pass --shots, or set youtube.shots in scriptcast.toml")
    return load_shots(path), path


def _audio(episode: Path, episode_id: str, given: Path | None, silent: bool) -> Path | None:
    if silent:
        return None
    if given is not None:
        if not given.is_file():
            raise ValueError(f"audio not found: {given}")
        return given
    for name in (f"{episode_id}.mp3", f"{episode.name}.mp3"):
        candidate = episode / name
        if candidate.is_file():
            return candidate
    raise ValueError(
        f"no {episode_id}.mp3 in {episode}; pass --audio or --no-audio"
    )


def _cmd_beats(args: argparse.Namespace) -> Path:
    shots, _path = _shots(args.episode_dir, args.shots)
    _episode_id, turns = load_turns(args.episode_dir)
    dest = args.out or (args.episode_dir / "video" / "beats.json")
    return write_beats(shots, turns, dest)


def _cmd_check(args: argparse.Namespace) -> Path:
    shots, path = _shots(args.episode_dir, args.shots)
    _episode_id, turns = load_turns(args.episode_dir)
    require_alignment(shots, turns)
    problems = check_shots(shots, path, current(args.episode_dir).root)
    if problems:
        raise ValueError("\n".join(problems))
    return path


def _cmd_conform(args: argparse.Namespace) -> Path:
    return conform(args.src, args.out, args.res, args.fps, args.frames)


def _cmd_cut(args: argparse.Namespace) -> Path:
    shots, _path = _shots(args.episode_dir, args.shots)
    episode_id, turns = load_turns(args.episode_dir)
    audio = _audio(args.episode_dir, episode_id, args.audio, args.no_audio)
    return cut_segment(
        args.episode_dir, shots, turns, args.seg, args.res, args.fps, audio,
    )


def _cmd_sheet(args: argparse.Namespace) -> Path:
    shots, _path = _shots(args.episode_dir, args.shots)
    _episode_id, turns = load_turns(args.episode_dir)
    require_alignment(shots, turns)
    parse_size(args.res)
    marks = frame_marks(turns, args.fps)
    wanted = [shot for shot in shots if shot.seg == args.seg]
    if not wanted:
        raise ValueError(f"no shots in segment {args.seg}")
    runs = runs_of(shots)
    out = args.episode_dir / "video" / "preview" / args.res / "sheet"
    out.mkdir(parents=True, exist_ok=True)
    for shot in wanted:
        src = run_path(args.episode_dir, shot.primary, args.res, args.fps)
        if not src.is_file():
            raise ValueError(f"{shot.id}: no run at {src}")
        start = (marks[shot.index] - marks[runs[shot.primary][0].index]) / args.fps
        dest = out / f"{shot.id}.png"
        _ffmpeg([
            "-v", "error", "-y", "-ss", f"{start:.6f}", "-i", str(src),
            "-frames:v", "1", str(dest),
        ])
    return out


def _cmd_render(args: argparse.Namespace) -> Path:
    shots, path = _shots(args.episode_dir, args.shots)
    command = args.render_command or current(args.episode_dir).youtube_render
    if not command:
        raise ValueError("pass --command, or set youtube.render in scriptcast.toml")
    primaries = [args.primary] if args.primary else list(runs_of(shots))
    for primary in primaries:
        render_hook(command, args.episode_dir, path, primary)
    return path


def _cmd_figcraft(args: argparse.Namespace) -> Path:
    shots, shots_path = _shots(args.episode_dir, args.shots)
    _episode_id, turns = load_turns(args.episode_dir)
    require_alignment(shots, turns)
    runs = runs_of(shots)
    if args.primary not in runs:
        raise ValueError(f"no run {args.primary}")
    run = runs[args.primary]
    if run[0].file not in CARD_FILES:
        raise ValueError(f"{args.primary} is a {run[0].file or 'unset'} shot, not a card")
    card_path = resolve_scene(run[0].scene, shots_path, current(args.episode_dir).root)
    try:
        card = json.loads(card_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"Cannot read {card_path}: {exc}") from exc
    sheet = beat_sheet(run, turns)
    directory = args.episode_dir / "video" / "figcraft" / args.primary
    directory.mkdir(parents=True, exist_ok=True)
    commands, exports = card_commands(card, sheet["beats"], directory, args.res)
    binary = args.figcraft or shutil.which("figcraft-cli")
    if not binary:
        raise ValueError("figcraft-cli is not on PATH; pass --figcraft")
    read_root = current(args.episode_dir).root
    session = start(binary, read_root, directory)
    try:
        export_beats(session, commands, exports)
    finally:
        session.close()
    counts = hold_frames(sheet, args.fps)
    pngs = [(Path(export["path"]), counts[i]) for i, export in enumerate(exports)]
    return stills_to_run(pngs, run_path(args.episode_dir, args.primary, args.res, args.fps), args.res, args.fps)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="scriptcast-youtube",
        description="Cut a 16:9 picture onto the episode clock. The default is 4K.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def episode(cmd: argparse.ArgumentParser) -> None:
        cmd.add_argument("episode_dir", type=Path)
        cmd.add_argument("--shots", type=Path)

    def geometry(cmd: argparse.ArgumentParser) -> None:
        cmd.add_argument("--res", default=DEFAULT_RES)
        cmd.add_argument("--fps", type=int, default=30)

    beats = sub.add_parser("beats", help="write reveal times from the manifest")
    episode(beats)
    beats.add_argument("--out", type=Path)

    check = sub.add_parser("check", help="confirm each shot names a file that exists")
    episode(check)

    conform_cmd = sub.add_parser("conform", help="fit an external video to a run length")
    conform_cmd.add_argument("--src", type=Path, required=True)
    conform_cmd.add_argument("--out", type=Path, required=True)
    conform_cmd.add_argument("--frames", type=int, required=True)
    geometry(conform_cmd)

    cut = sub.add_parser("cut", help="cut one segment and lay the episode audio under it")
    episode(cut)
    cut.add_argument("--seg", required=True)
    cut.add_argument("--audio", type=Path)
    cut.add_argument("--no-audio", action="store_true")
    geometry(cut)

    sheet = sub.add_parser("sheet", help="write one still per shot from its run")
    episode(sheet)
    sheet.add_argument("--seg", required=True)
    geometry(sheet)

    render = sub.add_parser("render", help="run the show's render command for a primary")
    episode(render)
    render.add_argument("--primary")
    render.add_argument("--command", dest="render_command")

    figcraft = sub.add_parser("figcraft", help="draw one FigCraft still per beat and build the run")
    episode(figcraft)
    figcraft.add_argument("--primary", required=True)
    figcraft.add_argument("--figcraft", help="path to figcraft-cli")
    geometry(figcraft)

    args = parser.parse_args(argv)
    handlers = {
        "beats": _cmd_beats,
        "check": _cmd_check,
        "conform": _cmd_conform,
        "cut": _cmd_cut,
        "sheet": _cmd_sheet,
        "render": _cmd_render,
        "figcraft": _cmd_figcraft,
    }
    try:
        output = handlers[args.command](args)
    except (OSError, ValueError, RuntimeError, FigCraftError, UnicodeError) as exc:
        print(f"scriptcast-youtube: {exc}", file=sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
