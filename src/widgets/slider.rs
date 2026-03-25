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

const DEFAULT_HORIZONTAL_THUMB_TEXTURE: &str = "data/ui/sliderbar-button-horizontal.png";
const DECORATED_TRACK_BACKGROUND_TEXTURE: &str = "data/ui/sliderbar-background.png";
const DECORATED_TRACK_POST_TEXTURE: &str = "data/ui/sliderbar-post.png";
const DECORATED_TRACK_RIGHT_ORNAMENT_TEXTURE: &str = "data/ui/sliderbar-ornament-right.png";
const DECORATED_TRACK_LEFT_WIDTH: f32 = 16.0;
const DECORATED_TRACK_RIGHT_WIDTH: f32 = 32.0;
const THUMB_FRAME_COLOR: &str = "0.08,0.06,0.03,0.82";
const THUMB_PLATE_COLOR: &str = "0.40,0.30,0.12,0.20";
const THUMB_HIGHLIGHT_COLOR: &str = "1.0,0.95,0.84,0.20";
const THUMB_GRIP_COLOR: &str = "0.20,0.14,0.04,0.20";

pub fn slider_widget(spec: SliderWidget<'_>) -> Element {
    let pct = normalize(spec.value, spec.min, spec.max).clamp(0.0, 1.0);
    let thumb_x = ((spec.width - spec.thumb_width) * pct).to_string();
    let track_name = DynName(format!("{}Track", spec.name));
    let thumb_texture = spec
        .thumb_texture
        .unwrap_or(DEFAULT_HORIZONTAL_THUMB_TEXTURE);
    let center_width =
        (spec.width - DECORATED_TRACK_LEFT_WIDTH - DECORATED_TRACK_RIGHT_WIDTH).max(0.0);
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
                background_color: spec.track_color,
                anchor {
                    point: AnchorPoint::Center,
                    relative_point: AnchorPoint::Center,
                }
                texture {
                    name: {DynName(format!("{}TrackLeft", spec.name))},
                    width: DECORATED_TRACK_LEFT_WIDTH,
                    height: {spec.track_height},
                    texture_file: DECORATED_TRACK_POST_TEXTURE,
                    anchor {
                        point: AnchorPoint::Left,
                        relative_point: AnchorPoint::Left,
                    }
                }
                texture {
                    name: {DynName(format!("{}TrackCenter", spec.name))},
                    width: {center_width},
                    height: {spec.track_height},
                    texture_file: DECORATED_TRACK_BACKGROUND_TEXTURE,
                    anchor {
                        point: AnchorPoint::Left,
                        relative_point: AnchorPoint::Left,
                        x: {DECORATED_TRACK_LEFT_WIDTH.to_string()},
                    }
                }
                texture {
                    name: {DynName(format!("{}TrackRightOrnament", spec.name))},
                    width: DECORATED_TRACK_RIGHT_WIDTH,
                    height: {spec.track_height},
                    texture_file: DECORATED_TRACK_RIGHT_ORNAMENT_TEXTURE,
                    anchor {
                        point: AnchorPoint::Right,
                        relative_point: AnchorPoint::Right,
                    }
                }
            }
            statusbar {
                name: {DynName(format!("{}Fill", spec.name))},
                width: {spec.width},
                height: {spec.track_height},
                value: {pct},
                min: 0.0,
                max: 1.0,
                statusbar_color: spec.fill_color,
                anchor {
                    point: AnchorPoint::Left,
                    relative_to: {&track_name},
                    relative_point: AnchorPoint::Left,
                }
            }
            r#frame {
                name: {DynName(format!("{}ThumbFrame", spec.name))},
                width: {spec.thumb_width},
                height: {spec.thumb_height},
                background_color: THUMB_FRAME_COLOR,
                anchor {
                    point: AnchorPoint::Left,
                    relative_to: {&track_name},
                    relative_point: AnchorPoint::Left,
                    x: {thumb_x},
                }
                r#frame {
                    name: {DynName(format!("{}ThumbPlate", spec.name))},
                    width: {spec.thumb_width - 4.0},
                    height: {spec.thumb_height - 4.0},
                    background_color: THUMB_PLATE_COLOR,
                    anchor {
                        point: AnchorPoint::Center,
                        relative_point: AnchorPoint::Center,
                    }
                    texture {
                        name: {DynName(format!("{}ThumbTexture", spec.name))},
                        width: {spec.thumb_width - 2.0},
                        height: {spec.thumb_height - 2.0},
                        texture_file: thumb_texture,
                        anchor {
                            point: AnchorPoint::Center,
                            relative_point: AnchorPoint::Center,
                        }
                    }
                    r#frame {
                        name: {DynName(format!("{}ThumbHighlight", spec.name))},
                        width: {spec.thumb_width - 8.0},
                        height: 2.0,
                        background_color: THUMB_HIGHLIGHT_COLOR,
                        anchor {
                            point: AnchorPoint::Top,
                            relative_point: AnchorPoint::Top,
                            y: "2",
                        }
                    }
                    r#frame {
                        name: {DynName(format!("{}ThumbGrip", spec.name))},
                        width: 3.0,
                        height: {spec.thumb_height - 10.0},
                        background_color: THUMB_GRIP_COLOR,
                        anchor {
                            point: AnchorPoint::Center,
                            relative_point: AnchorPoint::Center,
                        }
                    }
                }
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

    fn find_statusbar_child(
        children: &[crate::widget_def::WidgetChild],
    ) -> Option<&crate::widget_def::WidgetDef> {
        children.iter().find_map(|child| match child {
            crate::widget_def::WidgetChild::Widget(widget)
                if widget.effective_tag() == "StatusBar" =>
            {
                Some(widget)
            }
            crate::widget_def::WidgetChild::Fragment(children) => find_statusbar_child(children),
            _ => None,
        })
    }

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

    #[test]
    fn slider_widget_emits_slider_root_and_statusbar_fill() {
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
        assert_eq!(root.children.len(), 3);
        let fill = find_statusbar_child(&root.children).expect("expected fill widget");
        assert_eq!(fill.effective_tag(), "StatusBar");
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
            Some(DEFAULT_HORIZONTAL_THUMB_TEXTURE)
        );
    }
}
