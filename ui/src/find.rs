//! Find displayed words, navigate turns, and plan byte-range body replacements.
use crate::script::{self, ScriptTurn};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Words,
    Turns,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    pub mode: Mode,
    pub match_case: bool,
    pub whole_word: bool,
    pub regular_expression: bool,
    pub include_clips: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Words,
            match_case: false,
            whole_word: false,
            regular_expression: false,
            include_clips: true,
        }
    }
}

pub struct Matcher(Option<Regex>);
impl Matcher {
    pub fn new(query: &str, options: &Options) -> Result<Self, String> {
        if query.is_empty() {
            return Ok(Self(None));
        }
        let pattern = if options.regular_expression {
            query.to_owned()
        } else {
            regex::escape(query)
        };
        let pattern = if options.whole_word {
            format!(r"\b(?:{pattern})\b")
        } else {
            pattern
        };
        RegexBuilder::new(&pattern)
            .case_insensitive(!options.match_case)
            .build()
            .map(|regex| Self(Some(regex)))
            .map_err(|_| "Invalid pattern".into())
    }
    pub fn ranges(&self, text: &str) -> Vec<Range<usize>> {
        self.0.as_ref().map_or_else(Vec::new, |regex| {
            regex.find_iter(text).map(|m| m.range()).collect()
        })
    }
    pub fn matches_turn(&self, text: &str, speaker: &str, index: usize) -> bool {
        self.0.as_ref().is_none_or(|regex| {
            regex.is_match(text)
                || regex.is_match(speaker)
                || regex.is_match(&format!("{index:03}"))
        })
    }
    fn replacement(
        &self,
        body: &str,
        range: &Range<usize>,
        replacement: &str,
        regex: bool,
    ) -> String {
        if regex
            && let Some(captures) = self
                .0
                .as_ref()
                .and_then(|r| r.captures_at(body, range.start))
            && captures.get(0).is_some_and(|m| m.range() == *range)
        {
            let mut expanded = String::new();
            captures.expand(replacement, &mut expanded);
            expanded
        } else {
            replacement.to_owned()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub row: usize,
    pub displayed: Range<usize>,
    pub body: Option<Range<usize>>,
}

fn tag_range(body: &str) -> Option<Range<usize>> {
    let tag = script::clip_tag(body)?;
    let start = body.find(&tag)?;
    Some(start..start + tag.len())
}

fn overlaps(range: &Range<usize>, tag: &Range<usize>) -> bool {
    (range.start < tag.end && range.end > tag.start)
        || (range.is_empty() && range.start > tag.start && range.start < tag.end)
}

pub fn occurrences(
    row: usize,
    body: &str,
    matcher: &Matcher,
    options: &Options,
) -> Vec<Occurrence> {
    let tag = tag_range(body);
    if !options.include_clips && tag.is_some() {
        return vec![];
    }
    let displayed = script::display_body(body);
    let stripped = tag.as_ref().map_or_else(
        || body.to_owned(),
        |r| format!("{}{}", &body[..r.start], &body[r.end..]),
    );
    let leading = if tag.is_some() {
        stripped.len() - stripped.trim_start().len()
    } else {
        0
    };
    matcher
        .ranges(&displayed)
        .into_iter()
        .map(|range| {
            let start = range.start + leading;
            let end = range.end + leading;
            let raw = match &tag {
                Some(tag) if start < tag.start && end > tag.start => None,
                Some(tag) if start >= tag.start => Some(start + tag.len()..end + tag.len()),
                _ => Some(start..end),
            };
            Occurrence {
                row,
                displayed: range,
                body: raw,
            }
        })
        .collect()
}

pub fn navigate(current: Option<usize>, count: usize, previous: bool) -> Option<usize> {
    if count == 0 {
        None
    } else {
        Some(match current.filter(|i| *i < count) {
            Some(0) if previous => count - 1,
            Some(i) if previous => i - 1,
            Some(i) => (i + 1) % count,
            None if previous => count - 1,
            None => 0,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub edits: Vec<(ScriptTurn, String)>,
    pub occurrences: usize,
    pub skipped: usize,
}
impl Plan {
    pub fn preview(&self) -> String {
        let mut text = format!(
            "Replace {} occurrences in {} turns",
            self.occurrences,
            self.edits.len()
        );
        if self.skipped > 0 {
            text.push_str(&format!(" · {} skipped in clip tags", self.skipped));
        }
        text
    }
}

/// Match bodies, not display strings: clip tags are protected even for spanning patterns.
pub fn plan(
    turns: &[ScriptTurn],
    matcher: &Matcher,
    options: &Options,
    replacement: &str,
    current: Option<&Occurrence>,
) -> Plan {
    let mut plan = Plan::default();
    for turn in turns {
        if current.is_some_and(|m| m.row != turn.index) {
            continue;
        }
        let tag = tag_range(&turn.body);
        if !options.include_clips && tag.is_some() {
            continue;
        }
        let mut ranges = Vec::new();
        for range in matcher.ranges(&turn.body) {
            if tag.as_ref().is_some_and(|tag| overlaps(&range, tag)) {
                if current.is_none() {
                    plan.skipped += 1;
                }
                continue;
            }
            if current.is_none_or(|m| m.body.as_ref() == Some(&range)) {
                ranges.push(range);
            }
        }
        if ranges.is_empty() {
            continue;
        }
        let mut body = turn.body.clone();
        for range in ranges.iter().rev() {
            let text =
                matcher.replacement(&turn.body, range, replacement, options.regular_expression);
            body.replace_range(range.clone(), &text);
        }
        plan.occurrences += ranges.len();
        plan.edits.push((turn.clone(), body));
    }
    plan
}
