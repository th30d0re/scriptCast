"""Second-opinion ASR for failed verification turns (Phonon-2 beside Whisper).

Reads Whisper's hearing from stress.json (does not re-run the verifier) and
runs Phonon-2 on each turn's processed WAVs. Aligns expected / Whisper / Phonon
token streams and writes advisory stress_ensemble.json. Never mutates stress.json.
"""
from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Callable

from scriptcast.asr.engines import phonon_words
from scriptcast.asr.merge import fill_gaps
from scriptcast.markup import tokenize_markup
from scriptcast.parser import parse_transcript
from scriptcast.project import current
from scriptcast.tools.verify_render import _normalize, _score
from scriptcast.tools.word_timings import turn_selection


def atomic_write(path: Path, data: dict) -> None:
    """Publish only a complete JSON file; keep the previous output on failure."""
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temporary = Path(handle.name)
            json.dump(data, handle, ensure_ascii=False, indent=2, allow_nan=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def phonon_segment_text(
    audio_path: Path,
    *,
    phonon_words_fn: Callable | None = None,
) -> str:
    """Phonon-2 primary plus windowed gap-fill; return joined word text."""
    primary, passes, duration = (phonon_words_fn or phonon_words)(audio_path)
    merged, _filled, _unfilled = fill_gaps(primary, passes, duration)
    parts = []
    for word in merged:
        text = str(word.get("word", word.get("text", ""))).strip()
        if text:
            parts.append(text)
    return " ".join(parts)


def _expected_from_script(script_turn) -> str:
    chunks = [
        c.text.strip()
        for c in script_turn.markup_chunks
        if c.kind == "speech" and c.text and c.text.strip()
    ]
    return " ".join(chunks).strip()


def _engine_tokens_for_span(opcodes, engine: list[str], start: int, end: int) -> list[str]:
    """Engine tokens aligned to expected[start:end] (inserts only when start==end)."""
    out: list[str] = []
    for tag, i1, i2, j1, j2 in opcodes:
        if tag == "insert":
            if start == end == i1:
                out.extend(engine[j1:j2])
            continue
        lo, hi = max(i1, start), min(i2, end)
        if lo >= hi:
            continue
        if tag == "equal":
            out.extend(engine[j1 + (lo - i1) : j1 + (hi - i1)])
        elif i1 >= start and i2 <= end:
            out.extend(engine[j1:j2])
        else:
            out.extend(engine[j1:j2])
    return out


def span_verdict(
    expected: list[str], whisper: list[str], phonon: list[str]
) -> str:
    """Classify one disagreeing span."""
    if not whisper or not phonon:
        if not whisper and not phonon and not expected:
            return "uncertain"
        return "coverage_gap"
    wh_s = " ".join(whisper)
    ph_s = " ".join(phonon)
    exp_s = " ".join(expected)
    if wh_s == ph_s and wh_s != exp_s:
        return "likely_audio"
    if wh_s != ph_s and (wh_s == exp_s or ph_s == exp_s):
        return "likely_recognition"
    if wh_s != ph_s and wh_s != exp_s and ph_s != exp_s:
        return "uncertain"
    return "uncertain"


def summary_verdict(spans: list[dict]) -> str:
    # No differing spans: both engines now hear exactly the script (for example
    # after a re-render), so the original failure no longer reproduces.
    if not spans:
        return "matches_script"
    if any(span["verdict"] == "likely_audio" for span in spans):
        return "likely_audio"
    if spans and all(span["verdict"] == "likely_recognition" for span in spans):
        return "likely_recognition"
    return "mixed"


def align_spans(expected: str, whisper: str, phonon: str) -> list[dict]:
    """Spans where at least one engine differs from expected (token indices)."""
    exp = _normalize(expected)
    wh = _normalize(whisper)
    ph = _normalize(phonon)
    wh_ops = list(difflib.SequenceMatcher(None, exp, wh).get_opcodes())
    ph_ops = list(difflib.SequenceMatcher(None, exp, ph).get_opcodes())

    dirty = [False] * len(exp)
    inserts: dict[int, bool] = {}
    for ops in (wh_ops, ph_ops):
        for tag, i1, i2, _j1, _j2 in ops:
            if tag == "equal":
                continue
            if tag == "insert":
                inserts[i1] = True
            else:
                for index in range(i1, i2):
                    dirty[index] = True

    spans: list[dict] = []

    def emit(start: int, end: int) -> None:
        expected_toks = exp[start:end]
        whisper_toks = _engine_tokens_for_span(wh_ops, wh, start, end)
        phonon_toks = _engine_tokens_for_span(ph_ops, ph, start, end)
        if (
            whisper_toks == expected_toks
            and phonon_toks == expected_toks
            and expected_toks
        ):
            return
        if not expected_toks and not whisper_toks and not phonon_toks:
            return
        spans.append(
            {
                "start_token": start,
                "end_token": end,
                "expected": " ".join(expected_toks),
                "whisper": " ".join(whisper_toks),
                "phonon": " ".join(phonon_toks),
                "verdict": span_verdict(expected_toks, whisper_toks, phonon_toks),
            }
        )

    index = 0
    while index < len(exp):
        if inserts.pop(index, False):
            emit(index, index)
        if not dirty[index]:
            index += 1
            continue
        end = index + 1
        while end < len(exp) and dirty[end]:
            end += 1
        emit(index, end)
        index = end
    if inserts.pop(len(exp), False):
        emit(len(exp), len(exp))
    return spans


def _load_stress(path: Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list):
        raise ValueError("stress.json must be a JSON array")
    return data


def failed_turn_ids(stress: list[dict]) -> set[str]:
    return {
        str(entry["turn_id"])
        for entry in stress
        if entry.get("failed") and entry.get("turn_id")
    }


def build_ensemble(
    episode_dir: Path,
    manifest: dict,
    stress: list[dict],
    script_turns,
    turns: set[int] | None = None,
    *,
    phonon_fn: Callable[[Path], str] | None = None,
) -> dict[str, dict]:
    """Run Phonon-2 on selected failed turns; return per-turn_id payloads."""
    by_id = {turn.turn_id: turn for turn in script_turns}
    stress_by_id = {
        str(entry["turn_id"]): entry for entry in stress if entry.get("turn_id")
    }
    manifest_by_id = {turn["turn_id"]: turn for turn in manifest["turns"]}

    if turns is not None:
        wanted_ids = {
            turn["turn_id"]
            for turn in manifest["turns"]
            if turn["turn_index"] in turns
        }
        missing = turns - {turn["turn_index"] for turn in manifest["turns"]}
        if missing:
            raise ValueError(f"Turns absent from manifest: {sorted(missing)}")
    else:
        wanted_ids = failed_turn_ids(stress)
        if not wanted_ids:
            return {}

    result: dict[str, dict] = {}
    for turn_id in sorted(wanted_ids):
        entry = stress_by_id.get(turn_id)
        manifest_turn = manifest_by_id.get(turn_id)
        if manifest_turn is None:
            raise ValueError(f"Turn id {turn_id} absent from manifest")
        script_turn = by_id.get(turn_id)
        if script_turn is None:
            raise ValueError(
                f"Turn {manifest_turn['turn_index']} no longer matches the script; "
                "re-render first"
            )

        expected = ""
        heard = ""
        if entry is not None:
            expected = str(entry.get("expected") or "")
            heard = str(entry.get("heard") or "")
        if not expected.strip():
            expected = _expected_from_script(script_turn)
        if not expected.strip():
            raise ValueError(f"No expected text for turn {turn_id}")

        start = manifest_turn["start_ms"]
        duration = (manifest_turn["end_ms"] - start) / 1000
        if duration < 0 or not manifest_turn["segments"]:
            raise ValueError(
                f"Invalid audio window for turn {manifest_turn['turn_index']}"
            )

        phonon_parts: list[str] = []
        for segment in sorted(
            manifest_turn["segments"], key=lambda item: item["start_ms"]
        ):
            audio = (episode_dir / segment["segment_wav"]).resolve()
            if not audio.is_relative_to(episode_dir.resolve()):
                raise ValueError(f"Segment outside episode: {audio}")
            if not audio.is_file():
                raise ValueError(f"Missing WAV: {audio}")
            offset = (segment["start_ms"] - start) / 1000
            if offset < 0:
                raise ValueError("Segment starts before its turn")
            if phonon_fn is not None:
                text = phonon_fn(audio)
            else:
                text = phonon_segment_text(audio)
            if text.strip():
                phonon_parts.append(text.strip())
        phonon = " ".join(phonon_parts)

        score_whisper, _ = _score(expected, heard)
        score_phonon, _ = _score(expected, phonon)
        spans = align_spans(expected, heard, phonon)
        payload = {
            "score_whisper": round(score_whisper, 3),
            "score_phonon": round(score_phonon, 3),
            "spans": spans,
            "summary": summary_verdict(spans),
        }
        result[turn_id] = payload
        print(
            f"Turn {manifest_turn['turn_index']} ({turn_id}): "
            f"whisper {payload['score_whisper']:.3f} "
            f"phonon {payload['score_phonon']:.3f} "
            f"{payload['summary']} ({len(spans)} spans)",
            flush=True,
        )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("episode_dir", type=Path)
    parser.add_argument("--script", type=Path)
    parser.add_argument("--turns", type=turn_selection)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args(argv)
    try:
        episode_dir = args.episode_dir.resolve()
        manifest_path = episode_dir / "episode_manifest.json"
        stress_path = episode_dir / "stress.json"
        manifest_bytes = manifest_path.read_bytes()
        stress_bytes = stress_path.read_bytes()
        manifest = json.loads(manifest_bytes)
        stress = _load_stress(stress_path)
        script = args.script
        if script is None:
            source = manifest.get("source_file")
            if not source:
                source = json.loads((episode_dir / "render_state.json").read_text())[
                    "source_file"
                ]
            script = current(episode_dir).resolve(source)
        output = args.out or episode_dir / "stress_ensemble.json"
        inputs = {
            manifest_path,
            stress_path,
            script.resolve(),
            episode_dir / "render_state.json",
        }
        inputs.update(
            (episode_dir / segment["segment_wav"]).resolve()
            for turn in manifest["turns"]
            for segment in turn["segments"]
        )
        if output.resolve() in inputs or output.suffix.lower() != ".json":
            raise ValueError(
                "Output must be a .json file and cannot overwrite an input"
            )
        if output.resolve() == stress_path.resolve():
            raise ValueError("Refusing to overwrite stress.json")

        script_turns = tokenize_markup(parse_transcript(script))
        data = {
            "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "stress_sha256": hashlib.sha256(stress_bytes).hexdigest(),
            "turns": build_ensemble(
                episode_dir,
                manifest,
                stress,
                script_turns,
                args.turns,
            ),
        }
        if manifest_path.read_bytes() != manifest_bytes:
            raise ValueError("Manifest changed during ensemble; rerun")
        if stress_path.read_bytes() != stress_bytes:
            raise ValueError("stress.json changed during ensemble; rerun")
        atomic_write(output, data)
        print(f"Wrote {len(data['turns'])} turn(s) to {output}")
        return 0
    except (OSError, ValueError, KeyError, TypeError, FileNotFoundError) as exc:
        print(f"scriptcast-verify-ensemble: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
