use eframe::egui;
use scriptcast_ui::theme;
#[test]
fn embedded_fonts_have_real_glyphs_and_render_into_atlas() {
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    ctx.begin_pass(egui::RawInput::default());
    ctx.fonts_mut(|fonts| {
        for font in [
            theme::font(theme::BODY),
            theme::mono(theme::NUMERIC),
            theme::bold(theme::BUTTON),
        ] {
            let text = "Toussaint 00:19.740";
            // egui 0.36.2 has_glyph compares face identities, yielding false for
            // all chars when the family has only the replacement glyph's face.
            // Compare actual rasterized atlas regions at the same origin instead.
            let replacement =
                fonts.layout_no_wrap("\u{10ffff}".into(), font.clone(), theme::TEXT_PRIMARY);
            let replacement_uv = replacement.rows[0].glyphs[0].uv_rect;
            for c in text.chars().filter(|c| !c.is_whitespace()) {
                let glyph = fonts.layout_no_wrap(c.to_string(), font.clone(), theme::TEXT_PRIMARY);
                let uv = glyph.rows[0].glyphs[0].uv_rect;
                assert_ne!(
                    uv, replacement_uv,
                    "replacement glyph for {c:?} in {font:?}"
                );
                assert!(!glyph.rows[0].visuals.mesh.vertices.is_empty());
            }
            let galley = fonts.layout_no_wrap(text.into(), font.clone(), theme::TEXT_PRIMARY);
            assert!(galley.size().x > 0.0);
            assert!(
                galley.size().y <= theme::TABLE_ROW_HEIGHT,
                "font clips row: {:?}",
                galley.size()
            );
            println!("{font:?}: {:?}", galley.size());
        }
    });
    let mut output = ctx.end_pass();
    assert!(!output.textures_delta.set.is_empty());
    output.textures_delta.clear();
}
