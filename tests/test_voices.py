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
        self.assertEqual(voices["narrator"].engine, "kokoro")

    def test_elevenlabs_voice_requires_voice_id(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  kareem:",
                        "    engine: elevenlabs",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "elevenlabs_voice_id"):
                load_voices(voices_path)

    def test_elevenlabs_voice_loads_correctly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  kareem:",
                        "    engine: elevenlabs",
                        "    elevenlabs_voice_id: VlUmeC1Uzj3NnwiVR9K9",
                        "    character_profile: Test profile",
                    ]
                ),
                encoding="utf-8",
            )

            voices = load_voices(voices_path)

        self.assertEqual(voices["kareem"].engine, "elevenlabs")
        self.assertEqual(voices["kareem"].elevenlabs_voice_id, "VlUmeC1Uzj3NnwiVR9K9")
        self.assertEqual(voices["kareem"].character_profile, "Test profile")

    def test_kokoro_voice_requires_kokoro_voice_and_lang_code(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  narrator:",
                        "    engine: kokoro",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "kokoro_voice"):
                load_voices(voices_path)

    def test_mlx_chatterbox_voice_loads_correctly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  kareem:",
                        "    engine: mlx_chatterbox",
                        "    reference_audio: voices/kareem_reference.wav",
                        "    character_profile: Black male perspective",
                        "    exaggeration: 0.3",
                    ]
                ),
                encoding="utf-8",
            )

            voices = load_voices(voices_path)

        self.assertEqual(voices["kareem"].engine, "mlx_chatterbox")
        self.assertEqual(voices["kareem"].reference_audio, "voices/kareem_reference.wav")
        self.assertEqual(voices["kareem"].character_profile, "Black male perspective")
        self.assertEqual(voices["kareem"].exaggeration, 0.3)

    def test_mlx_chatterbox_voice_requires_reference_audio(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  kareem:",
                        "    engine: mlx_chatterbox",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "reference_audio"):
                load_voices(voices_path)

    def test_unknown_engine_raises_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            voices_path = Path(tmp) / "voices.yaml"
            voices_path.write_text(
                "\n".join(
                    [
                        "speakers:",
                        "  narrator:",
                        "    engine: unknown_engine",
                        "    kokoro_voice: af_heart",
                        "    lang_code: a",
                    ]
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unknown engine"):
                load_voices(voices_path)
