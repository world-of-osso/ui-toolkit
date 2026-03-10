/// A typed frame name for use in `name:` and `relative_to:` RSX attributes.
/// Ensures the same constant is used at both definition and reference sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameName(pub &'static str);

impl FrameName {
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl dioxus_core::IntoAttributeValue for FrameName {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.0.to_string())
    }
}

/// WoW-style anchor points for UI frame positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorPoint {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl AnchorPoint {
    /// Parse a WoW anchor string like "TOPLEFT", "CENTER", etc.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "TOPLEFT" => Some(Self::TopLeft),
            "TOP" => Some(Self::Top),
            "TOPRIGHT" => Some(Self::TopRight),
            "LEFT" => Some(Self::Left),
            "CENTER" => Some(Self::Center),
            "RIGHT" => Some(Self::Right),
            "BOTTOMLEFT" => Some(Self::BottomLeft),
            "BOTTOM" => Some(Self::Bottom),
            "BOTTOMRIGHT" => Some(Self::BottomRight),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "TOPLEFT",
            Self::Top => "TOP",
            Self::TopRight => "TOPRIGHT",
            Self::Left => "LEFT",
            Self::Center => "CENTER",
            Self::Right => "RIGHT",
            Self::BottomLeft => "BOTTOMLEFT",
            Self::Bottom => "BOTTOM",
            Self::BottomRight => "BOTTOMRIGHT",
        }
    }
}

impl dioxus_core::IntoAttributeValue for AnchorPoint {
    fn into_value(self) -> dioxus_core::AttributeValue {
        dioxus_core::AttributeValue::Text(self.as_str().to_string())
    }
}

/// A resolved anchor linking one frame's point to another frame's point with offsets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub point: AnchorPoint,
    pub relative_to: Option<u64>,
    pub relative_point: AnchorPoint,
    pub x_offset: f32,
    pub y_offset: f32,
}

/// Given a rectangle at (x, y) with size (w, h), return the pixel position
/// of the named anchor point.
pub fn anchor_position(point: AnchorPoint, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w * 0.5, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h * 0.5),
        AnchorPoint::Center => (x + w * 0.5, y + h * 0.5),
        AnchorPoint::Right => (x + w, y + h * 0.5),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w * 0.5, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

/// Given that a frame's anchor point should be at (target_x, target_y),
/// return the top-left corner position for a frame of size (w, h).
pub fn frame_position_from_anchor(
    point: AnchorPoint,
    target_x: f32,
    target_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (target_x, target_y),
        AnchorPoint::Top => (target_x - w * 0.5, target_y),
        AnchorPoint::TopRight => (target_x - w, target_y),
        AnchorPoint::Left => (target_x, target_y - h * 0.5),
        AnchorPoint::Center => (target_x - w * 0.5, target_y - h * 0.5),
        AnchorPoint::Right => (target_x - w, target_y - h * 0.5),
        AnchorPoint::BottomLeft => (target_x, target_y - h),
        AnchorPoint::Bottom => (target_x - w * 0.5, target_y - h),
        AnchorPoint::BottomRight => (target_x - w, target_y - h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_all_points() {
        assert_eq!(AnchorPoint::from_str("TOPLEFT"), Some(AnchorPoint::TopLeft));
        assert_eq!(AnchorPoint::from_str("TOP"), Some(AnchorPoint::Top));
        assert_eq!(
            AnchorPoint::from_str("TOPRIGHT"),
            Some(AnchorPoint::TopRight)
        );
        assert_eq!(AnchorPoint::from_str("LEFT"), Some(AnchorPoint::Left));
        assert_eq!(AnchorPoint::from_str("CENTER"), Some(AnchorPoint::Center));
        assert_eq!(AnchorPoint::from_str("RIGHT"), Some(AnchorPoint::Right));
        assert_eq!(
            AnchorPoint::from_str("BOTTOMLEFT"),
            Some(AnchorPoint::BottomLeft)
        );
        assert_eq!(AnchorPoint::from_str("BOTTOM"), Some(AnchorPoint::Bottom));
        assert_eq!(
            AnchorPoint::from_str("BOTTOMRIGHT"),
            Some(AnchorPoint::BottomRight)
        );
    }

    #[test]
    fn from_str_invalid() {
        assert_eq!(AnchorPoint::from_str("INVALID"), None);
        assert_eq!(AnchorPoint::from_str("topleft"), None);
        assert_eq!(AnchorPoint::from_str(""), None);
    }

    #[test]
    fn anchor_position_topleft() {
        let (px, py) = anchor_position(AnchorPoint::TopLeft, 10.0, 20.0, 100.0, 50.0);
        assert_eq!((px, py), (10.0, 20.0));
    }

    #[test]
    fn anchor_position_center() {
        let (px, py) = anchor_position(AnchorPoint::Center, 10.0, 20.0, 100.0, 50.0);
        assert_eq!((px, py), (60.0, 45.0));
    }

    #[test]
    fn anchor_position_bottomright() {
        let (px, py) = anchor_position(AnchorPoint::BottomRight, 10.0, 20.0, 100.0, 50.0);
        assert_eq!((px, py), (110.0, 70.0));
    }

    #[test]
    fn frame_position_roundtrip() {
        let (x, y, w, h) = (10.0, 20.0, 100.0, 50.0);
        let points = [
            AnchorPoint::TopLeft,
            AnchorPoint::Top,
            AnchorPoint::TopRight,
            AnchorPoint::Left,
            AnchorPoint::Center,
            AnchorPoint::Right,
            AnchorPoint::BottomLeft,
            AnchorPoint::Bottom,
            AnchorPoint::BottomRight,
        ];
        for point in points {
            let (ax, ay) = anchor_position(point, x, y, w, h);
            let (rx, ry) = frame_position_from_anchor(point, ax, ay, w, h);
            assert!(
                (rx - x).abs() < f32::EPSILON && (ry - y).abs() < f32::EPSILON,
                "Roundtrip failed for {point:?}: got ({rx}, {ry}), expected ({x}, {y})",
            );
        }
    }
}
