//! Discrepancy review cards: Whisper-only diffs and ensemble span mapping.
use crate::stress_ensemble::EnsembleSpan;
use serde::{Deserialize, Serialize};

/// Persisted Whisper / Ensemble preference for the discrepancy view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewMode {
    #[default]
    WhisperOnly,
    Ensemble,
}

impl ReviewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::WhisperOnly => "Whisper only",
            Self::Ensemble => "Ensemble",
        }
    }
}

/// One difference card shown in the left column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffCard {
    pub start_token: usize,
    pub end_token: usize,
    pub expected: String,
    pub heard: String,
    /// Accent source label; absent in Whisper-only mode.
    pub source: Option<&'static str>,
    pub verdict: &'static str,
    pub hint: Option<&'static str>,
    /// Danger-coloured lines under the card (ensemble warnings when present).
    pub warnings: Vec<String>,
}

/// Move card selection with wrap-free bounds (used by `[` / `]`).
pub fn select_delta(selected: usize, n: usize, delta: i32) -> usize {
    if n == 0 {
        return 0;
    }
    let cur = selected.min(n - 1);
    if delta < 0 {
        cur.saturating_sub((-delta) as usize)
    } else {
        (cur + delta as usize).min(n - 1)
    }
}

/// Tokenise like `verify_render._normalize` / `stress_ensemble`.
pub fn normalize_tokens(text: &str) -> Vec<String> {
    let mut lowered = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() {
            lowered.push(ch.to_ascii_lowercase());
        } else {
            lowered.push(' ');
        }
    }
    lowered
        .split_whitespace()
        .filter(|w| !w.is_empty() && !is_number_token(w))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn is_number_token(w: &str) -> bool {
    if w.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    NUMBER_WORDS.contains(&w)
}

const NUMBER_WORDS: &[&str] = &[
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
    "fourty",
    "fifty",
    "sixty",
    "seventy",
    "eighty",
    "ninety",
    "hundred",
    "thousand",
    "million",
    "billion",
    "forties",
    "fifties",
    "sixties",
    "seventies",
    "eighties",
    "nineties",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tag {
    Equal,
    Replace,
    Delete,
    Insert,
}

/// SequenceMatcher-style opcodes over token slices (same approach as review.rs).
pub(crate) fn opcodes(a: &[String], b: &[String]) -> Vec<(Tag, usize, usize, usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            let start_i = i;
            let start_j = j;
            while i < n && j < m && a[i] == b[j] {
                i += 1;
                j += 1;
            }
            out.push((Tag::Equal, start_i, i, start_j, j));
        } else if j < m && (i == n || dp[i][j + 1] >= dp[i + 1][j]) {
            let start_j = j;
            while j < m && (i == n || (i < n && a[i] != b[j] && dp[i][j + 1] >= dp[i + 1][j])) {
                j += 1;
                if i < n && j < m && a[i] == b[j] {
                    break;
                }
            }
            if i < n && start_j < j {
                let start_i = i;
                let take = (j - start_j).min(n - i);
                if take > 0 && dp[i + 1][start_j] < dp[i][start_j + 1] {
                    out.push((Tag::Insert, i, i, start_j, j));
                } else if take > 0 {
                    i += take;
                    out.push((Tag::Replace, start_i, i, start_j, start_j + take));
                    if start_j + take < j {
                        out.push((Tag::Insert, i, i, start_j + take, j));
                    }
                } else {
                    out.push((Tag::Insert, i, i, start_j, j));
                }
            } else {
                out.push((Tag::Insert, i, i, start_j, j));
            }
        } else if i < n {
            let start_i = i;
            while i < n && (j == m || (j < m && a[i] != b[j] && dp[i + 1][j] >= dp[i][j + 1])) {
                i += 1;
                if i < n && j < m && a[i] == b[j] {
                    break;
                }
            }
            out.push((Tag::Delete, start_i, i, j, j));
        } else {
            break;
        }
    }
    out
}

fn engine_tokens_for_span(
    ops: &[(Tag, usize, usize, usize, usize)],
    engine: &[String],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut out = Vec::new();
    for &(tag, i1, i2, j1, j2) in ops {
        if tag == Tag::Insert {
            if start == end && start == i1 {
                out.extend(engine[j1..j2].iter().cloned());
            }
            continue;
        }
        let lo = i1.max(start);
        let hi = i2.min(end);
        if lo >= hi {
            continue;
        }
        if tag == Tag::Equal {
            let off = lo - i1;
            let len = hi - lo;
            out.extend(engine[j1 + off..j1 + off + len].iter().cloned());
        } else {
            out.extend(engine[j1..j2].iter().cloned());
        }
    }
    out
}

/// Whisper-only cards: dirty spans of expected vs Whisper `heard`.
pub fn whisper_cards(expected: &str, heard: &str) -> Vec<DiffCard> {
    let exp = normalize_tokens(expected);
    let wh = normalize_tokens(heard);
    let ops = opcodes(&exp, &wh);
    let mut dirty = vec![false; exp.len()];
    let mut inserts: Vec<bool> = vec![false; exp.len() + 1];
    for &(tag, i1, i2, _, _) in &ops {
        match tag {
            Tag::Equal => {}
            Tag::Insert => inserts[i1] = true,
            Tag::Replace | Tag::Delete => {
                for slot in dirty.iter_mut().take(i2).skip(i1) {
                    *slot = true;
                }
            }
        }
    }
    let mut cards = Vec::new();
    let mut emit = |start: usize, end: usize| {
        let expected_toks = &exp[start..end];
        let heard_toks = engine_tokens_for_span(&ops, &wh, start, end);
        if expected_toks == heard_toks.as_slice() && !expected_toks.is_empty() {
            return;
        }
        if expected_toks.is_empty() && heard_toks.is_empty() {
            return;
        }
        cards.push(DiffCard {
            start_token: start,
            end_token: end,
            expected: expected_toks.join(" "),
            heard: heard_toks.join(" "),
            source: None,
            verdict: "Whisper heard differently",
            hint: None,
            warnings: Vec::new(),
        });
    };
    let mut index = 0usize;
    while index < exp.len() {
        if inserts[index] {
            emit(index, index);
            inserts[index] = false;
        }
        if !dirty[index] {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < exp.len() && dirty[end] {
            end += 1;
        }
        emit(index, end);
        index = end;
    }
    if inserts[exp.len()] {
        emit(exp.len(), exp.len());
    }
    cards
}

/// Map one ensemble span to a review card (verdict lines + source label).
pub fn ensemble_card(span: &EnsembleSpan) -> DiffCard {
    let (verdict, hint) = ensemble_verdict_lines(&span.verdict);
    let heard = heard_from_span(span);
    DiffCard {
        start_token: span.start_token,
        end_token: span.end_token,
        expected: span.expected.clone(),
        heard,
        source: source_label(span),
        verdict,
        hint,
        warnings: Vec::new(),
    }
}

pub fn ensemble_cards(spans: &[EnsembleSpan]) -> Vec<DiffCard> {
    spans.iter().map(ensemble_card).collect()
}

fn heard_from_span(span: &EnsembleSpan) -> String {
    if !span.phonon.trim().is_empty() {
        span.phonon.clone()
    } else {
        span.whisper.clone()
    }
}

/// Source from stored fields only — never invent Gap-fill.
pub fn source_label(span: &EnsembleSpan) -> Option<&'static str> {
    let phonon = !span.phonon.trim().is_empty();
    let whisper = !span.whisper.trim().is_empty();
    match (phonon, whisper) {
        (false, true) => Some("Whisper backfill"),
        // Stored data cannot distinguish Phonon primary from gap-fill.
        (true, _) => Some("Phonon"),
        (false, false) => None,
    }
}

pub fn ensemble_verdict_lines(verdict: &str) -> (&'static str, Option<&'static str>) {
    match verdict {
        "likely_audio" => (
            "Engines agree, but not with the script",
            Some("Likely audio problem · listen to confirm"),
        ),
        "likely_recognition" => ("Engines disagree · listen first", None),
        "uncertain" => ("Engines disagree with the script and each other", None),
        "coverage_gap" => ("One engine skipped · coverage gap", None),
        _ => ("Engines disagree with the script and each other", None),
    }
}

/// Approximate seek offset (ms into the turn) for a card's first expected token.
pub fn seek_ms_for_card(card: &DiffCard, words: &[crate::word_timings::Word]) -> Option<u64> {
    if words.is_empty() {
        return None;
    }
    let tokens = normalize_tokens(&card.expected);
    let needle = tokens.first()?;
    let idx = words.iter().position(|w| {
        normalize_tokens(&w.text)
            .first()
            .is_some_and(|t| t == needle)
    })?;
    // Prefer the span's token index when it lands inside the word list.
    let at = if card.start_token < words.len() {
        card.start_token
    } else {
        idx
    };
    Some(words.get(at).unwrap_or(&words[idx]).start_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whisper_only_builds_spans_from_fixture_text() {
        let cards = whisper_cards(
            "Anglo-Norman poer and potis descends",
            "Anglo-Norman Poeir and potes descends",
        );
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].expected, "poer");
        assert_eq!(cards[0].heard, "poeir");
        assert_eq!(cards[0].verdict, "Whisper heard differently");
        assert!(cards[0].source.is_none());
        assert_eq!(cards[1].expected, "potis");
        assert_eq!(cards[1].heard, "potes");
        let spain = whisper_cards(
            "the Spain chapter of the story",
            "the Portugal chapter of the story",
        );
        assert_eq!(spain.len(), 1);
        assert_eq!(spain[0].expected, "spain");
        assert_eq!(spain[0].heard, "portugal");
    }

    #[test]
    fn ensemble_spans_map_to_verdict_and_source() {
        let audio = EnsembleSpan {
            start_token: 1,
            end_token: 2,
            expected: "spain".into(),
            whisper: "portugal".into(),
            phonon: "portugal".into(),
            verdict: "likely_audio".into(),
        };
        let card = ensemble_card(&audio);
        assert_eq!(card.verdict, "Engines agree, but not with the script");
        assert_eq!(card.hint, Some("Likely audio problem · listen to confirm"));
        assert_eq!(card.source, Some("Phonon"));
        assert_eq!(card.heard, "portugal");

        let backfill = EnsembleSpan {
            start_token: 0,
            end_token: 1,
            expected: "pati".into(),
            whisper: "potty".into(),
            phonon: String::new(),
            verdict: "coverage_gap".into(),
        };
        let card = ensemble_card(&backfill);
        assert_eq!(card.source, Some("Whisper backfill"));
        assert_eq!(card.verdict, "One engine skipped · coverage gap");
        assert_eq!(card.heard, "potty");

        let recog = EnsembleSpan {
            start_token: 2,
            end_token: 3,
            expected: "potis".into(),
            whisper: "potes".into(),
            phonon: "potis".into(),
            verdict: "likely_recognition".into(),
        };
        let card = ensemble_card(&recog);
        assert_eq!(card.verdict, "Engines disagree · listen first");
        assert_eq!(card.source, Some("Phonon"));
    }

    #[test]
    fn bracket_selection_stays_in_bounds() {
        assert_eq!(select_delta(0, 0, 1), 0);
        assert_eq!(select_delta(0, 3, -1), 0);
        assert_eq!(select_delta(0, 3, 1), 1);
        assert_eq!(select_delta(2, 3, 1), 2);
        assert_eq!(select_delta(2, 3, -1), 1);
        assert_eq!(select_delta(5, 3, -1), 1);
    }
}
