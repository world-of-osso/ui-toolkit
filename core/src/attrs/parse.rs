use crate::frame::{Border, NineSlice};
use crate::widgets::font_string::{JustifyH, Outline};

pub(super) fn parse_nine_slice(s: &str) -> Option<NineSlice> {
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

pub(super) fn parse_justify_h(s: &str) -> JustifyH {
    match s {
        "LEFT" => JustifyH::Left,
        "RIGHT" => JustifyH::Right,
        _ => JustifyH::Center,
    }
}

pub(super) fn parse_outline(s: &str) -> Outline {
    match s {
        "OUTLINE" | "Outline" | "outline" | "NORMAL" | "Normal" | "normal" => Outline::Outline,
        "THICKOUTLINE" | "ThickOutline" | "thick_outline" | "THICK" | "thick" => {
            Outline::ThickOutline
        }
        _ => Outline::None,
    }
}

pub(super) fn parse_border(s: &str) -> Option<Border> {
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

pub(super) fn parse_vec2(s: &str) -> Option<[f32; 2]> {
    let parts: Vec<_> = s.split(',').map(str::trim).collect();
    if parts.len() != 2 {
        return None;
    }
    Some([parts[0].parse().ok()?, parts[1].parse().ok()?])
}

pub(super) fn format_color(color: [f32; 4]) -> String {
    format!("{},{},{},{}", color[0], color[1], color[2], color[3])
}

pub(super) fn format_vec2(offset: [f32; 2]) -> String {
    format!("{},{}", offset[0], offset[1])
}

pub(super) fn format_outline(outline: Outline) -> String {
    match outline {
        Outline::None => "NONE",
        Outline::Outline => "OUTLINE",
        Outline::ThickOutline => "THICKOUTLINE",
    }
    .to_string()
}
