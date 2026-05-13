//
//  EpisodeManifest.swift
//  Redefining_racism
//
//  Created by Emmanuel on 5/10/26.
//

import Foundation

// MARK: - SegmentEntry

struct SegmentEntry: Codable, Sendable {
    let chunkIndex: Int
    let segmentWav: String
    let durationMs: Int
    let startMs: Int
    let endMs: Int
    let gapAfterMs: Int
    let checksum: String

    private enum CodingKeys: String, CodingKey {
        case chunkIndex = "chunk_index"
        case segmentWav = "segment_wav"
        case durationMs = "duration_ms"
        case startMs = "start_ms"
        case endMs = "end_ms"
        case gapAfterMs = "gap_after_ms"
        case checksum
    }
}

// MARK: - TurnEntry

struct TurnEntry: Codable, Sendable {
    let turnIndex: Int
    let speakerId: String
    let sourceTimestamp: String
    let segments: [SegmentEntry]
    let startMs: Int
    let endMs: Int

    private enum CodingKeys: String, CodingKey {
        case turnIndex = "turn_index"
        case speakerId = "speaker_id"
        case sourceTimestamp = "source_timestamp"
        case segments
        case startMs = "start_ms"
        case endMs = "end_ms"
    }
}

// MARK: - SpeakerEntry

struct SpeakerEntry: Codable, Sendable {
    let speakerId: String
    let displayName: String
    let voice: String
    let turnCount: Int

    private enum CodingKeys: String, CodingKey {
        case speakerId = "speaker_id"
        case displayName = "display_name"
        case voice
        case turnCount = "turn_count"
    }
}

// MARK: - EpisodeManifest

struct EpisodeManifest: Codable, Sendable {
    let schemaVersion: String
    let episodeId: String
    let sourceFile: String
    let modelId: String
    let engine: String
    let sampleRate: Int
    let createdAt: String
    let speakers: [SpeakerEntry]
    let turns: [TurnEntry]

    private enum CodingKeys: String, CodingKey {
        case schemaVersion = "schema_version"
        case episodeId = "episode_id"
        case sourceFile = "source_file"
        case modelId = "model_id"
        case engine
        case sampleRate = "sample_rate"
        case createdAt = "created_at"
        case speakers
        case turns
    }
}
