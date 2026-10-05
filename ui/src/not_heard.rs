//! Expected words Whisper did not hear, for Failed-row transcript marks.
use crate::discrepancy::{self, Tag};
use std::ops::Range;

#[derive(Clone, Debug)]
struct Token {
    start: usize,
    end: usize,
    norm: String,
}

fn norm_word(raw: &str) -> String {
    raw.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

/// Whitespace tokens with byte ranges, normalised like `verify_render._normalize`.
fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for raw in text.split_whitespace() {
        let start = offset + text[offset..].find(raw).expect("split token in text");
        let end = start + raw.len();
        offset = end;
        let norm = norm_word(raw);
        if norm.is_empty() || discrepancy::is_number_token(&norm) {
            continue;
        }
        out.push(Token { start, end, norm });
    }
    out
}

/// Byte ranges in `expected` for words deleted or replaced versus `heard`.
pub fn not_heard(expected: &str, heard: &str) -> Vec<(usize, usize)> {
    let exp = tokens(expected);
    let heard_norms = discrepancy::normalize_tokens(heard);
    let exp_norms: Vec<String> = exp.iter().map(|t| t.norm.clone()).collect();
    let mut ranges = Vec::new();
    for &(tag, i1, i2, _, _) in &discrepancy::opcodes(&exp_norms, &heard_norms) {
        if !matches!(tag, Tag::Delete | Tag::Replace) {
            continue;
        }
        for token in &exp[i1..i2] {
            ranges.push((token.start, token.end));
        }
    }
    ranges
}

/// Map stress `expected`/`heard` marks onto `display` text. Empty if mapping fails.
pub fn not_heard_in_display(expected: &str, heard: &str, display: &str) -> Vec<(usize, usize)> {
    let in_expected = not_heard(expected, heard);
    if in_expected.is_empty() {
        return Vec::new();
    }
    if expected == display {
        return in_expected;
    }
    let exp = tokens(expected);
    let disp = tokens(display);
    let exp_norms: Vec<String> = exp.iter().map(|t| t.norm.clone()).collect();
    let disp_norms: Vec<String> = disp.iter().map(|t| t.norm.clone()).collect();
    let mut wanted = vec![false; exp.len()];
    for &(start, end) in &in_expected {
        let Some(index) = exp.iter().position(|t| t.start == start && t.end == end) else {
            return Vec::new();
        };
        wanted[index] = true;
    }
    let mut out = Vec::new();
    let mut mapped = 0usize;
    for &(tag, i1, i2, j1, j2) in &discrepancy::opcodes(&exp_norms, &disp_norms) {
        if tag != Tag::Equal {
            continue;
        }
        for (ei, di) in (i1..i2).zip(j1..j2) {
            if wanted[ei] {
                out.push((disp[di].start, disp[di].end));
                mapped += 1;
            }
        }
    }
    if mapped != wanted.iter().filter(|w| **w).count() {
        return Vec::new();
    }
    out
}

/// Merge neighbouring not-heard word ranges separated only by whitespace.
pub fn merge_ranges(text: &str, ranges: &[(usize, usize)]) -> Vec<Range<usize>> {
    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|r| r.0);
    let mut out: Vec<Range<usize>> = Vec::new();
    for (start, end) in sorted {
        if let Some(last) = out.last_mut()
            && end >= last.start
            && text[last.end..start].chars().all(char::is_whitespace)
        {
            last.end = end.max(last.end);
        } else {
            out.push(start..end);
        }
    }
    out
}

/// Inline summary under Failed transcript text (mode A).
pub fn summary_line(text: &str, ranges: &[Range<usize>], max_chars: usize) -> String {
    let words: Vec<&str> = ranges.iter().map(|r| &text[r.clone()]).collect();
    let count = words.len();
    let joined = words.join(" ");
    let prefix = format!("{count} words not heard: \"");
    let suffix = '"';
    let budget = max_chars.saturating_sub(prefix.len() + 1);
    let body = if joined.chars().count() <= budget {
        joined
    } else {
        let take = budget.saturating_sub(1);
        let mut cut: String = joined.chars().take(take).collect();
        cut.push('…');
        cut
    };
    format!("{prefix}{body}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_marks_missing_expected_words() {
        let ranges = not_heard(
            "A tool does one thing well. An agent decides when and how to use it.",
            "A tool does one thing well. An agent decides when to use it.",
        );
        let words: Vec<_> = ranges
            .iter()
            .map(|&(a, b)| {
                &"A tool does one thing well. An agent decides when and how to use it."[a..b]
            })
            .collect();
        assert_eq!(words, ["and", "how"]);
    }

    #[test]
    fn replacement_marks_expected_side() {
        let ranges = not_heard("the quick brown fox", "the slow brown fox");
        assert_eq!(ranges, [(4, 9)]);
        assert_eq!(&"the quick brown fox"[4..9], "quick");
    }

    #[test]
    fn punctuation_is_stripped_for_matching() {
        let ranges = not_heard("Hello, WORLD!", "Hello world");
        assert!(ranges.is_empty());
        let ranges = not_heard("Hello, WORLD!", "Hello");
        assert_eq!(ranges.len(), 1);
        assert_eq!(&"Hello, WORLD!"[ranges[0].0..ranges[0].1], "WORLD!");
    }

    #[test]
    fn edited_display_maps_by_tokens_or_shows_nothing() {
        let expected = "An agent decides when and how to use it.";
        let heard = "An agent decides when to use it.";
        let display = "An agent decides when and how to use tools.";
        let ranges = not_heard_in_display(expected, heard, display);
        let words: Vec<_> = ranges.iter().map(|&(a, b)| &display[a..b]).collect();
        assert_eq!(words, ["and", "how"]);

        let unmappable = "Completely different sentence here.";
        assert!(not_heard_in_display(expected, heard, unmappable).is_empty());
    }

    #[test]
    fn no_diff_returns_empty() {
        assert!(not_heard("same words here", "same words here").is_empty());
    }

    #[test]
    fn summary_truncates_with_ellipsis() {
        let text = "alpha beta gamma delta epsilon";
        let ranges = [0..5, 6..10, 11..16, 17..22, 23..30];
        let line = summary_line(text, &ranges, 40);
        assert!(line.starts_with("5 words not heard: \""));
        assert!(line.contains('…'), "{line}");
        assert!(line.ends_with('"'));
    }
}
