//! Camera, ordering and geometry shared by native UI projection.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::frame::WidgetData;
use crate::plugin::UiState;

mod backdrop;

/// Marker component for the 2D UI overlay camera.
#[derive(Component)]
pub struct UiCamera;

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
    pub(crate) indices: HashMap<u64, usize>,
    /// Registry frame count this order and its native projection were built from.
    pub(crate) projected_frames: usize,
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
        Self {
            indices,
            projected_frames: state.registry.frame_count(),
        }
    }
}

/// Rebuilds the order only when the projection is outdated; an unchanged order lets
/// `sync_registry` skip reconciling a settled registry.
pub(crate) fn prepare_ui_frame_order(mut state: ResMut<UiState>, mut order: ResMut<UiFrameOrder>) {
    let registry = &mut state.bypass_change_detection().registry;
    registry.resolve_pending_writes();
    if registry.projection_outdated(order.projected_frames) {
        *order = UiFrameOrder::from_state(&state);
    }
}

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

/// Native image size and offset, including proportional status-bar fill.
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

pub(crate) fn backdrop_part_geometry(
    frame: &crate::frame::Frame,
    part: u8,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
) -> (Transform, Vec2, Color) {
    backdrop::backdrop_part_geometry(frame, part, sort_idx, screen_w, screen_h)
}

pub fn texture_tint(frame: &crate::frame::Frame) -> Color {
    let (vertex_color, desaturated) = match &frame.widget_data {
        Some(WidgetData::Texture(tex)) => (tex.vertex_color, tex.desaturated),
        _ => ([1.0, 1.0, 1.0, 1.0], false),
    };
    let [r, g, b, a] = vertex_color;
    if desaturated {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        Color::srgba(lum, lum, lum, a * frame.effective_alpha)
    } else {
        Color::srgba(r, g, b, a * frame.effective_alpha)
    }
}
