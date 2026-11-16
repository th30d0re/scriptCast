//! R7: collapsible AI sidebar. Requests go to a [`RouterClient`]; the first
//! real client only appends them to a jsonl queue, because the Laya router
//! (R1) does not exist yet. Design: Figma `FyNyDUIlkH6Kjqvc3lcR2L` node 1:2.
use std::path::PathBuf;

use eframe::egui::{self, Rect, Ui, UiBuilder, vec2};

pub use crate::router_client::{
    Delivery, JsonlQueueClient, MemoryClient, RouterClient, RouterRequest, Source,
};
use crate::theme as t;

pub const WIDTH: f32 = 320.0;
pub const RAIL_WIDTH: f32 = 36.0;

/// Speech-to-text behind the mic button.
pub trait SpeechToText {
    /// Record and transcribe one utterance.
    fn dictate(&mut self) -> Result<String, String>;
}

/// Placeholder until mlx-whisper is wired (needs a model download).
pub struct SttStub;

impl SpeechToText for SttStub {
    fn dictate(&mut self) -> Result<String, String> {
        Err("Dictation needs mlx-whisper; not wired yet.".into())
    }
}

fn now_string() -> String {
    if crate::FIXED_CLOCK.load(std::sync::atomic::Ordering::Relaxed) {
        return "2026-01-01T00:00:00Z".into();
    }
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl Default for AiSidebar {
    fn default() -> Self {
        Self::new(
            Box::new(JsonlQueueClient::new(JsonlQueueClient::default_path())),
            Box::new(SttStub),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Author {
    You(Source),
    Router,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub author: Author,
    pub text: String,
    pub status: Option<String>,
}

pub struct AiSidebar {
    pub expanded: bool,
    /// No rail at all (headless snapshots of other screens).
    pub hidden: bool,
    pub enabled: bool,
    pub draft: String,
    pub messages: Vec<Message>,
    next_id: u64,
    client: Box<dyn RouterClient>,
    stt: Box<dyn SpeechToText>,
}

impl AiSidebar {
    pub fn new(client: Box<dyn RouterClient>, stt: Box<dyn SpeechToText>) -> Self {
        Self {
            expanded: false,
            hidden: false,
            enabled: true,
            draft: String::new(),
            messages: Vec::new(),
            next_id: 1,
            client,
            stt,
        }
    }

    /// Width the sidebar takes from the left edge.
    pub fn width(&self) -> f32 {
        if self.hidden {
            return 0.0;
        }
        if self.expanded { WIDTH } else { RAIL_WIDTH }
    }

    /// Send `text`. Empty text, or AI off, does nothing.
    pub fn submit(&mut self, text: &str, source: Source, episode: Option<PathBuf>) {
        let text = text.trim();
        if text.is_empty() || !self.enabled {
            return;
        }
        let request = RouterRequest {
            id: self.next_id,
            text: text.to_string(),
            source,
            episode,
            submitted_at: now_string(),
        };
        self.next_id += 1;
        let delivery = self.client.submit(&request);
        let status = match &delivery {
            Delivery::Queued => "queued".to_string(),
            Delivery::Failed(_) => "failed".to_string(),
        };
        self.messages.push(Message {
            author: Author::You(source),
            text: request.text,
            status: Some(status),
        });
        let reply = match delivery {
            Delivery::Queued => {
                "Queued for the Laya router. It is not built yet, so nothing runs until R1 lands."
                    .to_string()
            }
            Delivery::Failed(error) => format!("Could not queue: {error}"),
        };
        self.messages.push(Message {
            author: Author::Router,
            text: reply,
            status: None,
        });
    }

    fn dictate(&mut self, episode: Option<PathBuf>) {
        match self.stt.dictate() {
            Ok(text) => self.submit(&text, Source::Voice, episode),
            Err(error) => self.messages.push(Message {
                author: Author::Router,
                text: error,
                status: None,
            }),
        }
    }

    /// Paint into `rect` (the full left strip, height of the window).
    pub fn show(&mut self, ui: &mut Ui, rect: Rect, episode: Option<PathBuf>) {
        if self.hidden {
            return;
        }
        ui.painter().rect_filled(rect, 0, t::BG_SURFACE);
        ui.painter().vline(
            rect.right() - 0.5,
            rect.y_range(),
            egui::Stroke::new(1.0, t::BORDER_DEFAULT),
        );
        let inner = rect.shrink2(vec2(t::SPACE_12, t::SPACE_10));
        let mut ui = ui.new_child(UiBuilder::new().max_rect(if self.expanded {
            inner
        } else {
            rect.shrink(t::SPACE_4)
        }));
        if !self.expanded {
            ui.vertical_centered(|ui| {
                if ui
                    .add(egui::Button::new(t::text("»", t::TEXT_15, t::TEXT_MUTED)).frame(false))
                    .on_hover_text("Show assistant")
                    .clicked()
                {
                    self.expanded = true;
                }
                let colour = if self.enabled {
                    t::accent()
                } else {
                    t::TEXT_SUBTLE
                };
                ui.label(
                    egui::RichText::new("AI")
                        .font(t::bold(t::TEXT_11))
                        .color(colour),
                );
            });
            return;
        }
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(t::text("«", t::TEXT_15, t::TEXT_MUTED)).frame(false))
                .on_hover_text("Hide assistant")
                .clicked()
            {
                self.expanded = false;
            }
            ui.label(t::heading("Assistant", t::TEXT_15));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if self.enabled { "AI on" } else { "AI off" };
                let toggle = if self.enabled {
                    t::primary_button(ui, label, None, true)
                } else {
                    t::secondary_button(ui, label, None, true)
                };
                if toggle.clicked() {
                    self.enabled = !self.enabled;
                }
            });
        });
        ui.add_space(t::SPACE_8);
        // Composer is laid out bottom-up so it always fits; the list takes the rest.
        let enabled = self.enabled;
        let mut send = false;
        let mut dictate = false;
        let describe = self.client.describe();
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.label(t::text(describe, t::TEXT_11, t::TEXT_SUBTLE));
            ui.horizontal(|ui| {
                dictate = t::secondary_button(ui, "Dictate", None, enabled)
                    .on_hover_text("Speech to text with mlx-whisper")
                    .clicked();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ready = enabled && !self.draft.trim().is_empty();
                    send |= t::primary_button(ui, "Send", None, ready).clicked();
                });
            });
            ui.add_space(t::SPACE_4);
            let edit = ui.add_enabled(
                enabled,
                egui::TextEdit::multiline(&mut self.draft)
                    .hint_text("Ask the router… (Cmd+Enter to send)")
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            send |= edit.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter));
            ui.add_space(t::SPACE_8);
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("ai_sidebar_messages")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.messages.is_empty() {
                            ui.label(t::text(
                                "Requests you send here are queued for the Laya router.",
                                t::TEXT_13,
                                t::TEXT_SUBTLE,
                            ));
                        }
                        for message in &self.messages {
                            message_card(ui, message);
                            ui.add_space(t::SPACE_8);
                        }
                    });
            });
        });
        if send {
            let text = std::mem::take(&mut self.draft);
            self.submit(&text, Source::Typed, episode.clone());
        }
        if dictate {
            self.dictate(episode);
        }
    }
}

fn message_card(ui: &mut Ui, message: &Message) {
    let fill = match message.author {
        Author::You(_) => t::BG_SELECTED,
        Author::Router => t::BG_RAISED,
    };
    egui::Frame::new()
        .fill(fill)
        .corner_radius(t::RADIUS_CONTROL)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let who = match message.author {
                    Author::You(Source::Typed) => "You",
                    Author::You(Source::Voice) => "You (voice)",
                    Author::You(Source::Selection) => "You (selection)",
                    Author::Router => "Router",
                };
                ui.label(
                    egui::RichText::new(who)
                        .font(t::bold(t::TEXT_11))
                        .color(t::TEXT_MUTED),
                );
                if let Some(status) = &message.status {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(t::text(status, t::TEXT_11, t::STATUS_WARNING));
                    });
                }
            });
            ui.add(egui::Label::new(t::text(&message.text, t::TEXT_13, t::TEXT_PRIMARY)).wrap());
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_client_appends_one_line_per_request() {
        let dir = std::env::temp_dir().join(format!("r7-queue-{}", std::process::id()));
        let path = dir.join("queue.jsonl");
        let _ = std::fs::remove_dir_all(&dir);
        let mut sidebar = AiSidebar::new(Box::new(JsonlQueueClient::new(&path)), Box::new(SttStub));
        sidebar.submit("first", Source::Typed, None);
        sidebar.submit("  ", Source::Typed, None);
        sidebar.submit("second", Source::Voice, Some("/ep".into()));
        let lines: Vec<RouterRequest> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].text, "second");
        assert_eq!(lines[1].source, Source::Voice);
        assert_eq!(lines[1].id, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn disabled_sidebar_sends_nothing_and_stub_reports() {
        let mut sidebar = AiSidebar::new(Box::new(MemoryClient::default()), Box::new(SttStub));
        sidebar.enabled = false;
        sidebar.submit("hello", Source::Typed, None);
        assert!(sidebar.messages.is_empty());
        sidebar.enabled = true;
        sidebar.dictate(None);
        assert_eq!(sidebar.messages.len(), 1);
        assert!(sidebar.messages[0].text.contains("mlx-whisper"));
    }
}
