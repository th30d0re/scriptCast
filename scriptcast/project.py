"""Project configuration: a ``scriptcast.toml`` at the project root.

The project root is ``$SCRIPTCAST_PROJECT`` when set, otherwise the nearest
directory at or above the current working directory that contains a
``scriptcast.toml``, otherwise the current working directory.

Every key is optional and every path is relative to the project root unless
absolute. The project is resolved when a path is asked for, not at import, so
tools pick up the caller's working directory.
"""

from __future__ import annotations

import os
import math
import tomllib
from pathlib import Path

CONFIG_NAME = "scriptcast.toml"
ROOT_ENV_VAR = "SCRIPTCAST_PROJECT"

DEFAULTS: dict[str, str | int | float] = {
    "tail_ms": 400,
    "speech_threshold": 0.03,
    "voices": "voices.yaml",
    "pronunciations": "pronunciations.yaml",
    "speaker_rates": "speaker_rates.json",
    "clips": "archive/clips.yaml",
    "clip_sources": "archive/sources",
    "references": "voices/candidates",
    "scripts": "scripts",
    "outputs": "outputs",
    "omnivoice_python": ".venv-omnivoice/bin/python",
}


def find_root(start: Path | None = None) -> Path:
    """The project root for `start` (default: the current directory)."""
    env = os.environ.get(ROOT_ENV_VAR)
    if env:
        return Path(env).expanduser().resolve()
    start = Path(start) if start is not None else Path.cwd()
    start = start.expanduser().resolve()
    for directory in (start, *start.parents):
        if (directory / CONFIG_NAME).is_file():
            return directory
    return start


class Project:
    """A resolved `scriptcast.toml`: the root plus every configured path."""

    def __init__(self, root: Path) -> None:
        self.root = root
        raw: dict = {}
        config_path = root / CONFIG_NAME
        if config_path.is_file():
            with config_path.open("rb") as handle:
                raw = tomllib.load(handle)
        unknown = sorted(set(raw) - set(DEFAULTS))
        if unknown:
            raise ValueError(
                f"{config_path}: unknown key(s): {', '.join(unknown)}. "
                f"Known keys: {', '.join(sorted(DEFAULTS))}."
            )
        merged = {**DEFAULTS, **raw}
        self.tail_ms = merged["tail_ms"]
        self.speech_threshold = merged["speech_threshold"]
        if type(self.tail_ms) is not int or self.tail_ms < 0:
            raise ValueError("tail_ms must be a nonnegative integer")
        if (type(self.speech_threshold) not in (int, float)
                or not math.isfinite(self.speech_threshold)
                or self.speech_threshold < 0):
            raise ValueError("speech_threshold must be a finite nonnegative number")
        self._paths = {key: self.resolve(value) for key, value in merged.items()
                       if key not in {"tail_ms", "speech_threshold"}}

    def resolve(self, value: object) -> Path:
        """`value` as an absolute path; relative values anchor at the root."""
        path = Path(str(value)).expanduser()
        return path if path.is_absolute() else self.root / path

    def path(self, key: str) -> Path:
        if key not in self._paths:
            raise KeyError(
                f"unknown project key {key!r}; known keys: {', '.join(sorted(self._paths))}"
            )
        return self._paths[key]

    @property
    def voices(self) -> Path:
        return self._paths["voices"]

    @property
    def pronunciations(self) -> Path:
        return self._paths["pronunciations"]

    @property
    def speaker_rates(self) -> Path:
        return self._paths["speaker_rates"]

    @property
    def clips(self) -> Path:
        return self._paths["clips"]

    @property
    def clip_sources(self) -> Path:
        return self._paths["clip_sources"]

    @property
    def references(self) -> Path:
        return self._paths["references"]

    @property
    def scripts(self) -> Path:
        return self._paths["scripts"]

    @property
    def outputs(self) -> Path:
        return self._paths["outputs"]

    @property
    def omnivoice_python(self) -> Path:
        return self._paths["omnivoice_python"]


def current(start: Path | None = None) -> Project:
    """The project for `start` (default: the current directory)."""
    return Project(find_root(start))
