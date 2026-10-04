//! Native nine-slice geometry and UV mapping.

use crate::frame::NineSlice;
use bevy::prelude::*;

pub(crate) fn explicit_uv_rect_for_part(
    uv_rects: &[[f32; 4]; 9],
    part: u8,
    atlas_rect: Rect,
) -> Rect {
    let [left, right, top, bottom] = uv_rects[part as usize];
    let size = atlas_rect.max - atlas_rect.min;
    Rect {
        min: Vec2::new(
            atlas_rect.min.x + left * size.x,
            atlas_rect.min.y + top * size.y,
        ),
        max: Vec2::new(
            atlas_rect.min.x + right * size.x,
            atlas_rect.min.y + bottom * size.y,
        ),
    }
}

pub(crate) fn uv_rect_for_part(
    part: u8,
    w: f32,
    h: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) -> Rect {
    let (min_x, max_x, min_y, max_y) = match part {
        0 => (0.0, left, 0.0, top),
        1 => (left, w - right, 0.0, top),
        2 => (w - right, w, 0.0, top),
        3 => (0.0, left, top, h - bottom),
        4 => (left, w - right, top, h - bottom),
        5 => (w - right, w, top, h - bottom),
        6 => (0.0, left, h - bottom, h),
        7 => (left, w - right, h - bottom, h),
        _ => (w - right, w, h - bottom, h),
    };
    Rect {
        min: Vec2::new(min_x, min_y),
        max: Vec2::new(max_x, max_y),
    }
}

/// Compute the center position, size, and border flag for one nine-slice part.
/// Returns `(cx, cy, w, h, is_border)` in WoW screen space (top-left origin).
fn part_layout(
    part: u8,
    fx: f32,
    fy: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    iw: f32,
    ih: f32,
) -> (f32, f32, f32, f32, bool) {
    match part {
        0 => (fx + left * 0.5, fy + top * 0.5, left, top, true),
        1 => (fx + left + iw * 0.5, fy + top * 0.5, iw, top, true),
        2 => (
            fx + left + iw + right * 0.5,
            fy + top * 0.5,
            right,
            top,
            true,
        ),
        3 => (fx + left * 0.5, fy + top + ih * 0.5, left, ih, true),
        4 => (fx + left + iw * 0.5, fy + top + ih * 0.5, iw, ih, false),
        5 => (
            fx + left + iw + right * 0.5,
            fy + top + ih * 0.5,
            right,
            ih,
            true,
        ),
        6 => (
            fx + left * 0.5,
            fy + top + ih + bottom * 0.5,
            left,
            bottom,
            true,
        ),
        7 => (
            fx + left + iw * 0.5,
            fy + top + ih + bottom * 0.5,
            iw,
            bottom,
            true,
        ),
        _ => (
            fx + left + iw + right * 0.5,
            fy + top + ih + bottom * 0.5,
            right,
            bottom,
            true,
        ),
    }
}

fn layout_edges(ns: &NineSlice) -> (f32, f32, f32, f32) {
    if let Some([left, top, right, bottom]) = ns.edge_sizes {
        (left, top, right, bottom)
    } else {
        let horizontal = ns.edge_size;
        let vertical = ns.edge_size_v.unwrap_or(horizontal);
        (horizontal, vertical, horizontal, vertical)
    }
}

pub(crate) fn uv_edges(ns: &NineSlice) -> (f32, f32, f32, f32) {
    if let Some([left, top, right, bottom]) = ns.uv_edge_sizes {
        (left, top, right, bottom)
    } else if let Some([left, top, right, bottom]) = ns.edge_sizes {
        (left, top, right, bottom)
    } else {
        let horizontal = ns.uv_edge_size.unwrap_or(ns.edge_size);
        let vertical = ns.edge_size_v.unwrap_or(ns.edge_size);
        let uv_vertical = ns.uv_edge_size.unwrap_or(vertical);
        (horizontal, uv_vertical, horizontal, uv_vertical)
    }
}

fn part_color(ns: &NineSlice, is_border: bool, alpha: f32) -> Color {
    let [r, g, b, a] = if is_border {
        ns.border_color
    } else {
        ns.bg_color
    };
    Color::srgba(r, g, b, a * alpha)
}

/// Compute transform, size, color for one nine-slice part.
/// Parts: 0=TL, 1=T, 2=TR, 3=L, 4=Center, 5=R, 6=BL, 7=B, 8=BR
pub(crate) fn part_geometry(
    frame: &crate::frame::Frame,
    ns: &NineSlice,
    part: u8,
    screen_w: f32,
    screen_h: f32,
    z: f32,
) -> (Transform, Vec2, Color) {
    let (left, top, right, bottom) = layout_edges(ns);
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let iw = (frame.resolved_width() - left - right).max(0.0);
    let ih = (frame.resolved_height() - top - bottom).max(0.0);

    let (cx, cy, w, h, is_border) = part_layout(part, fx, fy, left, top, right, bottom, iw, ih);
    let color = part_color(ns, is_border, frame.effective_alpha);
    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    // Border parts render above center to prevent center fill from overpainting edges
    let part_z = if is_border { z + 0.0001 } else { z };
    (Transform::from_xyz(bx, by, part_z), Vec2::new(w, h), color)
}
