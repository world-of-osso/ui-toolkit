use std::collections::HashMap;
use std::sync::Mutex;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};

use crate::widgets::font_string::GameFont;

static CACHE: Mutex<Option<TextMeasureCache>> = Mutex::new(None);

struct TextMeasureCache {
    fonts: HashMap<GameFont, FontVec>,
    sizes: HashMap<(String, GameFont, u32), (f32, f32)>,
}

/// Measure text dimensions (width, height) for a given font and pixel size.
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

    let font_data = load_or_get_font(cache, font)?;
    let result = compute_size(font_data, text, font_size);
    cache.sizes.insert(cache_key, result);
    Some(result)
}

fn load_or_get_font(cache: &mut TextMeasureCache, font: GameFont) -> Option<&FontVec> {
    if !cache.fonts.contains_key(&font) {
        let bytes = std::fs::read(font.path()).ok()?;
        let fv = FontVec::try_from_vec(bytes).ok()?;
        cache.fonts.insert(font, fv);
    }
    cache.fonts.get(&font)
}

fn compute_size(font: &FontVec, text: &str, font_size: f32) -> (f32, f32) {
    let scaled = font.as_scaled(PxScale::from(font_size));
    let mut width = 0.0f32;
    let mut prev_glyph_id = None;
    for ch in text.chars() {
        let glyph_id = scaled.glyph_id(ch);
        if let Some(prev) = prev_glyph_id {
            width += scaled.kern(prev, glyph_id);
        }
        width += scaled.h_advance(glyph_id);
        prev_glyph_id = Some(glyph_id);
    }
    let height = scaled.height();
    (width.ceil(), height.ceil())
}

#[cfg(test)]
mod tests {
    use super::*;

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
