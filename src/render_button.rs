//! Button nine-slice sync and highlight overlay rendering.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::atlas;
use crate::frame::{Frame, NineSlice, WidgetData};
use crate::plugin::UiState;
use crate::render::{LoadedTexture, UI_RENDER_LAYER};
use crate::render_texture::{BlpLoaderRes, load_texture_source};
use crate::widgets::button::{ButtonData, ButtonState};
use crate::widgets::texture::TextureSource;

/// Marks a highlight overlay sprite entity for a button frame.
#[derive(Component)]
pub struct UiButtonHighlight(pub u64);

const BUTTON_NINE_SLICE_EDGE: f32 = 4.0;
const DEFAULT_BUTTON_ATLAS: &str = "defaultbutton-nineslice-up";
const DEFAULT_BUTTON_HIGHLIGHT: &str = "defaultbutton-nineslice-highlight";
const DEFAULT_BUTTON_PRESSED: &str = "defaultbutton-nineslice-pressed";
const DEFAULT_BUTTON_DISABLED: &str = "defaultbutton-nineslice-disabled";

fn button_nine_slice_metrics(
    tex: &TextureSource,
    frame_w: f32,
    frame_h: f32,
) -> ([f32; 4], [f32; 4]) {
    let TextureSource::Atlas(name) = tex else {
        let e = BUTTON_NINE_SLICE_EDGE;
        return ([e, e, e, e], [e, e, e, e]);
    };
    let Some(region) = atlas::get_region(name) else {
        let e = BUTTON_NINE_SLICE_EDGE;
        return ([e, e, e, e], [e, e, e, e]);
    };
    let uv = atlas::nine_slice_margins(name).unwrap_or([BUTTON_NINE_SLICE_EDGE; 4]);
    let display = [
        uv[0] * frame_w / region.width,
        uv[1] * frame_h / region.height,
        uv[2] * frame_w / region.width,
        uv[3] * frame_h / region.height,
    ];
    (display, uv)
}

/// Selects an authored base texture; a hover-only highlight is not a base image.
pub(crate) fn select_button_base_texture_source(btn: &ButtonData) -> Option<&TextureSource> {
    let source = match btn.state {
        ButtonState::Normal => btn.normal_texture.as_ref(),
        ButtonState::Pushed => btn.pushed_texture.as_ref().or(btn.normal_texture.as_ref()),
        ButtonState::Disabled => btn
            .disabled_texture
            .as_ref()
            .or(btn.normal_texture.as_ref()),
    }?;
    (!matches!(source, TextureSource::None)).then_some(source)
}

pub(crate) fn select_button_texture_source(btn: &ButtonData) -> Option<&TextureSource> {
    let source = match btn.state {
        ButtonState::Disabled => btn
            .disabled_texture
            .as_ref()
            .or(btn.normal_texture.as_ref()),
        ButtonState::Pushed => btn.pushed_texture.as_ref().or(btn.normal_texture.as_ref()),
        ButtonState::Normal if btn.hovered => btn
            .highlight_texture
            .as_ref()
            .or(btn.normal_texture.as_ref()),
        ButtonState::Normal => btn.normal_texture.as_ref(),
    }?;
    if matches!(source, TextureSource::None) {
        return None;
    }
    Some(source)
}

/// Converts button textures into nine-slice rendering based on current state.
pub fn sync_button_nine_slices(
    mut state: ResMut<UiState>,
    mut generated: Local<HashMap<u64, NineSlice>>,
) {
    generated.retain(|id, _| state.registry.get(*id).is_some());
    let ids: Vec<u64> = state
        .registry
        .frames_iter()
        .filter(|f| matches!(&f.widget_data, Some(WidgetData::Button(_))))
        .map(|f| f.id)
        .collect();

    for id in ids {
        sync_button_slice(&mut state, &mut generated, id);
    }
}

fn sync_button_slice(
    state: &mut ResMut<UiState>,
    generated: &mut HashMap<u64, NineSlice>,
    id: u64,
) {
    let frame = state.registry.get(id).expect("button frame exists");
    let Some(WidgetData::Button(button)) = &frame.widget_data else {
        return;
    };
    if should_skip_generated_slice(frame, button, generated) {
        clear_generated_button_slice(state, generated, id);
        return;
    }
    let source = select_button_texture_source(button)
        .cloned()
        .unwrap_or_else(|| default_button_texture(button));
    let desired = button_nine_slice(frame, source);
    if frame.nine_slice.as_ref() != Some(&desired) {
        state
            .registry
            .get_mut(id)
            .expect("button frame exists")
            .nine_slice = Some(desired.clone());
    }
    generated.insert(id, desired);
}

fn should_skip_generated_slice(
    frame: &Frame,
    button: &ButtonData,
    generated: &HashMap<u64, NineSlice>,
) -> bool {
    if button.use_default_skin {
        return false;
    }
    if frame.three_slice.is_some() {
        return true;
    }
    if frame
        .nine_slice
        .as_ref()
        .is_some_and(|slice| generated.get(&frame.id) != Some(slice))
    {
        return true;
    }
    select_button_base_texture_source(button).is_none()
}

fn button_nine_slice(frame: &Frame, source: TextureSource) -> NineSlice {
    let (display_edges, uv_edges) =
        button_nine_slice_metrics(&source, frame.resolved_width(), frame.resolved_height());
    NineSlice {
        edge_size: display_edges[0],
        edge_size_v: Some(display_edges[1]),
        edge_sizes: Some(display_edges),
        uv_edge_size: Some(uv_edges[0]),
        uv_edge_sizes: Some(uv_edges),
        bg_color: [1.0, 1.0, 1.0, 1.0],
        border_color: [1.0, 1.0, 1.0, 1.0],
        texture: Some(source),
        ..Default::default()
    }
}

fn clear_generated_button_slice(
    state: &mut ResMut<UiState>,
    generated: &mut HashMap<u64, NineSlice>,
    id: u64,
) {
    let Some(previous) = generated.remove(&id) else {
        return;
    };
    if state
        .registry
        .get(id)
        .and_then(|frame| frame.nine_slice.as_ref())
        == Some(&previous)
    {
        state
            .registry
            .get_mut(id)
            .expect("button frame exists")
            .nine_slice = None;
    }
}

fn default_button_texture(button: &ButtonData) -> TextureSource {
    let name = match button.state {
        ButtonState::Disabled => DEFAULT_BUTTON_DISABLED,
        ButtonState::Pushed => DEFAULT_BUTTON_PRESSED,
        ButtonState::Normal if button.hovered => DEFAULT_BUTTON_HIGHLIGHT,
        ButtonState::Normal => DEFAULT_BUTTON_ATLAS,
    };
    TextureSource::Atlas(name.to_string())
}

// --- Button highlight overlay ---

/// Manages highlight overlay sprites for hovered buttons.
pub fn sync_ui_button_highlights(
    state: Res<UiState>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    highlights: Query<(Entity, &UiButtonHighlight)>,
    visuals: Query<(&Transform, &Sprite)>,
    mut texture_cache: Local<HashMap<u32, Handle<Image>>>,
    mut file_texture_cache: Local<HashMap<String, Handle<Image>>>,
    mut missing_textures: Local<HashSet<u32>>,
    mut missing_file_textures: Local<HashSet<String>>,
    blp_loader: Option<Res<BlpLoaderRes>>,
) {
    let existing: HashMap<u64, Entity> = highlights.iter().map(|(e, h)| (h.0, e)).collect();
    let mut seen: HashSet<u64> = HashSet::new();
    let sw = state.registry.screen_width;
    let sh = state.registry.screen_height;

    for frame in state.registry.frames_iter() {
        let Some(source) = button_highlight_source(frame) else {
            continue;
        };
        seen.insert(frame.id);
        let Some(WidgetData::Button(btn)) = &frame.widget_data else {
            continue;
        };
        if !btn.hovered || btn.state == ButtonState::Disabled {
            if let Some(&entity) = existing.get(&frame.id) {
                commands.entity(entity).despawn();
            }
            continue;
        }
        let Some(texture) = load_texture_source(
            source,
            &mut images,
            &mut texture_cache,
            &mut file_texture_cache,
            &mut missing_textures,
            &mut missing_file_textures,
            blp_loader.as_deref(),
        ) else {
            continue;
        };
        upsert_highlight_sprite(frame, texture, sw, sh, &existing, &visuals, &mut commands);
    }

    despawn_stale_highlights(&existing, &seen, &mut commands);
}

pub(crate) fn button_highlight_source(frame: &crate::frame::Frame) -> Option<&TextureSource> {
    // Nine-slice buttons handle their own visual states; skip the flat highlight overlay.
    if frame.nine_slice.is_some() {
        return None;
    }
    let WidgetData::Button(btn) = frame.widget_data.as_ref()? else {
        return None;
    };
    btn.highlight_texture.as_ref()
}

fn upsert_highlight_sprite(
    frame: &crate::frame::Frame,
    texture: LoadedTexture,
    sw: f32,
    sh: f32,
    existing: &HashMap<u64, Entity>,
    visuals: &Query<(&Transform, &Sprite)>,
    commands: &mut Commands,
) {
    let Some(WidgetData::Button(button)) = &frame.widget_data else {
        return;
    };
    let alpha = frame.effective_alpha * 0.5;
    let color = Color::srgba(1.0, 1.0, 1.0, alpha);
    let button_size = Vec2::new(frame.resolved_width(), frame.resolved_height());
    let size = button.highlight_size.map_or(button_size, Vec2::from_array);
    let center_x = if button.highlight_size.is_some() {
        button_size.x / 2.0
    } else {
        frame.width.value() / 2.0
    };
    let bx = center_x + frame.layout_rect.as_ref().map_or(0.0, |r| r.x) - sw * 0.5;
    let by = sh * 0.5 - frame.layout_rect.as_ref().map_or(0.0, |r| r.y) - button_size.y * 0.5;
    let transform = Transform::from_xyz(bx, by, 500.0);
    let sprite = Sprite {
        color,
        custom_size: Some(size),
        image: texture.handle,
        rect: texture.rect,
        ..default()
    };
    if let Some(&entity) = existing.get(&frame.id) {
        crate::render::insert_changed_quad_visuals(
            commands,
            entity,
            visuals.get(entity).ok(),
            transform,
            sprite,
        );
    } else {
        commands.spawn((
            sprite,
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiButtonHighlight(frame.id),
        ));
    }
}

fn despawn_stale_highlights(
    existing: &HashMap<u64, Entity>,
    seen: &HashSet<u64>,
    commands: &mut Commands,
) {
    for (&frame_id, &entity) in existing {
        if !seen.contains(&frame_id) {
            commands.entity(entity).despawn();
        }
    }
}
