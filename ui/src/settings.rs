//! Settings window (⌘,): appearance, playback, and CLI command folder.

use crate::Browser;
use eframe::egui;
use scriptcast_ui::theme as t;

impl Browser {
    pub(crate) fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = self.settings_open;
        let mut appearance = self.preferences.appearance;
        let mut appearance_changed = false;
        let mut reset_appearance = false;
        let mut auto_advance = self.preferences.auto_advance;
        let mut gapless = self.preferences.gapless;
        let mut auto_changed = false;
        let mut gapless_changed = false;
        let mut choose_cli = false;
        let mut clear_cli = false;
        let speaker_names: Vec<String> = self
            .episode
            .as_ref()
            .map(|e| e.speakers.iter().map(|s| s.name.clone()).collect())
            .unwrap_or_default();
        let cli_dir = self.preferences.cli_dir.clone();
        let commands: Vec<(String, String)> = self
            .jobs
            .commands
            .iter()
            .map(|(name, result)| {
                let detail = result
                    .as_ref()
                    .map(|r| r.description())
                    .unwrap_or_else(|e| e.clone());
                (name.clone(), detail)
            })
            .collect();

        egui::Window::new("Settings")
            .id(egui::Id::new("settings"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(t::SETTINGS_WIDTH)
            .min_width(t::SETTINGS_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(t::BG_SURFACE)
                    .stroke(egui::Stroke::new(t::BORDER, t::BORDER_DEFAULT))
                    .corner_radius(t::RADIUS_DIALOG as u8)
                    .inner_margin(t::SPACE_16),
            )
            .show(ctx, |ui| {
                ui.set_width(t::SETTINGS_WIDTH - t::SPACE_16 * 2.0);
                ui.spacing_mut().item_spacing.y = t::SPACE_8;

                ui.label(t::heading("Appearance", t::SECTION_TITLE));
                ui.label(t::text(
                    "Very dark accents reduce contrast on accent button text.",
                    t::CAPTION,
                    t::TEXT_MUTED,
                ));
                ui.horizontal(|ui| {
                    ui.label(t::text("Accent", t::BODY, t::TEXT_PRIMARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if egui::widgets::color_picker::color_edit_button_srgb(
                            ui,
                            &mut appearance.accent,
                        )
                        .changed()
                        {
                            appearance_changed = true;
                        }
                    });
                });
                for i in 0..4 {
                    let label = speaker_names
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| format!("Speaker {}", i + 1));
                    ui.horizontal(|ui| {
                        ui.label(t::text(label, t::BODY, t::TEXT_PRIMARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if egui::widgets::color_picker::color_edit_button_srgb(
                                ui,
                                &mut appearance.speakers[i],
                            )
                            .changed()
                            {
                                appearance_changed = true;
                            }
                        });
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(t::text("Text size", t::BODY, t::TEXT_PRIMARY));
                    let mut percent = appearance.text_scale * 100.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut percent, 85.0..=130.0)
                                .suffix("%")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        appearance.text_scale = percent / 100.0;
                        appearance_changed = true;
                    }
                });
                if t::secondary_button(ui, "Reset appearance", None, true).clicked() {
                    reset_appearance = true;
                }

                ui.add_space(t::SPACE_8);
                ui.label(t::heading("Playback", t::SECTION_TITLE));
                if ui
                    .checkbox(
                        &mut auto_advance,
                        egui::RichText::new("Auto-advance")
                            .font(t::font(t::BODY))
                            .color(t::TEXT_PRIMARY),
                    )
                    .changed()
                {
                    auto_changed = true;
                }
                if ui
                    .checkbox(
                        &mut gapless,
                        egui::RichText::new("Gapless auto-advance")
                            .font(t::font(t::BODY))
                            .color(t::TEXT_PRIMARY),
                    )
                    .changed()
                {
                    gapless_changed = true;
                }

                ui.add_space(t::SPACE_8);
                ui.label(t::heading("Commands", t::SECTION_TITLE));
                ui.label(t::text(
                    "Folder with scriptCast commands",
                    t::SECONDARY,
                    t::TEXT_PRIMARY,
                ));
                ui.label(t::text(
                    cli_dir
                        .as_deref()
                        .filter(|path| !path.as_os_str().is_empty())
                        .map_or_else(
                            || "Unset — project virtualenvs, login PATH, then checkout".into(),
                            |p| p.display().to_string(),
                        ),
                    t::SECONDARY,
                    t::TEXT_MUTED,
                ));
                ui.horizontal(|ui| {
                    if t::secondary_button(ui, "Choose folder…", None, true).clicked() {
                        choose_cli = true;
                    }
                    if t::secondary_button(ui, "Clear", None, cli_dir.is_some()).clicked() {
                        clear_cli = true;
                    }
                });
                ui.add_space(t::SPACE_4);
                t::field_label(ui, "Resolved commands");
                if commands.is_empty() {
                    ui.label(t::text("Resolving…", t::CAPTION, t::TEXT_MUTED));
                } else {
                    for (name, detail) in &commands {
                        ui.label(t::text(
                            format!("{name} · {detail}"),
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                    }
                }
            });

        self.settings_open = open;
        if auto_changed {
            self.preferences.auto_advance = auto_advance;
        }
        if gapless_changed {
            self.preferences.gapless = gapless;
        }
        if reset_appearance {
            appearance = t::Appearance::default();
            appearance_changed = true;
        }
        if appearance_changed {
            self.preferences.appearance = appearance.clamped();
            t::set_appearance(ctx, self.preferences.appearance);
        }
        if choose_cli
            && let Some(path) = rfd::FileDialog::new()
                .set_title("Folder with scriptCast commands")
                .pick_folder()
        {
            self.preferences.cli_dir = Some(path);
            self.export_command = None;
            self.refresh_commands(ctx);
        }
        if clear_cli {
            self.preferences.cli_dir = None;
            self.export_command = None;
            self.refresh_commands(ctx);
        }
    }
}
