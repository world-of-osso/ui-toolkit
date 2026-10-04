//! Native backdrop and CSS border geometry.

use crate::frame::{Backdrop, Border};
use bevy::prelude::*;

pub(crate) fn edge_geometry(
    frame: &crate::frame::Frame,
    backdrop: &Backdrop,
    edge: u8,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    let [r, g, b, a] = backdrop.border_color.unwrap_or([1.0; 4]);
    let color = Color::srgba(r, g, b, a * frame.effective_alpha);
    let e = backdrop.edge_size;
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let fw = frame.resolved_width();
    let fh = frame.resolved_height();

    let (cx, cy, w, h) = match edge {
        0 => (fx + fw * 0.5, fy - e * 0.5, fw + e * 2.0, e),
        1 => (fx + fw * 0.5, fy + fh + e * 0.5, fw + e * 2.0, e),
        2 => (fx - e * 0.5, fy + fh * 0.5, e, fh),
        _ => (fx + fw + e * 0.5, fy + fh * 0.5, e, fh),
    };

    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    (Transform::from_xyz(bx, by, 9.5), Vec2::new(w, h), color)
}

pub(crate) fn css_edge_geometry(
    frame: &crate::frame::Frame,
    border: &Border,
    side: u8,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    let [r, g, b, a] = border.color;
    let color = Color::srgba(r, g, b, a * frame.effective_alpha);
    let e = border.width;
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let fw = frame
        .layout_rect
        .as_ref()
        .map_or(frame.resolved_width(), |r| r.width);
    let fh = frame
        .layout_rect
        .as_ref()
        .map_or(frame.resolved_height(), |r| r.height);

    // side: 0=top, 1=right, 2=bottom, 3=left
    let (cx, cy, w, h) = match side {
        0 => (fx + fw * 0.5, fy + e * 0.5, fw, e),      // top
        1 => (fx + fw - e * 0.5, fy + fh * 0.5, e, fh), // right
        2 => (fx + fw * 0.5, fy + fh - e * 0.5, fw, e), // bottom
        _ => (fx + e * 0.5, fy + fh * 0.5, e, fh),      // left
    };

    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    let z = frame.frame_level as f32 * 0.001 + 0.0005;
    (Transform::from_xyz(bx, by, z), Vec2::new(w, h), color)
}
