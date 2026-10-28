//! Sentence-aware text splitting that never cuts inside a `[...]` tag.
//!
//! All positions are `char` indices. A cut is "strictly inside" a tag when it
//! falls between the `[` and the `]`.

/// Tokens whose trailing period is not a sentence end (compared lower-case).
pub const ABBREVIATIONS: [&str; 12] = [
    "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "st", "vs", "etc", "us", "uk",
];

/// Written forms with internal periods (compared lower-case, final period included).
pub const DOTTED_ABBREVIATIONS: [&str; 6] = ["e.g.", "i.e.", "a.m.", "p.m.", "u.s.", "u.k."];

const SENTENCE_MARKS: [char; 6] = ['.', '!', '?', '。', '！', '？'];
const CLAUSE_MARKS: [char; 3] = [';', ',', ':'];
const EM_DASH: char = '—';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub extended_for_tag: bool,
}

/// `[start, end)` char range of one tag, `end` just past the `]`.
type Span = (usize, usize);

fn tag_spans(chars: &[char]) -> Result<Vec<Span>, String> {
    let mut spans = Vec::new();
    let mut open = None;
    for (i, &c) in chars.iter().enumerate() {
        match (c, open) {
            ('[', None) => open = Some(i),
            (']', Some(start)) => {
                spans.push((start, i + 1));
                open = None;
            }
            _ => {}
        }
    }
    match open {
        Some(start) => Err(format!("unclosed '[' at character {start}")),
        None => Ok(spans),
    }
}

fn has_content(chars: &[char]) -> bool {
    chars.iter().any(|c| !c.is_whitespace())
}

fn collect_lower(chars: &[char]) -> String {
    chars.iter().flat_map(|c| c.to_lowercase()).collect()
}

/// True when the period at `at` ends an abbreviation such as `Mr.` or `e.g.`.
fn is_abbreviation(chars: &[char], at: usize) -> bool {
    let mut word_start = at;
    while word_start > 0 && chars[word_start - 1].is_alphabetic() {
        word_start -= 1;
    }
    if ABBREVIATIONS.contains(&collect_lower(&chars[word_start..at]).as_str()) {
        return true;
    }
    let mut token_start = at;
    while token_start > 0
        && (chars[token_start - 1].is_alphabetic() || chars[token_start - 1] == '.')
    {
        token_start -= 1;
    }
    let token = collect_lower(&chars[token_start..=at]);
    DOTTED_ABBREVIATIONS.contains(&token.trim_start_matches('.'))
}

/// ASCII marks only count before whitespace or the end of the text, so that
/// decimals (`3.14`), file names and `e.g.` interiors never split.
fn followed_by_break(chars: &[char], at: usize) -> bool {
    chars.get(at + 1).is_none_or(|c| c.is_whitespace())
}

fn is_sentence_end(chars: &[char], at: usize) -> bool {
    let c = chars[at];
    if !SENTENCE_MARKS.contains(&c) {
        return false;
    }
    if !c.is_ascii() {
        return true;
    }
    followed_by_break(chars, at) && (c != '.' || !is_abbreviation(chars, at))
}

fn is_clause_boundary(chars: &[char], at: usize) -> bool {
    let c = chars[at];
    c == EM_DASH || (CLAUSE_MARKS.contains(&c) && followed_by_break(chars, at))
}

struct Splitter<'a> {
    chars: &'a [char],
    spans: &'a [Span],
}

impl Splitter<'_> {
    fn inside_tag(&self, pos: usize) -> bool {
        self.spans.iter().any(|&(s, e)| s < pos && pos < e)
    }

    /// Cut position after `end` and the whitespace that follows it.
    fn absorb_whitespace(&self, end: usize) -> usize {
        let mut cut = end;
        while self.chars.get(cut).is_some_and(|c| c.is_whitespace()) {
            cut += 1;
        }
        cut
    }

    /// Piece text would be `chars[start..end]`; may we cut there?
    fn usable(&self, start: usize, end: usize) -> bool {
        !self.inside_tag(end)
            && !self.inside_tag(self.absorb_whitespace(end))
            && has_content(&self.chars[start..end])
    }

    /// Next cut for the remainder at `start`. Returns the cut and whether the
    /// piece was extended past the limit for a tag.
    fn next_cut(&self, start: usize, max_chars: usize) -> (usize, bool) {
        let limit = start + max_chars;
        let candidates = || (start + 1..=limit).rev();

        let by_mark = |is_mark: fn(&[char], usize) -> bool| {
            candidates()
                .find(|&end| is_mark(self.chars, end - 1) && self.usable(start, end))
                .map(|end| self.absorb_whitespace(end))
        };
        if let Some(cut) = by_mark(is_sentence_end) {
            return (cut, false);
        }
        if let Some(cut) = by_mark(is_clause_boundary) {
            return (cut, false);
        }
        if let Some(end) =
            candidates().find(|&end| self.chars[end].is_whitespace() && self.usable(start, end))
        {
            return (self.absorb_whitespace(end), false);
        }

        match self.spans.iter().find(|&&(s, e)| s < limit && limit < e) {
            None => (limit, false),
            Some(&(s, _)) if has_content(&self.chars[start..s]) => (s, false),
            Some(&(_, e)) => (self.absorb_whitespace(e), true),
        }
    }
}

/// Split `text` into pieces of at most `max_chars` chars. A piece grows past
/// the limit only to keep a `[...]` tag whole, and then carries
/// `extended_for_tag`.
pub fn split(text: &str, max_chars: usize) -> Result<Vec<Piece>, String> {
    if max_chars == 0 {
        return Err("--max-chars must be at least 1".into());
    }
    let chars: Vec<char> = text.chars().collect();
    let spans = tag_spans(&chars)?;
    let piece = |from: usize, to: usize, extended_for_tag: bool| Piece {
        text: chars[from..to].iter().collect(),
        extended_for_tag,
    };
    if chars.len() <= max_chars {
        return Ok(vec![piece(0, chars.len(), false)]);
    }

    let splitter = Splitter {
        chars: &chars,
        spans: &spans,
    };
    let mut pieces = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        if chars.len() - start <= max_chars {
            pieces.push(piece(start, chars.len(), false));
            break;
        }
        let (cut, extended) = splitter.next_cut(start, max_chars);
        pieces.push(piece(start, cut, extended));
        start = cut;
    }
    Ok(pieces)
}

/// Split `text` once, at the whitespace nearest the midpoint that sits outside
/// a tag. Returns one piece when the text has no place to cut.
pub fn halve(text: &str) -> Result<Vec<Piece>, String> {
    let chars: Vec<char> = text.chars().collect();
    let spans = tag_spans(&chars)?;
    let len = chars.len();
    let mid = len / 2;
    let in_tag = |i: usize| spans.iter().any(|&(s, e)| s <= i && i < e);

    // Whitespace runs outside tags with text on both sides.
    let mut runs: Vec<Span> = Vec::new();
    let mut i = 0;
    while i < len {
        if chars[i].is_whitespace() && !in_tag(i) {
            let run_start = i;
            while i < len && chars[i].is_whitespace() && !in_tag(i) {
                i += 1;
            }
            if has_content(&chars[..run_start]) && has_content(&chars[i..]) {
                runs.push((run_start, i));
            }
        } else {
            i += 1;
        }
    }

    let mid_tag = spans.iter().find(|&&(s, e)| s < mid && mid < e);
    let run = match mid_tag {
        Some(&(s, e)) => runs
            .iter()
            .rev()
            .find(|r| r.1 <= s)
            .or_else(|| runs.iter().find(|r| r.0 >= e)),
        None => runs.iter().min_by_key(|&&(s, e)| {
            if mid < s {
                s - mid
            } else if mid >= e {
                mid - (e - 1)
            } else {
                0
            }
        }),
    };
    let cut = match (run, mid_tag) {
        (Some(&(_, end)), _) => Some(end),
        // No whitespace to use: fall back to a char boundary outside tags.
        (None, Some(&(s, _))) if s > 0 => Some(s),
        (None, Some(&(_, e))) if e < len => Some(e),
        (None, Some(_)) => None,
        (None, None) if mid > 0 => Some(mid),
        (None, None) => None,
    };

    let piece = |text: String| Piece {
        text,
        extended_for_tag: false,
    };
    Ok(match cut {
        Some(cut) => vec![
            piece(chars[..cut].iter().collect()),
            piece(chars[cut..].iter().collect()),
        ],
        None => vec![piece(text.to_string())],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(pieces: &[Piece]) -> Vec<&str> {
        pieces.iter().map(|p| p.text.as_str()).collect()
    }

    fn joined(pieces: &[Piece]) -> String {
        pieces.iter().map(|p| p.text.as_str()).collect()
    }

    #[test]
    fn short_text_is_one_piece() {
        let pieces = split("hello", 5).unwrap();
        assert_eq!(texts(&pieces), ["hello"]);
        assert!(!pieces[0].extended_for_tag);
    }

    #[test]
    fn abbreviation_period_is_not_a_sentence_end() {
        let input = "Mr. Smith went home. He sat.";
        let pieces = split(input, 20).unwrap();
        assert!(pieces[0].text.contains("home."));
        assert!(!pieces[0].text.contains("He"));
        assert_eq!(joined(&pieces), input);
    }

    #[test]
    fn written_abbreviations_do_not_split() {
        let input = "Use tools, e.g. a saw. Then rest.";
        let pieces = split(input, 16).unwrap();
        assert_eq!(texts(&pieces)[0], "Use tools, ");
        assert_eq!(joined(&pieces), input);
        let input = "Meet at 5 p.m. Bring food.";
        let pieces = split(input, 16).unwrap();
        assert!(pieces[0].text.contains("Bring") || pieces[0].text.ends_with("p.m. "));
        assert_eq!(joined(&pieces), input);
    }

    #[test]
    fn decimal_is_not_a_sentence_end() {
        let input = "3.14 is pi. Done.";
        for max in 4..input.chars().count() {
            let pieces = split(input, max).unwrap();
            assert!(
                !pieces[0].text.ends_with("3."),
                "split at 3. with max {max}: {pieces:?}"
            );
            assert_eq!(joined(&pieces), input);
        }
        let pieces = split(input, 11).unwrap();
        assert_eq!(texts(&pieces), ["3.14 is pi. ", "Done."]);
    }

    #[test]
    fn tag_at_limit_is_extended() {
        let input = "[pause:500ms] hello there friend";
        let pieces = split(input, 8).unwrap();
        assert!(pieces[0].text.starts_with("[pause:500ms]"));
        assert!(pieces[0].extended_for_tag);
        assert!(pieces[1..].iter().all(|p| !p.extended_for_tag));
        assert_eq!(joined(&pieces), input);
        for piece in &pieces {
            assert_eq!(
                piece.text.matches('[').count(),
                piece.text.matches(']').count()
            );
        }
    }

    #[test]
    fn cut_moves_before_a_tag_when_it_can() {
        let pieces = split("hello [pause:500ms] world", 8).unwrap();
        assert_eq!(texts(&pieces), ["hello ", "[pause:500ms] ", "world"]);
        assert_eq!(
            pieces
                .iter()
                .map(|p| p.extended_for_tag)
                .collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn mark_inside_a_tag_is_not_a_cut() {
        let pieces = split("[note: one. two] tail", 8).unwrap();
        assert_eq!(texts(&pieces), ["[note: one. two] ", "tail"]);
    }

    #[test]
    fn cjk_sentence_end_needs_no_space() {
        let pieces = split("你好。世界", 3).unwrap();
        assert_eq!(texts(&pieces), ["你好。", "世界"]);
    }

    #[test]
    fn semicolon_is_a_clause_boundary() {
        let pieces = split("alpha; beta gamma", 8).unwrap();
        assert!(!pieces[0].text.contains("beta"));
        assert_eq!(texts(&pieces)[0], "alpha; ");
    }

    #[test]
    fn colon_and_em_dash_are_clause_boundaries() {
        assert_eq!(texts(&split("alpha: beta gamma", 8).unwrap())[0], "alpha: ");
        assert_eq!(texts(&split("alpha—beta gamma", 8).unwrap())[0], "alpha—");
    }

    #[test]
    fn digit_grouping_comma_is_not_a_boundary() {
        let pieces = split("1,000 and 2,000", 3).unwrap();
        assert!(pieces.iter().all(|p| !p.text.ends_with("1,")));
    }

    #[test]
    fn falls_back_to_whitespace() {
        let pieces = split("alpha beta gamma", 8).unwrap();
        assert_eq!(texts(&pieces), ["alpha ", "beta ", "gamma"]);
    }

    #[test]
    fn falls_back_to_hard_cut() {
        let pieces = split("abcdefghij", 4).unwrap();
        assert_eq!(texts(&pieces), ["abcd", "efgh", "ij"]);
    }

    #[test]
    fn hard_cut_stops_before_a_tag() {
        let pieces = split("abcdef[pause:5ms]gh", 8).unwrap();
        assert_eq!(texts(&pieces), ["abcdef", "[pause:5ms]", "gh"]);
        assert_eq!(
            pieces
                .iter()
                .map(|p| p.extended_for_tag)
                .collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn unclosed_tag_is_an_error() {
        assert!(split("hello [pause", 5).is_err());
        assert!(split("[", 5).is_err());
        assert!(halve("a [b").is_err());
    }

    #[test]
    fn tags_do_not_nest() {
        // The first `]` closes; the second is plain text.
        let pieces = split("[a [b] c] d", 100).unwrap();
        assert_eq!(texts(&pieces), ["[a [b] c] d"]);
    }

    #[test]
    fn zero_limit_is_an_error() {
        assert!(split("x", 0).is_err());
    }

    #[test]
    fn halve_cuts_at_nearest_whitespace() {
        let pieces = halve("one two three four").unwrap();
        assert_eq!(texts(&pieces), ["one two ", "three four"]);
    }

    #[test]
    fn halve_avoids_tag_interior() {
        let pieces = halve("left [pause:10ms] right").unwrap();
        assert_eq!(texts(&pieces), ["left ", "[pause:10ms] right"]);
        for piece in &pieces {
            assert_eq!(
                piece.text.matches('[').count(),
                piece.text.matches(']').count()
            );
        }
    }

    #[test]
    fn halve_uses_whitespace_after_tag_when_none_before() {
        let pieces = halve("[pause:1000ms] go").unwrap();
        assert_eq!(texts(&pieces), ["[pause:1000ms] ", "go"]);
    }

    #[test]
    fn halve_without_whitespace_cuts_the_midpoint() {
        assert_eq!(texts(&halve("abcdef").unwrap()), ["abc", "def"]);
    }

    #[test]
    fn halve_of_a_lone_tag_is_one_piece() {
        assert_eq!(texts(&halve("[pause:5ms]").unwrap()), ["[pause:5ms]"]);
    }
}
