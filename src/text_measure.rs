use std::collections::HashMap;
use std::sync::Mutex;

use parley::fontique::{Blob, FontInfoOverride};
use parley::{FontContext, FontFamily, Layout, LayoutContext, StyleProperty};

use crate::widgets::font_string::GameFont;

static CACHE: Mutex<Option<TextMeasureCache>> = Mutex::new(None);

struct TextMeasureCache {
    fonts: HashMap<GameFont, (FontContext, LayoutContext<()>)>,
    sizes: HashMap<(String, GameFont, u32), (f32, f32)>,
}

/// Measure text dimensions (width, height) for a given font and pixel size; None when the
/// font is unavailable (no font directory set, or its file is unreadable).
/// Results are cached permanently — same (text, font, size) triple always returns
/// the cached value.
pub fn measure_text(text: &str, font: GameFont, font_size: f32) -> Option<(f32, f32)> {
    if text.is_empty() {
        return Some((0.0, 0.0));
    }
    let size_key = font_size.to_bits();
    let cache_key = (text.to_string(), font, size_key);

    let mut guard = CACHE.lock().ok()?;
    let cache = guard.get_or_insert_with(|| TextMeasureCache {
        fonts: HashMap::new(),
        sizes: HashMap::new(),
    });

    if let Some(&size) = cache.sizes.get(&cache_key) {
        return Some(size);
    }

    let (font_context, layout_context) = load_or_get_font(cache, font)?;
    let result = compute_size(font_context, layout_context, text, font, font_size);
    cache.sizes.insert(cache_key, result);
    Some(result)
}

fn load_or_get_font(
    cache: &mut TextMeasureCache,
    font: GameFont,
) -> Option<&mut (FontContext, LayoutContext<()>)> {
    if !cache.fonts.contains_key(&font) {
        let bytes = std::fs::read(font.path()?).ok()?;
        let mut context = FontContext::new();
        let faces = context.collection.register_fonts(
            Blob::from(bytes),
            Some(FontInfoOverride {
                family_name: Some(font.file_name()),
                ..Default::default()
            }),
        );
        if faces.is_empty() {
            return None;
        }
        cache.fonts.insert(font, (context, LayoutContext::new()));
    }
    cache.fonts.get_mut(&font)
}

fn compute_size(
    font_context: &mut FontContext,
    layout_context: &mut LayoutContext<()>,
    text: &str,
    font: GameFont,
    font_size: f32,
) -> (f32, f32) {
    let mut builder = layout_context.ranged_builder(font_context, text, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::named(
        font.file_name(),
    )));
    builder.push_default(StyleProperty::FontSize(font_size));
    let mut layout = Layout::new();
    builder.build_into(&mut layout, text);
    layout.break_all_lines(None);
    (layout.full_width().ceil(), layout.height().ceil())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::font_string::set_font_directory;
    use std::path::PathBuf;

    /// ui-toolkit ships no fonts; its own tests read the wow-ui-sim copies.
    const TEST_FONTS: &str = "/home/osso/Projects/wow/wow-ui-sim/fonts";

    fn measure_text(text: &str, font: GameFont, font_size: f32) -> Option<(f32, f32)> {
        set_font_directory(PathBuf::from(TEST_FONTS)).unwrap();
        super::measure_text(text, font, font_size)
    }

    #[test]
    fn font_paths_resolve_in_the_configured_directory_only() {
        set_font_directory(PathBuf::from(TEST_FONTS)).unwrap();
        assert_eq!(
            GameFont::FrizQuadrata.path(),
            Some(PathBuf::from(TEST_FONTS).join("FRIZQT__.TTF"))
        );
        assert_eq!(
            GameFont::ArialNarrow.path(),
            Some(PathBuf::from(TEST_FONTS).join("ARIALN.ttf"))
        );
        let error = set_font_directory(PathBuf::from("/elsewhere/fonts")).unwrap_err();
        assert!(error.contains("/elsewhere/fonts"), "{error}");
    }

    #[test]
    fn empty_text_returns_zero() {
        let (w, h) = measure_text("", GameFont::FrizQuadrata, 12.0).unwrap();
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn measure_returns_positive_dimensions() {
        let (w, h) = measure_text("Hello", GameFont::FrizQuadrata, 16.0).unwrap();
        assert!(w > 0.0, "width should be positive, got {w}");
        assert!(h > 0.0, "height should be positive, got {h}");
    }

    #[test]
    fn longer_text_is_wider() {
        let (w1, _) = measure_text("Hi", GameFont::FrizQuadrata, 14.0).unwrap();
        let (w2, _) = measure_text("Hello World", GameFont::FrizQuadrata, 14.0).unwrap();
        assert!(w2 > w1, "longer text should be wider: {w2} vs {w1}");
    }

    #[test]
    fn larger_font_is_taller() {
        let (_, h1) = measure_text("A", GameFont::FrizQuadrata, 10.0).unwrap();
        let (_, h2) = measure_text("A", GameFont::FrizQuadrata, 20.0).unwrap();
        assert!(h2 > h1, "larger font should be taller: {h2} vs {h1}");
    }

    #[test]
    fn cached_result_matches_fresh() {
        let first = measure_text("Cache", GameFont::FrizQuadrata, 12.0).unwrap();
        let second = measure_text("Cache", GameFont::FrizQuadrata, 12.0).unwrap();
        assert_eq!(first, second);
    }
}
