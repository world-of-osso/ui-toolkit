use super::texture::TextureSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct SliderData {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub orientation: Orientation,
    pub thumb_texture: Option<TextureSource>,
    pub obey_step_on_drag: bool,
    pub steps_per_page: u32,
}

impl Default for SliderData {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: 0.0,
            orientation: Orientation::Horizontal,
            thumb_texture: None,
            obey_step_on_drag: false,
            steps_per_page: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillStyle {
    Standard,
    Center,
}

#[derive(Debug, Clone)]
pub struct StatusBarData {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub fill_style: FillStyle,
    pub orientation: Orientation,
    pub reverse_fill: bool,
    pub color: [f32; 4],
    pub texture: Option<TextureSource>,
}

impl Default for StatusBarData {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            fill_style: FillStyle::Standard,
            orientation: Orientation::Horizontal,
            reverse_fill: false,
            color: [0.0, 1.0, 0.0, 1.0],
            texture: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slider_data() {
        let s = SliderData::default();
        assert_eq!(s.value, 0.0);
        assert_eq!(s.max, 1.0);
        assert_eq!(s.orientation, Orientation::Horizontal);
    }

    #[test]
    fn default_status_bar_data() {
        let sb = StatusBarData::default();
        assert_eq!(sb.fill_style, FillStyle::Standard);
        assert!(!sb.reverse_fill);
    }
}
