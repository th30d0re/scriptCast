"""Shared helpers for the opt-in JSON modes on the voice and clip tools.

A JSON mode prints one object to stdout. Logs stay on stderr. Success exits 0.
Failure exits 1 and prints ``{"error": "..."}``.
"""
from __future__ import annotations

import argparse
import json
import sys


def flag_present(name: str, argv: list[str] | None = None) -> bool:
    """True when ``name`` or ``name=value`` appears in ``argv`` (default: ``sys.argv``)."""
    tokens = sys.argv[1:] if argv is None else argv
    prefix = name + "="
    return any(token == name or token.startswith(prefix) for token in tokens)


def emit(payload: dict) -> None:
    """Print one JSON object as a single stdout line."""
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")


def fail(message: str) -> int:
    """Print ``{"error": message}`` and return the failure status."""
    emit({"error": message})
    return 1


class JsonArgumentParser(argparse.ArgumentParser):
    """Argparse that reports usage errors as JSON when a JSON flag is present.

    Without those flags, errors stay on stderr and the process exits 2, matching
    argparse.
    """

    def __init__(self, *args, json_flags: tuple[str, ...] = (), **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self.json_flags = json_flags

    def error(self, message: str) -> None:
        if any(flag_present(flag) for flag in self.json_flags):
            emit({"error": message})
            raise SystemExit(1)
        super().error(message)


def run_json(enabled: bool, body) -> int:
    """Run ``body``. In JSON mode, a string ``SystemExit`` or other exception becomes ``fail``."""
    if not enabled:
        return body()
    try:
        return body()
    except SystemExit as exc:
        if isinstance(exc.code, str):
            return fail(exc.code)
        raise
    except Exception as exc:
        return fail(str(exc))
