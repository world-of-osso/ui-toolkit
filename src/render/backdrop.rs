use bevy::prelude::{Color, Transform, Vec2};

use crate::plugin::UiState;

use super::UiBackdropQuad;

pub(super) fn should_keep_backdrop_part(state: &UiState, backdrop_part: &UiBackdropQuad) -> bool {
    let Some(frame) = state.registry.get(backdrop_part.0) else {
        return false;
    };
    if !super::uses_backdrop_parts(frame) {
        return false;
    }
    let size = backdrop_part_geometry(
        frame,
        backdrop_part.1,
        0,
        state.registry.screen_width,
        state.registry.screen_height,
    )
    .1;
    size.x > 0.0 && size.y > 0.0
}

pub(super) fn backdrop_part_geometry_for_id(
    state: &UiState,
    backdrop_part: &UiBackdropQuad,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    let frame = state
        .registry
        .get(backdrop_part.0)
        .expect("backdrop part should have a frame");
    backdrop_part_geometry(frame, backdrop_part.1, sort_idx, screen_w, screen_h)
}

pub(super) fn backdrop_part_geometry(
    frame: &crate::frame::Frame,
    part: u8,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    let (left, top, right, bottom) = frame
        .nine_slice
        .as_ref()
        .map(nine_slice_layout_edges)
        .unwrap_or_default();
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let interior_width = (frame.resolved_width() - left - right).max(0.0);
    let interior_height = (frame.resolved_height() - top - bottom).max(0.0);
    let (cx, cy, width, height) = backdrop_part_layout(
        part,
        fx,
        fy,
        left,
        top,
        right,
        bottom,
        interior_width,
        interior_height,
    );
    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    let z = sort_idx as f32 * 0.001 - 0.0002;
    (
        Transform::from_xyz(bx, by, z),
        Vec2::new(width, height),
        super::frame_color(frame),
    )
}

fn backdrop_part_layout(
    part: u8,
    fx: f32,
    fy: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    interior_width: f32,
    interior_height: f32,
) -> (f32, f32, f32, f32) {
    match part {
        0 => (
            fx + left + interior_width * 0.5,
            fy + top * 0.5,
            interior_width,
            top,
        ),
        1 => (
            fx + left * 0.5,
            fy + top + interior_height * 0.5,
            left,
            interior_height,
        ),
        2 => (
            fx + left + interior_width * 0.5,
            fy + top + interior_height * 0.5,
            interior_width,
            interior_height,
        ),
        3 => (
            fx + left + interior_width + right * 0.5,
            fy + top + interior_height * 0.5,
            right,
            interior_height,
        ),
        _ => (
            fx + left + interior_width * 0.5,
            fy + top + interior_height + bottom * 0.5,
            interior_width,
            bottom,
        ),
    }
}

fn nine_slice_layout_edges(ns: &crate::frame::NineSlice) -> (f32, f32, f32, f32) {
    if let Some([left, top, right, bottom]) = ns.edge_sizes {
        (left, top, right, bottom)
    } else {
        let horizontal = ns.edge_size;
        let vertical = ns.edge_size_v.unwrap_or(horizontal);
        (horizontal, vertical, horizontal, vertical)
    }
}
