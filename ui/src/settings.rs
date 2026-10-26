//! Settings window (⌘,): appearance, playback, and CLI command folder.

use crate::Browser;
use eframe::egui;
use crate::theme as t;

/// Title + Done chrome stay outside the scroll area; content scrolls under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SettingsLayout {
    pub scroll_max_height: f32,
    pub title_on_screen: bool,
}

/// Pure layout helper: chrome fits the viewport while content may exceed it.
pub fn settings_layout(screen_height: f32, title_height: f32, done_height: f32) -> SettingsLayout {
    let scroll_max_height = (screen_height * 0.8).max(0.0);
    let chrome = title_height + done_height;
    SettingsLayout {
        scroll_max_height,
        title_on_screen: chrome <= screen_height && title_height > 0.0,
    }
}

/// Bundle version plus short git hash from `build.rs` (copyable in Settings).
pub fn version_line() -> String {
    format!(
        "scriptCast {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("SCRIPTCAST_GIT")
    )
}

impl Browser {
    pub(crate) fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut close = false;
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
        let screen_height = ctx.content_rect().height();
        let layout = settings_layout(
            screen_height,
            t::SPACE_20 * 2.0,
            t::BUTTON_HEIGHT + t::SPACE_16,
        );
        let version = version_line();

        let response = egui::Modal::new(egui::Id::new("settings"))
            .backdrop_color(t::BG_SCRIM)
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

                ui.horizontal(|ui| {
                    ui.label(t::heading("Settings", t::SECTION_TITLE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(t::text("×", t::SECTION_TITLE, t::TEXT_MUTED))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });

                egui::ScrollArea::vertical()
                    .id_salt("settings_scroll")
                    .max_height(layout.scroll_max_height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = t::SPACE_8;

                        egui::CollapsingHeader::new("Project")
                            .default_open(self.project.tab == "toml")
                            .show(ui, |ui| self.project_settings_section(ui));
                        ui.separator();
                        ui.label(t::heading("Appearance", t::SECTION_TITLE));
                        ui.label(t::text(
                            "Very dark accents reduce contrast on accent button text.",
                            t::CAPTION,
                            t::TEXT_MUTED,
                        ));
                        ui.horizontal(|ui| {
                            ui.label(t::text("Accent", t::BODY, t::TEXT_PRIMARY));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if egui::widgets::color_picker::color_edit_button_srgb(
                                        ui,
                                        &mut appearance.accent,
                                    )
                                    .changed()
                                    {
                                        appearance_changed = true;
                                    }
                                },
                            );
                        });
                        let speaker_count = if speaker_names.is_empty() {
                            4
                        } else {
                            speaker_names.len()
                        };
                        for i in 0..speaker_count {
                            let label = speaker_names
                                .get(i)
                                .cloned()
                                .unwrap_or_else(|| format!("Speaker {}", i + 1));
                            let slot = i % appearance.speakers.len();
                            ui.horizontal(|ui| {
                                ui.label(t::text(label, t::BODY, t::TEXT_PRIMARY));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if egui::widgets::color_picker::color_edit_button_srgb(
                                            ui,
                                            &mut appearance.speakers[slot],
                                        )
                                        .changed()
                                        {
                                            appearance_changed = true;
                                        }
                                    },
                                );
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
                        ui.checkbox(
                            &mut self.preferences.read_along,
                            t::text("Read-along highlighting", t::BODY, t::TEXT_PRIMARY),
                        );
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
                        ui.label(t::heading("Editing", t::SECTION_TITLE));
                        ui.label(t::text("Edit transcripts in:", t::BODY, t::TEXT_PRIMARY));
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut self.preferences.edit_mode,
                                crate::editing::EditMode::Panel,
                                "Side panel",
                            );
                            ui.selectable_value(
                                &mut self.preferences.edit_mode,
                                crate::editing::EditMode::Row,
                                "Inline",
                            );
                        });
                        ui.checkbox(
                            &mut self.preferences.retime_on_open,
                            t::text("Retime script on open", t::BODY, t::TEXT_PRIMARY),
                        );
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
                                    || {
                                        "Unset — project virtualenvs, login PATH, then checkout"
                                            .into()
                                    },
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
                            for skip in &self.jobs.skipped {
                                ui.label(t::text(&skip.reason, t::CAPTION, t::STATUS_FAILED));
                            }
                            if let Some(warning) = &self.jobs.stale_warning {
                                ui.label(t::text(warning, t::CAPTION, t::STATUS_FAILED));
                            }
                            for (name, detail) in &commands {
                                ui.label(t::text(
                                    format!("{name} · {detail}"),
                                    t::CAPTION,
                                    t::TEXT_MUTED,
                                ));
                            }
                        }
                    });

                ui.add_space(t::SPACE_8);
                if t::primary_button(ui, "Done", None, true).clicked() {
                    close = true;
                }
                ui.add(
                    egui::Label::new(t::text(&version, t::CAPTION, t::TEXT_MUTED)).selectable(true),
                );
            });

        if close || response.should_close() {
            self.settings_open = false;
            if !self.dirty_project_files().is_empty() {
                self.project.data_open = true;
                self.project.close_prompt = true;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tall_content_keeps_title_chrome_on_screen() {
        let layout = settings_layout(800.0, 40.0, 46.0);
        assert!(layout.title_on_screen);
        assert!((layout.scroll_max_height - 640.0).abs() < f32::EPSILON);
        let layout = settings_layout(600.0, 40.0, 46.0);
        assert!(layout.title_on_screen);
        // Content may be thousands of points tall; chrome still fits the screen.
        assert!(layout.scroll_max_height < 600.0);
        let chrome = 40.0_f32 + 46.0;
        assert!(layout.title_on_screen && chrome < 600.0);
    }

    #[test]
    fn version_line_includes_package_version() {
        let line = version_line();
        assert!(!line.is_empty());
        assert!(
            line.contains(env!("CARGO_PKG_VERSION")),
            "version line missing package version: {line}"
        );
        assert!(line.starts_with("scriptCast "));
    }
}
