"""Generate Final Cut Pro XML (FCPXML) files for processed voice segments."""

from __future__ import annotations

import math
import xml.etree.ElementTree as ET
from pathlib import Path

import hashlib

from scriptcast.models import SegmentResult

_FCPXML_VERSION = "1.10"

# Use a standard 30fps format so FCP recognizes it.
# All time values are rounded to the nearest 1/30s frame boundary.
_FORMAT_ID = "r1"
_FORMAT_NAME = "FFVideoFormat1080p30"
_FORMAT_FRAME_DURATION = "1/30s"
_FRAME_RATE = 30  # frames per second


def _time_str(seconds: float, fps: int = _FRAME_RATE) -> str:
    """Format a time value in FCPXML rational seconds format.

    Values are rounded to the nearest frame boundary so FCP doesn't warn
    about items not aligning to edit frame boundaries. The default grid is
    1/30 s, which is what scriptcast-export already writes.
    """
    if seconds <= 0:
        return "0s"
    frames = int(round(seconds * fps))
    if frames == 0:
        return "0s"
    return f"{frames}/{fps}s"


def _ms_to_time_str(ms: int, fps: int = _FRAME_RATE) -> str:
    """Convert milliseconds to FCPXML time string, rounded to frame boundary."""
    return _time_str(ms / 1000.0, fps)


def _format_attributes(width: int, height: int, fps: int) -> dict[str, str]:
    """Format resource. The 1920×1080 30 fps export keeps its preset name."""
    if (width, height, fps) == (1920, 1080, _FRAME_RATE):
        name = _FORMAT_NAME
        frame_duration = _FORMAT_FRAME_DURATION
    else:
        name = f"FFVideoFormat{width}x{height}p{fps}"
        frame_duration = f"1/{fps}s"
    return {
        "id": _FORMAT_ID,
        "name": name,
        "frameDuration": frame_duration,
        "width": str(width),
        "height": str(height),
    }


def _build_asset(
    resource_id: int,
    segment: SegmentResult,
    project_root: Path,
    fps: int = _FRAME_RATE,
) -> ET.Element:
    """Build an FCPXML <asset> element referencing a WAV file."""
    wav_path = segment.wav_path
    duration_s = segment.duration_ms / 1000.0

    # Resolve to absolute path for src
    abs_path = wav_path.resolve()

    # Generate a stable UID from the file path
    uid = hashlib.md5(str(abs_path).encode("utf-8")).hexdigest().upper()

    asset = ET.Element(
        "asset",
        {
            "id": f"r{resource_id}",
            "name": wav_path.stem,
            "uid": uid,
            "start": "0s",
            "duration": _time_str(duration_s, fps),
            "hasVideo": "0",
            "hasAudio": "1",
            "audioSources": "1",
            "audioChannels": str(segment.audio_channels),
            "audioRate": str(segment.sample_rate),
        },
    )
    ET.SubElement(
        asset,
        "media-rep",
        {
            "kind": "original-media",
            "src": f"file://{abs_path.as_posix()}",
        },
    )
    return asset


def _build_asset_clip(
    resource_id: int,
    segment: SegmentResult,
    start_ms: int,
    lane: int,
    fps: int = _FRAME_RATE,
) -> ET.Element:
    """Build an FCPXML <asset-clip> element positioned on a lane."""
    duration_s = segment.duration_ms / 1000.0

    clip = ET.Element(
        "asset-clip",
        {
            "name": segment.wav_path.stem,
            "ref": f"r{resource_id}",
            "lane": str(lane),
            "offset": _ms_to_time_str(start_ms, fps),
            "duration": _time_str(duration_s, fps),
            "start": "0s",
            "audioRole": "Dialogue",
        },
    )
    return clip


def _build_picture_asset(resource_id: int, src: Path, duration_s: float, fps: int) -> ET.Element:
    """Build a silent picture asset. Motion files stay picture-only."""
    abs_path = src.resolve()
    uid = hashlib.md5(str(abs_path).encode("utf-8")).hexdigest().upper()
    asset = ET.Element(
        "asset",
        {
            "id": f"r{resource_id}",
            "name": abs_path.stem,
            "uid": uid,
            "start": "0s",
            "duration": _time_str(duration_s, fps),
            "hasVideo": "1",
            "hasAudio": "0",
            "format": _FORMAT_ID,
            "videoSources": "1",
        },
    )
    ET.SubElement(
        asset,
        "media-rep",
        {
            "kind": "original-media",
            "src": f"file://{abs_path.as_posix()}",
        },
    )
    return asset


def _build_picture_clip(
    resource_id: int,
    src: Path,
    start_ms: int,
    duration_s: float,
    lane: int,
    fps: int,
) -> ET.Element:
    """Place a picture on its own lane for the cue's span."""
    return ET.Element(
        "asset-clip",
        {
            "name": src.stem,
            "ref": f"r{resource_id}",
            "lane": str(lane),
            "offset": _ms_to_time_str(start_ms, fps),
            "duration": _time_str(duration_s, fps),
            "start": "0s",
        },
    )


def generate_fcpxml(
    segment_results: list[SegmentResult],
    output_path: Path,
    position_map: dict[tuple[str, int], int] | None = None,
    episode_id: str = "episode",
    *,
    width: int = 1920,
    height: int = 1080,
    fps: int = _FRAME_RATE,
    pictures: list[dict] | None = None,
) -> Path:
    """Generate an FCPXML file for Final Cut Pro from segment results.

    Each speaker gets their own lane (track) in the timeline. All clips are
    connected to a single gap clip on the primary storyline, which allows
    independent positioning per speaker.

    Args:
        segment_results: List of segment results with timing and file info.
        output_path: Path to write the .fcpxml file.
        position_map: Optional dict mapping (turn_id, chunk_index) to start_ms.
            When provided, unchanged segments use their stored start_ms instead
            of sequential placement.
        episode_id: Episode identifier used for project/event naming.
        width: Frame width. The default keeps the audio-session 1920×1080 format.
        height: Frame height.
        fps: Frame rate used for the format resource and for time rounding.
        pictures: Silent picture cues with start_ms, end_ms, and src. Each one
            becomes a hasVideo=1, hasAudio=0 asset on the lane above the speakers.

    Returns:
        Path to the generated .fcpxml file.
    """
    project_root = output_path.parent

    # Determine speaker order (first appearance)
    speaker_order: list[str] = []
    for segment in segment_results:
        if segment.speaker_id not in speaker_order:
            speaker_order.append(segment.speaker_id)

    # Assign each speaker a lane (1-based for connected clips)
    speaker_lane = {
        speaker_id: lane + 1 for lane, speaker_id in enumerate(speaker_order)
    }

    # Compute total duration and positioned segments
    positioned_segments: list[tuple[SegmentResult, int]] = []
    cursor_ms = 0
    total_duration_ms = 0
    for segment in segment_results:
        if position_map and (segment.turn_id, segment.chunk_index) in position_map:
            start_ms = position_map[(segment.turn_id, segment.chunk_index)]
        else:
            start_ms = cursor_ms
            cursor_ms += segment.duration_ms + segment.gap_after_ms
        positioned_segments.append((segment, start_ms))
        end_ms = start_ms + segment.duration_ms
        if end_ms > total_duration_ms:
            total_duration_ms = end_ms
    for picture in pictures or []:
        picture_end = int(picture["end_ms"])
        if picture_end > total_duration_ms:
            total_duration_ms = picture_end

    # Round total duration up to frame boundary for the gap
    total_duration_s = total_duration_ms / 1000.0
    total_frames = math.ceil(total_duration_s * fps)
    gap_duration_str = f"{total_frames}/{fps}s"

    # Build FCPXML
    fcpxml = ET.Element("fcpxml", {"version": _FCPXML_VERSION})

    # Resources
    resources = ET.SubElement(fcpxml, "resources")
    ET.SubElement(resources, "format", _format_attributes(width, height, fps))

    # Assets and clips
    resource_id = 2
    resource_ids: dict[int, str] = {}  # maps segment id (hash) to resource id

    clips_by_resource: list[tuple[str, SegmentResult, int]] = []  # (ref_id, segment, start_ms)

    for segment, start_ms in positioned_segments:
        # Use a unique key for deduplication (same file could appear multiple times
        # if position_map references it, but each segment is unique in practice)
        seg_key = id(segment)
        if seg_key not in resource_ids:
            resource_ids[seg_key] = f"r{resource_id}"
            asset = _build_asset(resource_id, segment, project_root, fps)
            resources.append(asset)
            resource_id += 1

        ref_id = resource_ids[seg_key]
        clips_by_resource.append((ref_id, segment, start_ms))

    # Library > Event > Project > Sequence > Spine
    library = ET.SubElement(fcpxml, "library")
    event = ET.SubElement(library, "event", {"name": episode_id})
    project = ET.SubElement(event, "project", {"name": episode_id})
    sequence = ET.SubElement(
        project,
        "sequence",
        {
            "format": _FORMAT_ID,
            "duration": gap_duration_str,
            "tcStart": "0s",
            "tcFormat": "NDF",
        },
    )
    spine = ET.SubElement(sequence, "spine")

    # Gap on primary storyline spanning the full duration.
    # Connected clips are children of the gap so they anchor to it.
    gap = ET.SubElement(
        spine,
        "gap",
        {
            "name": "Gap",
            "offset": "0s",
            "duration": gap_duration_str,
        },
    )

    # Add all clips as connected clips on their speaker's lane
    for ref_id, segment, start_ms in clips_by_resource:
        lane = speaker_lane[segment.speaker_id]
        clip = _build_asset_clip(
            int(ref_id[1:]),  # strip 'r' prefix
            segment,
            start_ms,
            lane,
            fps,
        )
        gap.append(clip)

    picture_lane = (max(speaker_lane.values()) if speaker_lane else 0) + 1
    for picture in pictures or []:
        src = Path(picture["src"])
        start_ms = int(picture["start_ms"])
        duration_s = (int(picture["end_ms"]) - start_ms) / 1000.0
        if _time_str(duration_s, fps) == "0s":
            continue
        asset = _build_picture_asset(resource_id, src, duration_s, fps)
        resources.append(asset)
        gap.append(_build_picture_clip(resource_id, src, start_ms, duration_s, picture_lane, fps))
        resource_id += 1

    # Pretty-print XML
    ET.indent(fcpxml, space="  ")
    xml_text = ET.tostring(fcpxml, encoding="unicode")
    xml_bytes = (
        '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_text
    ).encode("utf-8")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(xml_bytes)
    return output_path
