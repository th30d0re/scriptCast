//! Measured read-along text, painted inside the table's virtual items.
use eframe::egui::{self, Rect, Ui, pos2, vec2};
use crate::{
    not_heard, theme as t,
    word_timings::{Word, current_word},
};
use std::{ops::Range, sync::Arc};

/// Match the timing generator's ASCII alphanumeric normalisation, in script order.
fn normalize(word: &str) -> String {
    word.to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

pub fn word_mapping(text: &str, words: &[Word]) -> Option<Vec<Range<usize>>> {
    let mut offset = 0;
    let ranges: Vec<_> = text
        .split_whitespace()
        .map(|word| {
            let start = offset + text[offset..].find(word).unwrap();
            offset = start + word.len();
            start..offset
        })
        .collect();
    if ranges.len() != words.len()
        || ranges
            .iter()
            .zip(words)
            .any(|(range, word)| normalize(&text[range.clone()]) != normalize(&word.text))
    {
        return None;
    }
    Some(ranges)
}

pub fn active_range(text: &str, words: &[Word], position: u64) -> Option<Range<usize>> {
    let current = current_word(words, position)?;
    word_mapping(text, words)?.get(current).cloned()
}

/// How Failed-row not-heard words are drawn in the transcript cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotHeardStyle {
    /// Mode A (inline): danger bold text + summary line under the galley.
    Inline,
    /// Mode B (side panel): danger pill with a small "not heard" label.
    Pill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PillKind {
    Accent,
    Danger,
}

pub struct Transcript {
    pub galley: Arc<egui::Galley>,
    pills: Vec<(Rect, PillKind)>,
    summary: Option<Arc<egui::Galley>>,
}

#[derive(Clone, Copy)]
enum SpanKind {
    Normal,
    Active,
    NotHeard,
}

fn span_kind(index: usize, active: &Option<Range<usize>>, not_heard: &[Range<usize>]) -> SpanKind {
    if active.as_ref().is_some_and(|r| r.contains(&index)) {
        return SpanKind::Active;
    }
    if not_heard.iter().any(|r| r.contains(&index)) {
        return SpanKind::NotHeard;
    }
    SpanKind::Normal
}

fn bounds(
    text_len: usize,
    active: &Option<Range<usize>>,
    not_heard: &[Range<usize>],
) -> Vec<usize> {
    let mut points = vec![0, text_len];
    if let Some(r) = active {
        points.push(r.start.min(text_len));
        points.push(r.end.min(text_len));
    }
    for r in not_heard {
        points.push(r.start.min(text_len));
        points.push(r.end.min(text_len));
    }
    points.sort_unstable();
    points.dedup();
    points
}

/// Pill rects for a character range in the laid-out job (not source text).
fn pill_rects_for_job_chars(galley: &egui::Galley, chars: Range<usize>) -> Vec<Rect> {
    let mut index = 0usize;
    let mut out = Vec::new();
    for row in &galley.rows {
        let mut pill = Rect::NOTHING;
        for glyph in &row.glyphs {
            if chars.contains(&index) {
                let rect = Rect::from_min_size(
                    row.pos + vec2(glyph.pos.x, 0.0),
                    vec2(glyph.advance_width, row.size.y),
                );
                pill = pill.union(rect);
            }
            index += 1;
        }
        if row.ends_with_newline {
            index += 1;
        }
        if pill.is_positive() {
            // Pad vertically and on the left; keep the right edge flush with the
            // last glyph so the pill cannot cover the following word.
            out.push(Rect::from_min_max(
                pos2(
                    pill.min.x - t::READ_ALONG_PAD_X,
                    pill.min.y - t::READ_ALONG_PAD_Y,
                ),
                pos2(pill.max.x, pill.max.y + t::READ_ALONG_PAD_Y),
            ));
        }
    }
    out
}

fn append_section(
    job: &mut egui::text::LayoutJob,
    job_chars: &mut usize,
    text: &str,
    leading: f32,
    format: egui::TextFormat,
) -> Range<usize> {
    let start = *job_chars;
    let count = text.chars().count();
    job.append(text, leading, format);
    *job_chars += count;
    start..*job_chars
}

impl Transcript {
    pub fn measure(
        ui: &Ui,
        text: &str,
        width: f32,
        active: Option<Range<usize>>,
        not_heard_ranges: &[(usize, usize)],
        style: Option<NotHeardStyle>,
    ) -> Self {
        let merged = match style {
            Some(_) if !not_heard_ranges.is_empty() => {
                not_heard::merge_ranges(text, not_heard_ranges)
            }
            _ => Vec::new(),
        };
        let style = style.filter(|_| !merged.is_empty());
        let normal = egui::TextFormat::simple(t::font(t::BODY), t::TEXT_PRIMARY);
        let active_fmt = egui::TextFormat::simple(t::bold(t::BODY), t::ON_ACCENT);
        let not_heard_inline = egui::TextFormat::simple(t::bold(t::BODY), t::STATUS_FAILED);
        let not_heard_pill = egui::TextFormat::simple(t::bold(t::BODY), t::ON_DANGER);
        let label_fmt = egui::TextFormat::simple(t::font(t::NOT_HEARD_LABEL), t::ON_DANGER);

        let mut job = egui::text::LayoutJob::default();
        let mut job_chars = 0usize;
        let mut danger_job: Vec<Range<usize>> = Vec::new();
        let mut accent_job: Vec<Range<usize>> = Vec::new();
        let points = bounds(text.len(), &active, &merged);

        let mut i = 0usize;
        while i + 1 < points.len() {
            let start = points[i];
            let end = points[i + 1];
            if start >= end {
                i += 1;
                continue;
            }

            // Mode B: emit a full not-heard run, then a layout section for the label.
            if style == Some(NotHeardStyle::Pill)
                && matches!(span_kind(start, &active, &merged), SpanKind::NotHeard)
                && let Some(run) = merged.iter().find(|r| r.contains(&start))
                && !active.as_ref().is_some_and(|a| a == run)
            {
                let mut danger_parts: Vec<Range<usize>> = Vec::new();
                while i + 1 < points.len() && points[i] < run.end {
                    let s = points[i];
                    let e = points[i + 1].min(run.end);
                    if s < e {
                        if matches!(span_kind(s, &active, &merged), SpanKind::Active) {
                            let range = append_section(
                                &mut job,
                                &mut job_chars,
                                &text[s..e],
                                t::READ_ALONG_PAD_X,
                                active_fmt.clone(),
                            );
                            let tail = append_section(
                                &mut job,
                                &mut job_chars,
                                "\u{2009}",
                                t::READ_ALONG_PAD_X,
                                active_fmt.clone(),
                            );
                            accent_job.push(range.start..tail.end);
                        } else {
                            let range = append_section(
                                &mut job,
                                &mut job_chars,
                                &text[s..e],
                                t::READ_ALONG_PAD_X,
                                not_heard_pill.clone(),
                            );
                            if let Some(last) = danger_parts.last_mut()
                                && last.end == range.start
                            {
                                last.end = range.end;
                            } else {
                                danger_parts.push(range);
                            }
                        }
                    }
                    i += 1;
                    if points[i] >= run.end {
                        break;
                    }
                }
                if let Some(last) = danger_parts.last_mut() {
                    let label = append_section(
                        &mut job,
                        &mut job_chars,
                        " not heard",
                        0.0,
                        label_fmt.clone(),
                    );
                    // Thin space with leading padding gives the pill a right margin
                    // equal to its left one without covering the next word.
                    let tail = append_section(
                        &mut job,
                        &mut job_chars,
                        "\u{2009}",
                        t::READ_ALONG_PAD_X,
                        label_fmt.clone(),
                    );
                    last.end = tail.end.max(label.end);
                    danger_job.extend(danger_parts);
                }
                continue;
            }

            let slice = &text[start..end];
            match (span_kind(start, &active, &merged), style) {
                (SpanKind::Active, _) => {
                    let range = append_section(
                        &mut job,
                        &mut job_chars,
                        slice,
                        t::READ_ALONG_PAD_X,
                        active_fmt.clone(),
                    );
                    let tail = append_section(
                        &mut job,
                        &mut job_chars,
                        "\u{2009}",
                        t::READ_ALONG_PAD_X,
                        active_fmt.clone(),
                    );
                    accent_job.push(range.start..tail.end);
                }
                (SpanKind::NotHeard, Some(NotHeardStyle::Inline)) => {
                    append_section(
                        &mut job,
                        &mut job_chars,
                        slice,
                        0.0,
                        not_heard_inline.clone(),
                    );
                }
                _ => {
                    append_section(&mut job, &mut job_chars, slice, 0.0, normal.clone());
                }
            }
            i += 1;
        }
        if job_chars == 0 {
            append_section(&mut job, &mut job_chars, text, 0.0, normal);
        }

        job.wrap.max_width = width.max(0.0);
        let galley = ui.painter().layout_job(job);

        let mut pills = Vec::new();
        for range in &danger_job {
            for rect in pill_rects_for_job_chars(&galley, range.clone()) {
                pills.push((rect, PillKind::Danger));
            }
        }
        for range in &accent_job {
            for rect in pill_rects_for_job_chars(&galley, range.clone()) {
                pills.push((rect, PillKind::Accent));
            }
        }

        let summary = match style {
            Some(NotHeardStyle::Inline) => {
                let line = not_heard::summary_line(text, &merged, t::NOT_HEARD_SUMMARY_MAX);
                Some(
                    ui.painter()
                        .layout_no_wrap(line, t::font(t::SECONDARY), t::STATUS_FAILED),
                )
            }
            _ => None,
        };

        Self {
            galley,
            pills,
            summary,
        }
    }

    /// Backward-compatible measure used by older call sites and tests.
    pub fn measure_simple(ui: &Ui, text: &str, width: f32, active: Option<Range<usize>>) -> Self {
        Self::measure(ui, text, width, active, &[], None)
    }

    pub fn height(&self) -> f32 {
        let text_h = self.galley.size().y + t::SPACE_4 * 2.0;
        let summary_h = self
            .summary
            .as_ref()
            .map_or(0.0, |g| t::SPACE_4 + g.size().y);
        t::TABLE_ROW_HEIGHT.max(text_h + summary_h)
    }

    /// Top of the highlighted word, relative to the text origin.
    pub fn active_top(&self) -> Option<f32> {
        self.pills
            .iter()
            .find(|(_, kind)| *kind == PillKind::Accent)
            .map(|(rect, _)| rect.top())
    }

    pub fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let origin = pos2(rect.left(), rect.top() + t::SPACE_4);
        // Danger under accent so the read-along pill wins on overlap.
        for (pill, _) in self.pills.iter().filter(|(_, k)| *k == PillKind::Danger) {
            painter.rect_filled(
                pill.translate(origin.to_vec2()),
                t::READ_ALONG_RADIUS,
                t::DANGER_PILL,
            );
        }
        for (pill, _) in self.pills.iter().filter(|(_, k)| *k == PillKind::Accent) {
            painter.rect_filled(
                pill.translate(origin.to_vec2()),
                t::READ_ALONG_RADIUS,
                t::accent(),
            );
        }
        painter.galley(origin, self.galley.clone(), t::TEXT_PRIMARY);
        if let Some(summary) = &self.summary {
            let y = origin.y + self.galley.size().y + t::SPACE_4;
            painter.galley(pos2(origin.x, y), summary.clone(), t::STATUS_FAILED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<Word> {
        text.split_whitespace()
            .enumerate()
            .map(|(i, text)| Word {
                text: text.into(),
                start_ms: i as u64 * 100,
                end_ms: (i + 1) as u64 * 100,
            })
            .collect()
    }

    #[test]
    fn mapping_requires_equal_counts_and_ordered_normalized_words() {
        assert_eq!(
            word_mapping("one two", &words("one two")),
            Some(vec![0..3, 4..7])
        );
        assert_eq!(word_mapping("one two", &words("one")), None);
        assert_eq!(word_mapping("one", &words("one two")), None);
        assert_eq!(word_mapping("one two", &words("two one")), None);
        assert_eq!(
            word_mapping("“HELLO,”\nWorld!", &words("hello world")),
            Some(vec![0..12, 13..19])
        );
        assert_eq!(
            active_range("Hello, WORLD!", &words("hello world"), 150),
            Some(7..13)
        );
        assert_eq!(
            active_range("Hello, WORLD!", &words("hello world"), 250),
            None
        );
    }

    #[test]
    fn transcripts_grow_to_full_wrapped_text() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let short = Transcript::measure_simple(
                ui,
                "spoken",
                t::INSPECTOR_WIDTH_SELECTED_TURN,
                Some(0..6),
            );
            assert_eq!(short.galley.rows.len(), 1);
            assert_eq!(short.pills.len(), 1);
            let three = Transcript::measure_simple(
                ui,
                &"spoken word phrase ".repeat(12),
                t::INSPECTOR_WIDTH_SELECTED_TURN,
                None,
            );
            let six = Transcript::measure_simple(
                ui,
                &"spoken word phrase ".repeat(40),
                t::INSPECTOR_WIDTH_SELECTED_TURN,
                None,
            );
            assert!(
                three.galley.rows.len() >= 3,
                "rows={}",
                three.galley.rows.len()
            );
            assert!(six.galley.rows.len() >= 6, "rows={}", six.galley.rows.len());
            assert!(three.height() > short.height());
            assert!(six.height() > three.height());
            assert!(
                six.galley
                    .rows
                    .last()
                    .unwrap()
                    .glyphs
                    .last()
                    .map(|g| g.chr != '…')
                    .unwrap_or(true)
            );
            six.paint(
                ui,
                Rect::from_min_size(ui.max_rect().min, egui::Vec2::splat(t::SETTINGS_WIDTH)),
            );
        });
        output.textures_delta.clear();
    }

    #[test]
    fn inline_summary_increases_row_height() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let text = "An agent decides when and how to use it.";
            let plain = Transcript::measure_simple(ui, text, 320.0, None);
            let marked = Transcript::measure(
                ui,
                text,
                320.0,
                None,
                &[(27, 30), (31, 34)],
                Some(NotHeardStyle::Inline),
            );
            assert!(marked.summary.is_some());
            assert!(
                marked.height() > plain.height(),
                "plain={} marked={}",
                plain.height(),
                marked.height()
            );
        });
        output.textures_delta.clear();
    }

    #[test]
    fn pill_label_adds_layout_glyphs_and_danger_pills() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let text = "bias training or diversity education on the loop.";
            let plain = Transcript::measure_simple(ui, text, 280.0, None);
            let marked = Transcript::measure(
                ui,
                text,
                280.0,
                None,
                &[(5, 13), (44, 49)],
                Some(NotHeardStyle::Pill),
            );
            let glyph_count = |t: &Transcript| {
                t.galley
                    .rows
                    .iter()
                    .map(|row| row.glyphs.len())
                    .sum::<usize>()
            };
            assert!(
                glyph_count(&marked) > glyph_count(&plain),
                "plain={} marked={}",
                glyph_count(&plain),
                glyph_count(&marked)
            );
            assert!(
                marked
                    .pills
                    .iter()
                    .any(|(_, kind)| *kind == PillKind::Danger),
                "expected danger pills"
            );
        });
        output.textures_delta.clear();
    }

    #[test]
    fn pill_job_inserts_label_before_following_words() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let text =
                "like implicit bias training or diversity education, works on the loop. It should";
            let t0 = text.find("training").unwrap();
            let t1 = t0 + "training".len();
            let l0 = text.find("loop.").unwrap();
            let l1 = l0 + "loop.".len();
            let marked = Transcript::measure(
                ui,
                text,
                420.0,
                None,
                &[(t0, t1), (l0, l1)],
                Some(NotHeardStyle::Pill),
            );
            let mut laid = String::new();
            for row in &marked.galley.rows {
                for g in &row.glyphs {
                    laid.push(g.chr);
                }
                laid.push('|');
            }
            assert!(
                laid.contains("training not heard"),
                "label missing after training: {laid}"
            );
            let idx = laid.find("training not heard").unwrap();
            let after = &laid[idx..];
            let label_at = after.find("not heard").unwrap();
            let or_at = after
                .find("or")
                .unwrap_or_else(|| panic!("missing or in {after}"));
            assert!(label_at < or_at, "or should follow label: {after}");

            // Walk glyphs (no row markers): after the first "not heard", the next
            // letters are " or"; danger pills on that row must end before 'o'.
            let mut seen_first_label = false;
            let mut label_tail = 0usize;
            let mut or_left: Option<(f32, f32)> = None;
            let needle: Vec<char> = " not heard".chars().collect();
            let mut window: Vec<char> = Vec::new();
            for row in &marked.galley.rows {
                for glyph in &row.glyphs {
                    window.push(glyph.chr);
                    if window.len() > needle.len() {
                        window.remove(0);
                    }
                    if !seen_first_label && window.as_slice() == needle.as_slice() {
                        seen_first_label = true;
                        label_tail = 0;
                        continue;
                    }
                    if seen_first_label && or_left.is_none() {
                        label_tail += 1;
                        if glyph.chr == 'o' && label_tail > 0 {
                            or_left = Some((row.pos.x + glyph.pos.x, row.pos.y));
                            break;
                        }
                    }
                }
                if or_left.is_some() {
                    break;
                }
            }
            let (or_x, row_y) = or_left.expect("or glyph after first label");
            for (pill, kind) in &marked.pills {
                if *kind != PillKind::Danger {
                    continue;
                }
                if pill.bottom() < row_y || pill.top() > row_y + 20.0 {
                    continue;
                }
                assert!(
                    pill.right() <= or_x + 0.5,
                    "danger pill overlaps following word: right={} or_x={or_x} laid={laid}",
                    pill.right()
                );
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn danger_pill_covers_word_and_label_glyphs() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let text = "bias training or diversity on the loop. next";
            let t0 = text.find("training").unwrap();
            let l0 = text.find("loop.").unwrap();
            let marked = Transcript::measure(
                ui,
                text,
                500.0,
                None,
                &[(t0, t0 + 8), (l0, l0 + 5)],
                Some(NotHeardStyle::Pill),
            );
            let covered = |pill: Rect| {
                let mut s = String::new();
                for row in &marked.galley.rows {
                    for glyph in &row.glyphs {
                        let gx = row.pos.x + glyph.pos.x + glyph.advance_width / 2.0;
                        let gy = row.pos.y + row.size.y / 2.0;
                        if pill.contains(pos2(gx, gy)) {
                            s.push(glyph.chr);
                        }
                    }
                }
                s
            };
            let danger: Vec<_> = marked
                .pills
                .iter()
                .filter(|(_, k)| *k == PillKind::Danger)
                .map(|(r, _)| covered(*r))
                .collect();
            assert!(
                danger
                    .iter()
                    .any(|s| s.contains("training") && s.contains("not heard")),
                "training pill glyphs: {danger:?}"
            );
            assert!(
                danger
                    .iter()
                    .any(|s| s.contains("loop.") && s.contains("not heard")),
                "loop pill glyphs: {danger:?}"
            );
            assert!(
                danger
                    .iter()
                    .all(|s| !s.contains("or") && !s.contains("diversity")),
                "pill must not cover following words: {danger:?}"
            );
        });
        output.textures_delta.clear();
    }
}
