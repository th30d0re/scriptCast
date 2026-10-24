"""Run cinetic's render QA checks and combine their results.

The cinetic checkout is deliberately kept outside scriptCast.  This wrapper
calls its scripts in place and records enough detail to make a QA run useful
without coupling either project to the other's implementation.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence


_AUDIO_SUFFIXES = (".wav", ".mp3", ".m4a", ".aac", ".flac", ".ogg")


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _default_cinetic_root() -> Path:
    configured = os.environ.get("CINETIC_ROOT")
    if configured:
        return Path(configured).expanduser()
    return _repo_root().parent / "cinetic" / "skills" / "cinetic"


def _has_cv2() -> bool:
    return importlib.util.find_spec("cv2") is not None


def _help(command: Sequence[str]) -> str:
    result = subprocess.run(
        [*command, "--help"], capture_output=True, text=True, check=False
    )
    return result.stdout + result.stderr


def _summary(stdout: str, stderr: str, returncode: int) -> str:
    detail = stderr if stderr.strip() else stdout
    lines = [line.strip() for line in detail.splitlines() if line.strip()]
    if not lines:
        return "completed successfully" if returncode == 0 else f"exited {returncode}"
    text = lines[-1]
    return text if len(text) <= 500 else text[:497] + "..."


def _run_check(
    name: str,
    command: Sequence[str],
    out: Path,
) -> dict[str, object]:
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    log = [f"$ {' '.join(str(part) for part in command)}", ""]
    if result.stdout:
        log.extend(("--- stdout ---", result.stdout.rstrip()))
    if result.stderr:
        log.extend(("--- stderr ---", result.stderr.rstrip()))
    log.extend(("", f"exit_code: {result.returncode}"))
    (out / f"{name}.log").write_text("\n".join(log) + "\n")
    return {
        "status": "pass" if result.returncode == 0 else "fail",
        "exit_code": result.returncode,
        "summary": _summary(result.stdout, result.stderr, result.returncode),
    }


def _record_non_run(
    report: dict[str, dict[str, object]],
    out: Path,
    name: str,
    status: str,
    summary: str,
) -> None:
    report[name] = {"status": status, "exit_code": None, "summary": summary}
    (out / f"{name}.log").write_text(summary + "\n")


def _source_audio(video: Path) -> Path | None:
    candidates = sorted(
        path
        for path in video.parent.iterdir()
        if path.is_file() and path != video and path.suffix.lower() in _AUDIO_SUFFIXES
    )
    return candidates[0] if candidates else None


def run(video: Path, cinetic_root: Path, out: Path) -> dict[str, dict[str, object]]:
    scripts = cinetic_root / "scripts"
    out.mkdir(parents=True, exist_ok=True)
    report: dict[str, dict[str, object]] = {}

    probe = scripts / "probe.py"
    probe_help = _help([sys.executable, str(probe)])
    probe_command = [sys.executable, str(probe), str(video), "--spec", "@30"]
    if "--json" in probe_help:
        probe_command.extend(("--json", str(out / "probe.json")))
    report["probe"] = _run_check("probe", probe_command, out)

    if _has_cv2():
        report["forensics"] = _run_check(
            "forensics",
            [
                sys.executable,
                str(scripts / "forensics.py"),
                str(video),
                "--json",
                str(out / "forensics.json"),
            ],
            out,
        )
        report["sheet"] = _run_check(
            "sheet",
            [
                sys.executable,
                str(scripts / "sheet.py"),
                str(video),
                "--out",
                str(out / "sheet.png"),
                "--json",
                str(out / "sheet.json"),
            ],
            out,
        )
    else:
        reason = "missing dependency cv2"
        _record_non_run(report, out, "forensics", "skipped", reason)
        _record_non_run(report, out, "sheet", "skipped", reason)

    source = _source_audio(video)
    if source is None:
        _record_non_run(
            report, out, "check-sync", "skipped", "source audio not found in run directory"
        )
    else:
        report["check-sync"] = _run_check(
            "check-sync",
            [sys.executable, str(scripts / "check-sync.py"), str(video), str(source)],
            out,
        )

    lint = scripts / "lint-film.mjs"
    lint_help = _help(["node", str(lint)])
    if "[src]" in lint_help or "file or directory to scan" in lint_help:
        report["lint-film"] = _run_check(
            "lint-film", ["node", str(lint), str(_repo_root() / "video" / "src")], out
        )
    else:
        _record_non_run(
            report,
            out,
            "lint-film",
            "skipped",
            "lint-film.mjs does not advertise a source-path argument",
        )

    _record_non_run(
        report,
        out,
        "av-audit",
        "n/a",
        "scriptCast does not produce the cinetic cues file required by av-audit.py",
    )
    (out / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def _print_table(report: dict[str, dict[str, object]], stream=None) -> None:
    stream = stream or sys.stdout
    print(f"{'check':<14} {'status':<8} summary", file=stream)
    print(f"{'-' * 14} {'-' * 8} {'-' * 40}", file=stream)
    for check, result in report.items():
        print(f"{check:<14} {result['status']:<8} {result['summary']}", file=stream)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("video", type=Path)
    parser.add_argument("--cinetic-root", type=Path, default=_default_cinetic_root())
    parser.add_argument("--out", type=Path, default=Path("cinetic-qa"))
    parser.add_argument("--json", action="store_true", help="print the combined report as JSON")
    args = parser.parse_args()

    cinetic_root = args.cinetic_root.expanduser().resolve()
    if not cinetic_root.is_dir():
        parser.error(
            f"cinetic root does not exist: {cinetic_root} "
            "(pass --cinetic-root or set CINETIC_ROOT)"
        )
    video = args.video.expanduser().resolve()
    if not video.is_file():
        parser.error(f"video does not exist: {video}")

    report = run(video, cinetic_root, args.out.expanduser().resolve())
    if args.json:
        _print_table(report, sys.stderr)
        print(json.dumps(report, indent=2))
    else:
        _print_table(report)
    return 1 if any(item["status"] == "fail" for item in report.values()) else 0


if __name__ == "__main__":
    raise SystemExit(main())
