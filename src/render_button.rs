//! Button nine-slice sync and highlight overlay rendering.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::atlas;
use crate::frame::{NineSlice, WidgetData};
use crate::plugin::UiState;
use crate::render::{LoadedTexture, UI_RENDER_LAYER};
use crate::render_texture::{load_texture_source, BlpLoaderRes};
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

/// Returns `(edge_h, edge_v, uv_edge)` — scaled screen edges + original texture edge.
fn button_nine_slice_edges(tex: &TextureSource, frame_w: f32, frame_h: f32) -> (f32, f32, f32) {
    let TextureSource::Atlas(name) = tex else {
        let e = BUTTON_NINE_SLICE_EDGE;
        return (e, e, e);
    };
    let Some(region) = atlas::get_region(name) else {
        let e = BUTTON_NINE_SLICE_EDGE;
        return (e, e, e);
    };
    let base = region.nine_slice_edge.unwrap_or(BUTTON_NINE_SLICE_EDGE);
    (
        base * frame_w / region.width,
        base * frame_h / region.height,
        base,
    )
}

fn select_button_texture_source(btn: &ButtonData) -> Option<&TextureSource> {
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
pub fn sync_button_nine_slices(mut state: ResMut<UiState>) {
    let ids: Vec<u64> = state
        .registry
        .frames_iter()
        .filter(|f| matches!(&f.widget_data, Some(WidgetData::Button(_))))
        .map(|f| f.id)
        .collect();

    for id in ids {
        let texture = extract_button_texture(&state, id);
        let tex = texture.unwrap_or_else(|| default_button_texture(&state, id));
        let Some(frame) = state.registry.get_mut(id) else {
            continue;
        };
        let (eh, ev, uv_e) = button_nine_slice_edges(&tex, frame.width, frame.height);
        frame.nine_slice = Some(NineSlice {
            edge_size: eh,
            edge_size_v: Some(ev),
            uv_edge_size: Some(uv_e),
            bg_color: [1.0, 1.0, 1.0, 1.0],
            border_color: [1.0, 1.0, 1.0, 1.0],
            texture: Some(tex),
            ..Default::default()
        });
    }
}

fn default_button_texture(state: &UiState, id: u64) -> TextureSource {
    let name = match state.registry.get(id).and_then(|f| f.widget_data.as_ref()) {
        Some(WidgetData::Button(btn)) => match btn.state {
            ButtonState::Disabled => DEFAULT_BUTTON_DISABLED,
            ButtonState::Pushed => DEFAULT_BUTTON_PRESSED,
            ButtonState::Normal if btn.hovered => DEFAULT_BUTTON_HIGHLIGHT,
            ButtonState::Normal => DEFAULT_BUTTON_ATLAS,
        },
        _ => DEFAULT_BUTTON_ATLAS,
    };
    TextureSource::Atlas(name.to_string())
}

fn extract_button_texture(state: &UiState, id: u64) -> Option<TextureSource> {
    let frame = state.registry.get(id)?;
    let WidgetData::Button(btn) = &frame.widget_data.as_ref()? else {
        return None;
    };
    select_button_texture_source(btn).cloned()
}

// --- Button highlight overlay ---

/// Manages highlight overlay sprites for hovered buttons.
pub fn sync_ui_button_highlights(
    state: Res<UiState>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    highlights: Query<(Entity, &UiButtonHighlight)>,
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
        upsert_highlight_sprite(frame, texture, sw, sh, &existing, &mut commands);
    }

    despawn_stale_highlights(&existing, &seen, &mut commands);
}

fn button_highlight_source(frame: &crate::frame::Frame) -> Option<&TextureSource> {
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
    commands: &mut Commands,
) {
    let alpha = frame.effective_alpha * 0.5;
    let color = Color::srgba(1.0, 1.0, 1.0, alpha);
    let size = Vec2::new(frame.width, frame.height);
    let bx = frame
        .width
        .mul_add(0.5, frame.layout_rect.as_ref().map_or(0.0, |r| r.x))
        - sw * 0.5;
    let by = sh * 0.5 - frame.layout_rect.as_ref().map_or(0.0, |r| r.y) - frame.height * 0.5;
    let transform = Transform::from_xyz(bx, by, 500.0);
    let sprite = Sprite {
        color,
        custom_size: Some(size),
        image: texture.handle,
        rect: texture.rect,
        ..default()
    };
    if let Some(&entity) = existing.get(&frame.id) {
        commands.entity(entity).insert((transform, sprite));
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
