//! Measured two-line read-along strip, painted inside the table's virtual items.
use eframe::egui::{self, Rect, Ui, pos2, vec2};
use scriptcast_ui::{
    theme as t,
    word_timings::{Word, current_word},
};
use std::sync::Arc;

struct Token {
    galley: Arc<egui::Galley>,
    rect: Rect,
    active: bool,
}

pub struct Strip {
    tokens: Vec<Token>,
    pub height: f32,
}

impl Strip {
    pub fn measure(ui: &Ui, words: &[Word], position: u64, width: f32) -> Self {
        let current = current_word(words, position);
        let padding = vec2(t::READ_ALONG_PAD_X, t::READ_ALONG_PAD_Y);
        let line_height = t::line_height(ui, t::bold(t::READ_ALONG_TEXT)) + padding.y * 2.0;
        let width = (width - t::SPACE_16 * 2.0).max(0.0);
        let ellipsis =
            ui.painter()
                .layout_no_wrap("…".into(), t::font(t::READ_ALONG_TEXT), t::TEXT_PRIMARY);
        let mut tokens = Vec::new();
        let mut x = 0.0;
        let mut line = 0;
        for (index, word) in words.iter().enumerate() {
            let active = current == Some(index);
            let font = if active {
                t::bold(t::READ_ALONG_TEXT)
            } else {
                t::font(t::READ_ALONG_TEXT)
            };
            let colour = if active {
                t::ON_ACCENT
            } else {
                t::TEXT_PRIMARY
            };
            let galley = ui.painter().layout_no_wrap(word.text.clone(), font, colour);
            let token_width = galley.size().x + if active { padding.x * 2.0 } else { 0.0 };
            if x > 0.0 && x + token_width > width && line == 0 {
                line += 1;
                x = 0.0;
            }
            // Reserve an ellipsis on the last line while more words remain.
            let reserve = if index + 1 < words.len() {
                ellipsis.size().x + t::SPACE_4
            } else {
                0.0
            };
            if x + token_width > width
                || (line == t::READ_ALONG_LINES - 1 && x + token_width + reserve > width)
            {
                tokens.push(Token {
                    rect: Rect::from_min_size(
                        pos2(x, line as f32 * line_height),
                        vec2(ellipsis.size().x, line_height),
                    ),
                    galley: ellipsis,
                    active: false,
                });
                break;
            }
            tokens.push(Token {
                rect: Rect::from_min_size(
                    pos2(x, line as f32 * line_height),
                    vec2(token_width, line_height),
                ),
                galley,
                active,
            });
            x += token_width + t::SPACE_4;
        }
        Self {
            tokens,
            height: (line + 1) as f32 * line_height + t::SPACE_4 * 2.0,
        }
    }

    pub fn paint(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.rect_filled(rect, 0, t::BG_SELECTED);
        let origin = rect.min + vec2(t::SPACE_16, t::SPACE_4);
        for token in &self.tokens {
            let token_rect = token.rect.translate(origin.to_vec2());
            if token.active {
                painter.rect_filled(token_rect, t::READ_ALONG_RADIUS, t::accent());
            }
            let offset = vec2(
                if token.active {
                    t::READ_ALONG_PAD_X
                } else {
                    0.0
                },
                (token_rect.height() - token.galley.size().y) / 2.0,
            );
            painter.galley(
                token_rect.min + offset,
                token.galley.clone(),
                t::TEXT_PRIMARY,
            );
        }
    }
}

/// Both expansions follow their turn; read-along precedes an editor on the same turn.
pub fn row_top(row: usize, editor: Option<(usize, f32)>, read_along: Option<(usize, f32)>) -> f32 {
    row as f32 * t::TABLE_ROW_HEIGHT
        + [editor, read_along]
            .into_iter()
            .flatten()
            .filter(|(index, _)| row > *index)
            .map(|(_, height)| height)
            .sum::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_rows_include_each_expansion_once() {
        let editor = Some((1, t::EDITOR_MIN_HEIGHT));
        let read = Some((1, t::TABLE_ROW_HEIGHT));
        assert_eq!(row_top(1, editor, read), t::TABLE_ROW_HEIGHT);
        assert_eq!(
            row_top(2, editor, read),
            t::TABLE_ROW_HEIGHT * 3.0 + t::EDITOR_MIN_HEIGHT
        );
        assert_eq!(
            row_top(5, editor, read),
            t::TABLE_ROW_HEIGHT * 6.0 + t::EDITOR_MIN_HEIGHT
        );
        assert_eq!(
            row_top(2, editor, Some((3, t::TABLE_ROW_HEIGHT))),
            t::TABLE_ROW_HEIGHT * 2.0 + t::EDITOR_MIN_HEIGHT
        );
        assert_eq!(row_top(5, None, None), t::TABLE_ROW_HEIGHT * 5.0);
    }

    #[test]
    fn strips_wrap_to_two_lines_and_ellipsize() {
        let ctx = egui::Context::default();
        t::apply(&ctx);
        let words: Vec<_> = (0..100)
            .map(|i| Word {
                text: "spoken".into(),
                start_ms: i * 100,
                end_ms: (i + 1) * 100,
            })
            .collect();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let short = Strip::measure(ui, &words[..1], 0, t::INSPECTOR_WIDTH_SELECTED_TURN);
            assert_eq!(short.tokens.len(), 1);
            assert!(short.tokens[0].active);
            let long = Strip::measure(ui, &words, 0, t::INSPECTOR_WIDTH_SELECTED_TURN);
            assert!(long.height > short.height);
            assert!(long.tokens.len() < words.len());
            assert_eq!(long.tokens.last().unwrap().galley.text(), "…");
            long.paint(
                ui,
                Rect::from_min_size(ui.max_rect().min, egui::Vec2::splat(t::SETTINGS_WIDTH)),
            );
        });
        output.textures_delta.clear();
    }
}
