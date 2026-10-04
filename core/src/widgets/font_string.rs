use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameFont {
    #[default]
    FrizQuadrata,
    ArialNarrow,
}

static FONT_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

/// Set the directory holding the game font files (the client's `data/fonts`), once,
/// before the first text measurement; a second call must name the same directory.
pub fn set_font_directory(directory: PathBuf) -> Result<(), String> {
    let current = FONT_DIRECTORY.get_or_init(|| directory.clone());
    if *current == directory {
        Ok(())
    } else {
        Err(format!(
            "Font directory is {}, not {}",
            current.display(),
            directory.display()
        ))
    }
}

impl GameFont {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::FrizQuadrata => "FRIZQT__.TTF",
            Self::ArialNarrow => "ARIALN.ttf",
        }
    }

    /// The font file in the host-set font directory; None before the host set it.
    pub fn path(self) -> Option<PathBuf> {
        Some(FONT_DIRECTORY.get()?.join(self.file_name()))
    }

    pub fn from_attr(s: &str) -> Self {
        match s {
            "FrizQuadrata" => Self::FrizQuadrata,
            "ArialNarrow" => Self::ArialNarrow,
            _ => Self::default(),
        }
    }
}

impl std::fmt::Display for GameFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::FrizQuadrata => "FrizQuadrata",
            Self::ArialNarrow => "ArialNarrow",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyH {
    Left,
    Center,
    Right,
}

impl JustifyH {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "LEFT",
            Self::Center => "CENTER",
            Self::Right => "RIGHT",
        }
    }
}

impl std::fmt::Display for JustifyH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// RGBA color for use in Dioxus RSX attributes (font_color, background_color).
///
/// ```ignore
/// fontstring { font_color: FontColor::new(0.65, 0.65, 0.7, 1.0) }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontColor(pub [f32; 4]);

impl FontColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

impl std::fmt::Display for FontColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r, g, b, a] = self.0;
        write!(f, "{r},{g},{b},{a}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyV {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outline {
    None,
    Outline,
    ThickOutline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontStringData {
    pub text: String,
    pub font: GameFont,
    pub font_size: f32,
    pub color: [f32; 4],
    pub justify_h: JustifyH,
    pub justify_v: JustifyV,
    pub shadow_color: Option<[f32; 4]>,
    pub shadow_offset: [f32; 2],
    pub outline: Outline,
    pub word_wrap: bool,
    pub max_lines: Option<u32>,
    pub text_scale: f32,
}

impl Default for FontStringData {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: GameFont::default(),
            font_size: 12.0,
            color: [1.0, 1.0, 1.0, 1.0],
            justify_h: JustifyH::Center,
            justify_v: JustifyV::Middle,
            shadow_color: None,
            shadow_offset: [0.0, 0.0],
            outline: Outline::None,
            word_wrap: false,
            max_lines: None,
            text_scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font_string_data() {
        let fs = FontStringData::default();
        assert!(fs.text.is_empty());
        assert_eq!(fs.font, GameFont::FrizQuadrata);
        assert_eq!(fs.font_size, 12.0);
        assert_eq!(fs.justify_h, JustifyH::Center);
        assert_eq!(fs.justify_v, JustifyV::Middle);
        assert_eq!(fs.text_scale, 1.0);
    }
}
