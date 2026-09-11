/// Observed bounds in logical UI coordinates, populated from native Bevy layout.
/// These are measurement/input results, never authored layout constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
