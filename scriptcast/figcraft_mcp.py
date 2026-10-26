"""A small client for ``figcraft-cli mcp``.

The server speaks newline-delimited JSON-RPC over stdio. Headless mode never
needs the FigCraft window. This client builds nothing itself: it sends the
batch ScriptCast already prepared and exports the frames FigCraft returns.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path


class FigCraftError(RuntimeError):
    pass


class Session:
    def __init__(self, process: subprocess.Popen[str]) -> None:
        self._process = process
        self._next = 0

    def request(self, method: str, params: dict | None = None) -> dict:
        self._next += 1
        message = {"jsonrpc": "2.0", "id": self._next, "method": method}
        if params is not None:
            message["params"] = params
        self._send(message)
        while True:
            reply = self._read()
            if reply.get("id") != self._next:
                continue
            if "error" in reply:
                raise FigCraftError(str(reply["error"]))
            result = reply.get("result")
            if not isinstance(result, dict):
                raise FigCraftError(f"{method} returned no object")
            return result

    def notify(self, method: str) -> None:
        self._send({"jsonrpc": "2.0", "method": method})

    def close(self) -> None:
        if self._process.stdin is not None:
            self._process.stdin.close()
        try:
            self._process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self._process.kill()

    def _send(self, message: dict) -> None:
        if self._process.stdin is None:
            raise FigCraftError("FigCraft stdin is closed")
        self._process.stdin.write(json.dumps(message) + "\n")
        self._process.stdin.flush()

    def _read(self) -> dict:
        if self._process.stdout is None:
            raise FigCraftError("FigCraft stdout is closed")
        line = self._process.stdout.readline()
        if not line:
            raise FigCraftError("FigCraft closed the connection")
        try:
            reply = json.loads(line)
        except json.JSONDecodeError as exc:
            raise FigCraftError(f"FigCraft sent invalid JSON: {exc}") from exc
        if not isinstance(reply, dict):
            raise FigCraftError("FigCraft sent a non-object")
        return reply


def start(binary: str, read_root: Path, write_root: Path) -> Session:
    write_root.mkdir(parents=True, exist_ok=True)
    process = subprocess.Popen(
        [
            binary, "mcp",
            "--automation-read-root", str(read_root),
            "--automation-write-root", str(write_root),
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    session = Session(process)
    session.request("initialize", {
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": {"name": "scriptcast", "version": "0.2.0"},
    })
    session.notify("notifications/initialized")
    return session


def tool_text(result: dict) -> str:
    content = result.get("content")
    if isinstance(content, list):
        parts = [
            block.get("text") or ""
            for block in content
            if isinstance(block, dict) and block.get("type") == "text"
        ]
        return "\n".join(part for part in parts if part)
    return ""


def payload(result: dict) -> dict:
    """The JSON a tool put in its text content, or the result itself."""
    text = tool_text(result)
    if text:
        try:
            parsed = json.loads(text)
        except json.JSONDecodeError:
            parsed = None
        if isinstance(parsed, dict):
            return parsed
    return result


def call_tool(session: Session, name: str, arguments: dict) -> dict:
    result = session.request("tools/call", {"name": name, "arguments": arguments})
    if result.get("isError"):
        raise FigCraftError(tool_text(result) or f"{name} failed")
    return payload(result)


def _resolve(value: object, named: dict[str, int]) -> object:
    if isinstance(value, dict):
        if set(value) == {"$ref"}:
            name = value["$ref"]
            if name not in named:
                raise FigCraftError(f"no FigCraft id named {name}")
            return named[name]
        return {key: _resolve(item, named) for key, item in value.items()}
    if isinstance(value, list):
        return [_resolve(item, named) for item in value]
    return value


def export_beats(session: Session, commands: list[dict], exports: list[dict]) -> None:
    """Run each command on its own. This FigCraft build does not substitute ``$ref`` inside ``batch``."""
    named: dict[str, int] = {}
    for step in commands:
        result = call_tool(session, "run_command", {
            "command": step["command"],
            "params": _resolve(step.get("params") or {}, named),
        })
        if "as" in step:
            identifier = result.get("id")
            if not isinstance(identifier, int) or isinstance(identifier, bool):
                raise FigCraftError(f"{step['as']} did not return an id")
            named[step["as"]] = identifier
    for export in exports:
        identifier = named.get(export["frame"])
        if identifier is None:
            raise FigCraftError(f"FigCraft did not return a frame named {export['frame']}")
        call_tool(session, "export", {
            "ids": [identifier],
            "format": "png",
            "path": export["path"],
        })
