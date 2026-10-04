//! Native tiled texture layout.

use crate::frame::WidgetData;
use crate::widgets::texture::TextureSource;
use bevy::prelude::*;

const DEFAULT_TILE_SIZE: f32 = 64.0;

pub(crate) fn frame_tiling(f: &crate::frame::Frame) -> Option<(bool, bool)> {
    let WidgetData::Texture(tex) = f.widget_data.as_ref()? else {
        return None;
    };
    if tex.horiz_tile || tex.vert_tile {
        Some((tex.horiz_tile, tex.vert_tile))
    } else {
        None
    }
}

pub(crate) fn frame_tiled_fdid(f: &crate::frame::Frame) -> Option<u32> {
    let WidgetData::Texture(tex) = f.widget_data.as_ref()? else {
        return None;
    };
    match tex.source {
        TextureSource::FileDataId(fdid) => Some(fdid),
        _ => None,
    }
}

pub(crate) fn tile_size(frame: &crate::frame::Frame, horiz: bool, vert: bool) -> Vec2 {
    Vec2::new(
        if !vert {
            frame.resolved_width()
        } else {
            DEFAULT_TILE_SIZE
        },
        if !horiz {
            frame.resolved_height()
        } else {
            DEFAULT_TILE_SIZE
        },
    )
}

pub(crate) fn tile_positions(frame: &crate::frame::Frame, horiz: bool, vert: bool) -> Vec<Vec2> {
    let tile = tile_size(frame, horiz, vert);
    let cols = (frame.resolved_width() / tile.x).ceil() as u32;
    let rows = (frame.resolved_height() / tile.y).ceil() as u32;
    let ox = frame.layout_rect.as_ref().map_or(0.0, |r| r.x);
    let oy = frame.layout_rect.as_ref().map_or(0.0, |r| r.y);
    let mut out = Vec::with_capacity((cols * rows) as usize);
    for row in 0..rows {
        for col in 0..cols {
            out.push(Vec2::new(
                ox + col as f32 * tile.x + tile.x * 0.5,
                oy + row as f32 * tile.y + tile.y * 0.5,
            ));
        }
    }
    out
}
