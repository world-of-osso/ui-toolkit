use super::{Frame, WidgetData, format_color, format_vec2, parse_color, parse_vec2};
use crate::widgets::button::{ButtonData, ButtonTexture};
use crate::widgets::texture::{BlendMode, TextureSource};

type Slot = fn(&mut ButtonData) -> &mut Option<ButtonTexture>;

fn state_slot(state: &str) -> Option<Slot> {
    Some(match state {
        "normal" | "up" => |button| &mut button.normal_texture,
        "pushed" | "pressed" => |button| &mut button.pushed_texture,
        "disabled" => |button| &mut button.disabled_texture,
        "highlight" => |button| &mut button.highlight_texture,
        "checked" => |button| &mut button.checked_texture,
        "disabled_checked" => |button| &mut button.disabled_checked_texture,
        _ => return None,
    })
}

fn region_attribute(name: &str) -> Option<(Slot, &str)> {
    let attribute = name.strip_prefix("button_")?;
    for (prefix, field) in [("fdid_", "fdid"), ("atlas_", "atlas"), ("file_", "file")] {
        if let Some(state) = attribute.strip_prefix(prefix) {
            return Some((state_slot(state)?, field));
        }
    }
    // Longest state first: disabled_checked is distinct from disabled.
    for state in [
        "disabled_checked",
        "normal",
        "pushed",
        "disabled",
        "highlight",
        "checked",
    ] {
        if let Some(field) = attribute
            .strip_prefix(state)
            .and_then(|rest| rest.strip_prefix('_'))
        {
            return Some((state_slot(state)?, field));
        }
    }
    None
}

pub(super) fn read(frame: &Frame, name: &str) -> Option<String> {
    let Some(WidgetData::Button(button)) = &frame.widget_data else {
        return None;
    };
    if name == "button_checked" {
        return Some(button.checked.to_string());
    }
    let (slot, field) = region_attribute(name)?;
    let mut button = button.clone();
    let default = ButtonTexture::default();
    let region = slot(&mut button).as_ref().unwrap_or(&default);
    Some(match field {
        "blend_mode" => region.blend_mode.as_str().into(),
        "tex_coords" => format_color(region.tex_coords),
        "vertex_color" => format_color(region.vertex_color),
        "alpha" => region.alpha.to_string(),
        "desaturated" => region.desaturated.to_string(),
        "size" => return region.size.map(format_vec2),
        "offset" => format_vec2(region.offset),
        _ => return None,
    })
}

pub(super) fn apply(frame: &mut Frame, name: &str, value: &str) {
    let Some(WidgetData::Button(button)) = &mut frame.widget_data else {
        return;
    };
    if name == "button_checked" {
        button.checked = matches!(value, "true" | "TRUE" | "1");
        return;
    }
    let Some((slot, field)) = region_attribute(name) else {
        return;
    };
    // Parse on a copy so invalid metadata cannot leave an inserted or partially changed slot.
    let mut region = slot(button).clone().unwrap_or_default();
    if update_region(&mut region, field, value) {
        *slot(button) = Some(region);
    }
}

fn update_region(region: &mut ButtonTexture, field: &str, value: &str) -> bool {
    match field {
        "fdid" => {
            region.source =
                TextureSource::FileDataId(value.parse().expect("button FDID must be a u32"))
        }
        "atlas" => region.source = TextureSource::Atlas(value.into()),
        "file" => region.source = TextureSource::File(value.into()),
        "blend_mode" => region.blend_mode = BlendMode::from_alpha_mode(value),
        "tex_coords" => region.tex_coords = normalized_color(value),
        "vertex_color" => region.vertex_color = normalized_color(value),
        "alpha" => region.alpha = normalized_alpha(value),
        "desaturated" => region.desaturated = matches!(value, "true" | "TRUE" | "1"),
        "size" => region.size = Some(positive_size(value)),
        "offset" => region.offset = finite_offset(value),
        _ => return false,
    }
    true
}

fn normalized_color(value: &str) -> [f32; 4] {
    parse_color(value)
        .filter(|components| {
            components
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        })
        .unwrap_or_else(|| panic!("invalid normalized button region value: {value}"))
}

fn normalized_alpha(value: &str) -> f32 {
    let alpha: f32 = value.parse().expect("button region alpha must be numeric");
    assert!(
        alpha.is_finite() && (0.0..=1.0).contains(&alpha),
        "invalid button region alpha: {value}"
    );
    alpha
}

fn positive_size(value: &str) -> [f32; 2] {
    let size = finite_offset(value);
    assert!(
        size.iter().all(|v| *v > 0.0),
        "invalid button region size: {value}"
    );
    size
}

fn finite_offset(value: &str) -> [f32; 2] {
    let offset = parse_vec2(value).unwrap_or_else(|| panic!("expected button region x,y: {value}"));
    assert!(
        offset.iter().all(|v| v.is_finite()),
        "invalid button region x,y: {value}"
    );
    offset
}
