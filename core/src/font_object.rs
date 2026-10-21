//! Named Roman font objects from Retail Blizzard_Fonts_Shared/Shared/{Fonts,FontStyles}.xml.
//! Resolve inherited properties before a declaration applies its explicit local overrides.
use crate::frame::{Frame, WidgetData};
use crate::widgets::font_string::{FontStringData, GameFont, JustifyH, JustifyV};

const GOLD: [f32; 4] = [1.0, 0.82, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
const DISABLED: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const BLACK_SHADOW: FontProperty = FontProperty::Shadow([0.0, 0.0, 0.0, 1.0], [1.0, -1.0]);
const FRIZ: FontProperty = FontProperty::Face(GameFont::FrizQuadrata);

#[derive(Clone, Copy)]
enum FontProperty {
    Face(GameFont),
    Size(f32),
    Color([f32; 4]),
    Shadow([f32; 4], [f32; 2]),
    Horizontal(JustifyH),
    Vertical(JustifyV),
}

struct FontObject {
    name: &'static str,
    parent: Option<&'static str>,
    properties: &'static [FontProperty],
}

use FontProperty::{Color, Horizontal, Size, Vertical};

// Preserve the source chains: similarly named families do not imply the same size.
const OBJECTS: &[FontObject] = &[
    FontObject {
        name: "SystemFont_Shadow_Small",
        parent: None,
        properties: &[FRIZ, Size(10.0), BLACK_SHADOW],
    },
    FontObject {
        name: "SystemFont_Shadow_Small2",
        parent: None,
        properties: &[FRIZ, Size(11.0), BLACK_SHADOW],
    },
    FontObject {
        name: "SystemFont_Shadow_Med1",
        parent: None,
        properties: &[FRIZ, Size(12.0), BLACK_SHADOW],
    },
    FontObject {
        name: "SystemFont_Shadow_Med2",
        parent: None,
        properties: &[FRIZ, Size(14.0), BLACK_SHADOW],
    },
    FontObject {
        name: "SystemFont_Shadow_Large",
        parent: None,
        properties: &[FRIZ, Size(16.0), BLACK_SHADOW],
    },
    FontObject {
        name: "SystemFont_Med1",
        parent: None,
        properties: &[FRIZ, Size(12.0)],
    },
    FontObject {
        name: "SystemFont_Med2",
        parent: None,
        properties: &[FRIZ, Size(13.0)],
    },
    FontObject {
        name: "Tooltip_Med",
        parent: None,
        properties: &[FRIZ, Size(12.0)],
    },
    FontObject {
        name: "GameTooltipHeader",
        parent: None,
        properties: &[FRIZ, Size(14.0)],
    },
    FontObject {
        name: "GameTooltipText",
        parent: Some("Tooltip_Med"),
        properties: &[Color(WHITE), Horizontal(JustifyH::Left)],
    },
    FontObject {
        name: "GameFontNormal",
        parent: Some("SystemFont_Shadow_Med1"),
        properties: &[Color(GOLD)],
    },
    FontObject {
        name: "GameFontNormalLeft",
        parent: Some("GameFontNormal"),
        properties: &[Horizontal(JustifyH::Left)],
    },
    FontObject {
        name: "GameFontNormalSmall",
        parent: Some("SystemFont_Shadow_Small"),
        properties: &[Color(GOLD)],
    },
    FontObject {
        name: "GameFontNormalSmallLeft",
        parent: Some("GameFontNormalSmall"),
        properties: &[Horizontal(JustifyH::Left)],
    },
    FontObject {
        name: "GameFontNormalSmall2",
        parent: Some("SystemFont_Shadow_Small2"),
        properties: &[Color(GOLD)],
    },
    FontObject {
        name: "GameFontNormalMed1",
        parent: Some("SystemFont_Med2"),
        properties: &[BLACK_SHADOW, Color(GOLD)],
    },
    FontObject {
        name: "GameFontNormalMed2",
        parent: Some("SystemFont_Shadow_Med2"),
        properties: &[Color(GOLD)],
    },
    FontObject {
        name: "GameFontNormalLarge",
        parent: Some("SystemFont_Shadow_Large"),
        properties: &[Color(GOLD)],
    },
    FontObject {
        name: "GameFontHighlight",
        parent: Some("GameFontNormal"),
        properties: &[Color(WHITE)],
    },
    FontObject {
        name: "GameFontHighlightLeft",
        parent: Some("GameFontHighlight"),
        properties: &[Horizontal(JustifyH::Left)],
    },
    FontObject {
        name: "GameFontHighlightSmall",
        parent: Some("GameFontNormalSmall"),
        properties: &[Color(WHITE)],
    },
    FontObject {
        name: "GameFontHighlightSmallLeft",
        parent: Some("GameFontHighlightSmall"),
        properties: &[Horizontal(JustifyH::Left)],
    },
    FontObject {
        name: "GameFontHighlightSmallRight",
        parent: Some("GameFontHighlightSmall"),
        properties: &[Horizontal(JustifyH::Right)],
    },
    FontObject {
        name: "GameFontHighlightSmallLeftTop",
        parent: Some("GameFontHighlightSmallLeft"),
        properties: &[Vertical(JustifyV::Top)],
    },
    FontObject {
        name: "GameFontHighlightLarge",
        parent: Some("GameFontNormalLarge"),
        properties: &[Color(WHITE)],
    },
    FontObject {
        name: "GameFontDisable",
        parent: Some("GameFontNormal"),
        properties: &[Color(DISABLED)],
    },
    FontObject {
        name: "GameFontDisableSmall",
        parent: Some("GameFontNormalSmall"),
        properties: &[Color(DISABLED)],
    },
];

/// Resolve one supported source object. Unknown names are errors, not default fonts.
pub fn resolve_font_object(name: &str) -> FontStringData {
    let definition = OBJECTS
        .iter()
        .find(|object| object.name == name)
        .unwrap_or_else(|| panic!("unsupported Retail font object: {name}"));
    let mut font = definition
        .parent
        .map(resolve_font_object)
        .unwrap_or_default();
    for property in definition.properties {
        match *property {
            FontProperty::Face(value) => font.font = value,
            Size(value) => font.font_size = value,
            Color(value) => font.color = value,
            FontProperty::Shadow(color, offset) => {
                font.shadow_color = Some(color);
                font.shadow_offset = offset;
            }
            Horizontal(value) => font.justify_h = value,
            Vertical(value) => font.justify_v = value,
        }
    }
    font
}

/// Apply or remove declaration inheritance without changing text, wrapping or line limits.
pub(crate) fn apply_font_object(frame: &mut Frame, name: Option<&str>) {
    if name.is_none() && frame.font_object.is_none() {
        return;
    }
    let style = name.map(resolve_font_object).unwrap_or_default();
    let Some(WidgetData::FontString(font)) = &mut frame.widget_data else {
        panic!("font_object requires a FontString");
    };
    font.font = style.font;
    font.font_size = style.font_size;
    font.color = style.color;
    font.shadow_color = style.shadow_color;
    font.shadow_offset = style.shadow_offset;
    font.justify_h = style.justify_h;
    font.justify_v = style.justify_v;
    font.outline = style.outline;
    font.line_spacing = style.line_spacing;
    frame.font_object = name.map(str::to_owned);
}
