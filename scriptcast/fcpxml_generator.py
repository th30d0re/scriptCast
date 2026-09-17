"""Generate Final Cut Pro XML (FCPXML) files for processed voice segments."""

from __future__ import annotations

import math
import xml.etree.ElementTree as ET
from pathlib import Path

import hashlib

from voice_pipeline.models import SegmentResult

_FCPXML_VERSION = "1.10"

# Use a standard 30fps format so FCP recognizes it.
# All time values are rounded to the nearest 1/30s frame boundary.
_FORMAT_ID = "r1"
_FORMAT_NAME = "FFVideoFormat1080p30"
_FORMAT_FRAME_DURATION = "1/30s"
_FRAME_RATE = 30  # frames per second


def _time_str(seconds: float) -> str:
    """Format a time value in FCPXML rational seconds format.

    Values are rounded to the nearest frame boundary (1/30s) so FCP
    doesn't warn about items not aligning to edit frame boundaries.
    """
    if seconds <= 0:
        return "0s"
    # Round to nearest frame boundary
    frames = int(round(seconds * _FRAME_RATE))
    if frames == 0:
        return "0s"
    return f"{frames}/{_FRAME_RATE}s"


def _ms_to_time_str(ms: int) -> str:
    """Convert milliseconds to FCPXML time string, rounded to frame boundary."""
    return _time_str(ms / 1000.0)


def _build_asset(resource_id: int, segment: SegmentResult, project_root: Path) -> ET.Element:
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
            "duration": _time_str(duration_s),
            "hasVideo": "0",
            "hasAudio": "1",
            "audioSources": "1",
            "audioChannels": "2",
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
) -> ET.Element:
    """Build an FCPXML <asset-clip> element positioned on a lane."""
    duration_s = segment.duration_ms / 1000.0

    clip = ET.Element(
        "asset-clip",
        {
            "name": segment.wav_path.stem,
            "ref": f"r{resource_id}",
            "lane": str(lane),
            "offset": _ms_to_time_str(start_ms),
            "duration": _time_str(duration_s),
            "start": "0s",
            "audioRole": "Dialogue",
        },
    )
    return clip


def generate_fcpxml(
    segment_results: list[SegmentResult],
    output_path: Path,
    position_map: dict[tuple[str, int], int] | None = None,
    episode_id: str = "episode",
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

    # Round total duration up to frame boundary for the gap
    total_duration_s = total_duration_ms / 1000.0
    total_frames = math.ceil(total_duration_s * _FRAME_RATE)
    gap_duration_str = f"{total_frames}/{_FRAME_RATE}s"

    # Build FCPXML
    fcpxml = ET.Element("fcpxml", {"version": _FCPXML_VERSION})

    # Resources
    resources = ET.SubElement(fcpxml, "resources")
    ET.SubElement(
        resources,
        "format",
        {
            "id": _FORMAT_ID,
            "name": _FORMAT_NAME,
            "frameDuration": _FORMAT_FRAME_DURATION,
            "width": "1920",
            "height": "1080",
        },
    )

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
            asset = _build_asset(resource_id, segment, project_root)
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
        )
        gap.append(clip)

    # Pretty-print XML
    ET.indent(fcpxml, space="  ")
    xml_text = ET.tostring(fcpxml, encoding="unicode")
    xml_bytes = (
        '<?xml version="1.0" encoding="UTF-8"?>\n' + xml_text
    ).encode("utf-8")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(xml_bytes)
    return output_path
