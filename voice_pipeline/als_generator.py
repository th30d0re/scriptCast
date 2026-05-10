"""Generate minimal Ableton Live Set files for processed voice segments."""

from __future__ import annotations

import gzip
import xml.etree.ElementTree as ET
from pathlib import Path

from voice_pipeline.models import SegmentResult

_BPM = 120
_MS_PER_BEAT = 500.0
_LIVE_VERSION = "11.0.2"
_CREATOR = "Ableton Live 11.0.2"
_SPEAKERS = ["emmanuel_theodore", "ai_1", "ai_2"]
_SPEAKER_SET = set(_SPEAKERS)


def _ms_to_beats(ms: int) -> float:
    return ms / _MS_PER_BEAT


def _build_file_ref(wav_path: Path, project_root: Path) -> ET.Element:
    relative_path = wav_path.relative_to(project_root).as_posix()

    return ET.Element(
        "FileRef",
        {
            "HasRelativePath": "true",
            "RelativePath": relative_path,
            "Name": wav_path.name,
            "Type": "1",
        },
    )


def _build_audio_clip(
    clip_id: int,
    start_beats: float,
    length_beats: float,
    file_ref_elem: ET.Element,
) -> ET.Element:
    clip = ET.Element(
        "AudioClip",
        {"Id": str(clip_id), "Time": str(start_beats)},
    )

    ET.SubElement(clip, "Name", {"Value": file_ref_elem.attrib["Name"]})
    ET.SubElement(clip, "Time", {"Value": str(start_beats)})
    ET.SubElement(clip, "CurrentEnd", {"Value": str(start_beats + length_beats)})

    loop = ET.SubElement(clip, "Loop")
    ET.SubElement(loop, "LoopStart", {"Value": "0"})
    ET.SubElement(loop, "LoopEnd", {"Value": str(length_beats)})
    ET.SubElement(loop, "StartRelative", {"Value": "0"})
    ET.SubElement(loop, "LoopOn", {"Value": "false"})

    sample_ref = ET.SubElement(clip, "SampleRef")
    sample_ref.append(file_ref_elem)

    ET.SubElement(clip, "Disabled", {"Value": "false"})
    return clip


def _build_audio_track(
    track_id: int,
    speaker_id: str,
    clips: list[ET.Element],
) -> ET.Element:
    track = ET.Element("AudioTrack", {"Id": str(track_id)})

    name = ET.SubElement(track, "Name")
    ET.SubElement(name, "EffectiveName", {"Value": speaker_id})

    device_chain = ET.SubElement(track, "DeviceChain")
    main_sequence = ET.SubElement(device_chain, "MainSequence")
    clip_timeable = ET.SubElement(main_sequence, "ClipTimeable")
    arranged_clips = ET.SubElement(clip_timeable, "ArrangedClips")

    for clip in clips:
        arranged_clip = ET.SubElement(
            arranged_clips,
            "ArrangedClip",
            {"Time": clip.attrib["Time"]},
        )
        arranged_clip.append(clip)

    ET.SubElement(track, "AutomationEnvelopes")
    return track


def generate_als(segment_results: list[SegmentResult], output_path: Path) -> Path:
    project_root = output_path.parent

    for segment in segment_results:
        if segment.speaker_id not in _SPEAKER_SET:
            expected = ", ".join(_SPEAKERS)
            raise ValueError(
                f"Unknown speaker_id {segment.speaker_id!r}; expected one of: "
                f"{expected}"
            )

    positioned_segments: list[tuple[SegmentResult, int]] = []
    cursor_ms = 0
    for segment in segment_results:
        positioned_segments.append((segment, cursor_ms))
        cursor_ms += segment.duration_ms + segment.gap_after_ms

    clips_by_speaker: dict[str, list[ET.Element]] = {
        speaker_id: [] for speaker_id in _SPEAKERS
    }
    clip_id = 1
    for segment, start_ms in positioned_segments:
        start_beats = _ms_to_beats(start_ms)
        length_beats = _ms_to_beats(segment.duration_ms)
        file_ref = _build_file_ref(segment.wav_path, project_root)
        clip = _build_audio_clip(clip_id, start_beats, length_beats, file_ref)
        clips_by_speaker[segment.speaker_id].append(clip)
        clip_id += 1

    root = ET.Element(
        "Ableton",
        {
            "MajorVersion": _LIVE_VERSION,
            "MinorVersion": "0",
            "SchemaChangeCount": "0",
            "Creator": _CREATOR,
            "Revision": "",
        },
    )
    live_set = ET.SubElement(root, "LiveSet")

    tempo = ET.SubElement(live_set, "Tempo")
    ET.SubElement(tempo, "AutomationTarget")
    ET.SubElement(tempo, "ManualValue", {"Value": str(_BPM)})

    ET.SubElement(live_set, "MainSequence")

    tracks = ET.SubElement(live_set, "Tracks")
    for track_id, speaker_id in enumerate(_SPEAKERS, start=1):
        tracks.append(
            _build_audio_track(track_id, speaker_id, clips_by_speaker[speaker_id])
        )

    ET.SubElement(live_set, "MasterTrack")
    ET.SubElement(live_set, "Scenes")
    ET.SubElement(live_set, "Transport")

    ET.indent(root)
    xml_text = ET.tostring(root, encoding="unicode")
    xml_bytes = (
        '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_text
    ).encode("utf-8")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(output_path, "wb") as als_file:
        als_file.write(xml_bytes)

    return output_path
