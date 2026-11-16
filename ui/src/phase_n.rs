//! Phase N1 — full-window Mix editor.
use crate::{Browser, Transport, mix, theme as t, view};
use eframe::egui::{
    self, Align, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(crate) struct MixUi {
    pub open: bool,
    loaded_for: Option<PathBuf>,
    draft: mix::Mix,
    saved: mix::Mix,
    error: Option<String>,
    notice: Option<String>,
    ab_mixed: bool,
}

fn panel(ui: &mut Ui, id: &str, rect: Rect) -> Ui {
    ui.painter().rect(
        rect,
        t::RADIUS_CARD,
        t::BG_SURFACE,
        Stroke::new(t::BORDER, t::BORDER_DEFAULT),
        StrokeKind::Inside,
    );
    view::region(
        ui,
        id,
        view::inset(rect, t::SPACE_16, t::SPACE_12),
        t::BG_SURFACE,
    )
}

fn script_bodies(path: Option<&Path>) -> Vec<String> {
    let Some(text) = path.and_then(|p| fs::read_to_string(p).ok()) else {
        return vec![];
    };
    let header = regex::Regex::new(r"(?m)^.+\(\d{1,3}:\d{2}(?::\d{2})?\)\s*$").unwrap();
    let starts: Vec<_> = header.find_iter(&text).collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, hit)| {
            let end = starts.get(i + 1).map_or(text.len(), |next| next.start());
            text[hit.end()..end].trim().to_owned()
        })
        .collect()
}

fn matched_turn(selector: Option<&str>, bodies: &[String]) -> Option<(usize, String)> {
    let wanted = selector?.trim().to_lowercase();
    let hits: Vec<_> = bodies
        .iter()
        .enumerate()
        .filter(|(_, body)| body.to_lowercase().contains(&wanted))
        .collect();
    (hits.len() == 1).then(|| (hits[0].0, hits[0].1.clone()))
}

fn after_word_job(pattern: &str, body: &str) -> egui::text::LayoutJob {
    let words: Vec<_> = body.split_whitespace().collect();
    let hit = regex::Regex::new(pattern).ok().and_then(|pattern| {
        words
            .iter()
            .position(|word| pattern.is_match(&mix::normalize(word)))
    });
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "After word  ",
        0.0,
        egui::TextFormat {
            font_id: t::font(t::CAPTION),
            color: t::TEXT_MUTED,
            ..Default::default()
        },
    );
    if let Some(hit) = hit {
        let start = hit.saturating_sub(2);
        let end = (hit + 3).min(words.len());
        if start > 0 {
            job.append(
                "…",
                0.0,
                egui::TextFormat {
                    font_id: t::font(t::CAPTION),
                    color: t::TEXT_MUTED,
                    ..Default::default()
                },
            );
        }
        for (index, word) in words[start..end].iter().enumerate() {
            if index > 0 {
                job.append(
                    " ",
                    0.0,
                    egui::TextFormat {
                        font_id: t::font(t::CAPTION),
                        color: t::TEXT_MUTED,
                        ..Default::default()
                    },
                );
            }
            job.append(
                word,
                0.0,
                egui::TextFormat {
                    font_id: t::bold(t::CAPTION),
                    color: if start + index == hit {
                        t::accent()
                    } else {
                        t::TEXT_MUTED
                    },
                    ..Default::default()
                },
            );
        }
        if end < words.len() {
            job.append(
                "…",
                0.0,
                egui::TextFormat {
                    font_id: t::font(t::CAPTION),
                    color: t::TEXT_MUTED,
                    ..Default::default()
                },
            );
        }
    } else {
        job.append(
            pattern,
            0.0,
            egui::TextFormat {
                font_id: t::mono(t::CAPTION),
                color: t::TEXT_MUTED,
                ..Default::default()
            },
        );
    }
    job
}

impl Browser {
    pub(crate) fn open_mix(&mut self) {
        let Some(episode) = &self.episode else { return };
        let Some(root) = &episode.project_root else {
            self.jobs.notice = Some("Mix needs a project root containing scriptcast.toml".into());
            return;
        };
        let path = mix::config_path(root);
        match mix::load(&path) {
            Ok(draft) => {
                self.mix.saved = draft.clone();
                self.mix.draft = draft;
                self.mix.error = None;
                self.mix.notice = None;
                self.mix.loaded_for = Some(path);
                self.mix.open = true;
            }
            Err(error) => {
                self.mix.error = Some(error);
                self.mix.open = true;
                self.mix.loaded_for = Some(path);
            }
        }
    }

    fn mix_play(&mut self, path: PathBuf) {
        let result = self
            .audio
            .as_mut()
            .and_then(|result| result.as_mut().ok())
            .ok_or_else(|| "Audio output unavailable".to_owned())
            .and_then(|audio| audio.play(usize::MAX, &[path]));
        match result {
            Ok(()) => self.transport = Transport::Playing(usize::MAX),
            Err(error) => self.mix.notice = Some(error),
        }
    }

    fn mix_header(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        view::rule(ui, rect);
        let mut row = ui.new_child(
            UiBuilder::new()
                .max_rect(view::inset(rect, t::SPACE_20, 0.0))
                .layout(Layout::left_to_right(Align::Center)),
        );
        row.spacing_mut().item_spacing.x = t::SPACE_12;
        row.add(
            egui::Image::new(t::icon("audio-lines-22-accent"))
                .fit_to_exact_size(vec2(t::SPACE_22, t::SPACE_22)),
        );
        row.label(t::heading("scriptCast", t::BRAND));
        let (divider, _) =
            row.allocate_exact_size(vec2(t::BORDER, t::BUTTON_HEIGHT), Sense::hover());
        row.painter().rect_filled(divider, 0.0, t::BORDER_DEFAULT);
        if let Some(episode) = &self.episode {
            let project = episode.project_root.as_ref();
            let name = project
                .and_then(|p| p.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| episode.manifest.episode_id.clone());
            let root = project.map(|p| p.display().to_string()).unwrap_or_default();
            row.vertical(|ui| {
                ui.label(t::heading(name, t::TEXT_14));
                ui.label(t::text(root, t::CAPTION, t::TEXT_MUTED));
            });
        }
        let rest = row.available_rect_before_wrap();
        let mut actions = row.new_child(
            UiBuilder::new()
                .max_rect(rest)
                .layout(Layout::right_to_left(Align::Center)),
        );
        if t::secondary_button(&mut actions, "Project settings", None, true).clicked() {
            self.settings_open = true;
        }
    }

    fn mix_waveform(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        let Some(episode) = &self.episode else { return };
        view::line(
            ui,
            Rect::from_min_size(
                rect.min + vec2(t::SPACE_16, t::SPACE_8),
                vec2(rect.width(), 20.0),
            ),
            &format!(
                "{}  ·  waveform, overlays, gain, loudness",
                episode.manifest.episode_id
            ),
            t::mono(t::CAPTION),
            t::TEXT_MUTED,
        );
        let wave = Rect::from_min_max(
            rect.min + vec2(t::SPACE_16, 30.0),
            rect.max - vec2(t::SPACE_16, t::SPACE_8),
        );
        let count = episode.turns.len().max(1);
        for (i, turn) in episode.turns.iter().enumerate() {
            let slot = Rect::from_min_max(
                pos2(
                    wave.left() + wave.width() * i as f32 / count as f32,
                    wave.top(),
                ),
                pos2(
                    wave.left() + wave.width() * (i + 1) as f32 / count as f32,
                    wave.bottom(),
                ),
            );
            if let Some(env) = self.waveforms.as_ref().and_then(|waves| waves.get(i)) {
                let cols = crate::waveform::strip(
                    env,
                    ((turn.end_ms - turn.start_ms) as f32 / 1000.0).max(0.01),
                    0.08,
                );
                let step = slot.width() / cols.len().max(1) as f32;
                crate::waveform::paint_columns(
                    ui.painter(),
                    &cols,
                    slot.left(),
                    step,
                    (step * 0.55).max(1.0),
                    slot.center().y,
                    slot.height() * 0.45,
                    1.0,
                    ui.ctx().pixels_per_point(),
                    |_| t::BG_CONTROL,
                );
            }
        }
        let bodies = script_bodies(episode.script_path.as_deref());
        for overlay in &self.mix.draft.overlays {
            if let Some((index, _)) = matched_turn(overlay.selector(), &bodies) {
                let x = wave.left() + wave.width() * index as f32 / count as f32;
                ui.painter()
                    .circle_filled(pos2(x, wave.top() + 8.0), 4.0, t::STATUS_WARNING);
            }
        }
    }

    pub(crate) fn mix_screen(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, t::BG_WINDOW);
        let header = Rect::from_min_size(rect.min, vec2(rect.width(), t::EPISODE_HEADER_HEIGHT));
        let footer = Rect::from_min_max(pos2(rect.left(), rect.bottom() - 56.0), rect.max);
        let body = Rect::from_min_max(header.left_bottom(), footer.right_top());
        self.mix_header(&mut view::region(ui, "mix_header", header, t::BG_SURFACE));
        let loud =
            Rect::from_min_size(body.min + vec2(16.0, 12.0), vec2(body.width() - 32.0, 72.0));
        let mut loud_ui = panel(ui, "mix_loudness", loud);
        loud_ui.columns(4, |cols| {
            for (col, (label, value)) in cols.iter_mut().zip([
                ("Integrated", "—"),
                ("Short-term", "—"),
                ("True peak", "—"),
                ("Target", "−16 LUFS"),
            ]) {
                col.label(t::text(label, t::CAPTION, t::TEXT_MUTED));
                col.label(t::heading(value, t::TEXT_14));
            }
        });
        let timeline = Rect::from_min_size(
            pos2(loud.left(), loud.bottom() + 12.0),
            vec2(loud.width(), 96.0),
        );
        self.mix_waveform(&mut panel(ui, "mix_waveform", timeline));
        let content_top = timeline.bottom() + 12.0;
        let content_bottom = footer.top() - 12.0;
        let right_w = (body.width() * 0.37).max(360.0);
        let left = Rect::from_min_max(
            pos2(loud.left(), content_top),
            pos2(loud.right() - right_w - 12.0, content_bottom),
        );
        let right = Rect::from_min_max(
            pos2(left.right() + 12.0, content_top),
            pos2(loud.right(), content_bottom),
        );
        let gains_h = 112.0_f32.min(left.height() * 0.28);
        let overlays =
            Rect::from_min_max(left.min, pos2(left.right(), left.bottom() - gains_h - 12.0));
        let gains = Rect::from_min_max(pos2(left.left(), overlays.bottom() + 12.0), left.max);
        let bodies = self
            .episode
            .as_ref()
            .map(|e| script_bodies(e.script_path.as_deref()))
            .unwrap_or_default();
        let root = self.episode.as_ref().and_then(|e| e.project_root.clone());
        let mut play = None;
        let mut play_turn = None;
        let mut overlay_ui = panel(ui, "mix_overlays", overlays);
        overlay_ui.label(t::heading("Overlays", t::SECTION_TITLE));
        egui::ScrollArea::vertical().show(&mut overlay_ui, |ui| {
            let overlay_count = self.mix.draft.overlays.len();
            for (n, overlay) in self.mix.draft.overlays.iter_mut().enumerate() {
                let matched = matched_turn(overlay.selector(), &bodies);
                let failure = matched
                    .as_ref()
                    .and_then(|(i, body)| mix::unmatched_word_error(overlay, *i, body));
                let frame = egui::Frame::new()
                    .fill(if failure.is_some() {
                        t::BG_DANGER_ROW
                    } else {
                        t::BG_RAISED
                    })
                    .stroke(Stroke::new(
                        t::BORDER,
                        if failure.is_some() {
                            t::STATUS_FAILED
                        } else {
                            t::BORDER_DEFAULT
                        },
                    ))
                    .corner_radius(t::RADIUS_CARD as u8)
                    .inner_margin(t::SPACE_12);
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t::heading(&overlay.file, t::BODY));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Play").clicked() {
                                play = root.as_ref().map(|r| r.join(&overlay.file));
                            }
                        });
                    });
                    if let Some(error) = failure {
                        ui.label(t::text(error, t::BODY, t::STATUS_FAILED));
                    } else {
                        ui.label(t::text(
                            format!(
                                "Turn · {}",
                                overlay.selector().unwrap_or("missing selector")
                            ),
                            t::BODY,
                            t::TEXT_MUTED,
                        ));
                        if let Some((_, body)) = &matched {
                            ui.label(after_word_job(&overlay.after_word, body));
                        }
                        ui.label(t::text(
                            format!("Delay  {} ms", overlay.delay_ms),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    }
                    ui.horizontal(|ui| {
                        ui.label(t::text(
                            format!("Gain  {:.0} dB", overlay.gain_db),
                            t::BODY,
                            t::TEXT_PRIMARY,
                        ));
                        ui.add(
                            egui::Slider::new(
                                &mut overlay.gain_db,
                                mix::OVERLAY_GAIN_MIN..=mix::OVERLAY_GAIN_MAX,
                            )
                            .show_value(false),
                        );
                    });
                });
                if n + 1 < overlay_count {
                    ui.add_space(t::SPACE_12);
                }
            }
            if self.mix.draft.overlays.is_empty() {
                ui.label(t::text("No overlays in mix.yaml.", t::BODY, t::TEXT_MUTED));
            }
        });
        let mut gain_ui = panel(ui, "mix_gains", gains);
        gain_ui.label(t::heading("Turn gains", t::SECTION_TITLE));
        gain_ui.horizontal_wrapped(|ui| {
            for gain in &mut self.mix.draft.gains {
                ui.vertical(|ui| {
                    ui.label(t::text(
                        gain.selector().unwrap_or("Turn"),
                        t::CAPTION,
                        t::TEXT_MUTED,
                    ));
                    ui.label(t::text(
                        format!("{:+.1} dB", gain.gain_db),
                        t::BODY,
                        t::TEXT_PRIMARY,
                    ));
                    ui.add(
                        egui::Slider::new(
                            &mut gain.gain_db,
                            mix::TURN_GAIN_MIN..=mix::TURN_GAIN_MAX,
                        )
                        .show_value(false),
                    );
                    if ui.button("Play").clicked()
                        && let Some((index, _)) = matched_turn(gain.selector(), &bodies)
                    {
                        play_turn = Some(index);
                    }
                });
            }
        });
        let mut render = panel(ui, "mix_render", right);
        render.label(t::heading("Render mix", t::SECTION_TITLE));
        let render_spec = self.spec(&crate::runner::Preset::Mix);
        let can_render = render_spec.is_ok();
        let disabled_reason = render_spec.as_ref().err().cloned().unwrap_or_default();
        let asr_exists = self.episode.as_ref().is_some_and(|episode| {
            mix::asr_path(&episode.dir, &episode.manifest.episode_id).is_file()
        });
        if t::primary_button(&mut render, "Render mix", None, can_render)
            .on_disabled_hover_text(&disabled_reason)
            .clicked()
        {
            let path = self.mix.loaded_for.clone();
            match path
                .as_deref()
                .ok_or_else(|| "mix.yaml path is unavailable".to_owned())
                .and_then(|path| mix::save(path, &self.mix.draft))
            {
                Ok(()) => {
                    self.mix.saved = self.mix.draft.clone();
                    self.request_job(crate::runner::Preset::Mix, None, &ctx);
                }
                Err(error) => self.mix.error = Some(error),
            }
        }
        if !asr_exists {
            render.label(t::text(
                "Recognised words are missing.",
                t::CAPTION,
                t::STATUS_FAILED,
            ));
        } else if !can_render {
            render.label(t::text(disabled_reason, t::CAPTION, t::STATUS_FAILED));
        }
        render.add(
            egui::ProgressBar::new(self.render_progress().map_or(0.0, |p| {
                if p.total == 0 {
                    0.0
                } else {
                    p.n as f32 / p.total as f32
                }
            }))
            .show_percentage(),
        );
        if let Some(episode) = &self.episode {
            let mixed = mix::mixed_path(&episode.dir, &episode.manifest.episode_id);
            render.label(t::text("Output", t::CAPTION, t::TEXT_MUTED));
            render.label(
                egui::RichText::new(mixed.file_name().unwrap_or_default().to_string_lossy())
                    .font(t::mono(t::BODY))
                    .color(t::TEXT_PRIMARY),
            );
            render.add_space(t::SPACE_12);
            render.label(t::text("A/B", t::CAPTION, t::TEXT_MUTED));
            render.horizontal(|ui| {
                if ui
                    .selectable_label(!self.mix.ab_mixed, "Stitched")
                    .clicked()
                {
                    self.mix.ab_mixed = false;
                    if let Some(p) = stitched_path(&episode.dir) {
                        play = Some(p);
                    }
                }
                if ui
                    .add_enabled_ui(mixed.is_file(), |ui| {
                        ui.selectable_label(self.mix.ab_mixed, "Mixed")
                    })
                    .inner
                    .clicked()
                {
                    self.mix.ab_mixed = true;
                    play = Some(mixed);
                }
            });
            let (position, duration) = self
                .audio
                .as_ref()
                .and_then(|result| result.as_ref().ok())
                .map_or((0.0, 0.0), |a| {
                    (a.position().as_secs_f32(), a.duration().as_secs_f32())
                });
            let current = episode
                .turns
                .iter()
                .find(|turn| {
                    position * 1000.0 >= turn.start_ms as f32
                        && position * 1000.0 <= turn.end_ms as f32
                })
                .map(|turn| turn.index);
            render.label(t::text(
                format!(
                    "Turn {}   ·   {} / {}",
                    current.map_or("—".into(), |n| format!("{n:03}")),
                    clock(position),
                    clock(duration)
                ),
                t::BODY,
                t::TEXT_MUTED,
            ));
            let mut frac = if duration > 0.0 {
                position / duration
            } else {
                0.0
            };
            if render
                .add(egui::Slider::new(&mut frac, 0.0..=1.0).show_value(false))
                .changed()
                && let Some(audio) = self.audio.as_mut().and_then(|result| result.as_mut().ok())
            {
                audio.seek(std::time::Duration::from_secs_f32(duration * frac));
            }
        }
        if let Some(error) = self.mix.error.as_ref().or(self.mix.notice.as_ref()) {
            render.label(t::text(error, t::BODY, t::STATUS_FAILED));
        }
        if let Some(index) = play_turn {
            self.select(index);
            self.play_selected();
        }
        if let Some(path) = play {
            self.mix_play(path);
        }
        let mut foot = view::region(ui, "mix_footer", footer, t::BG_SURFACE);
        view::rule(&foot, footer);
        let mut row = foot.new_child(
            UiBuilder::new()
                .max_rect(view::inset(footer, t::SPACE_20, 0.0))
                .layout(Layout::right_to_left(Align::Center)),
        );
        if t::primary_button(
            &mut row,
            "Save mix.yaml",
            None,
            self.mix.draft != self.mix.saved,
        )
        .clicked()
            && let Some(path) = &self.mix.loaded_for
        {
            match mix::save(path, &self.mix.draft) {
                Ok(()) => {
                    self.mix.saved = self.mix.draft.clone();
                    self.mix.notice = Some("Saved mix.yaml".into())
                }
                Err(e) => self.mix.error = Some(e),
            }
        }
        if t::secondary_button(&mut row, "Revert", None, true).clicked() {
            self.open_mix();
        }
        if t::secondary_button(&mut row, "Close", None, true).clicked() {
            self.mix.open = false;
        }
        let status = Rect::from_min_max(
            footer.min + vec2(t::SPACE_20, 0.0),
            pos2(row.min_rect().left() - t::SPACE_16, footer.bottom()),
        );
        view::line(
            &foot,
            status,
            "Placement follows the script text, so it survives re-timing. The stitched MP3 is never changed.",
            t::font(t::CAPTION),
            t::TEXT_MUTED,
        );
    }
}

fn stitched_path(dir: &Path) -> Option<PathBuf> {
    let files: Vec<_> = fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|x| x.eq_ignore_ascii_case("mp3"))
                && !p
                    .file_stem()
                    .is_some_and(|x| x.to_string_lossy().ends_with("_mixed"))
        })
        .collect();
    (files.len() == 1).then(|| files[0].clone())
}

fn clock(seconds: f32) -> String {
    let total = seconds.max(0.0) as u64;
    format!(
        "{:02}:{:02}.{:03}",
        total / 60,
        total % 60,
        ((seconds.fract().abs()) * 1000.0) as u32
    )
}
