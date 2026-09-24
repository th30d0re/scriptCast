"""Render JSON cards using the adjacent, separately installed Remotion workspace."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import shlex
import shutil
import tempfile

import yaml

from scriptcast.parser import parse_transcript
from scriptcast.tools.video_plan import COMPONENTS, build_plan

VIDEO_DIR = Path(__file__).resolve().parents[2] / "video"
DEFAULT_CHROME = Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")

def workspace(parser, *, dependencies=True):
    if not (VIDEO_DIR / "src" / "index.ts").is_file():
        parser.error(f"Remotion workspace not found at {VIDEO_DIR}; run from a source installation")
    if dependencies and not (VIDEO_DIR / "node_modules" / ".bin" / "remotion").exists():
        parser.error(f"Install the video workspace dependencies with: cd {VIDEO_DIR} && npm install")


def browser_args():
    chrome = os.environ.get("SCRIPTCAST_CHROME")
    if not chrome and DEFAULT_CHROME.is_file():
        chrome = str(DEFAULT_CHROME)
    return [f"--browser-executable={chrome}"] if chrome else []


def stage_media(plan):
    """Copy media so the run remains renderable independently of original paths."""
    sources = [Path(plan["audio"]), *(Path(c["src"]) for c in plan["clips"])]
    for source in sources:
        if not source.is_file():
            raise ValueError(f"Missing media: {source}")
    parent = VIDEO_DIR / "public" / "episode"
    parent.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="run-", dir=parent))
    paths = {}
    for source in sources:
        if source not in paths:
            target = run / f"{len(paths):03d}{source.suffix}"
            shutil.copyfile(source, target)
            paths[source] = target.relative_to(VIDEO_DIR / "public").as_posix()
    plan["audio"] = paths[Path(plan["audio"])]
    for clip in plan["clips"]:
        clip["src"] = paths[Path(clip["src"])]


def render(args, parser):
    workspace(parser, dependencies=not args.plan_only)
    episode = args.episode_dir.resolve()
    audio = args.audio.resolve() if args.audio else None
    if audio is None:
        candidates = list(episode.glob("*.mp3"))
        if len(candidates) != 1:
            parser.error(f"Expected one *.mp3 in {episode}, found {len(candidates)}; use --audio")
        audio = candidates[0]
    try:
        if not args.cards.is_dir():
            raise ValueError(f"Cards directory not found: {args.cards}")
        cards = {p.stem: json.loads(p.read_text()) for p in sorted(args.cards.iterdir()) if p.suffix.lower() == ".json"}
        plan = build_plan(json.loads((episode / "episode_manifest.json").read_text()),
                          parse_transcript(args.script), json.loads(args.specs.read_text()),
                          yaml.safe_load(args.clips.read_text()), cards,
                          audio=str(audio), project_root=args.project_root.resolve())
        stage_media(plan)
        out = args.out.resolve()
        plan_path = Path(str(out) + ".plan.json")
        plan_path.parent.mkdir(parents=True, exist_ok=True)
        plan_path.write_text(json.dumps(plan, indent=2) + "\n")
    except (OSError, ValueError, KeyError, TypeError, yaml.YAMLError) as exc:
        parser.error(f"Cannot build episode plan: {exc}")
    if args.plan_only:
        print(f"wrote {plan_path}")
        return 0
    command = ["npx", "--no-install", "remotion", "render", "src/index.ts", "Episode",
               str(out), f"--props={plan_path}", "--codec", "h264", *browser_args()]
    if args.dry_run:
        print(shlex.join(command))
        return 0
    try:
        return subprocess.run(command, cwd=VIDEO_DIR, check=False).returncode
    except OSError as exc:
        parser.error(f"Could not run Remotion: {exc}")




def svg(args, parser):
    workspace(parser, dependencies=False)
    if not (VIDEO_DIR / "node_modules" / ".bin" / "tsc").is_file():
        parser.error(f"Install the video workspace dependencies with: cd {VIDEO_DIR} && npm install")
    command = ["npm", "run", "--silent", "quiver", "--", args.svg_command]
    if args.svg_command == "generate":
        command += [args.prompt, "--model", args.model, "--n", str(args.n)]
        if args.instructions is not None:
            command += ["--instructions", args.instructions]
    elif args.svg_command == "animate":
        command += [str(args.svg_path.resolve())]
        if args.prompt is not None:
            command += ["--prompt", args.prompt]
    if args.dry_run:
        command.append("--dry-run")
    try:
        return subprocess.run(command, cwd=VIDEO_DIR, check=False).returncode
    except OSError as exc:
        parser.error(f"Could not run Quiver CLI: {exc}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    still = sub.add_parser("still", help="Render one card to PNG")
    still.add_argument("card", type=Path)
    still.add_argument("out", type=Path)
    still.add_argument("--component", choices=sorted(COMPONENTS))
    episode = sub.add_parser("render", help="Assemble episode video from a resolved plan")
    episode.add_argument("episode_dir", type=Path)
    for name in ("script", "specs", "clips", "cards", "out"):
        episode.add_argument(f"--{name}", type=Path, required=True)
    episode.add_argument("--audio", type=Path)
    episode.add_argument("--project-root", type=Path, default=Path.cwd(),
                         help="Root for registry source paths (default: current directory)")
    mode = episode.add_mutually_exclusive_group()
    mode.add_argument("--plan-only", action="store_true")
    mode.add_argument("--dry-run", action="store_true")
    svg_parser = sub.add_parser("svg", help="Generate and animate cached SVG assets")
    svg_sub = svg_parser.add_subparsers(dest="svg_command", required=True)
    generate = svg_sub.add_parser("generate")
    generate.add_argument("prompt")
    generate.add_argument("--model", default="arrow-2")
    generate.add_argument("--instructions")
    generate.add_argument("--n", type=int, choices=range(1, 17), default=1)
    animate = svg_sub.add_parser("animate")
    animate.add_argument("svg_path", type=Path)
    animate.add_argument("--prompt")
    models = svg_sub.add_parser("models")
    for command_parser in (generate, animate, models):
        command_parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)
    if args.command == "svg":
        return svg(args, parser)
    if args.command == "render":
        return render(args, parser)
    card, out = args.card.resolve(), args.out.resolve()
    try:
        props = json.loads(card.read_text())
    except (OSError, ValueError) as exc:
        parser.error(f"Cannot read card JSON: {exc}")
    if not isinstance(props, dict):
        parser.error("Card JSON must be an object")
    component = args.component or props.get("component")
    if not isinstance(component, str) or component not in COMPONENTS:
        parser.error("Supply a supported component in JSON or with --component")
    workspace(parser)
    command = ["npx", "--no-install", "remotion", "still", "src/index.ts", component,
               str(out), f"--props={card}"]
    command.extend(browser_args())
    try:
        return subprocess.run(command, cwd=VIDEO_DIR, check=False).returncode
    except OSError as exc:
        parser.error(f"Could not run Remotion: {exc}")


if __name__ == "__main__":
    raise SystemExit(main())
