"""Generate Ableton Live Set files for processed voice segments."""

from __future__ import annotations

import copy
import gzip
import os
import shutil
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path
from zlib import crc32

from voice_pipeline.models import SegmentResult

_BPM = 120
_MS_PER_BEAT = 60000.0 / _BPM
_GENERATED_WARP_MARKER_ID_OFFSET = 1_000_000
_TEMPLATE_ENV_VAR = "VOICE_PIPELINE_ALS_TEMPLATE"
_PODCAST_TEMPLATE_RELATIVE_PATH = Path(
    "Contents/App-Resources/Core Library/Templates/Podcast & Radio.als"
)
_DEFAULT_SET_TEMPLATE_RELATIVE_PATH = Path(
    "Contents/App-Resources/Builtin/Templates/DefaultLiveSet.als"
)


def _ms_to_beats(ms: int) -> float:
    return ms / _MS_PER_BEAT


def _format_number(value: float | int) -> str:
    if isinstance(value, int) or value.is_integer():
        return str(int(value))
    return format(value, ".12g")


def _ableton_template_candidates() -> list[Path]:
    candidates: list[Path] = []
    env_template = os.environ.get(_TEMPLATE_ENV_VAR)
    if env_template:
        candidates.append(Path(env_template).expanduser())

    applications_dir = Path("/Applications")
    if applications_dir.exists():
        app_paths = sorted(
            applications_dir.glob("Ableton Live*.app"),
            key=lambda path: ("Live 12" not in path.name, path.name),
        )
        for app_path in app_paths:
            candidates.append(app_path / _PODCAST_TEMPLATE_RELATIVE_PATH)
            candidates.append(app_path / _DEFAULT_SET_TEMPLATE_RELATIVE_PATH)

    return candidates


def _force_template_tempo(root: ET.Element, bpm: float) -> None:
    """Pin the set's tempo to `bpm`, automation included.

    Beat positions here are computed from a fixed BPM. Ableton's stock
    "Podcast & Radio" template ships a constant automation envelope on the tempo
    parameter, and automation overrides the manual value, so the set actually
    runs at that automated tempo. When it disagrees with the BPM used to place
    clips, every clip's beat length is wrong by the tempo ratio and each one
    renders with a trailing empty stripe proportional to its own length.

    Setting Manual alone is not enough; the envelope has to agree.
    """
    target_ids: set[str] = set()
    for tempo in root.iter("Tempo"):
        manual = tempo.find("Manual")
        if manual is not None:
            manual.set("Value", _format_number(bpm))
        target = tempo.find("AutomationTarget")
        if target is not None and target.attrib.get("Id"):
            target_ids.add(target.attrib["Id"])

    if not target_ids:
        return

    for envelope in root.iter("AutomationEnvelope"):
        pointee = envelope.find(".//PointeeId")
        if pointee is None or pointee.attrib.get("Value") not in target_ids:
            continue
        for event in envelope.iter("FloatEvent"):
            event.set("Value", _format_number(bpm))


def _load_template_root() -> ET.Element:
    for template_path in _ableton_template_candidates():
        if not template_path.exists():
            continue
        with gzip.open(template_path, "rb") as template_file:
            root = ET.fromstring(template_file.read())
        _force_template_tempo(root, _BPM)
        return root

    checked = ", ".join(str(path) for path in _ableton_template_candidates())
    raise RuntimeError(
        "Could not find an Ableton Live .als template. Install Ableton Live 12 "
        f"or set {_TEMPLATE_ENV_VAR} to a valid template path. Checked: {checked}"
    )


def _relative_path_for_ableton(wav_path: Path, project_root: Path) -> str:
    try:
        return wav_path.resolve().relative_to(project_root.resolve()).as_posix()
    except ValueError:
        return os.path.relpath(wav_path, project_root).replace(os.sep, "/")


def _file_crc16(path: Path) -> int:
    checksum = 0
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            checksum = crc32(chunk, checksum)
    return checksum & 0xFFFF


def _build_file_ref(wav_path: Path, project_root: Path) -> ET.Element:
    file_ref = ET.Element("FileRef")
    ET.SubElement(file_ref, "RelativePathType", {"Value": "1"})
    ET.SubElement(
        file_ref,
        "RelativePath",
        {"Value": _relative_path_for_ableton(wav_path, project_root)},
    )
    ET.SubElement(file_ref, "Path", {"Value": str(wav_path.resolve())})
    ET.SubElement(file_ref, "Type", {"Value": "2"})
    ET.SubElement(file_ref, "LivePackName", {"Value": ""})
    ET.SubElement(file_ref, "LivePackId", {"Value": ""})
    ET.SubElement(file_ref, "OriginalFileSize", {"Value": str(wav_path.stat().st_size)})
    ET.SubElement(file_ref, "OriginalCrc", {"Value": str(_file_crc16(wav_path))})
    return file_ref


def _add_value(parent: ET.Element, tag: str, value: str | int | float | bool) -> ET.Element:
    if isinstance(value, bool):
        value = "true" if value else "false"
    elif isinstance(value, (int, float)):
        value = _format_number(float(value))

    return ET.SubElement(parent, tag, {"Value": str(value)})


def _build_audio_clip(
    clip_id: int,
    start_beats: float,
    length_beats: float,
    file_length_beats: float,
    segment: SegmentResult,
    project_root: Path,
) -> ET.Element:
    duration_seconds = segment.duration_ms / 1000.0
    sample_count = int(round(duration_seconds * segment.sample_rate))
    wav_path = segment.wav_path
    end_beats = start_beats + length_beats

    clip = ET.Element(
        "AudioClip",
        {"Id": str(clip_id), "Time": _format_number(start_beats)},
    )
    _add_value(clip, "LomId", "0")
    _add_value(clip, "LomIdView", "0")
    _add_value(clip, "CurrentStart", start_beats)
    _add_value(clip, "CurrentEnd", end_beats)

    loop = ET.SubElement(clip, "Loop")
    _add_value(loop, "LoopStart", 0)
    _add_value(loop, "LoopEnd", length_beats)
    _add_value(loop, "StartRelative", 0)
    _add_value(loop, "LoopOn", False)
    _add_value(loop, "OutMarker", length_beats)
    _add_value(loop, "HiddenLoopStart", 0)
    _add_value(loop, "HiddenLoopEnd", length_beats)

    _add_value(clip, "Name", wav_path.stem)
    _add_value(clip, "Annotation", "")
    _add_value(clip, "Color", "3")
    _add_value(clip, "LaunchMode", "0")
    _add_value(clip, "LaunchQuantisation", "0")

    time_signature = ET.SubElement(clip, "TimeSignature")
    signatures = ET.SubElement(time_signature, "TimeSignatures")
    remote_signature = ET.SubElement(signatures, "RemoteableTimeSignature", {"Id": "0"})
    _add_value(remote_signature, "Numerator", "4")
    _add_value(remote_signature, "Denominator", "4")
    _add_value(remote_signature, "Time", "0")

    envelopes = ET.SubElement(clip, "Envelopes")
    ET.SubElement(envelopes, "Envelopes")

    scroller = ET.SubElement(clip, "ScrollerTimePreserver")
    _add_value(scroller, "LeftTime", "0")
    _add_value(scroller, "RightTime", length_beats)

    time_selection = ET.SubElement(clip, "TimeSelection")
    _add_value(time_selection, "AnchorTime", "0")
    _add_value(time_selection, "OtherTime", "0")

    _add_value(clip, "Legato", False)
    _add_value(clip, "Ram", False)
    groove_settings = ET.SubElement(clip, "GrooveSettings")
    _add_value(groove_settings, "GrooveId", "-1")
    _add_value(clip, "Disabled", False)
    _add_value(clip, "VelocityAmount", "0")

    follow_action = ET.SubElement(clip, "FollowAction")
    _add_value(follow_action, "FollowTime", "4")
    _add_value(follow_action, "IsLinked", True)
    _add_value(follow_action, "LoopIterations", "1")
    _add_value(follow_action, "FollowActionA", "4")
    _add_value(follow_action, "FollowActionB", "0")
    _add_value(follow_action, "FollowChanceA", "100")
    _add_value(follow_action, "FollowChanceB", "0")
    _add_value(follow_action, "JumpIndexA", "1")
    _add_value(follow_action, "JumpIndexB", "1")
    _add_value(follow_action, "FollowActionEnabled", False)

    grid = ET.SubElement(clip, "Grid")
    _add_value(grid, "FixedNumerator", "1")
    _add_value(grid, "FixedDenominator", "16")
    _add_value(grid, "GridIntervalPixel", "20")
    _add_value(grid, "Ntoles", "2")
    _add_value(grid, "SnapToGrid", True)
    _add_value(grid, "Fixed", False)

    _add_value(clip, "FreezeStart", "0")
    _add_value(clip, "FreezeEnd", "0")
    _add_value(clip, "IsWarped", False)
    _add_value(clip, "TakeId", str(clip_id))
    _add_value(clip, "IsInKey", False)
    scale_information = ET.SubElement(clip, "ScaleInformation")
    _add_value(scale_information, "Root", "0")
    _add_value(scale_information, "Name", "0")

    sample_ref = ET.SubElement(clip, "SampleRef")
    sample_ref.append(_build_file_ref(wav_path, project_root))
    _add_value(sample_ref, "LastModDate", str(int(wav_path.stat().st_mtime)))
    ET.SubElement(sample_ref, "SourceContext")
    _add_value(sample_ref, "SampleUsageHint", "0")
    _add_value(sample_ref, "DefaultDuration", str(sample_count))
    _add_value(sample_ref, "DefaultSampleRate", str(segment.sample_rate))
    _add_value(sample_ref, "SamplesToAutoWarp", "0")

    onsets = ET.SubElement(clip, "Onsets")
    ET.SubElement(onsets, "UserOnsets")
    _add_value(onsets, "HasUserOnsets", False)

    _add_value(clip, "WarpMode", "0")
    _add_value(clip, "GranularityTones", "30")
    _add_value(clip, "GranularityTexture", "65")
    _add_value(clip, "FluctuationTexture", "25")
    _add_value(clip, "TransientResolution", "6")
    _add_value(clip, "TransientLoopMode", "2")
    _add_value(clip, "TransientEnvelope", "100")
    _add_value(clip, "ComplexProFormants", "100")
    _add_value(clip, "ComplexProEnvelope", "128")
    _add_value(clip, "Sync", False)
    _add_value(clip, "HiQ", False)
    _add_value(clip, "Fade", False)

    fades = ET.SubElement(clip, "Fades")
    _add_value(fades, "FadeInLength", "0")
    _add_value(fades, "FadeOutLength", "0")
    _add_value(fades, "ClipFadesAreInitialized", True)
    _add_value(fades, "CrossfadeInState", "0")
    _add_value(fades, "FadeInCurveSkew", "0")
    _add_value(fades, "FadeInCurveSlope", "0")
    _add_value(fades, "FadeOutCurveSkew", "0")
    _add_value(fades, "FadeOutCurveSlope", "0")
    _add_value(fades, "IsDefaultFadeIn", False)
    _add_value(fades, "IsDefaultFadeOut", False)

    _add_value(clip, "PitchCoarse", "0")
    _add_value(clip, "PitchFine", "0")
    _add_value(clip, "SampleVolume", "1")

    warp_markers = ET.SubElement(clip, "WarpMarkers")
    ET.SubElement(
        warp_markers,
        "WarpMarker",
        {
            "Id": str(_GENERATED_WARP_MARKER_ID_OFFSET + (clip_id * 2)),
            "SecTime": "0",
            "BeatTime": "0",
        },
    )
    ET.SubElement(
        warp_markers,
        "WarpMarker",
        {
            "Id": str(_GENERATED_WARP_MARKER_ID_OFFSET + (clip_id * 2) + 1),
            "SecTime": _format_number(duration_seconds),
            "BeatTime": _format_number(file_length_beats),
        },
    )
    ET.SubElement(clip, "SavedWarpMarkersForStretched")
    _add_value(clip, "MarkersGenerated", True)
    _add_value(clip, "IsSongTempoLeader", False)
    _add_value(clip, "AutoWarpPending", False)
    _add_value(clip, "WasMuted", False)

    return clip


def _set_track_name(track: ET.Element, name_value: str) -> None:
    name = track.find("./Name")
    if name is None:
        name = ET.SubElement(track, "Name")

    for tag in ("EffectiveName", "UserName"):
        child = name.find(f"./{tag}")
        if child is None:
            child = ET.SubElement(name, tag)
        child.attrib["Value"] = name_value


def _arrangement_events(track: ET.Element) -> ET.Element:
    events = track.find("./DeviceChain/MainSequencer/Sample/ArrangerAutomation/Events")
    if events is None:
        raise RuntimeError(
            "Ableton template audio track is missing "
            "DeviceChain/MainSequencer/Sample/ArrangerAutomation/Events"
        )
    return events


def _speaker_tracks(root: ET.Element, required_speaker_count: int) -> list[ET.Element]:
    audio_tracks = root.findall("./LiveSet/Tracks/AudioTrack")
    voice_tracks = [
        track
        for track in audio_tracks
        if (
            track.find("./Name/EffectiveName") is not None
            and track.find("./Name/EffectiveName").attrib.get("Value", "").startswith(
                "Voice "
            )
        )
    ]
    tracks = voice_tracks if len(voice_tracks) >= required_speaker_count else audio_tracks
    if len(tracks) < required_speaker_count:
        raise RuntimeError(
            f"Ableton template has {len(tracks)} usable audio track(s), "
            f"but {required_speaker_count} speaker track(s) are required."
        )
    return tracks[:required_speaker_count]


def _set_master_tempo(root: ET.Element) -> None:
    tempo_manual = root.find("./LiveSet/MainTrack/DeviceChain/Mixer/Tempo/Manual")
    if tempo_manual is not None:
        tempo_manual.attrib["Value"] = str(_BPM)


def generate_als(
    segment_results: list[SegmentResult],
    output_path: Path,
    position_map: dict[tuple[str, int], int] | None = None,
) -> Path:
    project_root = output_path.parent

    # Dynamic speaker discovery: order of first appearance
    speaker_order: list[str] = []
    for segment in segment_results:
        if segment.speaker_id not in speaker_order:
            speaker_order.append(segment.speaker_id)

    positioned_segments: list[tuple[SegmentResult, int]] = []
    cursor_ms = 0
    for segment in segment_results:
        if position_map and (segment.turn_id, segment.chunk_index) in position_map:
            start_ms = position_map[(segment.turn_id, segment.chunk_index)]
        else:
            start_ms = cursor_ms
        cursor_ms = start_ms + segment.speech_duration_ms + segment.gap_after_ms
        positioned_segments.append((segment, start_ms))

    clips_by_speaker: dict[str, list[ET.Element]] = {
        speaker_id: [] for speaker_id in speaker_order
    }
    clip_id = 1
    for segment, start_ms in positioned_segments:
        start_beats = _ms_to_beats(start_ms)
        length_beats = _ms_to_beats(segment.speech_duration_ms)
        file_length_beats = _ms_to_beats(segment.duration_ms)
        clip = _build_audio_clip(
            clip_id,
            start_beats,
            length_beats,
            file_length_beats,
            segment,
            project_root,
        )
        clips_by_speaker[segment.speaker_id].append(clip)
        clip_id += 1

    used_speakers = [
        speaker_id for speaker_id in speaker_order if clips_by_speaker[speaker_id]
    ] or [speaker_order[0]] if speaker_order else []

    if not used_speakers:
        raise ValueError("No segments to generate ALS from")

    root = _load_template_root()
    _set_master_tempo(root)
    tracks = _speaker_tracks(root, len(used_speakers))
    track_by_speaker = dict(zip(used_speakers, tracks))

    for speaker_id, track in track_by_speaker.items():
        _set_track_name(track, speaker_id)
        events = _arrangement_events(track)
        events.clear()
        for clip in clips_by_speaker[speaker_id]:
            events.append(copy.deepcopy(clip))

    ET.indent(root)
    xml_text = ET.tostring(root, encoding="unicode")
    xml_bytes = (
        '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_text
    ).encode("utf-8")

    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Backup existing ALS before overwriting
    if output_path.exists():
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = output_path.with_suffix(f".backup_{timestamp}.als")
        shutil.copy2(output_path, backup_path)

    with gzip.open(output_path, "wb") as als_file:
        als_file.write(xml_bytes)

    return output_path
