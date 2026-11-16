#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")/.."
mkdir -p dist
if command -v uv >/dev/null 2>&1; then
  if ! uv build --wheel; then
    python3 -m build --wheel
  fi
else
  python3 -m build --wheel
fi
