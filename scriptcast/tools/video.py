"""Render JSON cards using the adjacent, separately installed Remotion workspace."""
import argparse
import json
import os
from pathlib import Path
import subprocess

VIDEO_DIR = Path(__file__).resolve().parents[2] / "video"
DEFAULT_CHROME = Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
COMPONENTS = {"TimelineCard", "StatBarsCard", "TitleCard", "CompareCard"}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    still = sub.add_parser("still", help="Render one card to PNG")
    still.add_argument("card", type=Path)
    still.add_argument("out", type=Path)
    still.add_argument("--component", choices=sorted(COMPONENTS))
    args = parser.parse_args(argv)
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
    if not (VIDEO_DIR / "src" / "index.ts").is_file():
        parser.error(f"Remotion workspace not found at {VIDEO_DIR}; run from a source installation")
    if not (VIDEO_DIR / "node_modules" / ".bin" / "remotion").exists():
        parser.error(f"Install the video workspace dependencies with: cd {VIDEO_DIR} && npm install")
    command = ["npx", "--no-install", "remotion", "still", "src/index.ts", component,
               str(out), f"--props={card}"]
    chrome = os.environ.get("SCRIPTCAST_CHROME")
    if not chrome and DEFAULT_CHROME.is_file():
        chrome = str(DEFAULT_CHROME)
    if chrome:
        command.append(f"--browser-executable={chrome}")
    try:
        return subprocess.run(command, cwd=VIDEO_DIR, check=False).returncode
    except OSError as exc:
        parser.error(f"Could not run Remotion: {exc}")


if __name__ == "__main__":
    raise SystemExit(main())
