//! Tag spans keep the original UTF-8 text; TextEdit retains character-index caret mapping.
use crate::{structure::Registry, theme as t};
use eframe::egui::{self, text::LayoutJob};
use regex::Regex;
use serde::Deserialize;
use std::{ops::Range, sync::LazyLock};
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[[^\]\n]*\]").unwrap());
static PAUSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[pause:([0-9]+)(?:ms)?\]$").unwrap());

#[derive(Clone, Debug, Deserialize)]
pub struct Problem {
    pub index: usize,
    pub line: usize,
    pub tag: String,
    pub message: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Report {
    pub script_sha256: String,
    pub problems: Vec<Problem>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub range: Range<usize>,
    pub messages: Vec<String>,
    pub blocks_save: bool,
}
/// `<!-- ... -->` is an editorial note. Brackets inside it are not spoken.
fn comment_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut from = 0;
    while let Some(start) = text[from..].find("<!--") {
        let start = from + start;
        let after = start + 4;
        let Some(end_rel) = text[after..].find("-->") else {
            break;
        };
        let end = after + end_rel + 3;
        spans.push(start..end);
        from = end;
    }
    spans
}

fn inside_comment(range: &Range<usize>, comments: &[Range<usize>]) -> bool {
    comments
        .iter()
        .any(|span| range.start < span.end && span.start < range.end)
}

pub fn spans(body: &str, registry: &Registry) -> Vec<Span> {
    let comments = comment_spans(body);
    let mut clips = 0;
    TAG.find_iter(body)
        .filter(|m| !inside_comment(&m.range(), &comments))
        .map(|m| {
            let tag = m.as_str();
            let mut messages = Vec::new();
            let mut blocks_save = false;
            if let Some(c) = PAUSE.captures(tag) {
                if !c[1]
                    .parse::<u32>()
                    .is_ok_and(|ms| (1..=10000).contains(&ms))
                {
                    messages.push("pause must be between 1 and 10000 ms".into());
                    blocks_save = true;
                }
            } else if tag == "[beat]" || tag == "[emphasis]" {
            } else if tag.starts_with("[tone:") {
                if tag[6..tag.len() - 1].trim().is_empty() {
                    messages.push("tone must be non-empty".into());
                }
            } else if tag.starts_with("[clip:") {
                clips += 1;
                let id = &tag[6..tag.len() - 1];
                if id.is_empty() {
                    messages.push("clip id must be non-empty".into());
                } else if !registry.clips.iter().any(|c| c.id == id) {
                    messages.push(format!("clip '{id}' is not registered"));
                }
                if clips > 1 {
                    messages.push("clip appears more than once in this turn".into());
                }
                blocks_save = !messages.is_empty();
            } else {
                messages.push("unrecognised tag; it will be spoken aloud".into());
                blocks_save = tag.starts_with("[pause:");
            }
            Span {
                range: m.range(),
                messages,
                blocks_save,
            }
        })
        .collect()
}
pub fn can_save(body: &str, registry: &Registry) -> bool {
    !body.trim().is_empty()
        && !spans(body, registry).iter().any(|s| s.blocks_save)
        && !body.lines().any(|line| {
            ["[pause:", "[clip:"].iter().any(|prefix| {
                line.rfind(prefix)
                    .is_some_and(|start| !line[start..].contains(']'))
            })
        })
}
pub fn layout(text: &str, registry: &Registry, raw: bool, width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = width;
    let normal = egui::TextFormat {
        font_id: if raw {
            t::mono(t::BODY)
        } else {
            t::font(t::BODY)
        },
        color: t::TEXT_PRIMARY,
        ..Default::default()
    };
    if raw {
        job.append(text, 0.0, normal);
        return job;
    }
    let mut end = 0;
    for span in spans(text, registry) {
        job.append(&text[end..span.range.start], 0.0, normal.clone());
        let invalid = !span.messages.is_empty();
        job.append(
            &text[span.range.clone()],
            0.0,
            egui::TextFormat {
                font_id: t::font(t::CAPTION),
                color: if invalid { t::ON_DANGER } else { t::accent() },
                background: if invalid {
                    t::DANGER_PILL
                } else {
                    t::BG_SELECTED
                },
                expand_bg: t::TAG_BACKGROUND_EXPANSION,
                ..Default::default()
            },
        );
        end = span.range.end;
    }
    job.append(&text[end..], 0.0, normal);
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Registry {
        Registry {
            speakers: Vec::new(),
            clips: Vec::new(),
        }
    }

    #[test]
    fn an_editorial_comment_is_not_a_spoken_tag() {
        let body = "Because cognitive bandwidth is finite.\n<!-- [book Preface] -->";
        assert!(spans(body, &registry()).is_empty());
        assert!(can_save(body, &registry()));
        let spoken = spans("It says [other] aloud.", &registry());
        assert_eq!(spoken.len(), 1);
        assert!(spoken[0].messages[0].contains("unrecognised"));
    }
}
