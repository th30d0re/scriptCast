#!/usr/bin/env bash
set -u

ROOT="$HOME/Library/Application Support/scriptCast"
LOG="$ROOT/install.log"
APP_VERSION=""
WHEEL=""
WITH_OMNIVOICE=0
CHECK_ONLY=0

log() { printf '[%s] %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >&3; }
step() { printf 'STEP %s/6 %s\n' "$1" "$2"; log "STEP $1/6 $2"; }
ok() { printf 'OK %s\n' "$*"; log "OK $*"; }
warn() { printf 'WARN %s %s\n' "$1" "$2"; log "WARN $1 $2"; }
fail() { printf 'ERROR %s %s\n' "$1" "$2"; log "ERROR $1 $2"; exit 1; }
run_logged() { log "$*"; "$@" >>"$LOG" 2>&1; }

while [ "$#" -gt 0 ]; do
  case "$1" in
    --app-version) [ "$#" -ge 2 ] || fail E_NETWORK "--app-version needs a value"; APP_VERSION="$2"; shift 2 ;;
    --wheel) [ "$#" -ge 2 ] || fail E_NETWORK "--wheel needs a value"; WHEEL="$2"; shift 2 ;;
    --with-omnivoice) WITH_OMNIVOICE=1; shift ;;
    --check) CHECK_ONLY=1; shift ;;
    *) fail E_NETWORK "unknown argument: $1" ;;
  esac
done

if [ "$CHECK_ONLY" -eq 1 ]; then
  exec 3>/dev/null
else
  mkdir -p "$ROOT" 2>/dev/null || { echo 'ERROR E_DISK cannot create install root'; exit 1; }
  touch "$LOG" 2>/dev/null || { echo 'ERROR E_DISK cannot write install.log'; exit 1; }
  exec 3>>"$LOG"
fi

step 1 "checks"
[ "$(uname -s 2>/dev/null)" = Darwin ] && [ "$(uname -m 2>/dev/null)" = arm64 ] || fail E_ARCH "scriptCast requires arm64 macOS"
command -v ffmpeg >/dev/null 2>&1 || {
  if command -v brew >/dev/null 2>&1; then
    warn W_FFMPEG "ffmpeg missing; installing with Homebrew"
    [ "$CHECK_ONLY" -eq 1 ] || run_logged brew install ffmpeg || fail E_DISK "Homebrew could not install ffmpeg"
  else
    warn W_FFMPEG "ffmpeg missing; install Homebrew with https://brew.sh then run: brew install ffmpeg"
  fi
}
if command -v uv >/dev/null 2>&1; then UV=uv; else UV=""; warn W_UV "uv missing; falling back to python -m venv"; fi
ok "checks passed"

step 2 "python"
PYTHON=""
if [ -n "$UV" ]; then
  if [ "$CHECK_ONLY" -eq 1 ]; then PYTHON="$(uv python find '>=3.11' 2>/dev/null || true)"; else PYTHON="$(uv python find '>=3.11' 2>>"$LOG" || true)"; fi
fi
if [ -z "$PYTHON" ]; then
  for candidate in python3.13 python3.12 python3.11 \
    /opt/homebrew/bin/python3.13 /opt/homebrew/bin/python3.12 /opt/homebrew/bin/python3.11 \
    /Library/Frameworks/Python.framework/Versions/*/bin/python3 /usr/bin/python3; do
    if [ -x "$candidate" ] || command -v "$candidate" >/dev/null 2>&1; then
      if "$candidate" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3,11) else 1)' 2>/dev/null; then PYTHON="$candidate"; break; fi
    fi
  done
fi
[ -n "$PYTHON" ] || fail E_PYTHON "no python3 >= 3.11 found"
ok "$PYTHON ($($PYTHON -c 'import platform; print(platform.python_version())'))"

step 3 "venv"
[ "$CHECK_ONLY" -eq 1 ] && { ok "check mode; no environment changes"; exit 0; }
if [ -n "$UV" ]; then run_logged uv venv "$ROOT/venv" --python "$PYTHON" || fail E_VENV "could not create venv"; else run_logged "$PYTHON" -m venv "$ROOT/venv" || fail E_VENV "could not create venv"; fi
ok "created $ROOT/venv"

step 4 "package"
PIP="$ROOT/venv/bin/python"
SOURCE="$WHEEL"
if [ -z "$SOURCE" ]; then
  [ -n "$APP_VERSION" ] || fail E_NETWORK "--app-version is required unless --wheel is supplied"
  API="https://api.github.com/repos/th30d0re/scriptCast/releases/tags/ui-v$APP_VERSION"
  SOURCE="$($PIP -c 'import json,sys,urllib.request
try:
 d=json.load(urllib.request.urlopen(sys.argv[1]))
 print(next(a["browser_download_url"] for a in d["assets"] if a["name"].endswith(".whl")))
except Exception as e: print(e,file=sys.stderr); raise SystemExit(1)' "$API" 2>>"$LOG")" || fail E_NETWORK "could not find release wheel"
fi
if [ -n "$UV" ]; then run_logged uv pip install --python "$PIP" "$SOURCE" mlx-whisper || fail E_PIP "package installation failed"; else run_logged "$PIP" -m pip install "$SOURCE" mlx-whisper || fail E_PIP "package installation failed"; fi
if [ "$WITH_OMNIVOICE" -eq 1 ]; then run_logged "$PYTHON" -m venv "$ROOT/venv-omnivoice" || fail E_VENV "could not create OmniVoice venv"; run_logged "$ROOT/venv-omnivoice/bin/python" -m pip install torch==2.8.0 torchaudio==2.8.0 omnivoice || fail E_PIP "OmniVoice installation failed"; fi
ok "package installed"

step 5 "ffmpeg"
command -v ffmpeg >/dev/null 2>&1 && ok "ffmpeg available" || warn W_FFMPEG "ffmpeg is not available"

step 6 "verify"
run_logged "$ROOT/venv/bin/scriptcast" --help || fail E_PIP "scriptcast --help failed"
run_logged "$ROOT/venv/bin/python" -c 'import ruamel.yaml, mlx_whisper' || fail E_PIP "verification imports failed"
VERSION="$("$ROOT/venv/bin/python" -c 'from importlib.metadata import version; print(version("scriptcast"))' 2>>"$LOG")" || fail E_PIP "could not read installed version"
if [ -z "$APP_VERSION" ]; then APP_VERSION=""; fi
"$ROOT/venv/bin/python" - "$ROOT/install.json" "$VERSION" "$APP_VERSION" "$PYTHON" <<'PY' >>"$LOG" 2>&1
import datetime, json, sys
json.dump({"version": sys.argv[2], "app_version": sys.argv[3], "python": sys.argv[4], "installed_at": datetime.datetime.now(datetime.timezone.utc).isoformat()}, open(sys.argv[1], "w"), indent=2)
PY
ok "verification passed"
printf 'DONE %s\n' "$ROOT"
log "DONE $ROOT"
