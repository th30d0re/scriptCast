"""Score a candidate reference clip on how reliably it clones.

Choosing a voice by listening to one nice sample tells you how it sounds, not
whether it holds together across an episode. Chatterbox fails in specific
places: proper nouns, digit strings, comma-heavy lists, long sentences, and the
first word of an utterance. This renders a fixed passage set that stresses each
of those, transcribes the result, and scores it, so two candidates can be
compared on the same evidence.

    python3 tools/audition_voice.py candidates/*.wav --speaker aisha
    python3 tools/audition_voice.py voices/aisha_reference.wav --temperature 0.4,0.6,0.8

A reference wants to be clean mono speech at the engine's sample rate, a few
seconds long, with no music, no background, and no other speaker.
"""
from __future__ import annotations

import argparse
import asyncio
import json
import statistics
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import soundfile

from voice_pipeline.engine import ENGINE_REGISTRY
from voice_pipeline.models import VoiceConfig

# Drawn from the two rendered episodes, weighted toward the lines that have
# actually failed. Each one targets a mode seen in the wild.
PASSAGES = [
    # proper nouns
    "Third anchor. W. E. B. Du Bois, writing in nineteen fifteen, in an essay "
    "called The African Roots of War.",
    # a name plus a place the transcriber will not guess
    "Douglass was expected to secure it. Diplomatic pressure, with the implicit "
    "weight of naval force sitting behind it.",
    # comma-heavy list
    "Legal, because the code is literally written down. Statutes, ordinances, "
    "covenants, sentencing guidelines, zoning maps.",
    # spelled-out numbers
    "The calibration rests on one hundred forty six anchor cases, one for each "
    "historical event.",
    # long single sentence
    "It is saying that both an electrodynamic control architecture and a "
    "socioeconomic one are downstream implementations of the same universal "
    "dynamical systems equations, forced to optimize energy, labor, and "
    "suppression under structural constraints.",
    # short utterance, where the opening-babble mode shows up most
    "Now the second word. Virus.",
    # parallel clauses, which invite repetition
    "Change the statute and the reflex outlives it. Change the reflex and the "
    "statute reinstalls it.",
    # technical vocabulary
    "Psi sub m is the material wage, and it is the real component. Actual "
    "money, actual property access, actual infrastructure.",
]


async def _render(engine, text: str, config: VoiceConfig, path: Path, rate: int) -> None:
    audio = await engine.synthesize_chunk(text, config)
    soundfile.write(str(path), audio, rate)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("references", nargs="+", type=Path)
    ap.add_argument("--speaker", default="candidate")
    ap.add_argument("--engine", default="mlx_chatterbox")
    ap.add_argument("--model-id", default=None,
                    help="Engine model repo. Defaults to the pipeline's own.")
    ap.add_argument("--temperature", default="0.6",
                    help="Comma-separated values to sweep.")
    ap.add_argument("--cfg-weight", type=float, default=0.7)
    ap.add_argument("--model", choices=["tiny", "small", "medium"], default="small")
    ap.add_argument("--keep", type=Path, default=None,
                    help="Directory to keep the rendered auditions in.")
    args = ap.parse_args()

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from verify_render import _score

    import mlx_whisper
    from verify_render import _MODELS

    from voice_pipeline.__main__ import _DEFAULT_CHATTERBOX_MODEL
    model_id = args.model_id or _DEFAULT_CHATTERBOX_MODEL
    engine = ENGINE_REGISTRY[args.engine](model_id)
    temperatures = [float(x) for x in args.temperature.split(",")]
    out_root = args.keep or Path(tempfile.mkdtemp(prefix="audition_"))
    out_root.mkdir(parents=True, exist_ok=True)

    table = []
    for reference in args.references:
        if not reference.exists():
            print(f"missing: {reference}")
            continue
        for temperature in temperatures:
            config = VoiceConfig(
                speaker_id=args.speaker, engine=args.engine,
                reference_audio=str(reference),
                temperature=temperature, cfg_weight=args.cfg_weight,
            )
            scores = []
            worst = []
            for index, text in enumerate(PASSAGES):
                path = out_root / f"{reference.stem}_t{temperature}_{index:02d}.wav"
                asyncio.run(_render(engine, text, config, path, engine.sample_rate))
                heard = mlx_whisper.transcribe(
                    str(path), path_or_hf_repo=_MODELS[args.model], verbose=False
                )["text"].strip()
                score, run = _score(text, heard)
                scores.append(score)
                worst.append(run)
                if score < 0.85 or run > 2:
                    print(f"  {reference.name} t={temperature} passage {index}: "
                          f"{score:.3f} run {run}")
                    print(f"      want: {text[:90]}")
                    print(f"      got : {heard[:90]}")
            table.append(
                {
                    "reference": reference.name,
                    "temperature": temperature,
                    "mean": round(statistics.mean(scores), 4),
                    "min": round(min(scores), 4),
                    "clean": sum(1 for s, r in zip(scores, worst) if s >= 0.85 and r <= 2),
                    "of": len(PASSAGES),
                }
            )

    print(f"\n{'reference':<34} {'temp':>5} {'mean':>7} {'min':>7} {'clean':>7}")
    for row in sorted(table, key=lambda r: (-r["clean"], -r["mean"])):
        print(f"{row['reference']:<34} {row['temperature']:>5} {row['mean']:>7.4f} "
              f"{row['min']:>7.4f} {row['clean']:>3}/{row['of']}")
    if args.keep:
        print(f"\naudio kept in {out_root}")
        (out_root / "scores.json").write_text(json.dumps(table, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
