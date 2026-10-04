//! Text outline offsets shared by native text projection.

use crate::widgets::font_string::Outline;

pub(crate) fn outline_offsets(outline: Outline) -> &'static [(f32, f32)] {
    match outline {
        Outline::None => &[],
        Outline::Outline => &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)],
        Outline::ThickOutline => &[
            (-2.0, 0.0),
            (2.0, 0.0),
            (0.0, -2.0),
            (0.0, 2.0),
            (-1.4, -1.4),
            (1.4, -1.4),
            (-1.4, 1.4),
            (1.4, 1.4),
        ],
    }
}
