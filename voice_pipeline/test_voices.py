import tempfile
from pathlib import Path
from unittest import TestCase

from voice_pipeline.voices import load_voices


class LoadVoicesTests(TestCase):
    def test_missing_speed_defaults_to_one(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  narrator:",
                        "    kokoro_voice: af_heart",
                        "    lang_code: a",
                    ]
                ),
                encoding="utf-8",
            )

            voices = load_voices(voices_path)

        self.assertEqual(voices["narrator"].speaker_id, "narrator")
        self.assertEqual(voices["narrator"].kokoro_voice, "af_heart")
        self.assertEqual(voices["narrator"].lang_code, "a")
        self.assertEqual(voices["narrator"].speed, 1.0)
