use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::render::{UI_RENDER_LAYER, texture_tint};
use crate::render_texture::BlpLoaderRes;
use crate::widgets::texture::TextureSource;

const DEFAULT_TILE_SIZE: f32 = 64.0;

/// Marker component linking a tile sprite to a frame and tile index.
#[derive(Component)]
pub struct UiTile(pub u64, pub u32);

fn frame_tiling(f: &crate::frame::Frame) -> Option<(bool, bool)> {
    let WidgetData::Texture(tex) = f.widget_data.as_ref()? else {
        return None;
    };
    if tex.horiz_tile || tex.vert_tile {
        Some((tex.horiz_tile, tex.vert_tile))
    } else {
        None
    }
}

fn frame_tiled_fdid(f: &crate::frame::Frame) -> Option<u32> {
    let WidgetData::Texture(tex) = f.widget_data.as_ref()? else {
        return None;
    };
    match tex.source {
        TextureSource::FileDataId(fdid) => Some(fdid),
        _ => None,
    }
}

fn tile_size(frame: &crate::frame::Frame, horiz: bool, vert: bool) -> Vec2 {
    Vec2::new(
        if !vert {
            frame.width.value()
        } else {
            DEFAULT_TILE_SIZE
        },
        if !horiz {
            frame.height.value()
        } else {
            DEFAULT_TILE_SIZE
        },
    )
}

fn tile_positions(frame: &crate::frame::Frame, horiz: bool, vert: bool) -> Vec<Vec2> {
    let tile = tile_size(frame, horiz, vert);
    let cols = (frame.width.value() / tile.x).ceil() as u32;
    let rows = (frame.height.value() / tile.y).ceil() as u32;
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

fn load_tile_texture(
    fdid: u32,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Option<Handle<Image>> {
    crate::render_texture::load_fdid_texture(fdid, images, texture_cache, missing_textures, blp_loader)
}

fn spawn_or_update_tile(
    commands: &mut Commands,
    existing: &HashMap<(u64, u32), Entity>,
    frame_id: u64,
    tile_idx: u32,
    pos: Vec2,
    tile_sz: Vec2,
    color: Color,
    handle: Handle<Image>,
    screen_w: f32,
    screen_h: f32,
) {
    let bx = pos.x - screen_w * 0.5;
    let by = screen_h * 0.5 - pos.y;
    let transform = Transform::from_xyz(bx, by, 0.001);
    let sprite = Sprite {
        color,
        custom_size: Some(tile_sz),
        image: handle,
        ..default()
    };
    if let Some(&entity) = existing.get(&(frame_id, tile_idx)) {
        commands.entity(entity).insert((transform, sprite));
    } else {
        commands.spawn((
            sprite,
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiTile(frame_id, tile_idx),
        ));
    }
}

fn sync_tiled_frame(
    frame_id: u64,
    state: &UiState,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    existing: &HashMap<(u64, u32), Entity>,
    needed: &mut HashSet<(u64, u32)>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    blp_loader: Option<&BlpLoaderRes>,
    screen_w: f32,
    screen_h: f32,
) {
    let Some(frame) = state.registry.get(frame_id) else {
        return;
    };
    let Some((horiz, vert)) = frame_tiling(frame) else {
        return;
    };
    let Some(fdid) = frame_tiled_fdid(frame) else {
        return;
    };
    let Some(handle) = load_tile_texture(fdid, images, texture_cache, missing_textures, blp_loader) else {
        return;
    };
    let positions = tile_positions(frame, horiz, vert);
    let tile_sz = tile_size(frame, horiz, vert);
    let color = texture_tint(frame);
    for (idx, pos) in positions.iter().enumerate() {
        let tile_idx = idx as u32;
        needed.insert((frame_id, tile_idx));
        spawn_or_update_tile(
            commands,
            existing,
            frame_id,
            tile_idx,
            *pos,
            tile_sz,
            color,
            handle.clone(),
            screen_w,
            screen_h,
        );
    }
}

pub fn sync_ui_tiled_textures(
    mut state: ResMut<UiState>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    tiles: Query<(Entity, &UiTile)>,
    mut texture_cache: Local<HashMap<u32, Handle<Image>>>,
    mut missing_textures: Local<HashSet<u32>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;

    let existing: HashMap<(u64, u32), Entity> =
        tiles.iter().map(|(e, t)| ((t.0, t.1), e)).collect();
    let mut needed: HashSet<(u64, u32)> = HashSet::new();

    let frame_ids: Vec<u64> = state
        .registry
        .frames_iter()
        .filter(|f| f.visible && frame_tiling(f).is_some() && frame_tiled_fdid(f).is_some())
        .map(|f| f.id)
        .collect();

    for frame_id in frame_ids {
        sync_tiled_frame(
            frame_id,
            &state,
            &mut commands,
            &mut images,
            &existing,
            &mut needed,
            &mut texture_cache,
            &mut missing_textures,
            blp_loader.as_deref(),
            screen_w,
            screen_h,
        );
    }

    for (entity, tile) in tiles.iter() {
        if !needed.contains(&(tile.0, tile.1)) {
            commands.entity(entity).despawn();
        }
    }

    state.registry.render_dirty.clear();
}
