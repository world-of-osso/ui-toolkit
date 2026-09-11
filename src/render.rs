use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::render_texture::BlpLoaderRes;
use crate::widgets::texture::TextureSource;

mod backdrop;
mod visual;
/// Marker component for the 2D UI overlay camera.
#[derive(Component)]
pub struct UiCamera;

/// Links a Bevy sprite entity to a UI frame by its ID.
#[derive(Component)]
pub struct UiQuad(pub u64);

/// Links a Bevy sprite entity to a nine-slice-aware background part.
/// Parts: 0=Top, 1=Left, 2=Center, 3=Right, 4=Bottom
#[derive(Component)]
pub struct UiBackdropQuad(pub u64, pub u8);

/// Links a Bevy Text2d entity to a UI frame by its ID.
#[derive(Component)]
pub struct UiText(pub u64);

#[derive(Clone)]
pub struct LoadedTexture {
    pub handle: Handle<Image>,
    pub rect: Option<Rect>,
}

/// Render layer used for all UI elements, separate from the 3D scene.
pub const UI_RENDER_LAYER: usize = 1;

/// Spawns a 2D camera that renders after the 3D camera with a transparent background.
pub fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(UI_RENDER_LAYER),
        UiCamera,
    ));
}

/// Ordering prepared for one plugin render update.
#[derive(Default, Resource)]
pub(crate) struct UiFrameOrder {
    pub(crate) ids: Vec<u64>,
    pub(crate) indices: HashMap<u64, usize>,
}

impl UiFrameOrder {
    pub(crate) fn from_state(state: &UiState) -> Self {
        let ids = build_sorted_visible_frame_ids(state);
        let indices = ids
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        Self { ids, indices }
    }

    pub(crate) fn z(&self, id: u64) -> f32 {
        self.indices.get(&id).copied().unwrap_or(0) as f32 * 0.001
    }
}

pub(crate) fn prepare_ui_frame_order(state: Res<UiState>, mut order: ResMut<UiFrameOrder>) {
    *order = UiFrameOrder::from_state(&state);
}

/// Syncs the frame registry into Bevy sprite entities each frame.
pub fn sync_ui_quads(
    state: ResMut<UiState>,
    commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    quads: Query<(Entity, &UiQuad)>,
    backdrop_quads: Query<(Entity, &UiBackdropQuad)>,
    visuals: Query<(&Transform, &Sprite)>,
    texture_cache: Local<HashMap<u32, Handle<Image>>>,
    file_texture_cache: Local<HashMap<String, Handle<Image>>>,
    missing_textures: Local<HashSet<u32>>,
    missing_file_textures: Local<HashSet<String>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    let order = UiFrameOrder::from_state(&state);
    sync_ui_quads_with_order(
        state,
        &order,
        commands,
        images,
        quads,
        backdrop_quads,
        visuals,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    );
}

pub(crate) fn sync_ui_quads_prepared(
    state: ResMut<UiState>,
    order: Res<UiFrameOrder>,
    commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    quads: Query<(Entity, &UiQuad)>,
    backdrop_quads: Query<(Entity, &UiBackdropQuad)>,
    visuals: Query<(&Transform, &Sprite)>,
    texture_cache: Local<HashMap<u32, Handle<Image>>>,
    file_texture_cache: Local<HashMap<String, Handle<Image>>>,
    missing_textures: Local<HashSet<u32>>,
    missing_file_textures: Local<HashSet<String>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    sync_ui_quads_with_order(
        state,
        &order,
        commands,
        images,
        quads,
        backdrop_quads,
        visuals,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    );
}

fn sync_ui_quads_with_order(
    mut state: ResMut<UiState>,
    order: &UiFrameOrder,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    quads: Query<(Entity, &UiQuad)>,
    backdrop_quads: Query<(Entity, &UiBackdropQuad)>,
    visuals: Query<(&Transform, &Sprite)>,
    mut texture_cache: Local<HashMap<u32, Handle<Image>>>,
    mut file_texture_cache: Local<HashMap<String, Handle<Image>>>,
    mut missing_textures: Local<HashSet<u32>>,
    mut missing_file_textures: Local<HashSet<String>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;

    let visible_sorted_ids = &order.ids;
    let sorted_quad_ids: Vec<u64> = visible_sorted_ids
        .iter()
        .copied()
        .filter(|id| state.registry.get(*id).is_some_and(is_renderable))
        .filter(|id| {
            state
                .registry
                .get(*id)
                .is_some_and(|frame| !uses_backdrop_parts(frame))
        })
        .collect();
    let sorted_backdrop_ids: Vec<u64> = visible_sorted_ids
        .iter()
        .copied()
        .filter(|id| state.registry.get(*id).is_some_and(uses_backdrop_parts))
        .collect();
    let sort_map = &order.indices;

    update_or_despawn_quads(
        &state,
        sort_map,
        screen_w,
        screen_h,
        &mut commands,
        &mut images,
        &mut texture_cache,
        &mut file_texture_cache,
        &mut missing_textures,
        &mut missing_file_textures,
        &quads,
        &visuals,
        blp_loader.as_deref(),
    );
    update_or_despawn_backdrop_quads(
        &state,
        sort_map,
        screen_w,
        screen_h,
        &mut commands,
        &backdrop_quads,
        &visuals,
    );

    let existing: HashSet<u64> = quads.iter().map(|(_, q)| q.0).collect();
    spawn_new_quads(
        &state,
        &sorted_quad_ids,
        sort_map,
        &existing,
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
    let existing_backdrop_parts: HashSet<(u64, u8)> =
        backdrop_quads.iter().map(|(_, q)| (q.0, q.1)).collect();
    spawn_new_backdrop_quads(
        &state,
        &sorted_backdrop_ids,
        sort_map,
        &existing_backdrop_parts,
        screen_w,
        screen_h,
        &mut commands,
    );

    if !state.registry.render_dirty.is_empty() {
        state.registry.render_dirty.clear();
    }
}

fn update_or_despawn_quads(
    state: &UiState,
    sort_map: &HashMap<u64, usize>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    quads: &Query<(Entity, &UiQuad)>,
    visuals: &Query<(&Transform, &Sprite)>,
    blp_loader: Option<&BlpLoaderRes>,
) {
    for (entity, ui_quad) in quads {
        if let Some(&sort_idx) = sort_map.get(&ui_quad.0) {
            update_quad(
                state,
                entity,
                ui_quad.0,
                visuals.get(entity).ok(),
                sort_idx,
                screen_w,
                screen_h,
                commands,
                images,
                texture_cache,
                file_texture_cache,
                missing_textures,
                missing_file_textures,
                blp_loader,
            );
        } else {
            commands.entity(entity).despawn();
        }
    }
}

// --- Quad helpers ---

const BACKDROP_PART_COUNT: u8 = 5;

fn sort_frame_ids<'a>(frames: impl Iterator<Item = &'a crate::frame::Frame>) -> Vec<u64> {
    let mut frames: Vec<_> = frames
        .map(|f| (f.id, f.strata, f.frame_level, f.raise_order))
        .collect();
    frames.sort_unstable_by(|a, b| {
        a.1.cmp(&b.1)
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
            .then(a.0.cmp(&b.0))
    });
    frames.into_iter().map(|(id, _, _, _)| id).collect()
}

pub(crate) fn build_sorted_visible_frame_ids(state: &UiState) -> Vec<u64> {
    sort_frame_ids(state.registry.frames_iter().filter(|frame| {
        if !frame.visible {
            return false;
        }
        let (width, height) = effective_size(frame);
        width > 0.0 && height > 0.0
    }))
}

/// Effective size: layout_rect if available, else explicit width/height.
fn effective_size(f: &crate::frame::Frame) -> (f32, f32) {
    f.layout_rect
        .as_ref()
        .map(|r| (r.width, r.height))
        .unwrap_or((f.resolved_width(), f.resolved_height()))
}

pub(crate) fn is_renderable(f: &crate::frame::Frame) -> bool {
    if f.nine_slice.is_some() || f.three_slice.is_some() {
        // nine_slice/three_slice frames are rendered by their own systems,
        // but allow a background_color quad behind them (WoW backdropColor).
        return f.background_color.is_some();
    }
    let (w, h) = effective_size(f);
    f.visible
        && w > 0.0
        && h > 0.0
        && (f.background_color.is_some()
            || frame_texture_source(f).is_some()
            || frame_has_button_texture(f)
            || f.backdrop.as_ref().is_some_and(|b| b.bg_color.is_some())
            || matches!(f.widget_data, Some(WidgetData::StatusBar(_))))
}

fn uses_backdrop_parts(f: &crate::frame::Frame) -> bool {
    f.nine_slice.is_some() && f.background_color.is_some()
}

fn frame_has_button_texture(f: &crate::frame::Frame) -> bool {
    let Some(WidgetData::Button(btn)) = &f.widget_data else {
        return false;
    };
    btn.normal_texture.is_some() || btn.pushed_texture.is_some() || btn.disabled_texture.is_some()
}

fn frame_texture_source(f: &crate::frame::Frame) -> Option<&TextureSource> {
    let WidgetData::Texture(texture) = f.widget_data.as_ref()? else {
        return None;
    };
    if matches!(texture.source, TextureSource::None) {
        return None;
    }
    Some(&texture.source)
}

pub(crate) fn frame_transform(
    f: &crate::frame::Frame,
    sort_idx: usize,
    sw: f32,
    sh: f32,
) -> Transform {
    let (w, h) = effective_size(f);
    let bx = w.mul_add(0.5, f.layout_rect.as_ref().map_or(0.0, |r| r.x)) - sw * 0.5;
    let by = sh * 0.5 - f.layout_rect.as_ref().map_or(0.0, |r| r.y) - h * 0.5;
    let mut tf = Transform::from_xyz(bx, by, sort_idx as f32 * 0.001);
    if let Some(WidgetData::Texture(tex)) = &f.widget_data {
        if tex.rotation != 0.0 {
            tf.rotation = Quat::from_rotation_z(tex.rotation);
        }
    }
    tf
}

pub(crate) fn frame_color(f: &crate::frame::Frame) -> Color {
    let base = f
        .background_color
        .or_else(|| f.backdrop.as_ref().and_then(|b| b.bg_color));
    let [r, g, b, a] = base.unwrap_or([1.0, 1.0, 1.0, 1.0]);
    Color::srgba(r, g, b, a * f.effective_alpha)
}

/// Returns `(size, offset)` for the sprite quad.
///
/// For StatusBar, width is scaled by the fill fraction and the quad is
/// left-aligned by shifting right by half the difference between the full
/// frame width and the filled width.  All other frames use their full size
/// with no offset.
pub(crate) fn frame_sprite_params(f: &crate::frame::Frame) -> (Vec2, Vec2) {
    let (w, h) = effective_size(f);
    if let Some(WidgetData::StatusBar(sb)) = &f.widget_data {
        let fill =
            ((sb.value - sb.min) / (sb.max - sb.min).max(f64::EPSILON)).clamp(0.0, 1.0) as f32;
        let filled_w = w * fill;
        let offset_x = (filled_w - w) * 0.5;
        (Vec2::new(filled_w, h), Vec2::new(offset_x, 0.0))
    } else {
        (Vec2::new(w, h), Vec2::ZERO)
    }
}

fn update_quad(
    state: &UiState,
    entity: Entity,
    frame_id: u64,
    current: Option<(&Transform, &Sprite)>,
    sort_idx: usize,
    sw: f32,
    sh: f32,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) {
    let Some(frame) = state.registry.get(frame_id) else {
        return;
    };
    let (sprite_size, sprite_offset) = frame_sprite_params(frame);
    let mut transform = frame_transform(frame, sort_idx, sw, sh);
    transform.translation.x += sprite_offset.x;
    transform.translation.y += sprite_offset.y;
    let (color, image, rect) = frame_visual(
        frame,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    );
    insert_changed_quad_visuals(
        commands,
        entity,
        current,
        transform,
        Sprite {
            color,
            custom_size: Some(sprite_size),
            image,
            rect,
            ..default()
        },
    );
}

pub(super) fn insert_changed_quad_visuals(
    commands: &mut Commands,
    entity: Entity,
    current: Option<(&Transform, &Sprite)>,
    transform: Transform,
    sprite: Sprite,
) {
    let Some((old_transform, old_sprite)) = current else {
        commands.entity(entity).insert((transform, sprite));
        return;
    };
    if *old_transform != transform {
        commands.entity(entity).insert(transform);
    }
    if !sprites_equal(old_sprite, &sprite) {
        commands.entity(entity).insert(sprite);
    }
}

fn sprites_equal(left: &Sprite, right: &Sprite) -> bool {
    left.image == right.image
        && left.texture_atlas == right.texture_atlas
        && left.color == right.color
        && left.flip_x == right.flip_x
        && left.flip_y == right.flip_y
        && left.custom_size == right.custom_size
        && left.rect == right.rect
        && left.image_mode == right.image_mode
}

fn update_or_despawn_backdrop_quads(
    state: &UiState,
    sort_map: &HashMap<u64, usize>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    backdrop_quads: &Query<(Entity, &UiBackdropQuad)>,
    visuals: &Query<(&Transform, &Sprite)>,
) {
    for (entity, backdrop_part) in backdrop_quads {
        if should_keep_backdrop_part(state, backdrop_part) {
            let Some(&sort_idx) = sort_map.get(&backdrop_part.0) else {
                commands.entity(entity).despawn();
                continue;
            };
            let (transform, size, color) =
                backdrop_part_geometry_for_id(state, backdrop_part, sort_idx, screen_w, screen_h);
            insert_changed_quad_visuals(
                commands,
                entity,
                visuals.get(entity).ok(),
                transform,
                Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
            );
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_new_quads(
    state: &UiState,
    sorted_ids: &[u64],
    sort_map: &HashMap<u64, usize>,
    existing: &HashSet<u64>,
    sw: f32,
    sh: f32,
    commands: &mut Commands,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) {
    for &frame_id in sorted_ids {
        if existing.contains(&frame_id) {
            continue;
        }
        let Some(frame) = state.registry.get(frame_id) else {
            continue;
        };
        let sort_idx = sort_map[&frame_id];
        let (sprite_size, sprite_offset) = frame_sprite_params(frame);
        let mut transform = frame_transform(frame, sort_idx, sw, sh);
        transform.translation.x += sprite_offset.x;
        transform.translation.y += sprite_offset.y;
        let (color, image, rect) = frame_visual(
            frame,
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
                custom_size: Some(sprite_size),
                image,
                rect,
                ..default()
            },
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiQuad(frame_id),
        ));
    }
}

fn spawn_new_backdrop_quads(
    state: &UiState,
    sorted_ids: &[u64],
    sort_map: &HashMap<u64, usize>,
    existing: &HashSet<(u64, u8)>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    for &frame_id in sorted_ids {
        let Some(&sort_idx) = sort_map.get(&frame_id) else {
            continue;
        };
        for part in 0..BACKDROP_PART_COUNT {
            if existing.contains(&(frame_id, part)) {
                continue;
            }
            let backdrop_part = UiBackdropQuad(frame_id, part);
            if !should_keep_backdrop_part(state, &backdrop_part) {
                continue;
            }
            let (transform, size, color) =
                backdrop_part_geometry_for_id(state, &backdrop_part, sort_idx, screen_w, screen_h);
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                transform,
                RenderLayers::layer(UI_RENDER_LAYER),
                backdrop_part,
            ));
        }
    }
}

fn frame_visual(
    frame: &crate::frame::Frame,
    images: &mut Option<ResMut<Assets<Image>>>,
    texture_cache: &mut HashMap<u32, Handle<Image>>,
    file_texture_cache: &mut HashMap<String, Handle<Image>>,
    missing_textures: &mut HashSet<u32>,
    missing_file_textures: &mut HashSet<String>,
    blp_loader: Option<&BlpLoaderRes>,
) -> (Color, Handle<Image>, Option<Rect>) {
    visual::frame_visual(
        frame,
        images,
        texture_cache,
        file_texture_cache,
        missing_textures,
        missing_file_textures,
        blp_loader,
    )
}

fn should_keep_backdrop_part(state: &UiState, backdrop_part: &UiBackdropQuad) -> bool {
    backdrop::should_keep_backdrop_part(state, backdrop_part)
}

fn backdrop_part_geometry_for_id(
    state: &UiState,
    backdrop_part: &UiBackdropQuad,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    backdrop::backdrop_part_geometry_for_id(state, backdrop_part, sort_idx, screen_w, screen_h)
}

pub(crate) fn backdrop_part_geometry(
    frame: &crate::frame::Frame,
    part: u8,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    backdrop::backdrop_part_geometry(frame, part, sort_idx, screen_w, screen_h)
}

/// Apply vertex_color tinting, effective_alpha, and desaturation to textured frames.
pub fn texture_tint(frame: &crate::frame::Frame) -> Color {
    visual::texture_tint(frame)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Dimension;
    use crate::plugin::UiPlugin;
    use crate::widgets::button::ButtonState;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy::text::Font>();
        app.add_plugins(UiPlugin);
        app
    }

    #[derive(Resource, Default)]
    struct QuadChanges {
        transforms: usize,
        sprites: usize,
    }

    fn observe_quad_changes(
        transforms: Query<(), (Changed<Transform>, Or<(With<UiQuad>, With<UiBackdropQuad>)>)>,
        sprites: Query<(), (Changed<Sprite>, Or<(With<UiQuad>, With<UiBackdropQuad>)>)>,
        mut changes: ResMut<QuadChanges>,
    ) {
        changes.transforms = transforms.iter().count();
        changes.sprites = sprites.iter().count();
    }

    fn quad_change_fixture(nine_slice: bool) -> (App, u64) {
        let mut app = test_app();
        app.init_resource::<QuadChanges>();
        app.add_systems(PostUpdate, observe_quad_changes);
        app.update();
        let id = {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("Changes", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(100.0);
            frame.height = Dimension::Fixed(50.0);
            frame.background_color = Some([1.0, 0.0, 0.0, 1.0]);
            if nine_slice {
                frame.nine_slice = Some(crate::frame::NineSlice {
                    edge_size: 8.0,
                    ..default()
                });
            }
            id
        };
        app.update();
        (app, id)
    }

    #[test]
    fn quad_changes_settled_visuals_do_not_invalidate_components() {
        for nine_slice in [false, true] {
            let (mut app, id) = quad_change_fixture(nine_slice);
            // A mutable registry lookup marks dirty even without changing the output.
            app.world_mut()
                .resource_mut::<UiState>()
                .registry
                .get_mut(id)
                .unwrap();
            app.update();
            let changes = app.world().resource::<QuadChanges>();
            assert_eq!(changes.transforms, 0, "nine_slice={nine_slice}");
            assert_eq!(changes.sprites, 0, "nine_slice={nine_slice}");
        }
    }

    #[test]
    fn quad_changes_color_and_geometry_remain_live() {
        for nine_slice in [false, true] {
            let (mut app, id) = quad_change_fixture(nine_slice);
            app.update();
            app.world_mut()
                .resource_mut::<UiState>()
                .registry
                .get_mut(id)
                .unwrap()
                .background_color = Some([0.0, 1.0, 0.0, 1.0]);
            app.update();
            let changes = app.world().resource::<QuadChanges>();
            assert_eq!(changes.transforms, 0, "color must not invalidate geometry");
            assert_eq!(changes.sprites, if nine_slice { 5 } else { 1 });
            let mut sprites = app
                .world_mut()
                .query_filtered::<&Sprite, Or<(With<UiQuad>, With<UiBackdropQuad>)>>();
            assert!(
                sprites
                    .iter(app.world())
                    .all(|sprite| sprite.color == Color::srgba(0.0, 1.0, 0.0, 1.0))
            );
            {
                let mut ui = app.world_mut().resource_mut::<UiState>();
                let frame = ui.registry.get_mut(id).unwrap();
                frame.width = Dimension::Fixed(200.0);
                // Unanchored frames retain explicit layout rectangles until invalidated.
                frame.layout_rect = None;
            }
            app.update();
            let changes = app.world().resource::<QuadChanges>();
            assert!(changes.transforms > 0);
            assert!(changes.sprites > 0);
            let mut sprites = app.world_mut().query_filtered::<&Sprite, With<UiQuad>>();
            if !nine_slice {
                assert_eq!(
                    sprites.single(app.world()).unwrap().custom_size,
                    Some(Vec2::new(200.0, 50.0))
                );
            }
        }
    }

    #[test]
    fn ui_camera_spawned() {
        let mut app = test_app();
        app.update();
        let mut query = app.world_mut().query_filtered::<(), With<UiCamera>>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    #[test]
    fn creates_quad_for_visible_frame() {
        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("Test", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(100.0);
            frame.height = Dimension::Fixed(50.0);
            frame.background_color = Some([1.0, 0.0, 0.0, 1.0]);
        }
        app.update();
        let mut query = app.world_mut().query_filtered::<(), With<UiQuad>>();
        assert!(query.iter(app.world()).count() > 0);
    }

    #[test]
    fn no_quad_without_background_color() {
        let mut app = test_app();
        app.update();
        let baseline = {
            let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
            q.iter(app.world()).count()
        };
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("NoColor", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(100.0);
            frame.height = Dimension::Fixed(50.0);
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
        assert_eq!(q.iter(app.world()).count(), baseline);
    }

    #[test]
    fn despawns_quad_when_hidden() {
        let mut app = test_app();
        app.update();
        let baseline = {
            let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
            q.iter(app.world()).count()
        };
        let frame_id;
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            frame_id = ui.registry.create_frame("HideMe", None);
            let frame = ui.registry.get_mut(frame_id).unwrap();
            frame.width = Dimension::Fixed(100.0);
            frame.height = Dimension::Fixed(50.0);
            frame.background_color = Some([0.0, 1.0, 0.0, 1.0]);
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
        assert_eq!(q.iter(app.world()).count(), baseline + 1);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            ui.registry.set_hidden(frame_id, true);
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
        assert_eq!(q.iter(app.world()).count(), baseline);
    }

    #[test]
    fn backdrop_bg_color_renderable() {
        let mut app = test_app();
        app.update();
        let baseline = {
            let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
            q.iter(app.world()).count()
        };
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("Bd", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(100.0);
            frame.height = Dimension::Fixed(50.0);
            frame.backdrop = Some(crate::frame::Backdrop {
                bg_color: Some([0.1, 0.1, 0.1, 1.0]),
                ..Default::default()
            });
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiQuad>>();
        assert_eq!(q.iter(app.world()).count(), baseline + 1);
    }

    #[test]
    fn statusbar_sprite_params_proportional_to_fill() {
        let mut frame = crate::frame::Frame::new(1, None, crate::frame::WidgetType::StatusBar);
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(20.0);
        frame.widget_data = Some(WidgetData::StatusBar(
            crate::widgets::slider::StatusBarData {
                value: 0.5,
                min: 0.0,
                max: 1.0,
                ..Default::default()
            },
        ));
        let (size, offset) = frame_sprite_params(&frame);
        assert!(
            (size.x - 100.0).abs() < 0.01,
            "half fill → width 100, got {}",
            size.x
        );
        assert_eq!(size.y, 20.0);
        assert!(
            (offset.x - (-50.0)).abs() < 0.01,
            "offset_x should be -50, got {}",
            offset.x
        );
        assert_eq!(offset.y, 0.0);
    }

    #[test]
    fn statusbar_sprite_params_full_fill() {
        let mut frame = crate::frame::Frame::new(1, None, crate::frame::WidgetType::StatusBar);
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(20.0);
        frame.widget_data = Some(WidgetData::StatusBar(
            crate::widgets::slider::StatusBarData {
                value: 1.0,
                min: 0.0,
                max: 1.0,
                ..Default::default()
            },
        ));
        let (size, offset) = frame_sprite_params(&frame);
        assert!((size.x - 200.0).abs() < 0.01);
        assert!((offset.x).abs() < 0.01);
    }

    #[test]
    fn button_disabled_text_grey() {
        let btn = crate::widgets::button::ButtonData {
            state: ButtonState::Disabled,
            text: "Test".into(),
            ..Default::default()
        };
        let color = crate::render_text::extract_button_text(&btn, 1.0).color;
        let Color::Srgba(srgba) = color else {
            panic!("expected srgba")
        };
        assert!(srgba.red < 0.6, "disabled should be grey");
    }

    #[test]
    fn nine_slice_background_uses_cross_shaped_backdrop_parts() {
        use crate::frame::NineSlice;
        use crate::layout::LayoutRect;

        let mut frame = crate::frame::Frame::new(1, None, crate::frame::WidgetType::Frame);
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(40.0);
        frame.background_color = Some([0.15, 0.12, 0.09, 1.0]);
        frame.layout_rect = Some(LayoutRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 40.0,
        });
        frame.nine_slice = Some(NineSlice {
            edge_size: 8.0,
            ..Default::default()
        });

        let (_, top_size, _) = backdrop_part_geometry(&frame, 0, 0, 1920.0, 1080.0);
        assert_eq!(top_size, Vec2::new(184.0, 8.0));

        let (_, left_size, _) = backdrop_part_geometry(&frame, 1, 0, 1920.0, 1080.0);
        assert_eq!(left_size, Vec2::new(8.0, 24.0));

        let (_, center_size, _) = backdrop_part_geometry(&frame, 2, 0, 1920.0, 1080.0);
        assert_eq!(center_size, Vec2::new(184.0, 24.0));

        let (_, right_size, _) = backdrop_part_geometry(&frame, 3, 0, 1920.0, 1080.0);
        assert_eq!(right_size, Vec2::new(8.0, 24.0));

        let (_, bottom_size, _) = backdrop_part_geometry(&frame, 4, 0, 1920.0, 1080.0);
        assert_eq!(bottom_size, Vec2::new(184.0, 8.0));
    }

    #[test]
    fn nine_slice_background_spawns_backdrop_parts_instead_of_full_quad() {
        use crate::frame::NineSlice;

        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("NineSliceBackdrop", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = Dimension::Fixed(200.0);
            frame.height = Dimension::Fixed(40.0);
            frame.background_color = Some([0.15, 0.12, 0.09, 1.0]);
            frame.nine_slice = Some(NineSlice {
                edge_size: 8.0,
                ..Default::default()
            });
        }

        app.update();

        let backdrop_count = app
            .world_mut()
            .query_filtered::<(), With<UiBackdropQuad>>()
            .iter(app.world())
            .count();
        assert_eq!(backdrop_count, 5);

        let quad_count = app
            .world_mut()
            .query_filtered::<(), With<UiQuad>>()
            .iter(app.world())
            .count();
        assert_eq!(quad_count, 0);
    }
}
