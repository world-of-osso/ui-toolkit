//! Native three-slice geometry and source selection.

use crate::frame::ThreeSlice;
use crate::widgets::texture::TextureSource;
use bevy::prelude::*;

pub(crate) fn part_source(ts: &ThreeSlice, part: u8) -> &TextureSource {
    match part {
        0 => &ts.left,
        1 => &ts.center,
        _ => &ts.right,
    }
}

pub(crate) fn part_geometry(
    frame: &crate::frame::Frame,
    ts: &ThreeSlice,
    part: u8,
    screen_w: f32,
    screen_h: f32,
    z: f32,
) -> (Transform, Vec2, Color) {
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let fw = frame.resolved_width();
    let fh = frame.resolved_height();
    let cap = ts.cap_width;
    let center_w = (fw - cap * 2.0).max(0.0);

    let (cx, w) = match part {
        0 => (fx + cap * 0.5, cap),
        1 => (fx + cap + center_w * 0.5, center_w),
        _ => (fx + cap + center_w + cap * 0.5, cap),
    };
    let cy = fy + fh * 0.5;
    let [r, g, b, a] = ts.color;
    let color = Color::srgba(r, g, b, a * frame.effective_alpha);
    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    (Transform::from_xyz(bx, by, z), Vec2::new(w, fh), color)
}
