//! Full-screen discrepancy review (Phase H4).
use crate::{Browser, view};
use eframe::egui::{self, Align, Layout, RichText, Sense, StrokeKind, Ui};
use scriptcast_ui::{
    discrepancy::{self, DiffCard, ReviewMode},
    runner::Preset,
    theme as t,
};

struct LeftMeta<'a> {
    index: usize,
    episode: &'a str,
    speaker: &'a str,
    start: &'a str,
    dur: &'a str,
    stress_line: &'a Option<String>,
    has_ensemble: bool,
    ensemble_notice: Option<&'a str>,
}

struct LeftActions<'a> {
    mode: &'a mut ReviewMode,
    mode_changed: &'a mut bool,
    edit: &'a mut bool,
    run_ensemble: &'a mut bool,
    seek_ms: &'a mut Option<u64>,
}

impl Browser {
    pub(crate) fn open_discrepancy_review(&mut self) {
        let Some(turn) = self.selected else {
            return;
        };
        let failed = self.episode.as_ref().is_some_and(|e| {
            e.turns.get(turn).is_some_and(|t| {
                t.verification == scriptcast_ui::episode::VerificationState::Failed
            })
        });
        if !failed {
            return;
        }
        self.discrepancy_open = true;
        self.discrepancy_card = 0;
    }

    pub(crate) fn close_discrepancy_review(&mut self) {
        self.discrepancy_open = false;
        self.discrepancy_card = 0;
    }

    pub(crate) fn save_discrepancy_decision(&mut self, ctx: &egui::Context) {
        let Some(turn) = self.selected else {
            return;
        };
        let Some(e) = self.episode.as_ref() else {
            return;
        };
        let Some(tdata) = e.turns.get(turn) else {
            return;
        };
        let turn_id = tdata.id.clone();
        let expected = tdata
            .verify
            .as_ref()
            .map(|v| v.expected.clone())
            .unwrap_or_default();
        let heard = tdata
            .verify
            .as_ref()
            .map(|v| v.heard.clone())
            .unwrap_or_default();
        let phonon = e
            .stress_ensemble
            .get(&turn_id)
            .and_then(|s| s.spans.first())
            .map(|s| s.phonon.clone());
        self.ensure_review_form(&turn_id, &expected, &heard);
        self.save_review_decision(&turn_id, &expected, &heard, phonon.as_deref(), ctx);
    }

    pub(crate) fn discrepancy_cards(&self) -> Vec<DiffCard> {
        let Some(turn) = self.selected else {
            return Vec::new();
        };
        let Some(e) = self.episode.as_ref() else {
            return Vec::new();
        };
        let Some(turn_data) = e.turns.get(turn) else {
            return Vec::new();
        };
        let expected = turn_data
            .verify
            .as_ref()
            .map(|v| v.expected.as_str())
            .unwrap_or("");
        let heard = turn_data
            .verify
            .as_ref()
            .map(|v| v.heard.as_str())
            .unwrap_or("");
        match self.preferences.discrepancy_mode {
            ReviewMode::WhisperOnly => discrepancy::whisper_cards(expected, heard),
            ReviewMode::Ensemble => {
                if let Some(second) = e.stress_ensemble.get(&turn_data.id) {
                    discrepancy::ensemble_cards(&second.spans)
                } else {
                    Vec::new()
                }
            }
        }
    }

    pub(crate) fn discrepancy_review(&mut self, ui: &mut Ui) {
        let turn = match self.selected {
            Some(t) => t,
            None => {
                self.close_discrepancy_review();
                return;
            }
        };
        let meta = {
            let e = self.episode.as_ref().unwrap();
            let tdata = &e.turns[turn];
            let speaker = e.speakers[tdata.speaker_idx].name.clone();
            let episode = e.manifest.episode_id.clone();
            let start = view::time(tdata.start_ms);
            let dur = format!(
                "{:.3} s",
                tdata.end_ms.saturating_sub(tdata.start_ms) as f64 / 1000.0
            );
            let index = tdata.index;
            let turn_id = tdata.id.clone();
            let stress_line = tdata.verify.as_ref().map(|v| {
                if v.stress.is_empty() {
                    "Stress checks · none".into()
                } else {
                    let parts: Vec<_> = v
                        .stress
                        .iter()
                        .map(|s| format!("{} · {} · {}", s.word, s.reading, s.verdict))
                        .collect();
                    format!("Stress checks · {}", parts.join(" · "))
                }
            });
            let score = tdata.verify.as_ref().and_then(|v| v.score);
            let expected = tdata
                .verify
                .as_ref()
                .map(|v| v.expected.clone())
                .unwrap_or_default();
            let heard = tdata
                .verify
                .as_ref()
                .map(|v| v.heard.clone())
                .unwrap_or_default();
            let ensemble_notice = e.stress_ensemble.notice.clone();
            let has_ensemble = e.stress_ensemble.get(&turn_id).is_some();
            (
                index,
                episode,
                speaker,
                start,
                dur,
                turn_id,
                stress_line,
                score,
                expected,
                heard,
                ensemble_notice,
                has_ensemble,
            )
        };
        let (
            index,
            episode,
            speaker,
            start,
            dur,
            turn_id,
            stress_line,
            score,
            expected,
            heard,
            ensemble_notice,
            has_ensemble,
        ) = meta;

        self.ensure_review_form(&turn_id, &expected, &heard);

        let mut back = false;
        let mut edit = false;
        let mut mode = self.preferences.discrepancy_mode;
        let mut mode_changed = false;
        let mut run_ensemble = false;
        let mut seek_ms: Option<u64> = None;
        let mut save = false;
        let mut cancel = false;

        let full = ui.max_rect();
        let mut top = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(egui::Rect::from_min_size(
                    full.min,
                    egui::vec2(full.width(), t::BUTTON_HEIGHT + t::SPACE_8),
                ))
                .layout(Layout::left_to_right(Align::Center)),
        );
        if t::secondary_button(&mut top, "Back to turns", None, true).clicked() {
            back = true;
        }
        let body = egui::Rect::from_min_max(
            egui::pos2(full.left(), top.min_rect().bottom() + t::SPACE_8),
            full.max,
        );
        let decision_w = t::DISCREPANCY_DECISION_WIDTH.min(body.width() * 0.4);
        let gap = t::SPACE_16;
        let left_rect = egui::Rect::from_min_max(
            body.min,
            egui::pos2(
                (body.right() - decision_w - gap).max(body.left()),
                body.bottom(),
            ),
        );
        let right_rect =
            egui::Rect::from_min_max(egui::pos2(left_rect.right() + gap, body.top()), body.max);
        self.discrepancy_left(
            &mut view::region(ui, "disc_left", left_rect, t::BG_WINDOW),
            LeftMeta {
                index,
                episode: &episode,
                speaker: &speaker,
                start: &start,
                dur: &dur,
                stress_line: &stress_line,
                has_ensemble,
                ensemble_notice: ensemble_notice.as_deref(),
            },
            LeftActions {
                mode: &mut mode,
                mode_changed: &mut mode_changed,
                edit: &mut edit,
                run_ensemble: &mut run_ensemble,
                seek_ms: &mut seek_ms,
            },
        );
        self.discrepancy_right(
            &mut view::region(ui, "disc_right", right_rect, t::BG_WINDOW),
            score,
            &mut save,
            &mut cancel,
        );

        if mode_changed {
            self.preferences.discrepancy_mode = mode;
            self.discrepancy_card = 0;
        }
        if back || cancel {
            self.close_discrepancy_review();
        }
        if edit {
            self.open_editor(turn);
        }
        if run_ensemble {
            self.request_job(Preset::SecondOpinion, None, ui.ctx());
        }
        if save {
            let phonon = self
                .episode
                .as_ref()
                .and_then(|e| e.stress_ensemble.get(&turn_id))
                .and_then(|s| s.spans.first())
                .map(|s| s.phonon.clone());
            self.save_review_decision(&turn_id, &expected, &heard, phonon.as_deref(), ui.ctx());
        }
        if let Some(ms) = seek_ms {
            self.seek_discrepancy(turn, ms);
        }
    }

    fn discrepancy_left(&mut self, ui: &mut Ui, meta: LeftMeta<'_>, actions: LeftActions<'_>) {
        ui.label(t::heading(
            format!("Discrepancy review · Turn {:03}", meta.index),
            t::SECTION_TITLE,
        ));
        ui.label(t::text(
            format!(
                "{} · {} · {} · {}",
                meta.episode, meta.speaker, meta.start, meta.dur
            ),
            t::CAPTION,
            t::TEXT_MUTED,
        ));
        ui.add_space(t::SPACE_8);
        ui.horizontal(|ui| {
            for candidate in [ReviewMode::WhisperOnly, ReviewMode::Ensemble] {
                let selected = *actions.mode == candidate;
                let label = candidate.label();
                let response =
                    ui.selectable_label(selected, t::text(label, t::BODY, t::TEXT_PRIMARY));
                if response.clicked() && !selected {
                    *actions.mode = candidate;
                    *actions.mode_changed = true;
                }
            }
            ui.label(t::text(
                "Ensemble · not ground truth",
                t::CAPTION,
                t::TEXT_SUBTLE,
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if t::secondary_button(ui, "Edit script", None, true).clicked() {
                    *actions.edit = true;
                }
            });
        });
        ui.add_space(t::SPACE_8);
        ui.horizontal(|ui| {
            ui.label(t::text("Expected script", t::FIELD_LABEL, t::TEXT_MUTED));
            ui.add_space(ui.available_width() / 2.0 - t::SPACE_20);
            ui.label(t::text("Recognizer heard", t::FIELD_LABEL, t::TEXT_MUTED));
        });
        ui.add_space(t::SPACE_4);

        let cards = self.discrepancy_cards();
        if *actions.mode == ReviewMode::Ensemble && !meta.has_ensemble {
            let hint = meta.ensemble_notice.unwrap_or(
                "No second opinion yet for failed turns. \
                 Run Jobs → Second opinion (failed turns).",
            );
            ui.label(t::text(hint, t::BODY, t::TEXT_MUTED));
            let s = self.spec(&Preset::SecondOpinion);
            let run_hint = s
                .as_ref()
                .map(|s| s.resolved.description())
                .unwrap_or_else(|e| e.clone());
            if t::secondary_button(ui, "Run…", None, s.is_ok())
                .on_disabled_hover_text(&run_hint)
                .clicked()
            {
                *actions.run_ensemble = true;
            }
        }

        if self.discrepancy_card >= cards.len() && !cards.is_empty() {
            self.discrepancy_card = cards.len() - 1;
        }
        let selected = self.discrepancy_card;
        let n = cards.len();
        let turn_id = self
            .selected
            .and_then(|i| self.episode.as_ref().map(|e| e.turns[i].id.clone()));
        let words = turn_id.and_then(|id| {
            self.episode
                .as_ref()
                .and_then(|e| e.read_along.words(&id).map(|w| w.to_vec()))
        });

        let footer_h = t::SPACE_20 * 3.0;
        let scroll_h = (ui.available_height() - footer_h).max(t::SPACE_20 * 2.0);
        egui::ScrollArea::vertical()
            .id_salt("discrepancy_cards")
            .max_height(scroll_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, card) in cards.iter().enumerate() {
                    let is_sel = i == selected;
                    let stroke = if is_sel {
                        egui::Stroke::new(t::BORDER * 2.0, t::accent())
                    } else {
                        egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT)
                    };
                    let frame = egui::Frame::new()
                        .fill(t::BG_RAISED)
                        .stroke(stroke)
                        .corner_radius(t::RADIUS_CARD as u8)
                        .inner_margin(t::SPACE_12);
                    let response = frame
                        .show(ui, |ui| {
                            paint_diff_card(ui, card, i + 1, n);
                        })
                        .response
                        .interact(Sense::click());
                    if response.clicked() {
                        self.discrepancy_card = i;
                        if let Some(words) = &words
                            && let Some(ms) = discrepancy::seek_ms_for_card(card, words)
                        {
                            *actions.seek_ms = Some(ms);
                        }
                    }
                    ui.add_space(t::SPACE_8);
                }
            });

        ui.add_space(t::SPACE_8);
        ui.label(t::text(
            "Agreement is a clue, not a verdict. Listen before deciding.",
            t::CAPTION,
            t::TEXT_SUBTLE,
        ));
        if let Some(line) = meta.stress_line {
            ui.label(t::text(line, t::CAPTION, t::TEXT_MUTED));
        }
    }

    fn discrepancy_right(
        &mut self,
        ui: &mut Ui,
        score: Option<f64>,
        save: &mut bool,
        cancel: &mut bool,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("disc_right_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.discrepancy_right_inner(ui, score, save, cancel);
            });
    }

    fn discrepancy_right_inner(
        &mut self,
        ui: &mut Ui,
        score: Option<f64>,
        save: &mut bool,
        cancel: &mut bool,
    ) {
        ui.label(t::heading("Why did this turn fail?", t::SECTION_TITLE));
        let status = if self.review.as_ref().is_some_and(|f| f.saved) {
            "Manual decision · saved"
        } else {
            "Manual decision · not saved"
        };
        ui.label(t::text(status, t::CAPTION, t::TEXT_MUTED));
        ui.add_space(t::SPACE_8);

        let Some(form) = self.review.as_mut() else {
            return;
        };
        for outcome in scriptcast_ui::review::Outcome::all() {
            let selected = form.outcome == outcome;
            let stroke = if selected {
                egui::Stroke::new(t::BORDER, t::accent())
            } else {
                egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT)
            };
            let frame = egui::Frame::new()
                .fill(t::BG_RAISED)
                .stroke(stroke)
                .corner_radius(t::RADIUS_CONTROL as u8)
                .inner_margin(t::SPACE_10);
            let response = frame
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot, _) = ui.allocate_exact_size(
                            egui::vec2(t::SPACE_12, t::SPACE_12),
                            Sense::hover(),
                        );
                        ui.painter().circle_stroke(dot.center(), 5.0, stroke);
                        if selected {
                            ui.painter().circle_filled(dot.center(), 3.0, t::accent());
                        }
                        ui.vertical(|ui| {
                            ui.label(t::text(outcome.card_title(), t::BODY, t::TEXT_PRIMARY));
                            ui.label(t::text(outcome.card_detail(), t::CAPTION, t::TEXT_MUTED));
                        });
                    });
                })
                .response
                .interact(Sense::click());
            if response.clicked() {
                scriptcast_ui::review::choose_outcome(form, outcome);
            }
            ui.add_space(t::SPACE_6);
        }

        let can_save = !form.busy && self.review_save.is_none() && !self.jobs.busy();
        ui.horizontal(|ui| {
            if t::primary_button(ui, "Save decision", None, can_save).clicked() {
                *save = true;
            }
            if t::secondary_button(ui, "Cancel", None, true).clicked() {
                *cancel = true;
            }
        });
        ui.label(t::text(
            "Saved decisions can be undone. Saving never replaces audio. \
             Regeneration requires confirmation.",
            t::CAPTION,
            t::TEXT_SUBTLE,
        ));

        ui.add_space(t::SPACE_12);
        let score_line = match score {
            Some(s) => format!("{s:.3} / 0.90 · Original audio retained"),
            None => "Original audio retained".into(),
        };
        egui::CollapsingHeader::new("Provenance")
            .id_salt("disc_provenance")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(t::text(score_line, t::BODY, t::TEXT_PRIMARY));
            });
        egui::CollapsingHeader::new("Corrected transcription draft")
            .id_salt("disc_correction")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(t::text(
                    "Reviewer correction — raw recognition retained",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                let response = ui.add(
                    egui::TextEdit::multiline(&mut form.correction)
                        .desired_width(ui.available_width())
                        .desired_rows(2),
                );
                if response.changed() {
                    form.saved = false;
                }
            });
        egui::CollapsingHeader::new("Save for model improvement")
            .id_salt("disc_feedback")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(t::text(
                    "Opt-in off · no automatic retraining",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                let mut off = false;
                ui.add_enabled_ui(false, |ui| {
                    ui.checkbox(
                        &mut off,
                        t::text("Save for model improvement", t::BODY, t::TEXT_PRIMARY),
                    )
                    .on_disabled_hover_text("Not available yet");
                });
            });

        if form.saved {
            ui.label(t::text(
                scriptcast_ui::review::SAVED_BANNER,
                t::BODY,
                t::STATUS_PASSED,
            ));
        }
        if let Some(notice) = &form.notice
            && !form.saved
        {
            ui.label(t::text(notice, t::CAPTION, t::STATUS_FAILED));
        }
    }

    fn seek_discrepancy(&mut self, turn: usize, ms: u64) {
        let live = self.episode.is_some()
            && matches!(&self.audio, Some(Ok(_)))
            && self.device_error.is_none();
        if !live {
            return;
        }
        let pos = std::time::Duration::from_millis(ms);
        match self.transport {
            crate::Transport::Playing(_) | crate::Transport::Paused(_)
                if self.transport.turn() == Some(turn) =>
            {
                if let Some(Ok(audio)) = self.audio.as_mut() {
                    audio.seek(pos);
                }
            }
            _ => {
                self.select(turn);
                self.play_selected();
                if matches!(self.transport, crate::Transport::Playing(_))
                    && let Some(Ok(audio)) = self.audio.as_mut()
                {
                    audio.seek(pos);
                }
            }
        }
    }

    pub(crate) fn discrepancy_audio_strip(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, t::BG_SURFACE);
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        );
        let Some(turn) = self.selected else {
            return;
        };
        let index = self
            .episode
            .as_ref()
            .and_then(|e| e.turns.get(turn))
            .map(|t| t.index)
            .unwrap_or(turn);
        let state = match self.transport {
            crate::Transport::Playing(_) => "Playing",
            crate::Transport::Paused(_) => "Paused",
            crate::Transport::Idle => "Paused",
        };
        let mut row = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(view::inset(rect, t::SPACE_20, 0.0))
                .layout(Layout::left_to_right(Align::Center)),
        );
        row.label(t::text(
            format!("Turn {index:03} · Original rendered audio · {state}"),
            t::BODY,
            t::TEXT_PRIMARY,
        ));
        let track = egui::Rect::from_min_max(
            egui::pos2(
                row.cursor().left() + t::SPACE_16,
                rect.center().y - t::WAVE_VIEWPORT_HEIGHT / 2.0,
            ),
            egui::pos2(
                rect.right() - t::SPACE_20,
                rect.center().y + t::WAVE_VIEWPORT_HEIGHT / 2.0,
            ),
        );
        row.painter().rect(
            track,
            t::RADIUS_WAVE_VIEWPORT,
            t::BG_WINDOW,
            egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT),
            StrokeKind::Inside,
        );
        if let Some(env) = self.waveforms.as_ref().and_then(|w| w.get(turn)) {
            let dur_sec = self
                .selected_media
                .as_ref()
                .and_then(|(_, m)| m.duration)
                .map(|d| d.as_secs_f32())
                .unwrap_or(0.0);
            let bar_space = t::WAVE_BAR + t::WAVE_GAP;
            let n = (track.width() / bar_space) as usize;
            let max_h = track.height() * t::WAVE_VIEWPORT_FILL;
            let painter = row.painter().with_clip_rect(track);
            if n > 0 && dur_sec > 0.0 {
                for i in 0..n {
                    let t0 = (i as f32) / n as f32 * dur_sec;
                    let t1 = ((i + 1) as f32) / n as f32 * dur_sec;
                    let Some(peak) = scriptcast_ui::waveform::peak_in_range(env, t0, t1, dur_sec)
                    else {
                        continue;
                    };
                    let h = (peak * max_h).max(t::WAVE_MIN);
                    let x = track.left() + i as f32 * bar_space;
                    let y = track.center().y - h / 2.0;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(t::WAVE_BAR, h)),
                        0.0,
                        t::TEXT_PRIMARY,
                    );
                }
            }
        }
    }
}

fn paint_diff_card(ui: &mut Ui, card: &DiffCard, index: usize, n: usize) {
    ui.horizontal(|ui| {
        let half = ui.available_width() / 2.0 - t::SPACE_8;
        ui.allocate_ui_with_layout(
            egui::vec2(half, ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(half);
                paint_phrase(ui, &card.expected, &card.heard);
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(half, ui.available_height()),
            Layout::top_down(Align::Min),
            |ui| {
                ui.set_width(half);
                ui.horizontal(|ui| {
                    paint_phrase(ui, &card.heard, &card.expected);
                    if let Some(source) = card.source {
                        ui.label(
                            RichText::new(source)
                                .font(t::font(t::CAPTION))
                                .color(t::accent()),
                        );
                    }
                });
            },
        );
    });
    ui.label(t::text(
        format!("Difference {index} of {n}"),
        t::CAPTION,
        t::TEXT_MUTED,
    ));
    ui.label(
        RichText::new(card.verdict)
            .font(t::font(t::BODY))
            .color(t::TEXT_PRIMARY)
            .strong(),
    );
    if let Some(hint) = card.hint {
        ui.label(t::text(hint, t::CAPTION, t::TEXT_MUTED));
    }
    for warning in &card.warnings {
        ui.label(t::text(warning, t::CAPTION, t::STATUS_FAILED));
    }
}

fn paint_phrase(ui: &mut Ui, value: &str, reference: &str) {
    let reference: std::collections::HashSet<String> = discrepancy::normalize_tokens(reference)
        .into_iter()
        .collect();
    let mut job = egui::text::LayoutJob::default();
    let tokens = discrepancy::normalize_tokens(value);
    if tokens.is_empty() {
        job.append(
            "—",
            0.0,
            egui::TextFormat {
                font_id: t::font(t::BODY),
                color: t::TEXT_SUBTLE,
                ..Default::default()
            },
        );
    } else {
        for (i, word) in tokens.iter().enumerate() {
            if i > 0 {
                job.append(
                    " ",
                    0.0,
                    egui::TextFormat {
                        font_id: t::font(t::BODY),
                        color: t::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
            let differs = !reference.contains(word);
            job.append(
                word,
                0.0,
                egui::TextFormat {
                    font_id: t::font(t::BODY),
                    color: if differs {
                        t::STATUS_FAILED
                    } else {
                        t::TEXT_PRIMARY
                    },
                    ..Default::default()
                },
            );
        }
    }
    ui.label(job);
}
