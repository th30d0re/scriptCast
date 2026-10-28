//! Turn windows from `episode_manifest.json`, merged into speech spans.
//!
//! A gap shorter than the merge threshold is still "someone is talking": the
//! bed should not recover between two turns that almost touch. The threshold
//! matches the ScriptCast port of HyperFrames' `mergeIntervals` (`< gap`, so
//! a gap of exactly 300 ms stays open).

use serde::Deserialize;
use serde_json::Value;

use crate::MediaError;

/// Join turns separated by less than this. A gap of exactly this stays open.
pub const DEFAULT_MERGE_GAP_MS: u64 = 300;

/// One closed-open stretch of speech on the episode clock, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SpeechSpan {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// Turns that contained speech, and the spans left after merging.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechPlan {
    pub turn_count: usize,
    pub spans: Vec<SpeechSpan>,
}

/// Sort spans and join any two whose gap is shorter than `merge_gap_ms`.
///
/// Spans that touch or overlap always join. A span with `end_ms <= start_ms`
/// is dropped.
pub fn merge_spans(spans: &[SpeechSpan], merge_gap_ms: u64) -> Vec<SpeechSpan> {
    let mut ordered: Vec<SpeechSpan> = spans
        .iter()
        .copied()
        .filter(|span| span.end_ms > span.start_ms)
        .collect();
    ordered.sort_by_key(|span| (span.start_ms, span.end_ms));
    let mut merged: Vec<SpeechSpan> = Vec::new();
    for span in ordered {
        if let Some(prev) = merged.last_mut() {
            let overlaps = span.start_ms <= prev.end_ms;
            let gap = span.start_ms.saturating_sub(prev.end_ms);
            if overlaps || gap < merge_gap_ms {
                prev.end_ms = prev.end_ms.max(span.end_ms);
                continue;
            }
        }
        merged.push(span);
    }
    merged
}

/// Read turn `start_ms` / `end_ms` and merge them.
///
/// Times may be integers or whole-number JSON floats. Other manifest fields
/// are ignored.
pub fn spans_from_manifest(json: &str, merge_gap_ms: u64) -> Result<SpeechPlan, MediaError> {
    let file: ManifestTurns =
        serde_json::from_str(json).map_err(|err| MediaError::Manifest(err.to_string()))?;
    let turns: Vec<SpeechSpan> = file
        .turns
        .into_iter()
        .map(|turn| SpeechSpan {
            start_ms: turn.start_ms,
            end_ms: turn.end_ms,
        })
        .filter(|span| span.end_ms > span.start_ms)
        .collect();
    let turn_count = turns.len();
    Ok(SpeechPlan {
        turn_count,
        spans: merge_spans(&turns, merge_gap_ms),
    })
}

#[derive(Deserialize)]
struct ManifestTurns {
    #[serde(default)]
    turns: Vec<TurnTime>,
}

#[derive(Deserialize)]
struct TurnTime {
    #[serde(default, deserialize_with = "flexible_u64")]
    start_ms: u64,
    #[serde(default, deserialize_with = "flexible_u64")]
    end_ms: u64,
}

fn flexible_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(0);
    };
    match value {
        Value::Number(number) => {
            if let Some(whole) = number.as_u64() {
                return Ok(whole);
            }
            if let Some(float) = number.as_f64()
                && float.is_finite()
                && (0.0..u64::MAX as f64).contains(&float)
            {
                return Ok(float.round() as u64);
            }
            Err(serde::de::Error::custom(
                "start_ms and end_ms must be non-negative numbers",
            ))
        }
        _ => Err(serde::de::Error::custom(
            "start_ms and end_ms must be numbers",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start_ms: u64, end_ms: u64) -> SpeechSpan {
        SpeechSpan { start_ms, end_ms }
    }

    #[test]
    fn gap_shorter_than_the_threshold_merges_and_the_threshold_itself_does_not() {
        let close = [span(0, 1000), span(1299, 2000)];
        let merged = merge_spans(&close, 300);
        assert_eq!(merged, vec![span(0, 2000)]);

        let exact = [span(0, 1000), span(1300, 2000)];
        assert_eq!(
            merge_spans(&exact, 300),
            vec![span(0, 1000), span(1300, 2000)]
        );
    }

    #[test]
    fn overlap_touch_and_unsorted_input() {
        assert_eq!(
            merge_spans(&[span(0, 1000), span(500, 1500)], 0),
            vec![span(0, 1500)]
        );
        assert_eq!(
            merge_spans(&[span(0, 1000), span(1000, 1500)], 0),
            vec![span(0, 1500)]
        );
        assert_eq!(
            merge_spans(&[span(5000, 6000), span(0, 1000)], 0),
            vec![span(0, 1000), span(5000, 6000)]
        );
        assert_eq!(merge_spans(&[span(5, 5), span(1, 2)], 0), vec![span(1, 2)]);
    }

    #[test]
    fn manifest_json_ignores_unknown_fields_and_rounds_floats() {
        let plan = spans_from_manifest(
            r#"{"episode_id":"x","turns":[
                {"start_ms":0,"end_ms":1000,"speaker_id":"a"},
                {"start_ms":1100.2,"end_ms":2000.0}
            ]}"#,
            300,
        )
        .unwrap();
        assert_eq!(plan.turn_count, 2);
        assert_eq!(plan.spans, vec![span(0, 2000)]);
    }

    #[test]
    fn manifest_rejects_garbage_and_keeps_an_empty_episode() {
        assert!(spans_from_manifest("nope", 300).is_err());
        let plan = spans_from_manifest("{}", 300).unwrap();
        assert_eq!(plan.turn_count, 0);
        assert!(plan.spans.is_empty());
    }

    #[test]
    fn fixture_episode_collapses_to_one_span() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../ui/tests/fixtures/episode_small/episode_manifest.json"
        );
        let json = std::fs::read_to_string(path).unwrap();
        let plan = spans_from_manifest(&json, DEFAULT_MERGE_GAP_MS).unwrap();
        assert_eq!(plan.turn_count, 3);
        assert_eq!(plan.spans, vec![span(0, 800)]);
    }
}
