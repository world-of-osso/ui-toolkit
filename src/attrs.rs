use std::collections::HashSet;
use std::path::Path;

use crate::atlas;
use crate::frame::{Border, Dimension, FlexAlign, FlexDirection, FlexJustify, FlexLayout, Frame, NineSlice, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::strata::{DrawLayer, FrameStrata};
use crate::widgets::button::ButtonData;
use crate::widgets::font_string::{GameFont, JustifyH};
use crate::widgets::texture::TextureSource;

fn parse_dimension(value: &str) -> Dimension {
    match value {
        "fill" | "Fill" => Dimension::Fill,
        _ => value.parse::<f32>().map(Dimension::Fixed).unwrap_or_default(),
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
        "button" | "Button" => Some(WidgetType::Button),
        "editbox" | "EditBox" => Some(WidgetType::EditBox),
        "fontstring" | "FontString" => Some(WidgetType::FontString),
        "texture" | "Texture" => Some(WidgetType::Texture),
        _ => None,
    }
}

/// Read the current value of a frame attribute as a string, for change detection.
pub(crate) fn read_attribute(registry: &FrameRegistry, frame_id: u64, name: &str) -> Option<String> {
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
        "alpha" => Some(format!("{}", frame.alpha)),
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
            _ => None,
        },
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
    if let Some(frame) = registry.get_mut(frame_id) {
        if apply_flex_attr(frame, name, value) { return None; }
    }
    if name == "hidden" {
        match value {
            "true" | "TRUE" | "1" => registry.set_hidden(frame_id, true),
            "false" | "FALSE" | "0" => registry.set_hidden(frame_id, false),
            _ => {}
        }
        return None;
    }
    if name == "alpha" {
        if let Ok(v) = value.parse::<f32>() {
            registry.set_alpha(frame_id, v);
        }
        return None;
    }
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

fn apply_flex_attr(frame: &mut Frame, name: &str, value: &str) -> bool {
    match name {
        "layout" => {
            let dir = match value {
                "flex-row" => FlexDirection::Row,
                _ => FlexDirection::Column,
            };
            frame.flex_layout.get_or_insert_with(FlexLayout::default).direction = dir;
        }
        "gap" => {
            if let Ok(v) = value.parse::<f32>() {
                frame.flex_layout.get_or_insert_with(FlexLayout::default).gap = v;
            }
        }
        "justify" => {
            let j = match value {
                "center" => FlexJustify::Center,
                "end" => FlexJustify::End,
                "space-between" => FlexJustify::SpaceBetween,
                _ => FlexJustify::Start,
            };
            frame.flex_layout.get_or_insert_with(FlexLayout::default).justify = j;
        }
        "align" => {
            let a = match value {
                "start" => FlexAlign::Start,
                "end" => FlexAlign::End,
                "stretch" => FlexAlign::Stretch,
                _ => FlexAlign::Center,
            };
            frame.flex_layout.get_or_insert_with(FlexLayout::default).align = a;
        }
        "padding" => {
            if let Ok(v) = value.parse::<f32>() {
                frame.flex_layout.get_or_insert_with(FlexLayout::default).padding = v;
            }
        }
        _ => return false,
    }
    true
}

fn apply_frame_attr(frame: &mut Frame, name: &str, value: &str) {
    match name {
        "width" => { frame.width = parse_dimension(value); }
        "height" => { frame.height = parse_dimension(value); }
        "mouse_enabled" => match value { "true" | "TRUE" | "1" => frame.mouse_enabled = true, "false" | "FALSE" | "0" => frame.mouse_enabled = false, _ => {} },
        "movable" => match value { "true" | "TRUE" | "1" => frame.movable = true, "false" | "FALSE" | "0" => frame.movable = false, _ => {} },
        "frame_level" => { if let Ok(v) = value.parse::<f32>() { frame.frame_level = v as i32; } }
        "strata" => { frame.strata = FrameStrata::from_str(value).unwrap_or_default(); }
        "draw_layer" => { frame.draw_layer = DrawLayer::from_str(value).unwrap_or_default(); }
        "background_color" => { if let Some(color) = parse_color(value) { frame.background_color = Some(color); } }
        "nine_slice" => { if let Some(ns) = parse_nine_slice(value) { frame.nine_slice = Some(ns); } }
        "border" => { frame.border = parse_border(value); }
        "onclick" => { frame.onclick = Some(value.to_string()); }
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



fn parse_border(s: &str) -> Option<Border> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 3 { return None; }
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
    if parts.len() != 4 { return None; }
    let mut color = [0.0; 4];
    for (i, part) in parts.into_iter().enumerate() { color[i] = part.parse().ok()?; }
    Some(color)
}
