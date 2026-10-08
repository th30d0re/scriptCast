"""Machine-readable, append-only progress events for command-line tools."""

from __future__ import annotations

import json
import signal
from datetime import datetime, timezone
from pathlib import Path
from types import FrameType
from typing import Any


class ProgressCancelled(BaseException):
    """Raised when an opted-in command receives SIGTERM."""


class Progress:
    def __init__(self, path: Path | None) -> None:
        self.path = path
        self._handle = None
        self._finished = False
        self._previous_sigterm: Any = None

    def __enter__(self) -> "Progress":
        if self.path is not None:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            self._handle = self.path.open("a", encoding="utf-8")
            self._previous_sigterm = signal.getsignal(signal.SIGTERM)
            signal.signal(signal.SIGTERM, self._cancel)
        return self

    def __exit__(self, exc_type, exc, traceback) -> bool:
        del exc_type, traceback
        if exc is not None and not self._finished:
            message = "cancelled" if isinstance(exc, ProgressCancelled) else str(exc)
            self.error(message or exc.__class__.__name__)
        elif exc is None and not self._finished:
            self.done()
        if self._handle is not None:
            signal.signal(signal.SIGTERM, self._previous_sigterm)
            self._handle.close()
        if isinstance(exc, ProgressCancelled):
            raise SystemExit(130) from None
        return False

    def _cancel(self, signum: int, frame: FrameType | None) -> None:
        del signum, frame
        raise ProgressCancelled()

    def emit(self, event: str, **fields: object) -> None:
        if self._handle is None:
            return
        payload = {"event": event, **fields}
        self._handle.write(json.dumps(payload, separators=(",", ":")) + "\n")
        self._handle.flush()

    def start(self, tool: str, total: int) -> None:
        self.emit(
            "start",
            tool=tool,
            total=total,
            ts=datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        )

    def done(self, *, ok: bool = True, outputs: dict[str, str] | None = None) -> None:
        if self._finished:
            return
        self.emit("done", ok=ok, outputs=outputs or {})
        self._finished = True

    def error(self, message: str) -> None:
        if self._finished:
            return
        self.emit("error", message=message)
        self._finished = True
