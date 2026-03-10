use std::collections::HashSet;
use std::path::Path;

use dioxus_core::AttributeValue;

use crate::atlas;
use crate::frame::{Frame, NineSlice, WidgetData};
use crate::registry::FrameRegistry;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::ButtonData;
use crate::widgets::font_string::{GameFont, JustifyH};
use crate::widgets::texture::TextureSource;

/// Apply an attribute. Returns `Some((frame_id, spec))` if the anchor couldn't
/// be resolved yet (cross-component name reference). Caller should defer it.
pub(crate) fn apply_attribute(
    registry: &mut FrameRegistry,
    frame_id: u64,
    name: &str,
    value: &AttributeValue,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) -> Option<(u64, String)> {
    if name == "name" {
        if let Some(s) = as_text(value) {
            registry.set_name(frame_id, s.to_string());
        }
        return None;
    }
    if name == "stretch" {
        return apply_stretch_attr(registry, frame_id, value);
    }
    let Some(frame) = registry.get_mut(frame_id) else {
        return None;
    };
    apply_frame_attr(frame, name, value);
    apply_widget_text_attrs(frame, name, value, validated_paths, missing_paths);
    apply_widget_texture_attrs(frame, name, value, validated_paths, missing_paths);
    None
}

fn apply_stretch_attr(
    registry: &mut FrameRegistry,
    frame_id: u64,
    value: &AttributeValue,
) -> Option<(u64, String)> {
    if matches!(value, AttributeValue::Bool(true)) {
        let parent_id = registry.get(frame_id).and_then(|f| f.parent_id);
        let _ = registry.stretch_to_fill(frame_id, parent_id);
    }
    None
}

fn apply_frame_attr(frame: &mut Frame, name: &str, value: &AttributeValue) {
    match name {
        "width" => assign_f32(value, |v| frame.width = v),
        "height" => assign_f32(value, |v| frame.height = v),
        "alpha" => assign_f32(value, |v| frame.alpha = v),
        "shown" => assign_bool(value, |v| frame.shown = v),
        "mouse_enabled" => assign_bool(value, |v| frame.mouse_enabled = v),
        "movable" => assign_bool(value, |v| frame.movable = v),
        "frame_level" => assign_f32(value, |v| frame.frame_level = v as i32),
        "strata" => {
            if let Some(s) = as_text(value) {
                frame.strata = FrameStrata::from_str(s).unwrap_or_default();
            }
        }
        "draw_layer" => {
            if let Some(s) = as_text(value) {
                frame.draw_layer = DrawLayer::from_str(s).unwrap_or_default();
            }
        }
        "background_color" => {
            if let Some(s) = as_text(value)
                && let Some(color) = parse_color(s)
            {
                frame.background_color = Some(color);
            }
        }
        "nine_slice" => {
            if let Some(s) = as_text(value)
                && let Some(ns) = parse_nine_slice(s)
            {
                frame.nine_slice = Some(ns);
            }
        }
        _ => {}
    }
}

fn apply_widget_text_attrs(
    frame: &mut Frame,
    name: &str,
    value: &AttributeValue,
    _validated_paths: &mut HashSet<String>,
    _missing_paths: &mut HashSet<String>,
) {
    match name {
        "text" => apply_text_attr(frame, value),
        "font" => {
            if let Some(s) = as_text(value) {
                let game_font = GameFont::from_attr(s);
                match &mut frame.widget_data {
                    Some(WidgetData::FontString(fs)) => fs.font = game_font,
                    Some(WidgetData::EditBox(eb)) => eb.font = game_font,
                    _ => {}
                }
            }
        }
        "font_size" => assign_f32(value, |v| apply_font_size(frame, v)),
        "font_color" => {
            if let Some(s) = as_text(value)
                && let Some(color) = parse_color(s)
            {
                match &mut frame.widget_data {
                    Some(WidgetData::FontString(fs)) => fs.color = color,
                    Some(WidgetData::EditBox(eb)) => eb.text_color = color,
                    _ => {}
                }
            }
        }
        "justify_h" => {
            if let Some(s) = as_text(value) {
                let jh = parse_justify_h(s);
                if let Some(WidgetData::FontString(fs)) = &mut frame.widget_data {
                    fs.justify_h = jh;
                }
            }
        }
        "password" => assign_bool(value, |v| {
            if let Some(WidgetData::EditBox(eb)) = &mut frame.widget_data {
                eb.password = v;
            }
        }),
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
    value: &AttributeValue,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    match name {
        "texture_file" => {
            if let Some(s) = as_text(value) {
                if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                    check_path(validated_paths, missing_paths, "texture_file", s);
                    td.source = TextureSource::File(s.to_string());
                }
            }
        }
        "texture_fdid" => assign_f32(value, |v| {
            if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                let fdid = v as u32;
                let path = format!("data/textures/{fdid}.blp");
                check_path(validated_paths, missing_paths, "texture_fdid", &path);
                td.source = TextureSource::FileDataId(fdid);
            }
        }),
        "texture_atlas" => {
            if let Some(s) = as_text(value) {
                if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                    if atlas::get_region(s).is_none() {
                        eprintln!("[UI] texture_atlas not found: {s}");
                    }
                    td.source = TextureSource::Atlas(s.to_string());
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

fn apply_text_attr(frame: &mut Frame, value: &AttributeValue) {
    if let Some(s) = as_text(value) {
        match &mut frame.widget_data {
            Some(WidgetData::FontString(fs)) => fs.text = s.to_string(),
            Some(WidgetData::EditBox(eb)) => eb.text = s.to_string(),
            Some(WidgetData::Button(bd)) => bd.text = s.to_string(),
            _ => {}
        }
    }
}

fn apply_button_texture(
    frame: &mut Frame,
    value: &AttributeValue,
    apply: impl FnOnce(&mut ButtonData, TextureSource),
) {
    if let Some(s) = as_text(value) {
        if let Some(WidgetData::Button(bd)) = &mut frame.widget_data {
            if atlas::get_region(s).is_none() {
                eprintln!("[UI] button atlas not found: {s}");
            }
            apply(bd, TextureSource::Atlas(s.to_string()));
        }
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

pub(crate) fn apply_static_attribute(
    registry: &mut FrameRegistry,
    frame_id: u64,
    name: &str,
    namespace: Option<&str>,
    value: &str,
    pending: &mut Vec<(u64, String)>,
    validated_paths: &mut HashSet<String>,
    missing_paths: &mut HashSet<String>,
) {
    let _ = namespace;
    let attr = AttributeValue::Text(value.to_string());
    if let Some(p) = apply_attribute(
        registry,
        frame_id,
        name,
        &attr,
        validated_paths,
        missing_paths,
    ) {
        pending.push(p);
    }
}

pub(crate) fn as_text(value: &AttributeValue) -> Option<&str> {
    match value {
        AttributeValue::Text(s) => Some(s),
        _ => None,
    }
}

pub(crate) fn assign_f32(value: &AttributeValue, mut assign: impl FnMut(f32)) {
    match value {
        AttributeValue::Float(v) => assign(*v as f32),
        AttributeValue::Int(v) => assign(*v as f32),
        AttributeValue::Text(s) => {
            if let Ok(v) = s.parse::<f32>() {
                assign(v);
            }
        }
        _ => {}
    }
}

pub(crate) fn assign_bool(value: &AttributeValue, mut assign: impl FnMut(bool)) {
    match value {
        AttributeValue::Bool(v) => assign(*v),
        AttributeValue::Text(s) => match s.as_str() {
            "true" | "TRUE" | "1" => assign(true),
            "false" | "FALSE" | "0" => assign(false),
            _ => {}
        },
        _ => {}
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
