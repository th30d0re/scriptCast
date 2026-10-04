//! Measured two-line read-along text, painted inside the table's virtual items.
use eframe::egui::{self, Rect, Ui, pos2, vec2};
use scriptcast_ui::{
    theme as t,
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

pub struct Transcript {
    pub galley: Arc<egui::Galley>,
    pills: Vec<Rect>,
}

impl Transcript {
    pub fn measure(ui: &Ui, text: &str, width: f32, active: Option<Range<usize>>) -> Self {
        let normal = egui::TextFormat::simple(t::font(t::BODY), t::TEXT_PRIMARY);
        let mut job = egui::text::LayoutJob::default();
        if let Some(range) = &active {
            job.append(&text[..range.start], 0.0, normal.clone());
            job.append(
                &text[range.clone()],
                t::READ_ALONG_PAD_X,
                egui::TextFormat::simple(t::bold(t::BODY), t::ON_ACCENT),
            );
            job.append(&text[range.end..], t::READ_ALONG_PAD_X, normal);
        } else {
            job.append(text, 0.0, normal);
        }
        job.wrap.max_width = width.max(0.0);
        job.wrap.max_rows = t::READ_ALONG_LINES;
        // Reserve an ellipsis on the last line while more words remain.
        job.wrap.overflow_character = Some('…');
        let galley = ui.painter().layout_job(job);
        let mut pills = Vec::new();
        if let Some(range) = active {
            let chars = text[..range.start].chars().count()..text[..range.end].chars().count();
            let mut index = 0;
            for row in &galley.rows {
                let mut pill = Rect::NOTHING;
                for glyph in &row.glyphs {
                    // An elided glyph is synthetic and must never acquire a word highlight.
                    if chars.contains(&index) && glyph.chr != '…' {
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
                    pills.push(pill.expand2(vec2(t::READ_ALONG_PAD_X, t::READ_ALONG_PAD_Y)));
                }
            }
        }
        Self { galley, pills }
    }

    pub fn height(&self) -> f32 {
        let minimum = if self.galley.rows.len() > 1 {
            t::TABLE_ROW_HEIGHT_2
        } else {
            t::TABLE_ROW_HEIGHT
        };
        minimum.max(self.galley.size().y + t::SPACE_4 * 2.0)
    }

    pub fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let origin = pos2(rect.left(), rect.top() + t::SPACE_4);
        for pill in &self.pills {
            painter.rect_filled(
                pill.translate(origin.to_vec2()),
                t::READ_ALONG_RADIUS,
                t::accent(),
            );
        }
        painter.galley(origin, self.galley.clone(), t::TEXT_PRIMARY);
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
    fn transcripts_wrap_to_two_lines_and_ellipsize() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let short =
                Transcript::measure(ui, "spoken", t::INSPECTOR_WIDTH_SELECTED_TURN, Some(0..6));
            assert_eq!(short.galley.rows.len(), 1);
            assert_eq!(short.pills.len(), 1);
            let long = Transcript::measure(
                ui,
                &"spoken ".repeat(100),
                t::INSPECTOR_WIDTH_SELECTED_TURN,
                None,
            );
            assert!(long.height() > short.height());
            assert_eq!(long.galley.rows.len(), 2);
            assert_eq!(
                long.galley.rows.last().unwrap().glyphs.last().unwrap().chr,
                '…'
            );
            long.paint(
                ui,
                Rect::from_min_size(ui.max_rect().min, egui::Vec2::splat(t::SETTINGS_WIDTH)),
            );
        });
        output.textures_delta.clear();
    }
}
