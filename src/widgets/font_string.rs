#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameFont {
    #[default]
    FrizQuadrata,
    ArialNarrow,
}

impl GameFont {
    pub fn path(self) -> &'static str {
        match self {
            Self::FrizQuadrata => "/home/osso/Projects/wow/wow-ui-sim/fonts/FRIZQT__.TTF",
            Self::ArialNarrow => "/home/osso/Projects/wow/wow-ui-sim/fonts/ARIALN.ttf",
        }
    }

    pub fn from_attr(s: &str) -> Self {
        match s {
            "FrizQuadrata" => Self::FrizQuadrata,
            "ArialNarrow" => Self::ArialNarrow,
            _ => Self::default(),
        }
    }
}

impl dioxus_core::IntoAttributeValue for GameFont {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(match self {
            Self::FrizQuadrata => "FrizQuadrata".to_string(),
            Self::ArialNarrow => "ArialNarrow".to_string(),
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

impl dioxus_core::IntoAttributeValue for JustifyH {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.as_str().to_string())
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

impl dioxus_core::IntoAttributeValue for FontColor {
    fn into_value(self) -> dioxus_core::AttributeValue {
        let [r, g, b, a] = self.0;
        dioxus_core::AttributeValue::Text(format!("{r},{g},{b},{a}"))
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

#[derive(Debug, Clone)]
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
