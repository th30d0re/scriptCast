//! Design tokens from the offline snapshot; all visual constants live here.
use eframe::egui::{self, Color32, FontFamily, FontId, RichText};
pub const FONT_SCALE: f32 = 1.0;
pub const BG_WINDOW: Color32 = Color32::from_rgb(23, 25, 28);
pub const BG_SURFACE: Color32 = Color32::from_rgb(30, 32, 36);
pub const BG_RAISED: Color32 = Color32::from_rgb(39, 42, 47);
pub const BG_CONTROL: Color32 = Color32::from_rgb(48, 52, 58);
pub const BG_SEGMENT: Color32 = Color32::from_rgb(44, 48, 55);
pub const BG_ANNOTATION: Color32 = Color32::from_rgb(34, 38, 45);
pub const BG_SELECTED: Color32 = Color32::from_rgb(42, 59, 86);
pub const BG_DANGER_ROW: Color32 = Color32::from_rgb(44, 34, 39);
pub const BG_DANGER: Color32 = Color32::from_rgb(64, 40, 45);
pub const BG_SCRIM: Color32 = Color32::from_rgba_unmultiplied_const(16, 18, 22, 201);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(54, 58, 65);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(237, 240, 244);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(164, 171, 182);
pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(113, 121, 133);
pub const ACCENT: Color32 = Color32::from_rgb(140, 175, 243);
pub const ON_ACCENT: Color32 = Color32::from_rgb(21, 28, 41);
pub const STATUS_PASSED: Color32 = Color32::from_rgb(154, 190, 176);
pub const STATUS_FAILED: Color32 = Color32::from_rgb(240, 130, 135);
pub const SPEAKER: [Color32; 4] = [
    Color32::from_rgb(147, 177, 213),
    Color32::from_rgb(178, 160, 201),
    Color32::from_rgb(146, 183, 170),
    Color32::from_rgb(197, 178, 141),
];
pub const OPACITY_DISABLED: f32 = 0.45;
pub const OPACITY_OVERVIEW_SECTION_OUTSIDE_VISIBLE_RANGE: f32 = 0.48;
pub const RADIUS_OVERVIEW_SECTION: f32 = 1.0;
pub const RADIUS_SEGMENT: f32 = 2.0;
pub const RADIUS_SEEK_TRACK: f32 = 2.0;
pub const RADIUS_WAVE_VIEWPORT: f32 = 3.0;
pub const RADIUS_CONTROL: f32 = 5.0;
pub const RADIUS_WINDOW: f32 = 8.0;
pub const RADIUS_CARD: f32 = 8.0;
pub const RADIUS_DIALOG: f32 = 12.0;
pub const TITLE_BAR_HEIGHT: f32 = 38.0;
pub const EPISODE_HEADER_HEIGHT: f32 = 76.0;
pub const TIMELINE_HEIGHT: f32 = 188.0;
pub const TIMELINE_OVERVIEW_HEIGHT: f32 = 17.0;
pub const TIMELINE_VISIBLE_RANGE_HEIGHT: f32 = 21.0;
pub const TIMELINE_RULER_HEIGHT: f32 = 18.0;
pub const TIMELINE_LANE_HEIGHT: f32 = 20.0;
pub const TIMELINE_LANE_GAP: f32 = 4.0;
pub const TIMELINE_LANE_LABEL_WIDTH: f32 = 66.0;
pub const TIMELINE_LANES_TO_LABEL_GAP: f32 = 12.0;
pub const TIMELINE_RULER_LEFT_PAD: f32 = 80.0;
pub const WAVE_BAR: f32 = 1.5;
pub const WAVE_GAP: f32 = 1.5;
pub const WAVE_MIN: f32 = 1.0;
pub const WAVE_VIEWPORT_HEIGHT: f32 = 36.0;
pub const WAVE_VIEWPORT_SECONDS: f32 = 6.0;
pub const WAVE_VIEWPORT_FILL: f32 = 0.8;
pub const WAVE_FADE: f32 = 32.0;
pub const WAVE_PLAYHEAD: f32 = 2.0;
pub const FILTER_BAR_HEIGHT: f32 = 42.0;
pub const SEARCH_WIDTH: f32 = 270.0;
pub const SEARCH_HEIGHT: f32 = 27.0;
pub const TABLE_HEADER_HEIGHT: f32 = 30.0;
pub const TABLE_ROW_HEIGHT: f32 = 28.0;
pub const TABLE_COL_TURN: f32 = 44.0;
pub const TABLE_COL_TIME: f32 = 92.0;
pub const TABLE_COL_SPEAKER: f32 = 82.0;
pub const TABLE_COL_DURATION: f32 = 60.0;
pub const TABLE_COL_VERIFICATION: f32 = 106.0;
pub const TABLE_COL_GAP: f32 = 12.0;
pub const INSPECTOR_WIDTH_SELECTED_TURN: f32 = 376.0;
pub const INSPECTOR_WIDTH_VERIFICATION: f32 = 436.0;
pub const INSPECTOR_WIDTH_JOB_RUNNER: f32 = 510.0;
pub const INSPECTOR_HEADING_HEIGHT: f32 = 39.0;
pub const PLAYBACK_BAR_HEIGHT: f32 = 64.0;
pub const PLAYBACK_CURRENT_AUDIO_WIDTH: f32 = 226.0;
pub const SEEK_TRACK_HEIGHT: f32 = 4.0;
pub const PROGRESS_HEIGHT: f32 = 3.0;
pub const STATUS_BAR_HEIGHT: f32 = 26.0;
pub const BUTTON_HEIGHT: f32 = 30.0;
pub const CLI_POPOVER_WIDTH: f32 = 360.0;
pub const BUTTON_PAD_X: f32 = 11.0;
pub const BUTTON_ICON: f32 = 16.0;
pub const BUTTON_ICON_GAP: f32 = 7.0;
pub const DIALOG_WIDTH: f32 = 540.0;
pub const DIALOG_PADDING: f32 = 24.0;
pub const STATE_CARD_WIDTH: f32 = 680.0;
pub const SPACE_2: f32 = 2.0;
pub const SPACE_4: f32 = 4.0;
pub const SPACE_5: f32 = 5.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_7: f32 = 7.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_9: f32 = 9.0;
pub const SPACE_10: f32 = 10.0;
pub const SPACE_11: f32 = 11.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_14: f32 = 14.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_18: f32 = 18.0;
pub const SPACE_20: f32 = 20.0;
pub const SPACE_22: f32 = 22.0;
pub const TEXT_9: f32 = 9.0;
pub const TEXT_10: f32 = 10.0;
pub const TEXT_11: f32 = 11.0;
pub const TEXT_12: f32 = 12.0;
pub const TEXT_13: f32 = 13.0;
pub const TEXT_14: f32 = 14.0;
pub const TEXT_15: f32 = 15.0;
pub const TEXT_18: f32 = 18.0;
pub const TEXT_20: f32 = 20.0;
pub const TEXT_22: f32 = 22.0;
pub const TEXT_30: f32 = 30.0;
pub const BRAND: f32 = 18.0;
pub const EPISODE_NAME: f32 = 15.0;
pub const HERO_TITLE: f32 = 30.0;
pub const DIALOG_TITLE: f32 = 20.0;
pub const STATE_TITLE: f32 = 18.0;
pub const SECTION_TITLE: f32 = 12.0;
pub const BUTTON: f32 = 12.0;
pub const BODY: f32 = 12.0;
pub const SECONDARY: f32 = 11.0;
pub const CAPTION: f32 = 10.0;
pub const FIELD_LABEL: f32 = 10.0;
pub const SPEAKER_LABEL: f32 = 12.0;
pub const NUMERIC: f32 = 11.0;
pub const SEGMENT_LABEL: f32 = 9.0;
pub const LOG_LINE: f32 = 11.0;
pub const RULER_TICK: f32 = 10.0;
pub const BORDER: f32 = 1.0;
pub const WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
// Supplemental dimensions explicitly specified in DESIGN.md / screen 01 TSX.
pub const WINDOW_MIN: [f32; 2] = [1024.0, 640.0];
pub const EMPTY_LEFT: f32 = 410.0;
pub const EMPTY_RIGHT: f32 = 386.0;
pub const EMPTY_GAP: f32 = 64.0;
pub const EMPTY_DIVIDER: f32 = 350.0;
pub const HERO_ICON: f32 = 42.0;
pub fn icon(name: &str) -> egui::ImageSource<'static> {
    match name {
        "audio-lines-22-accent" => {
            egui::include_image!("../design/assets/icons/audio-lines-22-accent.svg")
        }
        "circle-alert-22-danger" => {
            egui::include_image!("../design/assets/icons/circle-alert-22-danger.svg")
        }
        "folder-open-16-on-accent" => {
            egui::include_image!("../design/assets/icons/folder-open-16-on-accent.svg")
        }
        "folder-open-16-text" => {
            egui::include_image!("../design/assets/icons/folder-open-16-text.svg")
        }
        "folder-open-22-accent" => {
            egui::include_image!("../design/assets/icons/folder-open-22-accent.svg")
        }
        "folder-open-42-accent" => {
            egui::include_image!("../design/assets/icons/folder-open-42-accent.svg")
        }
        "headphones-22-accent" => {
            egui::include_image!("../design/assets/icons/headphones-22-accent.svg")
        }
        "list-15-text-muted" => {
            egui::include_image!("../design/assets/icons/list-15-text-muted.svg")
        }
        "loader-circle-12-accent" => {
            egui::include_image!("../design/assets/icons/loader-circle-12-accent.svg")
        }
        "loader-circle-22-accent" => {
            egui::include_image!("../design/assets/icons/loader-circle-22-accent.svg")
        }
        "pause-16-on-accent" => {
            egui::include_image!("../design/assets/icons/pause-16-on-accent.svg")
        }
        "play-16-on-accent" => egui::include_image!("../design/assets/icons/play-16-on-accent.svg"),
        "rotate-cw-16-text" => egui::include_image!("../design/assets/icons/rotate-cw-16-text.svg"),
        "settings-16-text" => egui::include_image!("../design/assets/icons/settings-16-text.svg"),
        "search-14-text-muted" => {
            egui::include_image!("../design/assets/icons/search-14-text-muted.svg")
        }
        "shield-check-26-accent" => {
            egui::include_image!("../design/assets/icons/shield-check-26-accent.svg")
        }
        "square-16-text" => egui::include_image!("../design/assets/icons/square-16-text.svg"),
        "terminal-22-danger" => {
            egui::include_image!("../design/assets/icons/terminal-22-danger.svg")
        }
        "triangle-alert-16-danger" => {
            egui::include_image!("../design/assets/icons/triangle-alert-16-danger.svg")
        }
        "unplug-22-accent" => egui::include_image!("../design/assets/icons/unplug-22-accent.svg"),
        "volume-x-22-danger" => {
            egui::include_image!("../design/assets/icons/volume-x-22-danger.svg")
        }
        "window-controls-49-text-subtle" => {
            egui::include_image!("../design/assets/icons/window-controls-49-text-subtle.svg")
        }
        _ => panic!("unknown embedded icon: {name}"),
    }
}

pub fn speaker_colour(i: usize) -> Color32 {
    SPEAKER[i % SPEAKER.len()]
}
pub fn font(size: f32) -> FontId {
    FontId::new(size * FONT_SCALE, FontFamily::Proportional)
}
pub fn mono(size: f32) -> FontId {
    FontId::new(size * FONT_SCALE, FontFamily::Monospace)
}
pub fn bold(size: f32) -> FontId {
    FontId::new(
        size * FONT_SCALE,
        FontFamily::Name("OpenDyslexic Bold".into()),
    )
}
pub fn text(value: impl Into<String>, size: f32, colour: Color32) -> RichText {
    RichText::new(value).font(font(size)).color(colour)
}
pub fn heading(value: impl Into<String>, size: f32) -> RichText {
    RichText::new(value).font(bold(size)).color(TEXT_PRIMARY)
}
pub fn field_label(ui: &mut egui::Ui, value: &str) -> egui::Response {
    ui.label(
        RichText::new(value.to_uppercase())
            .font(bold(FIELD_LABEL))
            .color(TEXT_MUTED),
    )
}

/// Upper bound for text-sized boxes (lane labels, table columns) so one long
/// name cannot eat the layout.
pub const TEXT_WIDTH_CAP: f32 = 200.0;

/// Width of `value` laid out on a single line in `font`.
pub fn text_width(ui: &egui::Ui, value: impl Into<String>, font: FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(value.into(), font, TEXT_MUTED)
        .size()
        .x
}
/// Height of `value` wrapped to `wrap_width` in `font`.
pub fn wrapped_height(ui: &egui::Ui, value: &str, font: FontId, wrap_width: f32) -> f32 {
    ui.painter()
        .layout(value.to_string(), font, TEXT_MUTED, wrap_width)
        .size()
        .y
}
/// Height of a single line of `font`.
pub fn line_height(ui: &egui::Ui, font: FontId) -> f32 {
    ui.painter().fonts_mut(|f| f.row_height(&font))
}
/// `measured` with the design token as floor and [`TEXT_WIDTH_CAP`] as ceiling.
pub fn fit_text_width(measured: f32, min: f32) -> f32 {
    measured.clamp(min, TEXT_WIDTH_CAP)
}

pub fn apply(ctx: &egui::Context) {
    // No default-font fallback: failure to load an embedded font is a test failure.
    // OpenDyslexic 3 is the face; OpenDyslexic 0.99 stays registered behind it
    // (and behind the mono face) only for symbols OpenDyslexic 3 lacks:
    // arrows, check mark, bullet, ▶.
    //
    // egui centres a row on the font's ascent/descent box. These faces reserve
    // far more space above the letters than below (OpenDyslexic 3: ascent 1556,
    // descent 426), so text sits visibly low next to icons. Each offset moves
    // the glyphs so the midpoint of cap height and x-height lands on the box
    // centre, measured from the 'H' and 'x' outlines of each file.
    let mut fonts = egui::FontDefinitions::empty();
    for (name, bytes, y_offset_factor) in [
        (
            "OpenDyslexic3",
            include_bytes!("../assets/fonts/OpenDyslexic3-Regular.ttf").as_slice(),
            -0.213,
        ),
        (
            "OpenDyslexic3 Bold",
            include_bytes!("../assets/fonts/OpenDyslexic3-Bold.ttf").as_slice(),
            -0.213,
        ),
        (
            "OpenDyslexic",
            include_bytes!("../assets/fonts/OpenDyslexic-Regular.otf").as_slice(),
            -0.038,
        ),
        (
            "OpenDyslexic Bold",
            include_bytes!("../assets/fonts/OpenDyslexic-Bold.otf").as_slice(),
            -0.038,
        ),
        (
            "OpenDyslexicMono",
            include_bytes!("../assets/fonts/OpenDyslexicMono-Regular.otf").as_slice(),
            -0.056,
        ),
    ] {
        let tweak = egui::FontTweak {
            y_offset_factor,
            ..Default::default()
        };
        fonts.font_data.insert(
            name.into(),
            egui::FontData::from_static(bytes).tweak(tweak).into(),
        );
    }
    fonts.families.insert(
        FontFamily::Proportional,
        vec!["OpenDyslexic3".into(), "OpenDyslexic".into()],
    );
    fonts.families.insert(
        FontFamily::Monospace,
        vec!["OpenDyslexicMono".into(), "OpenDyslexic".into()],
    );
    fonts.families.insert(
        FontFamily::Name("OpenDyslexic Bold".into()),
        vec!["OpenDyslexic3 Bold".into(), "OpenDyslexic Bold".into()],
    );
    ctx.set_fonts(fonts);
    let mut style = egui::Style {
        visuals: egui::Visuals::dark(),
        ..Default::default()
    };
    style.visuals.panel_fill = BG_WINDOW;
    style.visuals.window_fill = BG_SURFACE;
    style.visuals.extreme_bg_color = BG_WINDOW;
    style.visuals.faint_bg_color = BG_SURFACE;
    style.visuals.override_text_color = Some(TEXT_PRIMARY);
    style.visuals.selection.bg_fill = BG_SELECTED;
    style.visuals.selection.stroke = egui::Stroke::new(BORDER, ACCENT);
    style.visuals.window_stroke = egui::Stroke::new(BORDER, BORDER_DEFAULT);
    style.visuals.window_corner_radius = egui::CornerRadius::same(RADIUS_WINDOW as u8);
    style.visuals.disabled_alpha = OPACITY_DISABLED;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.bg_fill = BG_CONTROL;
        widget.weak_bg_fill = BG_CONTROL;
        widget.bg_stroke = egui::Stroke::new(BORDER, BORDER_DEFAULT);
        widget.fg_stroke = egui::Stroke::new(BORDER, TEXT_PRIMARY);
        widget.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL as u8);
    }
    style.visuals.widgets.hovered.bg_stroke.color = ACCENT;
    style.visuals.widgets.active.bg_fill = BG_SELECTED;
    style.spacing.item_spacing = egui::vec2(SPACE_8, SPACE_6);
    style.spacing.button_padding = egui::vec2(BUTTON_PAD_X, 0.0);
    style.spacing.interact_size.y = BUTTON_HEIGHT;
    style.text_styles = [
        (egui::TextStyle::Body, font(BODY)),
        (egui::TextStyle::Button, bold(BUTTON)),
        (egui::TextStyle::Small, font(SECONDARY)),
        (egui::TextStyle::Heading, bold(STATE_TITLE)),
        (egui::TextStyle::Monospace, mono(NUMERIC)),
    ]
    .into();
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_style_of(egui::Theme::Dark, style);
    egui_extras::install_image_loaders(ctx);
}

pub fn primary_button(
    ui: &mut egui::Ui,
    label: &str,
    icon_name: Option<&str>,
    enabled: bool,
) -> egui::Response {
    button(ui, label, icon_name, enabled, true)
}
pub fn secondary_button(
    ui: &mut egui::Ui,
    label: &str,
    icon_name: Option<&str>,
    enabled: bool,
) -> egui::Response {
    button(ui, label, icon_name, enabled, false)
}
/// A square secondary button showing only an icon; callers add a tooltip.
pub fn icon_button(ui: &mut egui::Ui, icon_name: &str, enabled: bool) -> egui::Response {
    ui.scope(|ui| {
        // Padding that centres the icon in a BUTTON_HEIGHT square.
        ui.spacing_mut().button_padding = egui::Vec2::splat((BUTTON_HEIGHT - BUTTON_ICON) / 2.0);
        ui.add_enabled(
            enabled,
            egui::Button::image(
                egui::Image::new(icon(icon_name)).fit_to_exact_size(egui::Vec2::splat(BUTTON_ICON)),
            )
            .fill(BG_CONTROL)
            .stroke(egui::Stroke::new(BORDER, BORDER_DEFAULT))
            .corner_radius(RADIUS_CONTROL as u8)
            .min_size(egui::Vec2::splat(BUTTON_HEIGHT)),
        )
    })
    .inner
}
fn button(
    ui: &mut egui::Ui,
    label: &str,
    icon_name: Option<&str>,
    enabled: bool,
    primary: bool,
) -> egui::Response {
    let text = RichText::new(label).font(bold(BUTTON)).color(if primary {
        ON_ACCENT
    } else {
        TEXT_PRIMARY
    });
    let button = if let Some(name) = icon_name {
        egui::Button::new((
            egui::Image::new(icon(name)).fit_to_exact_size(egui::Vec2::splat(BUTTON_ICON)),
            text,
        ))
    } else {
        egui::Button::new(text)
    };
    ui.add_enabled(
        enabled,
        button
            .gap(BUTTON_ICON_GAP)
            .wrap_mode(egui::TextWrapMode::Extend)
            .fill(if primary { ACCENT } else { BG_CONTROL })
            .stroke(egui::Stroke::new(BORDER, BORDER_DEFAULT))
            .corner_radius(RADIUS_CONTROL as u8)
            .min_size(egui::vec2(0.0, BUTTON_HEIGHT)),
    )
}

pub const JOB_LOG_HEIGHT: f32 = 180.0;
pub const RESTORE_LIST_HEIGHT: f32 = 180.0;
pub const CONFIRM_ICON: f32 = 26.0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_text_width_clamps_between_token_and_cap() {
        assert_eq!(fit_text_width(10.0, TABLE_COL_SPEAKER), TABLE_COL_SPEAKER);
        assert_eq!(fit_text_width(150.0, TABLE_COL_SPEAKER), 150.0);
        assert_eq!(fit_text_width(500.0, TABLE_COL_SPEAKER), TEXT_WIDTH_CAP);
    }
}
