"""Persistent OmniVoice synthesis worker.

Runs inside its own virtualenv. OmniVoice wants torch 2.8, transformers 5.16 and
gradio; the render pipeline runs on MLX with torch 2.11, and installing one into
the other's environment breaks whichever loses. So this process owns the model
and the pipeline talks to it over stdin and stdout.

The model costs about a minute to load, which is why the worker is persistent
rather than a subprocess per turn.

Protocol, one JSON object per line in each direction:

    in   {"text": "...", "ref_audio": "path.wav", "ref_text": "...", "out": "path.wav"}
    out  {"ok": true, "out": "path.wav", "samples": 123456}
         {"ok": false, "error": "..."}

Started by voice_pipeline.engine.OmniVoiceEngine; not run by hand.
"""
from __future__ import annotations

import json
import sys


def main() -> int:
    import soundfile
    import torch
    from omnivoice import OmniVoice

    model_id = sys.argv[1] if len(sys.argv) > 1 else "k2-fsa/OmniVoice"
    device = sys.argv[2] if len(sys.argv) > 2 else "mps"
    model = OmniVoice.from_pretrained(model_id, device_map=device, dtype=torch.float16)
    # The pipeline waits for this line before sending work.
    print(json.dumps({"ready": True}), flush=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            audio = model.generate(
                text=request["text"],
                ref_audio=request["ref_audio"],
                ref_text=request["ref_text"],
                speed=request.get("speed"),
            )
            samples = audio[0]
            soundfile.write(request["out"], samples, 24000)
            print(json.dumps({"ok": True, "out": request["out"],
                              "samples": int(len(samples))}), flush=True)
        except Exception as error:  # reported to the caller, worker stays up
            print(json.dumps({"ok": False, "error": f"{type(error).__name__}: {error}"}),
                  flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
