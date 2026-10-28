//! Strength as the six numbers the carve works in.
//!
//! Ported from HyperFrames `carveProfile` (`packages/core/src/audioCarve.ts`).
//! Quarter strength is the old panel default: 6 dB, 3 bands, Q 1.4.

use serde::Serialize;

/// Third-octave speech centres, 160 Hz through 6 kHz.
pub const CANDIDATE_CENTERS_HZ: [f64; 9] = [
    160.0, 250.0, 400.0, 630.0, 1000.0, 1600.0, 2500.0, 4000.0, 6000.0,
];

/// Panel default. HyperFrames' own CLI uses 0.8; this is the gentler studio default.
pub const DEFAULT_STRENGTH: f64 = 0.25;

/// What one strength setting expands to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarveProfile {
    /// Deepest cut on the strongest band, as a positive number of dB.
    pub max_cut_db: f64,
    /// How many of the nine candidate bands to dip.
    pub bands: usize,
    pub q: f64,
    /// 0 ranks bands by voice power, 1 ranks them by proximity to 2 kHz.
    pub intelligibility_bias: f64,
    /// Fixed bed reduction while speech is open, in positive dB.
    pub duck_db: f64,
    /// HyperFrames' measured level-match target. Rendering does not use it:
    /// the manifest already says when speech is present, so the bed ducks by
    /// [`Self::duck_db`] for the whole window.
    pub headroom_db: f64,
}

/// One peaking cut. `gain_db` is negative.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CarveBand {
    pub freq_hz: f64,
    pub gain_db: f64,
    pub q: f64,
}

/// Map a strength knob onto the carve profile.
///
/// A non-finite strength is treated as 0.5, matching HyperFrames. Values
/// outside 0..1 clamp.
pub fn carve_profile(strength: f64) -> CarveProfile {
    let s = if strength.is_finite() {
        strength.clamp(0.0, 1.0)
    } else {
        0.5
    };
    CarveProfile {
        max_cut_db: round2(2.0 + s * 16.0),
        bands: (1.0 + s * 6.0).round().clamp(1.0, 6.0) as usize,
        q: round2(1.1 + s * 1.2),
        intelligibility_bias: round2((0.6 + s * 0.4).min(1.0)),
        duck_db: round2(s * 24.0),
        headroom_db: round2(6.0 + s * 12.0),
    }
}

pub(crate) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "{left} != {right}");
    }

    #[test]
    fn quarter_strength_is_the_old_panel_default() {
        let profile = carve_profile(0.25);
        close(profile.max_cut_db, 6.0);
        assert_eq!(profile.bands, 3);
        close(profile.q, 1.4);
        close(profile.intelligibility_bias, 0.7);
        close(profile.duck_db, 6.0);
        close(profile.headroom_db, 9.0);
    }

    #[test]
    fn ends_of_the_knob() {
        let off = carve_profile(0.0);
        close(off.duck_db, 0.0);
        assert_eq!(off.bands, 1);
        close(off.max_cut_db, 2.0);
        close(off.q, 1.1);
        close(off.intelligibility_bias, 0.6);

        let full = carve_profile(1.0);
        close(full.max_cut_db, 18.0);
        assert_eq!(full.bands, 6);
        close(full.duck_db, 24.0);
        close(full.intelligibility_bias, 1.0);
        close(full.q, 2.3);
        close(full.headroom_db, 18.0);

        let mid = carve_profile(0.5);
        close(mid.max_cut_db, 10.0);
        assert_eq!(mid.bands, 4);
        close(mid.q, 1.7);
        close(mid.intelligibility_bias, 0.8);
        close(mid.duck_db, 12.0);
        close(mid.headroom_db, 12.0);
    }

    #[test]
    fn clamps_and_treats_nan_as_half() {
        assert_eq!(carve_profile(9.0).bands, carve_profile(1.0).bands);
        close(carve_profile(9.0).max_cut_db, carve_profile(1.0).max_cut_db);
        close(carve_profile(-9.0).duck_db, 0.0);
        close(
            carve_profile(f64::NAN).max_cut_db,
            carve_profile(0.5).max_cut_db,
        );
    }

    #[test]
    fn every_step_moves_the_same_direction() {
        let steps: Vec<CarveProfile> = [0.0, 0.25, 0.5, 0.75, 1.0]
            .into_iter()
            .map(carve_profile)
            .collect();
        for pair in steps.windows(2) {
            assert!(pair[1].max_cut_db > pair[0].max_cut_db);
            assert!(pair[1].bands >= pair[0].bands);
            assert!(pair[1].q > pair[0].q);
            assert!(pair[1].duck_db > pair[0].duck_db);
            assert!(pair[1].headroom_db > pair[0].headroom_db);
            assert!(pair[1].intelligibility_bias >= pair[0].intelligibility_bias);
        }
    }
}
