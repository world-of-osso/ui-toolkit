use std::collections::HashMap;

use bevy::prelude::*;
use bevy::text::Font;

use crate::widgets::font_string::GameFont;

#[derive(Resource, Default)]
pub struct FontRegistry {
    cache: HashMap<GameFont, Handle<Font>>,
}

impl FontRegistry {
    pub fn get(&mut self, font: GameFont, font_assets: &mut Assets<Font>) -> Handle<Font> {
        if let Some(handle) = self.cache.get(&font) {
            return handle.clone();
        }
        let bytes = std::fs::read(font.path())
            .unwrap_or_else(|e| panic!("failed to read font {:?} at {}: {}", font, font.path(), e));
        ab_glyph::FontRef::try_from_slice(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse font {:?}: {}", font, e));
        let f = Font::from_bytes(bytes);
        let handle = font_assets.add(f);
        self.cache.insert(font, handle.clone());
        handle
    }
}
