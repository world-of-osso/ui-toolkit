use std::collections::HashSet;
use std::path::Path;

use crate::atlas;
use crate::frame::{
    Border, Dimension, FlexAlign, FlexDirection, FlexJustify, FlexLayout, Frame, NineSlice,
    WidgetData, WidgetType,
};
use crate::registry::FrameRegistry;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::{ButtonData, ButtonState};
use crate::widgets::font_string::{GameFont, JustifyH};
use crate::widgets::slider::{FillStyle, Orientation};
use crate::widgets::texture::TextureSource;

fn parse_dimension(value: &str) -> Dimension {
    match value {
        "fill" | "Fill" => Dimension::Fill,
        _ => value
            .parse::<f32>()
            .map(Dimension::Fixed)
            .unwrap_or_default(),
    }
}

fn format_dimension(dim: Dimension) -> String {
    match dim {
        Dimension::Fill => "fill".to_string(),
        Dimension::Fixed(v) => format!("{v}"),
    }
}

pub(crate) fn tag_to_widget_type(tag: &str) -> Option<WidgetType> {
    match tag {
        "frame" | "r#frame" | "Frame" => Some(WidgetType::Frame),
        "panel" | "Panel" => Some(WidgetType::Panel),
        "button" | "Button" => Some(WidgetType::Button),
        "editbox" | "EditBox" => Some(WidgetType::EditBox),
        "fontstring" | "FontString" => Some(WidgetType::FontString),
        "slider" | "Slider" => Some(WidgetType::Slider),
        "statusbar" | "StatusBar" => Some(WidgetType::StatusBar),
        "texture" | "Texture" => Some(WidgetType::Texture),
        _ => None,
    }
}

/// Read the current value of a frame attribute as a string, for change detection.
pub(crate) fn read_attribute(
    registry: &FrameRegistry,
    frame_id: u64,
    name: &str,
) -> Option<String> {
    let frame = registry.get(frame_id)?;
    read_frame_attr(frame, name)
        .or_else(|| read_widget_text_attr(frame, name))
        .or_else(|| read_widget_texture_attr(frame, name))
}

fn read_frame_attr(frame: &Frame, name: &str) -> Option<String> {
    match name {
        "name" => frame.name.clone(),
        "width" => Some(format_dimension(frame.width)),
        "height" => Some(format_dimension(frame.height)),
        "strata" => Some(format!("{:?}", frame.strata)),
        "onclick" => frame.onclick.clone(),
        "hidden" => Some(if frame.visible { "false" } else { "true" }.to_string()),
        "disabled" => match &frame.widget_data {
            Some(WidgetData::Button(b)) => Some(
                if b.state == ButtonState::Disabled {
                    "true"
                } else {
                    "false"
                }
                .to_string(),
            ),
            _ => None,
        },
        "alpha" => Some(format!("{}", frame.alpha)),
        "style" => frame.panel_style.clone(),
        "three_slice_style" => frame.three_slice_style.clone(),
        _ => None,
    }
}

fn read_widget_text_attr(frame: &Frame, name: &str) -> Option<String> {
    match name {
        "text" => match &frame.widget_data {
            Some(WidgetData::FontString(fs)) => Some(fs.text.clone()),
            Some(WidgetData::Button(b)) => Some(b.text.clone()),
            Some(WidgetData::EditBox(eb)) => Some(eb.text.clone()),
            _ => None,
        },
        "font_size" => match &frame.widget_data {
            Some(WidgetData::FontString(fs)) => Some(format!("{}", fs.font_size)),
            Some(WidgetData::EditBox(eb)) => Some(format!("{}", eb.font_size)),
            _ => None,
        },
        "value" => read_slider_numeric_attr(frame, |slider| slider.value, |sb| sb.value),
        "min" => read_slider_numeric_attr(frame, |slider| slider.min, |sb| sb.min),
        "max" => read_slider_numeric_attr(frame, |slider| slider.max, |sb| sb.max),
        "password" => match &frame.widget_data {
            Some(WidgetData::EditBox(eb)) => Some(format!("{}", eb.password)),
            _ => None,
        },
        _ => None,
    }
}

fn read_widget_texture_attr(frame: &Frame, name: &str) -> Option<String> {
    match name {
        "texture_file" => match &frame.widget_data {
            Some(WidgetData::Texture(t)) => match &t.source {
                TextureSource::File(p) => Some(p.clone()),
                _ => None,
            },
            Some(WidgetData::Slider(slider)) => match &slider.thumb_texture {
                Some(TextureSource::File(p)) => Some(p.clone()),
                _ => None,
            },
            Some(WidgetData::StatusBar(sb)) => match &sb.texture {
                Some(TextureSource::File(p)) => Some(p.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn read_slider_numeric_attr(
    frame: &Frame,
    slider: impl FnOnce(&crate::widgets::slider::SliderData) -> f64,
    statusbar: impl FnOnce(&crate::widgets::slider::StatusBarData) -> f64,
) -> Option<String> {
    match &frame.widget_data {
        Some(WidgetData::Slider(data)) => Some(format!("{}", slider(data))),
        Some(WidgetData::StatusBar(data)) => Some(format!("{}", statusbar(data))),
        _ => None,
    }
}

pub(crate) fn apply_attribute(
    registry: &mut FrameRegistry,
    frame_id: u64,
    name: &str,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) -> Option<(u64, String)> {
    if apply_registry_attr(registry, frame_id, name, value) {
        return None;
    }
    if name == "stretch" {
        return apply_stretch_attr(registry, frame_id, value);
    }
    if name == "style" {
        registry.apply_panel_style(frame_id, value);
        return None;
    }
    if name == "three_slice_style" {
        registry.apply_three_slice_style(frame_id, value);
        return None;
    }
    let Some(frame) = registry.get_mut(frame_id) else {
        return None;
    };
    apply_flex_attr(frame, name, value);
    apply_frame_attr(frame, name, value);
    apply_widget_text_attrs(frame, name, value, validated_paths, missing_paths);
    apply_slider_attrs(frame, name, value, validated_paths, missing_paths);
    apply_widget_texture_attrs(frame, name, value, validated_paths, missing_paths);
    None
}

/// Handle attributes that need registry-level access.
fn apply_registry_attr(
    registry: &mut FrameRegistry,
    frame_id: u64,
    name: &str,
    value: &str,
) -> bool {
    match name {
        "name" => registry.set_name(frame_id, value.to_string()),
        "hidden" => set_bool_via(value, |v| registry.set_hidden(frame_id, v)),
        "alpha" => {
            if let Ok(v) = value.parse::<f32>() {
                registry.set_alpha(frame_id, v);
            }
        }
        "disabled" => apply_disabled_attr(registry, frame_id, value),
        _ => return false,
    }
    true
}

fn apply_disabled_attr(registry: &mut FrameRegistry, frame_id: u64, value: &str) {
    let disabled = matches!(value, "true" | "TRUE" | "1");
    if let Some(frame) = registry.get_mut(frame_id) {
        if let Some(WidgetData::Button(bd)) = &mut frame.widget_data {
            if disabled {
                bd.state = ButtonState::Disabled;
            } else if bd.state == ButtonState::Disabled {
                bd.state = ButtonState::Normal;
            }
        }
    }
}

fn apply_stretch_attr(
    registry: &mut FrameRegistry,
    frame_id: u64,
    value: &str,
) -> Option<(u64, String)> {
    if matches!(value, "true" | "TRUE" | "1") {
        let parent_id = registry.get(frame_id).and_then(|f| f.parent_id);
        let _ = registry.stretch_to_fill(frame_id, parent_id);
    }
    None
}

macro_rules! flex {
    ($frame:expr) => {
        $frame.flex_layout.get_or_insert_with(FlexLayout::default)
    };
}

fn apply_flex_attr(frame: &mut Frame, name: &str, value: &str) {
    match name {
        "layout" => {
            flex!(frame).direction = match value {
                "flex-row" => FlexDirection::Row,
                "flex-row-wrap" => FlexDirection::RowWrap,
                _ => FlexDirection::Column,
            };
        }
        "gap" => {
            if let Ok(v) = value.parse::<f32>() {
                flex!(frame).gap = v;
            }
        }
        "justify" => {
            flex!(frame).justify = parse_flex_justify(value);
        }
        "align" => {
            flex!(frame).align = parse_flex_align(value);
        }
        "padding" => {
            if let Ok(v) = value.parse::<f32>() {
                flex!(frame).padding = v;
            }
        }
        _ => {}
    }
}

fn parse_flex_justify(value: &str) -> FlexJustify {
    match value {
        "center" => FlexJustify::Center,
        "end" => FlexJustify::End,
        "space-between" => FlexJustify::SpaceBetween,
        _ => FlexJustify::Start,
    }
}

fn parse_flex_align(value: &str) -> FlexAlign {
    match value {
        "start" => FlexAlign::Start,
        "end" => FlexAlign::End,
        "stretch" => FlexAlign::Stretch,
        _ => FlexAlign::Center,
    }
}

fn apply_frame_attr(frame: &mut Frame, name: &str, value: &str) {
    match name {
        "width" => frame.width = parse_dimension(value),
        "height" => frame.height = parse_dimension(value),
        "mouse_enabled" => set_bool(&mut frame.mouse_enabled, value),
        "movable" => set_bool(&mut frame.movable, value),
        "frame_level" => {
            if let Ok(v) = value.parse::<f32>() {
                frame.frame_level = v as i32;
            }
        }
        "strata" => frame.strata = FrameStrata::from_str(value).unwrap_or_default(),
        "draw_layer" => frame.draw_layer = DrawLayer::from_str(value).unwrap_or_default(),
        "background_color" => frame.background_color = parse_color(value),
        "nine_slice" => {
            if let Some(ns) = parse_nine_slice(value) {
                frame.nine_slice = Some(ns);
            }
        }
        "border" => frame.border = parse_border(value),
        "onclick" => {
            frame.onclick = Some(value.to_string());
            frame.mouse_enabled = true;
        }
        _ => {}
    }
}

fn set_bool(target: &mut bool, value: &str) {
    match value {
        "true" | "TRUE" | "1" => *target = true,
        "false" | "FALSE" | "0" => *target = false,
        _ => {}
    }
}

fn set_bool_via(value: &str, f: impl FnOnce(bool)) {
    match value {
        "true" | "TRUE" | "1" => f(true),
        "false" | "FALSE" | "0" => f(false),
        _ => {}
    }
}

fn apply_widget_text_attrs(
    frame: &mut Frame,
    name: &str,
    value: &str,
    _validated_paths: &mut HashSet<String>,
    _missing_paths: &mut HashSet<String>,
) {
    match name {
        "text" => apply_text_attr(frame, value),
        "font" => {
            let gf = GameFont::from_attr(value);
            match &mut frame.widget_data {
                Some(WidgetData::FontString(fs)) => fs.font = gf,
                Some(WidgetData::EditBox(eb)) => eb.font = gf,
                _ => {}
            }
        }
        "font_size" => {
            if let Ok(v) = value.parse::<f32>() {
                apply_font_size(frame, v);
            }
        }
        "font_color" => {
            if let Some(color) = parse_color(value) {
                match &mut frame.widget_data {
                    Some(WidgetData::FontString(fs)) => fs.color = color,
                    Some(WidgetData::EditBox(eb)) => eb.text_color = color,
                    _ => {}
                }
            }
        }
        "justify_h" => {
            let jh = parse_justify_h(value);
            if let Some(WidgetData::FontString(fs)) = &mut frame.widget_data {
                fs.justify_h = jh;
            }
        }
        "password" => match value {
            "true" | "TRUE" | "1" => {
                if let Some(WidgetData::EditBox(eb)) = &mut frame.widget_data {
                    eb.password = true;
                }
            }
            "false" | "FALSE" | "0" => {
                if let Some(WidgetData::EditBox(eb)) = &mut frame.widget_data {
                    eb.password = false;
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn check_path(
    validated: &mut HashSet<String>,
    missing: &mut HashSet<String>,
    label: &str,
    path: &str,
) {
    if validated.contains(path) || missing.contains(path) {
        return;
    }
    if Path::new(path).exists() {
        validated.insert(path.to_string());
    } else {
        eprintln!("[UI] {label} not found: {path}");
        missing.insert(path.to_string());
    }
}

fn apply_font_size(frame: &mut Frame, v: f32) {
    if v <= 0.0 || v > 72.0 {
        eprintln!("[UI] font_size out of range (0..72]: {v}");
    }
    match &mut frame.widget_data {
        Some(WidgetData::FontString(fs)) => fs.font_size = v,
        Some(WidgetData::EditBox(eb)) => eb.font_size = v,
        Some(WidgetData::Button(bd)) => bd.font_size = v,
        _ => {}
    }
}

fn apply_widget_texture_attrs(
    frame: &mut Frame,
    name: &str,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    match name {
        "texture_file" => apply_texture_file(frame, value, validated_paths, missing_paths),
        "texture_fdid" => apply_texture_fdid(frame, value, validated_paths, missing_paths),
        "texture_atlas" => apply_texture_atlas(frame, value),
        "vertex_color" => {
            if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                if let Some(color) = parse_color(value) {
                    td.vertex_color = color;
                }
            }
        }
        "button_atlas_up" => {
            apply_button_texture(frame, value, |bd, src| bd.normal_texture = Some(src))
        }
        "button_atlas_pressed" => {
            apply_button_texture(frame, value, |bd, src| bd.pushed_texture = Some(src))
        }
        "button_atlas_highlight" => {
            apply_button_texture(frame, value, |bd, src| bd.highlight_texture = Some(src))
        }
        "button_atlas_disabled" => {
            apply_button_texture(frame, value, |bd, src| bd.disabled_texture = Some(src))
        }
        _ => {}
    }
}

fn apply_texture_file(
    frame: &mut Frame,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    check_path(validated_paths, missing_paths, "texture_file", value);
    let source = TextureSource::File(value.to_string());
    apply_texture_source(frame, source);
}

fn apply_texture_fdid(
    frame: &mut Frame,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    let Ok(v) = value.parse::<f32>() else { return };
    let fdid = v as u32;
    let path = format!("data/textures/{fdid}.blp");
    check_path(validated_paths, missing_paths, "texture_fdid", &path);
    apply_texture_source(frame, TextureSource::FileDataId(fdid));
}

fn apply_texture_atlas(frame: &mut Frame, value: &str) {
    if atlas::get_region(value).is_none() {
        eprintln!("[UI] texture_atlas not found: {value}");
    }
    apply_texture_source(frame, TextureSource::Atlas(value.to_string()));
}

fn apply_texture_source(frame: &mut Frame, source: TextureSource) {
    match &mut frame.widget_data {
        Some(WidgetData::Texture(td)) => td.source = source,
        Some(WidgetData::Slider(slider)) => slider.thumb_texture = Some(source),
        Some(WidgetData::StatusBar(sb)) => sb.texture = Some(source),
        _ => {}
    }
}

fn apply_slider_attrs(
    frame: &mut Frame,
    name: &str,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    match name {
        "value" => apply_slider_numeric_attr(
            frame,
            value,
            |slider, v| slider.value = v,
            |sb, v| sb.value = v,
        ),
        "min" => {
            apply_slider_numeric_attr(frame, value, |slider, v| slider.min = v, |sb, v| sb.min = v)
        }
        "max" => {
            apply_slider_numeric_attr(frame, value, |slider, v| slider.max = v, |sb, v| sb.max = v)
        }
        "orientation" => apply_orientation_attr(frame, value),
        "thumb_texture" => apply_thumb_texture(frame, value, validated_paths, missing_paths),
        "statusbar_color" => apply_statusbar_color(frame, value),
        "fill_style" => apply_fill_style(frame, value),
        "reverse_fill" => apply_reverse_fill(frame, value),
        _ => {}
    }
}

fn apply_slider_numeric_attr(
    frame: &mut Frame,
    value: &str,
    slider_apply: impl FnOnce(&mut crate::widgets::slider::SliderData, f64),
    statusbar_apply: impl FnOnce(&mut crate::widgets::slider::StatusBarData, f64),
) {
    let Ok(v) = value.parse::<f64>() else { return };
    match &mut frame.widget_data {
        Some(WidgetData::Slider(slider)) => slider_apply(slider, v),
        Some(WidgetData::StatusBar(sb)) => statusbar_apply(sb, v),
        _ => {}
    }
}

fn apply_orientation_attr(frame: &mut Frame, value: &str) {
    let orientation = match value {
        "vertical" | "VERTICAL" | "Vertical" => Some(Orientation::Vertical),
        "horizontal" | "HORIZONTAL" | "Horizontal" => Some(Orientation::Horizontal),
        _ => None,
    };
    let Some(orientation) = orientation else {
        return;
    };
    match &mut frame.widget_data {
        Some(WidgetData::Slider(slider)) => slider.orientation = orientation,
        Some(WidgetData::StatusBar(sb)) => sb.orientation = orientation,
        _ => {}
    }
}

fn apply_thumb_texture(
    frame: &mut Frame,
    value: &str,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    if value.is_empty() || value.eq_ignore_ascii_case("none") {
        if let Some(WidgetData::Slider(slider)) = &mut frame.widget_data {
            slider.thumb_texture = None;
        }
        return;
    }
    check_path(validated_paths, missing_paths, "thumb_texture", value);
    if let Some(WidgetData::Slider(slider)) = &mut frame.widget_data {
        slider.thumb_texture = Some(TextureSource::File(value.to_string()));
    }
}

fn apply_statusbar_color(frame: &mut Frame, value: &str) {
    if let Some(color) = parse_color(value)
        && let Some(WidgetData::StatusBar(sb)) = &mut frame.widget_data
    {
        sb.color = color;
    }
}

fn apply_fill_style(frame: &mut Frame, value: &str) {
    if let Some(WidgetData::StatusBar(sb)) = &mut frame.widget_data {
        sb.fill_style = match value {
            "center" | "CENTER" | "Center" => FillStyle::Center,
            _ => FillStyle::Standard,
        };
    }
}

fn apply_reverse_fill(frame: &mut Frame, value: &str) {
    if let Some(WidgetData::StatusBar(sb)) = &mut frame.widget_data {
        set_bool(&mut sb.reverse_fill, value);
    }
}

fn apply_text_attr(frame: &mut Frame, value: &str) {
    match &mut frame.widget_data {
        Some(WidgetData::FontString(fs)) => fs.text = value.to_string(),
        Some(WidgetData::EditBox(eb)) => eb.text = value.to_string(),
        Some(WidgetData::Button(bd)) => bd.text = value.to_string(),
        _ => {}
    }
}

fn apply_button_texture(
    frame: &mut Frame,
    value: &str,
    apply: impl FnOnce(&mut ButtonData, TextureSource),
) {
    if let Some(WidgetData::Button(bd)) = &mut frame.widget_data {
        if atlas::get_region(value).is_none() {
            eprintln!("[UI] button atlas not found: {value}");
        }
        apply(bd, TextureSource::Atlas(value.to_string()));
    }
}

fn parse_nine_slice(s: &str) -> Option<NineSlice> {
    let parts: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if parts.len() != 9 {
        return None;
    }
    Some(NineSlice {
        edge_size: parts[0],
        bg_color: [parts[1], parts[2], parts[3], parts[4]],
        border_color: [parts[5], parts[6], parts[7], parts[8]],
        ..Default::default()
    })
}

fn parse_justify_h(s: &str) -> JustifyH {
    match s {
        "LEFT" => JustifyH::Left,
        "RIGHT" => JustifyH::Right,
        _ => JustifyH::Center,
    }
}

fn parse_border(s: &str) -> Option<Border> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let width_str = parts[0].strip_suffix("px")?;
    let width: f32 = width_str.parse().ok()?;
    // parts[1] is "solid" (or any style keyword — we skip it)
    let color = parse_border_color(parts[2])?;
    Some(Border { width, color })
}

fn parse_border_color(s: &str) -> Option<[f32; 4]> {
    match s {
        "red" => Some([1.0, 0.0, 0.0, 1.0]),
        "green" => Some([0.0, 1.0, 0.0, 1.0]),
        "blue" => Some([0.0, 0.0, 1.0, 1.0]),
        "white" => Some([1.0, 1.0, 1.0, 1.0]),
        "black" => Some([0.0, 0.0, 0.0, 1.0]),
        "gold" => Some([1.0, 0.82, 0.0, 1.0]),
        "yellow" => Some([1.0, 1.0, 0.0, 1.0]),
        _ => parse_color(s),
    }
}

pub(crate) fn parse_color(s: &str) -> Option<[f32; 4]> {
    let parts: Vec<_> = s.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return None;
    }
    let mut color = [0.0; 4];
    for (i, part) in parts.into_iter().enumerate() {
        color[i] = part.parse().ok()?;
    }
    Some(color)
}
