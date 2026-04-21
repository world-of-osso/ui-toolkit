//! Three-slice frame rendering — 3 sprites per frame with three_slice set.
//! Parts: 0=Left cap, 1=Center stretch, 2=Right cap

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::render::{UI_RENDER_LAYER, build_sorted_visible_frame_ids};
use super::render_texture::{BlpLoaderRes, load_texture_source_pub};
use crate::frame::ThreeSlice;
use crate::plugin::UiState;
use crate::widgets::texture::TextureSource;

/// Links a Bevy sprite to a three-slice part (frame_id, part 0-2).
#[derive(Component)]
pub struct UiThreeSlicePart(pub u64, pub u8);

/// Syncs three-slice sprites (3 per frame that has three_slice set).
pub fn sync_ui_three_slices(
    state: Res<UiState>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    parts: Query<(Entity, &UiThreeSlicePart)>,
    mut texture_cache: Local<HashMap<u32, Handle<Image>>>,
    mut file_texture_cache: Local<HashMap<String, Handle<Image>>>,
    mut missing_textures: Local<HashSet<u32>>,
    mut missing_file_textures: Local<HashSet<String>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;
    let z_map = build_z_map(&state);

    let mut existing: HashSet<(u64, u8)> = HashSet::new();
    for (entity, part) in &parts {
        if should_keep(&state, part.0) {
            existing.insert((part.0, part.1));
            let z = z_map.get(&part.0).copied().unwrap_or(0.0);
            update_part(
                &state,
                entity,
                part,
                screen_w,
                screen_h,
                z,
                &mut commands,
                &mut images,
                &mut texture_cache,
                &mut file_texture_cache,
                &mut missing_textures,
                &mut missing_file_textures,
                blp_loader.as_deref(),
            );
        } else {
            commands.entity(entity).despawn();
        }
    }

    spawn_missing(
        &state,
        &existing,
        &z_map,
        screen_w,
        screen_h,
        &mut commands,
        &mut images,
        &mut texture_cache,
        &mut file_texture_cache,
        &mut missing_textures,
        &mut missing_file_textures,
        blp_loader.as_deref(),
    );
}

fn build_z_map(state: &UiState) -> HashMap<u64, f32> {
    build_sorted_visible_frame_ids(state)
        .iter()
        .copied()
        .enumerate()
        .map(|(i, id)| (id, i as f32 * 0.001))
        .collect()
}

fn should_keep(state: &UiState, frame_id: u64) -> bool {
    state
        .registry
        .get(frame_id)
        .is_some_and(|f| f.visible && f.three_slice.is_some())
}

#[allow(clippy::too_many_arguments)]
fn update_part(
    state: &UiState,
    entity: Entity,
    part: &UiThreeSlicePart,
    screen_w: f32,
    screen_h: f32,
    z: f32,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) {
    let Some(frame) = state.registry.get(part.0) else {
        return;
    };
    let Some(ts) = &frame.three_slice else {
        return;
    };
    let (transform, size, color) = part_geometry(frame, ts, part.1, screen_w, screen_h, z);
    let image = resolve_texture(
        part_source(ts, part.1),
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    );
    commands.entity(entity).insert((
        transform,
        Sprite {
            color,
            custom_size: Some(size),
            image,
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_missing(
    state: &UiState,
    existing: &HashSet<(u64, u8)>,
    z_map: &HashMap<u64, f32>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) {
    for frame in state.registry.frames_iter() {
        if !frame.visible {
            continue;
        }
        let Some(ts) = &frame.three_slice else {
            continue;
        };
        let z = z_map.get(&frame.id).copied().unwrap_or(0.0);
        for p in 0..3u8 {
            if existing.contains(&(frame.id, p)) {
                continue;
            }
            let (transform, size, color) = part_geometry(frame, ts, p, screen_w, screen_h, z);
            let image = resolve_texture(
                part_source(ts, p),
                images,
                texture_cache,
                file_texture_cache,
                missing_textures,
                missing_file_textures,
                blp_loader,
            );
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(size),
                    image,
                    ..default()
                },
                transform,
                RenderLayers::layer(UI_RENDER_LAYER),
                UiThreeSlicePart(frame.id, p),
            ));
        }
    }
}

fn part_source(ts: &ThreeSlice, part: u8) -> &TextureSource {
    match part {
        0 => &ts.left,
        1 => &ts.center,
        _ => &ts.right,
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_texture(
    source: &TextureSource,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> Handle<Image> {
    if matches!(source, TextureSource::None) {
        return Handle::default();
    }
    load_texture_source_pub(
        source,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )
    .map(|t| t.handle)
    .unwrap_or_default()
}

/// Compute transform, size, color for one three-slice part.
fn part_geometry(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Dimension;
    use crate::plugin::UiPlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);
        app
    }

    #[test]
    fn three_slice_spawns_3_parts() {
        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("ThreeSliceFrame", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(200.0);
            frame.height = Dimension::Fixed(32.0);
            frame.three_slice = Some(ThreeSlice {
                cap_width: 25.0,
                left: TextureSource::File("left.png".to_string()),
                center: TextureSource::File("center.png".to_string()),
                right: TextureSource::File("right.png".to_string()),
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }
        app.update();
        let mut q = app
            .world_mut()
            .query_filtered::<(), With<UiThreeSlicePart>>();
        assert_eq!(q.iter(app.world()).count(), 3);
    }

    #[test]
    fn frame_without_three_slice_spawns_no_parts() {
        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("PlainFrame", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(200.0);
            frame.height = Dimension::Fixed(32.0);
        }
        app.update();
        let mut q = app
            .world_mut()
            .query_filtered::<(), With<UiThreeSlicePart>>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }
}
