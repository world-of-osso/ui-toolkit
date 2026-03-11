use std::collections::HashSet;
use std::path::Path;

use crate::atlas;
use crate::frame::{Frame, NineSlice, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::ButtonData;
use crate::widgets::font_string::{GameFont, JustifyH};
use crate::widgets::texture::TextureSource;

pub(crate) fn tag_to_widget_type(tag: &str) -> Option<WidgetType> {
    match tag {
        "frame" | "r#frame" | "Frame" => Some(WidgetType::Frame),
        "button" | "Button" => Some(WidgetType::Button),
        "editbox" | "EditBox" => Some(WidgetType::EditBox),
        "fontstring" | "FontString" => Some(WidgetType::FontString),
        "texture" | "Texture" => Some(WidgetType::Texture),
        _ => None,
    }
}

pub(crate) fn apply_attribute(
    registry: &mut FrameRegistry, frame_id: u64, name: &str, value: &str,
    validated_paths: &mut HashSet<String>, missing_paths: &mut HashSet<String>,
) -> Option<(u64, String)> {
    if name == "name" {
        registry.set_name(frame_id, value.to_string());
        return None;
    }
    if name == "stretch" { return apply_stretch_attr(registry, frame_id, value); }
    let Some(frame) = registry.get_mut(frame_id) else { return None };
    apply_frame_attr(frame, name, value);
    apply_widget_text_attrs(frame, name, value, validated_paths, missing_paths);
    apply_widget_texture_attrs(frame, name, value, validated_paths, missing_paths);
    None
}

fn apply_stretch_attr(registry: &mut FrameRegistry, frame_id: u64, value: &str) -> Option<(u64, String)> {
    if matches!(value, "true" | "TRUE" | "1") {
        let parent_id = registry.get(frame_id).and_then(|f| f.parent_id);
        let _ = registry.stretch_to_fill(frame_id, parent_id);
    }
    None
}

fn apply_frame_attr(frame: &mut Frame, name: &str, value: &str) {
    match name {
        "width" => { if let Ok(v) = value.parse::<f32>() { frame.width = v; } }
        "height" => { if let Ok(v) = value.parse::<f32>() { frame.height = v; } }
        "alpha" => { if let Ok(v) = value.parse::<f32>() { frame.alpha = v; } }
        "shown" => match value { "true" | "TRUE" | "1" => frame.shown = true, "false" | "FALSE" | "0" => frame.shown = false, _ => {} },
        "mouse_enabled" => match value { "true" | "TRUE" | "1" => frame.mouse_enabled = true, "false" | "FALSE" | "0" => frame.mouse_enabled = false, _ => {} },
        "movable" => match value { "true" | "TRUE" | "1" => frame.movable = true, "false" | "FALSE" | "0" => frame.movable = false, _ => {} },
        "frame_level" => { if let Ok(v) = value.parse::<f32>() { frame.frame_level = v as i32; } }
        "strata" => { frame.strata = FrameStrata::from_str(value).unwrap_or_default(); }
        "draw_layer" => { frame.draw_layer = DrawLayer::from_str(value).unwrap_or_default(); }
        "background_color" => { if let Some(color) = parse_color(value) { frame.background_color = Some(color); } }
        "nine_slice" => { if let Some(ns) = parse_nine_slice(value) { frame.nine_slice = Some(ns); } }
        _ => {}
    }
}

fn apply_widget_text_attrs(frame: &mut Frame, name: &str, value: &str, _validated_paths: &mut HashSet<String>, _missing_paths: &mut HashSet<String>) {
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
            if let Ok(v) = value.parse::<f32>() { apply_font_size(frame, v); }
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
            if let Some(WidgetData::FontString(fs)) = &mut frame.widget_data { fs.justify_h = jh; }
        }
        "password" => match value {
            "true" | "TRUE" | "1" => { if let Some(WidgetData::EditBox(eb)) = &mut frame.widget_data { eb.password = true; } }
            "false" | "FALSE" | "0" => { if let Some(WidgetData::EditBox(eb)) = &mut frame.widget_data { eb.password = false; } }
            _ => {}
        },
        _ => {}
    }
}

fn check_path(validated: &mut HashSet<String>, missing: &mut HashSet<String>, label: &str, path: &str) {
    if validated.contains(path) || missing.contains(path) { return; }
    if Path::new(path).exists() { validated.insert(path.to_string()); }
    else { eprintln!("[UI] {label} not found: {path}"); missing.insert(path.to_string()); }
}

fn apply_font_size(frame: &mut Frame, v: f32) {
    if v <= 0.0 || v > 72.0 { eprintln!("[UI] font_size out of range (0..72]: {v}"); }
    match &mut frame.widget_data {
        Some(WidgetData::FontString(fs)) => fs.font_size = v,
        Some(WidgetData::EditBox(eb)) => eb.font_size = v,
        Some(WidgetData::Button(bd)) => bd.font_size = v,
        _ => {}
    }
}

fn apply_widget_texture_attrs(frame: &mut Frame, name: &str, value: &str, validated_paths: &mut HashSet<String>, missing_paths: &mut HashSet<String>) {
    match name {
        "texture_file" => {
            if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                check_path(validated_paths, missing_paths, "texture_file", value);
                td.source = TextureSource::File(value.to_string());
            }
        }
        "texture_fdid" => {
            if let Ok(v) = value.parse::<f32>() {
                if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                    let fdid = v as u32;
                    let path = format!("data/textures/{fdid}.blp");
                    check_path(validated_paths, missing_paths, "texture_fdid", &path);
                    td.source = TextureSource::FileDataId(fdid);
                }
            }
        }
        "texture_atlas" => {
            if let Some(WidgetData::Texture(td)) = &mut frame.widget_data {
                if atlas::get_region(value).is_none() { eprintln!("[UI] texture_atlas not found: {value}"); }
                td.source = TextureSource::Atlas(value.to_string());
            }
        }
        "button_atlas_up" => apply_button_texture(frame, value, |bd, src| bd.normal_texture = Some(src)),
        "button_atlas_pressed" => apply_button_texture(frame, value, |bd, src| bd.pushed_texture = Some(src)),
        "button_atlas_highlight" => apply_button_texture(frame, value, |bd, src| bd.highlight_texture = Some(src)),
        "button_atlas_disabled" => apply_button_texture(frame, value, |bd, src| bd.disabled_texture = Some(src)),
        _ => {}
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

fn apply_button_texture(frame: &mut Frame, value: &str, apply: impl FnOnce(&mut ButtonData, TextureSource)) {
    if let Some(WidgetData::Button(bd)) = &mut frame.widget_data {
        if atlas::get_region(value).is_none() { eprintln!("[UI] button atlas not found: {value}"); }
        apply(bd, TextureSource::Atlas(value.to_string()));
    }
}

fn parse_nine_slice(s: &str) -> Option<NineSlice> {
    let parts: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if parts.len() != 9 { return None; }
    Some(NineSlice { edge_size: parts[0], bg_color: [parts[1], parts[2], parts[3], parts[4]], border_color: [parts[5], parts[6], parts[7], parts[8]], ..Default::default() })
}

fn parse_justify_h(s: &str) -> JustifyH { match s { "LEFT" => JustifyH::Left, "RIGHT" => JustifyH::Right, _ => JustifyH::Center } }

pub(crate) fn apply_static_attribute(registry: &mut FrameRegistry, frame_id: u64, name: &str, namespace: Option<&str>, value: &str, pending: &mut Vec<(u64, String)>, validated_paths: &mut HashSet<String>, missing_paths: &mut HashSet<String>) {
    let _ = namespace;
    if let Some(p) = apply_attribute(registry, frame_id, name, value, validated_paths, missing_paths) { pending.push(p); }
}

pub(crate) fn parse_color(s: &str) -> Option<[f32; 4]> {
    let parts: Vec<_> = s.split(',').map(str::trim).collect();
    if parts.len() != 4 { return None; }
    let mut color = [0.0; 4];
    for (i, part) in parts.into_iter().enumerate() { color[i] = part.parse().ok()?; }
    Some(color)
}
