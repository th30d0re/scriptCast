"""Resolve authored shot lists to manifest timing with optional manuscript checks."""
from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, field
from pathlib import Path

SCHEMA_VERSION = 1

_HEADING = re.compile(r"^##\s+(?P<id>[A-Za-z0-9\-]+)\s+[—-]\s+(?P<title>.+?)\s*$")
_BULLET = re.compile(r"^-\s+\*\*(?P<label>[^*]+?):\*\*\s*(?P<value>.*)$")
_ANCHOR = re.compile(r"`?(?P<speaker>[A-Za-z][A-Za-z .'-]*?)\s*\((?P<ts>\d{1,2}:\d{2})\)`?")
_THROUGH = re.compile(r"(?P<ts>\d{1,2}:\d{2})")
_TAG = re.compile(r"\[(book|data|design|source)\]")
_CITE = re.compile(r"`?((?:Paper|chapters|figures)/[\w./-]+?\.(?:tex|csv|json|ipynb))((?::\d+)(?:,\s*\d+)*)?`?")
# The provenance key each list declares: "[book] — stated in
# Paper/The_Original_Power.tex, cited by line". So a bare `:149` is a complete
# citation against that file, and a note that has already named a path can go
# on with `:164` for a second line in the same one.
BOOK = "Paper/The_Original_Power.tex"
_BARE_LINE = re.compile(r"`:(\d+)`")
# The durable form: a phrase quoted from the manuscript, which survives editing
# the way a line number does not. The spec resolves it to a current line.
_QUOTE = re.compile(r'`"([^"]{12,})"`')
_LABEL = re.compile(r"`((?:ch|sec|subsec|eq|fig|tab|app):[\w:.\-]+)`")
_SPEAKER_ONLY = re.compile(r"`(?P<speaker>[A-Za-z][A-Za-z .'-]*?)`\s*(?P<cue>.*)")

# Shots that depict a real event or person want licensed material, not a model.
_ARCHIVAL = re.compile(
    r"\b(footage|photograph|photo|newsreel|archival|recording|film of|"
    r"press conference|interview)\b", re.I)


@dataclass
class Shot:
    id: str
    title: str
    fields: dict[str, str] = field(default_factory=dict)
    body: list[str] = field(default_factory=list)

    @property
    def text(self) -> str:
        return " ".join(list(self.fields.values()) + self.body)


def parse(path: Path) -> list[Shot]:
    shots: list[Shot] = []
    current: Shot | None = None
    label: str | None = None
    for raw in path.read_text().splitlines():
        heading = _HEADING.match(raw)
        if heading:
            current = Shot(heading.group("id"), heading.group("title"))
            shots.append(current)
            label = None
            continue
        if current is None:
            continue
        bullet = _BULLET.match(raw)
        if bullet:
            label = bullet.group("label").strip()
            current.fields[label] = bullet.group("value").strip()
            continue
        stripped = raw.strip()
        if not stripped or stripped == "---":
            label = None
            continue
        if raw.startswith(("  ", "\t")) and label:
            current.fields[label] = (current.fields[label] + " " + stripped).strip()
        elif not raw.startswith("#"):
            current.body.append(stripped)
            label = None
    return shots


def _manifest_lookup(manifest_path: Path, script_path: Path | None = None
                     ) -> tuple[dict, dict]:
    """Build (display name -> speaker id) and ((speaker, mm:ss) -> turn).

    The shot lists anchor to the script's timestamps, and `scriptcast-retime`
    rewrites those after a render, so a manifest's own `source_timestamp`
    drifts out of agreement with the list it was written against. Turn *order*
    does not drift. Given the script, anchors resolve through its turn index
    into the manifest; without it, fall back to matching the manifest's own
    timestamps, which only works before a retime.
    """
    manifest = json.loads(manifest_path.read_text())
    by_name = {s["display_name"].lower(): s["speaker_id"] for s in manifest["speakers"]}
    by_index = {turn["turn_index"]: turn for turn in manifest["turns"]}
    turns: dict[tuple[str, str], dict] = {}

    if script_path and script_path.exists():
        from scriptcast.parser import parse_transcript
        for turn in parse_transcript(script_path):
            stamp = getattr(turn, "timestamp_mmss", None)
            target = by_index.get(turn.turn_index)
            if stamp and target:
                turns.setdefault((turn.speaker_id, stamp), target)
            by_name.setdefault(turn.display_name.lower(), turn.speaker_id)
    if not turns:
        for turn in manifest["turns"]:
            turns.setdefault((turn["speaker_id"], turn["source_timestamp"]), turn)
    return by_name, turns


def _seconds(ts: str) -> int:
    minutes, secs = ts.split(":")
    return int(minutes) * 60 + int(secs)


def spec_for(shot: Shot, names: dict | None = None, turns: dict | None = None,
             *, book: Path | None = None, repo: Path | None = None) -> dict:
    tags = sorted(set(_TAG.findall(shot.text)))
    citations = []
    labels = []
    quotes = []
    if book is not None or repo is not None:
        for match in _CITE.finditer(shot.text):
            # A note can cite several lines of one file: "...tex:2049, 2054".
            lines = [int(n) for n in re.findall(r"\d+", match.group(2) or "")]
            for line in lines or [None]:
                entry = {"path": match.group(1)}
                if line is not None:
                    entry["line"] = line
                if entry not in citations:
                    citations.append(entry)

        # Bare line numbers attach to the path the note already named, or to the
        # manuscript when it named none.
        default = next((c["path"] for c in citations if c["path"].endswith(".tex")), str(book) if book is not None else BOOK)
        for match in _BARE_LINE.finditer(shot.text):
            entry = {"path": default, "line": int(match.group(1))}
            if entry not in citations:
                citations.append(entry)

        quotes = [m.group(1) for m in _QUOTE.finditer(shot.text)]

        labels = []
        for match in _LABEL.finditer(shot.text):
            if match.group(1) not in labels:
                labels.append(match.group(1))

    for match in re.finditer(r"https?://[^\s<>`\"]+", shot.text):
        url = match.group().rstrip(".,;:!?'”")
        # Drop Markdown/prose closing parentheses, preserving URL parentheses.
        while url.endswith(")") and url.count(")") > url.count("("):
            url = url[:-1]
        entry = {"url": url}
        if entry not in citations:
            citations.append(entry)
    for match in re.finditer(r"\bCards?\s+(\d+(?:(?:\s*,\s*(?:and\s+)?|\s+and\s+)\d+)*)\b",
                             shot.text, re.I):
        for number in re.findall(r"\d+", match.group(1)):
            entry = {"card": f"Card {int(number)}"}
            if entry not in citations:
                citations.append(entry)

    anchor: dict = {}
    raw_anchor = shot.fields.get("Anchor", "")
    found = _ANCHOR.search(raw_anchor)
    if found:
        speaker = found.group("speaker").strip()
        anchor = {"speaker": speaker, "timestamp": found.group("ts")}
        if names is not None and turns is not None:
            speaker_id = names.get(speaker.lower())
            turn = turns.get((speaker_id, found.group("ts"))) if speaker_id else None
            if turn:
                anchor.update({"speaker_id": speaker_id, "turn_index": turn["turn_index"],
                               "start_ms": turn["start_ms"], "end_ms": turn["end_ms"]})
            elif speaker_id:
                anchor["speaker_id"] = speaker_id
    elif raw_anchor:
        # Some anchors name a speaker and then describe the moment in prose
        # instead of giving a timestamp. Keep the cue so a person can resolve
        # it; nothing downstream can place it on a timeline.
        prose = _SPEAKER_ONLY.match(raw_anchor)
        if prose:
            anchor = {"speaker": prose.group("speaker").strip(),
                      "cue": prose.group("cue").strip(), "unresolved": True}

    hold: dict = {}
    raw_hold = shot.fields.get("Hold", "")
    through = _THROUGH.search(raw_hold)
    if through:
        hold["until"] = through.group("ts")
        if anchor.get("timestamp"):
            hold["script_ms"] = max(
                0, (_seconds(through.group("ts")) - _seconds(anchor["timestamp"])) * 1000)

    # Everything that describes what is on screen, in the order it was written.
    described = {k: v for k, v in shot.fields.items()
                 if k not in ("Anchor", "Hold", "Note") and v}
    seed = "; ".join(f"{k}: {v}" for k, v in described.items())

    return {
        "id": shot.id,
        "title": shot.title,
        "anchor": anchor,
        "hold": hold,
        "type": shot.fields.get("Type", ""),
        "described": described,
        "note": shot.fields.get("Note", ""),
        "provenance": {"tags": tags, "citations": citations,
                        "labels": labels, "quotes": quotes},
        "suggested_track": "archival" if _ARCHIVAL.search(shot.text) else "vector",
        # Assembled from `described`, not authored. A prompt-writing pass with
        # the chapter's own text should replace it before anything is rendered.
        "prompt_seed": f"{shot.title}. {seed}".strip(),
    }


def _normalize(text: str) -> str:
    return re.sub(r"\s+", " ", re.sub(r"[^a-z0-9 ]", " ", text.lower())).strip()


def resolve_quotes(specs: list[dict], repo: Path | None = None,
                   book: Path | None = None) -> None:
    """Turn each quoted phrase into the line it currently sits on.

    The markdown holds the quote because a quote does not drift; the spec holds
    the line because that is what an editor wants to jump to. Regenerating the
    spec is what keeps the two in agreement.
    """
    if repo is None and book is None:
        return
    manuscript = (repo or Path.cwd()) / (book if book is not None else BOOK)
    if not manuscript.exists():
        return
    flat = [_normalize(line) for line in manuscript.read_text(errors="replace").splitlines()]
    for spec in specs:
        resolved = []
        for quote in spec["provenance"].get("quotes", []):
            needle = _normalize(quote)
            line = next((n for n, text in enumerate(flat, start=1) if needle in text), None)
            resolved.append({"quote": quote, "path": str(book) if book is not None else BOOK, "line": line})
        if resolved:
            spec["provenance"]["resolved_quotes"] = resolved


def validate(specs: list[dict], repo: Path | None = None,
             book: Path | None = None) -> list[tuple[str, str, str]]:
    """Returns (level, shot id, message). `error` means the list disagrees with
    the repository; `warn` means a human should look."""
    problems: list[tuple[str, str, str]] = []
    for spec in specs:
        sid = spec["id"]
        tags = spec["provenance"]["tags"]
        cites = spec["provenance"]["citations"]

        if not spec["anchor"]:
            problems.append(("error", sid, "no anchor, so it cannot be placed on a timeline"))
        elif spec["anchor"].get("unresolved"):
            problems.append(("warn", sid, "anchor is prose, not a timestamp: "
                                          f"{spec['anchor'].get('cue', '')[:60]}"))
        elif "start_ms" not in spec["anchor"] and spec["anchor"].get("speaker_id"):
            problems.append(("warn", sid,
                             f"anchor {spec['anchor']['speaker']} "
                             f"({spec['anchor']['timestamp']}) is not a turn in the manifest"))

        if not tags:
            problems.append(("warn", sid, "no provenance tag"))

        if "source" in tags and not any("url" in c or "card" in c for c in cites):
            problems.append(("error", sid, "tagged [source] with no URL or card reference"))
        if repo is None and book is None:
            continue
        cites = [c for c in cites if "path" in c]
        labels = spec["provenance"].get("labels", [])
        quotes = spec["provenance"].get("resolved_quotes", [])
        for entry in quotes:
            if entry["line"] is None:
                problems.append(("error", sid,
                                 f"quoted phrase is not in the manuscript: {entry['quote'][:50]!r}"))
        if ("book" in tags and not any(c["path"].endswith(".tex") for c in cites)
                and not labels and not quotes):
            problems.append(("error", sid, "tagged [book] with no line or label"))
        for label in labels:
            manuscript = (repo or Path.cwd()) / (book if book is not None else BOOK)
            if manuscript.exists() and f"\\label{{{label}}}" not in manuscript.read_text():
                problems.append(("error", sid, f"cited label does not exist: {label}"))
        if "data" in tags and not any("/data/" in c["path"] for c in cites):
            problems.append(("error", sid, "tagged [data] with no file under Paper/data/"))

        for cite in cites:
            target = (repo or Path.cwd()) / cite["path"]
            if not target.exists():
                problems.append(("error", sid, f"cited file is missing: {cite['path']}"))
                continue
            if "line" in cite and target.suffix == ".tex":
                total = sum(1 for _ in target.open(errors="replace"))
                if cite["line"] > total:
                    problems.append(("error", sid,
                                     f"{cite['path']}:{cite['line']} is past the end "
                                     f"of the file ({total} lines)"))
    return problems


def build(shotlist: Path, manifest: Path | None, script: Path | None = None,
          *, book: Path | None = None, repo: Path | None = None) -> dict:
    names, turns = (None, None)
    if manifest:
        names, turns = _manifest_lookup(manifest, script)
    shots = parse(shotlist)
    specs = [spec_for(shot, names, turns, book=book, repo=repo) for shot in shots]
    return {
        "schema_version": SCHEMA_VERSION,
        "source": shotlist.name,
        "manifest": manifest.name if manifest else None,
        "script": script.name if script else None,
        "shot_count": len(specs),
        "shots": specs,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("shotlist", type=Path)
    ap.add_argument("--manifest", type=Path, default=None)
    ap.add_argument("--script", type=Path, default=None,
                    help="the episode script, so anchors resolve through turn "
                         "order rather than timestamps a retime has moved")
    ap.add_argument("--out", type=Path, default=None)
    ap.add_argument("--strict", action="store_true", help="exit non-zero on any error")
    ap.add_argument("--book", type=Path, help="enable manuscript checks using this file")
    ap.add_argument("--repo", type=Path,
                    help="enable manuscript checks relative to this root; default book: " + BOOK)
    args = ap.parse_args()

    document = build(args.shotlist, args.manifest, args.script, book=args.book, repo=args.repo)
    resolve_quotes(document["shots"], args.repo, args.book)
    problems = validate(document["shots"], args.repo, args.book)
    document["validation"] = [{"level": lvl, "shot": sid, "message": msg}
                              for lvl, sid, msg in problems]

    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(document, indent=2) + "\n")

    tracks = {}
    resolved = 0
    for spec in document["shots"]:
        tracks[spec["suggested_track"]] = tracks.get(spec["suggested_track"], 0) + 1
        resolved += "start_ms" in spec["anchor"]
    print(f"{args.shotlist.name}: {document['shot_count']} shots, "
          f"{resolved} anchored to real time, "
          + ", ".join(f"{n} {k}" for k, n in sorted(tracks.items())))

    errors = [p for p in problems if p[0] == "error"]
    warns = [p for p in problems if p[0] == "warn"]
    for level, sid, msg in errors + warns:
        print(f"  {level:5s} {sid:8s} {msg}")
    if args.out:
        print(f"wrote {args.out}")
    return 1 if (args.strict and errors) else 0


if __name__ == "__main__":
    raise SystemExit(main())
