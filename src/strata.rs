/// WoW frame strata levels controlling render order of UI frames.
/// Higher strata render on top of lower ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum FrameStrata {
    World = 0,
    Background = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Dialog = 5,
    Fullscreen = 6,
    FullscreenDialog = 7,
    Tooltip = 8,
}

impl FrameStrata {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::World => "WORLD",
            Self::Background => "BACKGROUND",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Dialog => "DIALOG",
            Self::Fullscreen => "FULLSCREEN",
            Self::FullscreenDialog => "FULLSCREEN_DIALOG",
            Self::Tooltip => "TOOLTIP",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "WORLD" => Some(Self::World),
            "BACKGROUND" => Some(Self::Background),
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            "DIALOG" => Some(Self::Dialog),
            "FULLSCREEN" => Some(Self::Fullscreen),
            "FULLSCREEN_DIALOG" => Some(Self::FullscreenDialog),
            "TOOLTIP" => Some(Self::Tooltip),
            _ => None,
        }
    }
}

impl dioxus_core::IntoAttributeValue for FrameStrata {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.as_str().to_string())
    }
}

impl Default for FrameStrata {
    fn default() -> Self {
        Self::Medium
    }
}

/// WoW draw layer levels controlling render order within a single frame.
/// Higher layers render on top of lower ones within the same strata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum DrawLayer {
    Background = 0,
    Border = 1,
    Artwork = 2,
    Overlay = 3,
    Highlight = 4,
}

impl DrawLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Background => "BACKGROUND",
            Self::Border => "BORDER",
            Self::Artwork => "ARTWORK",
            Self::Overlay => "OVERLAY",
            Self::Highlight => "HIGHLIGHT",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "BACKGROUND" => Some(Self::Background),
            "BORDER" => Some(Self::Border),
            "ARTWORK" => Some(Self::Artwork),
            "OVERLAY" => Some(Self::Overlay),
            "HIGHLIGHT" => Some(Self::Highlight),
            _ => None,
        }
    }
}

impl dioxus_core::IntoAttributeValue for DrawLayer {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.as_str().to_string())
    }
}

impl Default for DrawLayer {
    fn default() -> Self {
        Self::Artwork
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strata_ordering() {
        assert!(FrameStrata::World < FrameStrata::Background);
        assert!(FrameStrata::Background < FrameStrata::Medium);
        assert!(FrameStrata::Medium < FrameStrata::Tooltip);
        assert!(FrameStrata::Low < FrameStrata::High);
        assert!(FrameStrata::Dialog < FrameStrata::Fullscreen);
        assert!(FrameStrata::Fullscreen < FrameStrata::FullscreenDialog);
    }

    #[test]
    fn draw_layer_ordering() {
        assert!(DrawLayer::Background < DrawLayer::Artwork);
        assert!(DrawLayer::Artwork < DrawLayer::Highlight);
        assert!(DrawLayer::Border < DrawLayer::Overlay);
    }

    #[test]
    fn strata_from_str_valid() {
        assert_eq!(FrameStrata::from_str("WORLD"), Some(FrameStrata::World));
        assert_eq!(
            FrameStrata::from_str("BACKGROUND"),
            Some(FrameStrata::Background)
        );
        assert_eq!(FrameStrata::from_str("LOW"), Some(FrameStrata::Low));
        assert_eq!(FrameStrata::from_str("MEDIUM"), Some(FrameStrata::Medium));
        assert_eq!(FrameStrata::from_str("HIGH"), Some(FrameStrata::High));
        assert_eq!(FrameStrata::from_str("DIALOG"), Some(FrameStrata::Dialog));
        assert_eq!(
            FrameStrata::from_str("FULLSCREEN"),
            Some(FrameStrata::Fullscreen)
        );
        assert_eq!(
            FrameStrata::from_str("FULLSCREEN_DIALOG"),
            Some(FrameStrata::FullscreenDialog)
        );
        assert_eq!(FrameStrata::from_str("TOOLTIP"), Some(FrameStrata::Tooltip));
    }

    #[test]
    fn strata_from_str_invalid() {
        assert_eq!(FrameStrata::from_str("medium"), None);
        assert_eq!(FrameStrata::from_str(""), None);
        assert_eq!(FrameStrata::from_str("INVALID"), None);
    }

    #[test]
    fn draw_layer_from_str_valid() {
        assert_eq!(
            DrawLayer::from_str("BACKGROUND"),
            Some(DrawLayer::Background)
        );
        assert_eq!(DrawLayer::from_str("BORDER"), Some(DrawLayer::Border));
        assert_eq!(DrawLayer::from_str("ARTWORK"), Some(DrawLayer::Artwork));
        assert_eq!(DrawLayer::from_str("OVERLAY"), Some(DrawLayer::Overlay));
        assert_eq!(DrawLayer::from_str("HIGHLIGHT"), Some(DrawLayer::Highlight));
    }

    #[test]
    fn draw_layer_from_str_invalid() {
        assert_eq!(DrawLayer::from_str("artwork"), None);
        assert_eq!(DrawLayer::from_str(""), None);
        assert_eq!(DrawLayer::from_str("INVALID"), None);
    }

    #[test]
    fn strata_default_is_medium() {
        assert_eq!(FrameStrata::default(), FrameStrata::Medium);
    }

    #[test]
    fn draw_layer_default_is_artwork() {
        assert_eq!(DrawLayer::default(), DrawLayer::Artwork);
    }
}
