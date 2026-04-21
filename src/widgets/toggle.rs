use std::fmt;

use crate::anchor::AnchorPoint;
use crate::rsx;
use crate::widget_def::Element;

struct DynName(String);

impl fmt::Display for DynName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy)]
pub struct ToggleWidget<'a> {
    pub name: &'a str,
    pub action: &'a str,
    pub right_selected: bool,
    pub width: f32,
    pub height: f32,
    pub left_label: &'a str,
    pub right_label: &'a str,
    pub background_color: &'a str,
    pub active_color: &'a str,
    pub border: &'a str,
    pub active_text_color: &'a str,
    pub idle_text_color: &'a str,
    pub x: &'a str,
}

pub fn toggle_widget(spec: ToggleWidget<'_>) -> Element {
    let active_x = segment_x(spec.right_selected, spec.width);
    rsx! {
        r#frame {
            name: {DynName(spec.name.to_string())},
            width: {spec.width},
            height: {spec.height},
            background_color: spec.background_color,
            border: spec.border,
            anchor {
                point: AnchorPoint::Right,
                relative_point: AnchorPoint::Right,
                x: {spec.x},
            }
            {toggle_active_panel(&spec, &active_x)}
            {toggle_segment(&spec, "Left", spec.left_label, !spec.right_selected)}
            {toggle_segment(&spec, "Right", spec.right_label, spec.right_selected)}
        }
    }
}

fn toggle_active_panel(spec: &ToggleWidget<'_>, active_x: &str) -> Element {
    rsx! {
        r#frame {
            name: {DynName(format!("{}Active", spec.name))},
            width: {segment_width(spec.width)},
            height: {spec.height},
            background_color: spec.active_color,
            anchor {
                point: AnchorPoint::Left,
                relative_point: AnchorPoint::Left,
                x: {active_x},
            }
        }
    }
}

fn toggle_segment(spec: &ToggleWidget<'_>, side: &str, label: &str, active: bool) -> Element {
    let x = segment_x(side == "Right", spec.width);
    rsx! {
        r#frame {
            name: {DynName(format!("{}{side}", spec.name))},
            width: {segment_width(spec.width)},
            height: {spec.height},
            anchor {
                point: AnchorPoint::Left,
                relative_point: AnchorPoint::Left,
                x: {x},
            }
            {toggle_segment_label(spec, side, label, active)}
            {toggle_segment_hitbox(spec, side, active)}
        }
    }
}

fn toggle_segment_label(spec: &ToggleWidget<'_>, side: &str, label: &str, active: bool) -> Element {
    let color = text_color(spec, active);
    rsx! {
        fontstring {
            name: {DynName(format!("{}{side}Label", spec.name))},
            width: {segment_width(spec.width)},
            height: {spec.height},
            text: {label},
            font_size: 14.0,
            color: color,
            justify_h: "CENTER",
            anchor {
                point: AnchorPoint::Center,
                relative_point: AnchorPoint::Center,
            }
        }
    }
}

fn toggle_segment_hitbox(spec: &ToggleWidget<'_>, side: &str, active: bool) -> Element {
    if active {
        return Vec::new();
    }
    rsx! {
        r#frame {
            name: {DynName(format!("{}{side}Hit", spec.name))},
            width: {segment_width(spec.width)},
            height: {spec.height},
            onclick: {spec.action},
        }
    }
}

fn segment_width(width: f32) -> f32 {
    width * 0.5
}

fn segment_x(right: bool, width: f32) -> String {
    if right {
        segment_width(width).to_string()
    } else {
        "0".to_string()
    }
}

fn text_color<'a>(spec: &'a ToggleWidget<'_>, active: bool) -> &'a str {
    if active {
        spec.active_text_color
    } else {
        spec.idle_text_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_widget<'a>(
        children: &'a [crate::widget_def::WidgetChild],
        name: &str,
    ) -> Option<&'a crate::widget_def::WidgetDef> {
        children.iter().find_map(|child| match child {
            crate::widget_def::WidgetChild::Widget(widget) => {
                if widget.name.as_deref() == Some(name) {
                    Some(widget)
                } else {
                    find_widget(&widget.children, name)
                }
            }
            crate::widget_def::WidgetChild::Fragment(children) => find_widget(children, name),
            crate::widget_def::WidgetChild::Dynamic => None,
        })
    }

    #[test]
    fn toggle_widget_emits_active_panel_and_inactive_hitbox() {
        let el = toggle_widget(ToggleWidget {
            name: "MuteToggle",
            action: "toggle:mute",
            right_selected: true,
            width: 170.0,
            height: 28.0,
            left_label: "Off",
            right_label: "On",
            background_color: "0,0,0,1",
            active_color: "1,1,1,1",
            border: "1px solid 1,1,1,1",
            active_text_color: "1,1,1,1",
            idle_text_color: "0.5,0.5,0.5,1",
            x: "-8",
        });

        assert!(find_widget(&el, "MuteToggleActive").is_some());
        assert!(find_widget(&el, "MuteToggleLeftHit").is_some());
        assert!(find_widget(&el, "MuteToggleRightHit").is_none());
    }
}
