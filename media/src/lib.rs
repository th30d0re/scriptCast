//! Duck a music bed under speech, and dip the bands that mask a voice.
//!
//! Speech times come from an episode manifest (`start_ms` / `end_ms` on each
//! turn). The bed is ordinary planar `f32` audio. Filter coefficients are
//! soundcraft-dsp peaking biquads.
//!
//! The band picker and the strength curve are adapted from HyperFrames
//! (Apache-2.0). See `NOTICE`.

#![forbid(unsafe_code)]

mod analyse;
mod cli;
mod envelope;
mod profile;
mod render;
mod span;

pub use analyse::{CarveSource, analyse_carve_bands, intelligibility_bands, mix_carve_sources};
pub use cli::run;
pub use envelope::{DEFAULT_ATTACK_MS, DEFAULT_RELEASE_MS, speech_engagement};
pub use profile::{CANDIDATE_CENTERS_HZ, CarveBand, CarveProfile, DEFAULT_STRENGTH, carve_profile};
pub use render::{CarveSettings, carve_bed};
pub use span::{DEFAULT_MERGE_GAP_MS, SpeechPlan, SpeechSpan, merge_spans, spans_from_manifest};

use std::fmt;

/// Failures from manifest parsing and from rendering a bed.
#[derive(Debug, PartialEq, Eq)]
pub enum MediaError {
    /// Sample rate was zero.
    SampleRate,
    /// Channels of one bed had different lengths.
    RaggedChannels,
    /// `episode_manifest.json` was missing fields or was not JSON.
    Manifest(String),
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaError::SampleRate => write!(f, "sample rate must be greater than 0"),
            MediaError::RaggedChannels => {
                write!(f, "every channel must have the same number of frames")
            }
            MediaError::Manifest(msg) => write!(f, "episode manifest: {msg}"),
        }
    }
}

impl std::error::Error for MediaError {}
