use std::fmt;

use super::texture::TextureSource;
use crate::anchor::AnchorPoint;
use crate::rsx;
use crate::widget_def::Element;

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

impl SliderData {
    /// Increment value by one step (or 0.1 if step is zero). Clamped to max.
    pub fn increment(&mut self) {
        let step = if self.step > 0.0 { self.step } else { 0.1 };
        self.value = (self.value + step).min(self.max);
    }

    /// Decrement value by one step (or 0.1 if step is zero). Clamped to min.
    pub fn decrement(&mut self) {
        let step = if self.step > 0.0 { self.step } else { 0.1 };
        self.value = (self.value - step).max(self.min);
    }

    /// Set value with clamping to [min, max].
    pub fn set_clamped(&mut self, value: f64) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Whether the slider is at its minimum value.
    pub fn is_at_min(&self) -> bool {
        self.value <= self.min
    }

    /// Whether the slider is at its maximum value.
    pub fn is_at_max(&self) -> bool {
        self.value >= self.max
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

struct DynName(String);

impl fmt::Display for DynName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy)]
pub struct SliderWidget<'a> {
    pub name: &'a str,
    pub action: &'a str,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub width: f32,
    pub interactive_height: f32,
    pub track_height: f32,
    pub thumb_width: f32,
    pub thumb_height: f32,
    pub thumb_texture: Option<&'a str>,
    pub track_color: &'a str,
    pub fill_color: &'a str,
    pub x: &'a str,
}

const DEFAULT_HORIZONTAL_HANDLE_TEXTURE: &str = "data/ui/sliderbar-handle.ktx2";
const TRACK_LEFT: &str = "data/ui/sliderbar-track-left.ktx2";
const TRACK_CENTER: &str = "data/ui/sliderbar-track-center.ktx2";
const TRACK_RIGHT: &str = "data/ui/sliderbar-track-right.ktx2";
const FILL_LEFT: &str = "data/ui/sliderbar-track-filled-left.ktx2";
const FILL_CENTER: &str = "data/ui/sliderbar-track-filled-center.ktx2";
const CAP_WIDTH: f32 = 8.0;
const FILL_BORDER: f32 = 2.0;

pub fn slider_widget(spec: SliderWidget<'_>) -> Element {
    let pct = normalize(spec.value, spec.min, spec.max).clamp(0.0, 1.0);
    let thumb_x_val = (spec.width - spec.thumb_width) * pct;
    let thumb_x = thumb_x_val.to_string();
    let track_name = DynName(format!("{}Track", spec.name));
    let thumb_texture = spec
        .thumb_texture
        .unwrap_or(DEFAULT_HORIZONTAL_HANDLE_TEXTURE);
    let track_center_w = (spec.width - CAP_WIDTH * 2.0).max(0.0);
    let track_center_x = CAP_WIDTH.to_string();
    let fill_center_w = (thumb_x_val + spec.thumb_width * 0.5 - CAP_WIDTH).max(0.0);
    let fill_center_x = CAP_WIDTH.to_string();
    let fill_height = spec.track_height - FILL_BORDER * 2.0;
    let show_fill = pct > 0.001;
    rsx! {
        slider {
            name: {DynName(spec.name.to_string())},
            width: {spec.width},
            height: {spec.interactive_height},
            value: {spec.value},
            min: {spec.min},
            max: {spec.max},
            thumb_texture: {thumb_texture},
            mouse_enabled: true,
            onclick: {spec.action},
            anchor {
                point: AnchorPoint::Left,
                relative_point: AnchorPoint::Left,
                x: {spec.x},
            }
            r#frame {
                name: {&track_name},
                width: {spec.width},
                height: {spec.track_height},
                anchor {
                    point: AnchorPoint::Center,
                    relative_point: AnchorPoint::Center,
                }
                // Empty track: left cap + center + right cap
                texture {
                    name: {DynName(format!("{}TrackL", spec.name))},
                    width: CAP_WIDTH,
                    height: {spec.track_height},
                    texture_file: TRACK_LEFT,
                    anchor {
                        point: AnchorPoint::Left,
                        relative_point: AnchorPoint::Left,
                    }
                }
                texture {
                    name: {DynName(format!("{}TrackC", spec.name))},
                    width: {track_center_w},
                    height: {spec.track_height},
                    texture_file: TRACK_CENTER,
                    anchor {
                        point: AnchorPoint::Left,
                        relative_point: AnchorPoint::Left,
                        x: {track_center_x},
                    }
                }
                texture {
                    name: {DynName(format!("{}TrackR", spec.name))},
                    width: CAP_WIDTH,
                    height: {spec.track_height},
                    texture_file: TRACK_RIGHT,
                    anchor {
                        point: AnchorPoint::Right,
                        relative_point: AnchorPoint::Right,
                    }
                }
                // Filled track: left cap + center (no right cap — ends at handle)
                {fill_left_cap(spec.name, fill_height, show_fill)}
                {fill_center(spec.name, fill_center_w, fill_height, &fill_center_x, show_fill)}
                texture {
                    name: {DynName(format!("{}Handle", spec.name))},
                    width: {spec.thumb_width},
                    height: {spec.thumb_height},
                    texture_file: thumb_texture,
                    anchor {
                        point: AnchorPoint::Left,
                        relative_point: AnchorPoint::Left,
                        x: {thumb_x},
                    }
                }
            }
        }
    }
}

fn fill_left_cap(name: &str, height: f32, show: bool) -> Element {
    if !show {
        return vec![];
    }
    rsx! {
        texture {
            name: {DynName(format!("{name}FillL"))},
            width: CAP_WIDTH,
            height: {height},
            texture_file: FILL_LEFT,
            anchor {
                point: AnchorPoint::Left,
                relative_point: AnchorPoint::Left,
            }
        }
    }
}

fn fill_center(name: &str, width: f32, height: f32, x: &str, show: bool) -> Element {
    if !show || width <= 0.0 {
        return vec![];
    }
    rsx! {
        texture {
            name: {DynName(format!("{name}FillC"))},
            width: {width},
            height: {height},
            texture_file: FILL_CENTER,
            anchor {
                point: AnchorPoint::Left,
                relative_point: AnchorPoint::Left,
                x: {x},
            }
        }
    }
}

fn normalize(value: f32, min: f32, max: f32) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        0.0
    } else {
        (value - min) / (max - min)
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

    // --- Stepper: increment/decrement with bounds clamping ---

    #[test]
    fn increment_advances_by_step() {
        let mut s = SliderData {
            value: 0.5,
            step: 0.1,
            ..Default::default()
        };
        s.increment();
        assert!((s.value - 0.6).abs() < 0.001);
    }

    #[test]
    fn decrement_retreats_by_step() {
        let mut s = SliderData {
            value: 0.5,
            step: 0.1,
            ..Default::default()
        };
        s.decrement();
        assert!((s.value - 0.4).abs() < 0.001);
    }

    #[test]
    fn increment_clamped_at_max() {
        let mut s = SliderData {
            value: 0.95,
            step: 0.1,
            ..Default::default()
        };
        s.increment();
        assert_eq!(s.value, 1.0);
        assert!(s.is_at_max());
    }

    #[test]
    fn decrement_clamped_at_min() {
        let mut s = SliderData {
            value: 0.05,
            step: 0.1,
            ..Default::default()
        };
        s.decrement();
        assert_eq!(s.value, 0.0);
        assert!(s.is_at_min());
    }

    #[test]
    fn increment_at_max_stays_at_max() {
        let mut s = SliderData {
            value: 1.0,
            step: 0.1,
            ..Default::default()
        };
        s.increment();
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn decrement_at_min_stays_at_min() {
        let mut s = SliderData {
            value: 0.0,
            step: 0.1,
            ..Default::default()
        };
        s.decrement();
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn increment_uses_default_step_when_zero() {
        let mut s = SliderData::default(); // step = 0.0
        s.increment();
        assert!((s.value - 0.1).abs() < 0.001);
    }

    #[test]
    fn set_clamped_within_range() {
        let mut s = SliderData::default();
        s.set_clamped(0.5);
        assert_eq!(s.value, 0.5);
    }

    #[test]
    fn set_clamped_above_max() {
        let mut s = SliderData::default();
        s.set_clamped(5.0);
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn set_clamped_below_min() {
        let mut s = SliderData::default();
        s.set_clamped(-1.0);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn is_at_min_and_max() {
        let min = SliderData::default();
        assert!(min.is_at_min());
        assert!(!min.is_at_max());
        let max = SliderData {
            value: 1.0,
            ..Default::default()
        };
        assert!(!max.is_at_min());
        assert!(max.is_at_max());
    }

    #[test]
    fn custom_range_increment() {
        let mut s = SliderData {
            value: 50.0,
            min: 0.0,
            max: 100.0,
            step: 10.0,
            ..Default::default()
        };
        s.increment();
        assert!((s.value - 60.0).abs() < 0.001);
        s.increment();
        s.increment();
        s.increment();
        s.increment();
        assert_eq!(s.value, 100.0); // clamped
    }

    #[test]
    fn slider_widget_emits_slider_root_with_track_frame() {
        let el = slider_widget(SliderWidget {
            name: "MasterVolume",
            action: "options_slider:master_volume",
            value: 0.5,
            min: 0.0,
            max: 1.0,
            width: 270.0,
            interactive_height: 28.0,
            track_height: 10.0,
            thumb_width: 18.0,
            thumb_height: 22.0,
            thumb_texture: None,
            track_color: "0,0,0,1",
            fill_color: "1,1,1,1",
            x: "286",
        });
        let crate::widget_def::WidgetChild::Widget(root) = &el[0] else {
            panic!("expected root widget")
        };
        assert_eq!(root.effective_tag(), "Slider");
        // Root slider contains one track frame child.
        assert_eq!(root.children.len(), 1);
    }

    #[test]
    fn slider_widget_uses_default_thumb_texture() {
        let el = slider_widget(SliderWidget {
            name: "MasterVolume",
            action: "options_slider:master_volume",
            value: 0.5,
            min: 0.0,
            max: 1.0,
            width: 270.0,
            interactive_height: 28.0,
            track_height: 10.0,
            thumb_width: 18.0,
            thumb_height: 22.0,
            thumb_texture: None,
            track_color: "0,0,0,1",
            fill_color: "1,1,1,1",
            x: "286",
        });
        let crate::widget_def::WidgetChild::Widget(root) = &el[0] else {
            panic!("expected root widget")
        };
        assert_eq!(
            root.attrs
                .iter()
                .find(|attr| attr.effective_name() == "thumb_texture")
                .map(|attr| attr.value_str()),
            Some(DEFAULT_HORIZONTAL_HANDLE_TEXTURE)
        );
    }
}
