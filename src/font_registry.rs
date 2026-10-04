use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::text::Font;

use crate::widgets::font_string::GameFont;

#[derive(Resource)]
pub struct FontRegistry {
    directory: PathBuf,
    cache: HashMap<GameFont, Handle<Font>>,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::with_directory("/home/osso/Projects/wow/wow-ui-sim/fonts")
    }
}

impl FontRegistry {
    pub fn with_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, font: GameFont, font_assets: &mut Assets<Font>) -> Handle<Font> {
        if let Some(handle) = self.cache.get(&font) {
            return handle.clone();
        }
        let path = self.directory.join(font.file_name());
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read font {font:?} at {}: {e}", path.display()));
        ab_glyph::FontRef::try_from_slice(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse font {:?}: {}", font, e));
        let f = Font::from_bytes(bytes);
        let handle = font_assets.add(f);
        self.cache.insert(font, handle.clone());
        handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FontDirectory(PathBuf);

    impl FontDirectory {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "ui-toolkit-fonts-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("create isolated font fixture directory");
            Self(path)
        }
    }

    impl Drop for FontDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).expect("remove font fixture directory");
        }
    }

    #[test]
    fn configured_directory_loads_real_fonts_and_caches_handles() {
        let directory = FontDirectory::new();
        let mut registry = FontRegistry::with_directory(&directory.0);
        let mut assets = Assets::<Font>::default();
        for font in [GameFont::FrizQuadrata, GameFont::ArialNarrow] {
            let destination = directory.0.join(font.file_name());
            std::fs::write(&destination, bevy::text::DEFAULT_FONT_DATA)
                .expect("write embedded real font fixture");
            let handle = registry.get(font, &mut assets);
            assert!(assets.get(&handle).is_some());
            std::fs::remove_file(destination).unwrap();
            assert_eq!(registry.get(font, &mut assets), handle);
        }
        assert_eq!(assets.len(), 2);
    }

    #[test]
    fn missing_configured_font_fails_without_loading_default_font() {
        let directory = FontDirectory::new();
        let mut registry = FontRegistry::with_directory(&directory.0);
        let mut assets = Assets::<Font>::default();
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.get(GameFont::FrizQuadrata, &mut assets);
        }))
        .expect_err("missing configured font must fail");
        let message = error.downcast_ref::<String>().expect("font error message");
        assert!(message.contains(&directory.0.join("FRIZQT__.TTF").display().to_string()));
        assert!(assets.is_empty());
    }

    #[test]
    fn invalid_configured_font_fails_explicitly() {
        let directory = FontDirectory::new();
        std::fs::write(directory.0.join("FRIZQT__.TTF"), b"not a font").unwrap();
        let mut registry = FontRegistry::with_directory(&directory.0);
        let mut assets = Assets::<Font>::default();
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry.get(GameFont::FrizQuadrata, &mut assets);
        }))
        .expect_err("invalid configured font must fail");
        let message = error.downcast_ref::<String>().expect("font error message");
        assert!(message.contains("failed to parse font FrizQuadrata"));
        assert!(assets.is_empty());
    }
}
