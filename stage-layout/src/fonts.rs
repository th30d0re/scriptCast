//! IBM Plex faces bundled with the crate, plus extra font files from the caller.
//!
//! FigCraft's API for this is [`figcraft_text::load_font_bytes`] (`FontDb::add_bytes`
//! underneath). It accepts SFNT fonts (TTF/OTF). The `@fontsource` packages Stage loads
//! in the browser are WOFF/WOFF2, which `skrifa` does not parse, so the crate ships the
//! OFL IBM Plex TTF files (v6.4.2) instead of those WOFF slices.

use std::sync::Once;

/// One font file (TTF or OTF bytes).
#[derive(Clone, Debug)]
pub struct FontBytes {
    pub bytes: Vec<u8>,
}

impl FontBytes {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self { bytes: bytes.into() }
    }
}

const BUNDLED: &[&[u8]] = &[
    include_bytes!("../fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../fonts/IBMPlexSans-Medium.ttf"),
    include_bytes!("../fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../fonts/IBMPlexMono-Medium.ttf"),
];

/// Opt out of the machine's fonts, then load the bundled faces once and `extra` every call.
///
/// System fonts would make line breaks depend on whichever IBM Plex (or fallback) a machine
/// happens to have installed. After this, shaping uses the bundled faces plus `extra`.
pub fn register(extra: &[FontBytes]) {
    figcraft_text::opt_out_of_system_fonts();
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for bytes in BUNDLED {
            figcraft_text::load_font_bytes(bytes.to_vec());
        }
    });
    for font in extra {
        figcraft_text::load_font_bytes(font.bytes.clone());
    }
}
