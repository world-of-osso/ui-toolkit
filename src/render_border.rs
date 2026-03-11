//! Backdrop border rendering — emits 4 edge sprites per frame with a border.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::frame::{Backdrop, Border};
use crate::plugin::UiState;

use super::render::UI_RENDER_LAYER;

/// Links a Bevy sprite to a frame border edge (frame_id, edge_index 0-3).
#[derive(Component)]
pub struct UiBorder(pub u64, pub u8);

/// Links a Bevy sprite to a CSS-style border edge (frame_id, side: 0=top,1=right,2=bottom,3=left).
#[derive(Component)]
pub struct UiBorderPart(pub u64, pub u8);

/// Syncs backdrop borders (4 edge sprites per frame that has a backdrop with border_color).
pub fn sync_ui_borders(
    state: Res<UiState>,
    mut commands: Commands,
    borders: Query<(Entity, &UiBorder)>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;

    let mut existing: HashSet<(u64, u8)> = HashSet::new();
    for (entity, border) in &borders {
        if should_keep_border(&state, border) {
            existing.insert((border.0, border.1));
            update_border(&state, entity, border, screen_w, screen_h, &mut commands);
        } else {
            commands.entity(entity).despawn();
        }
    }

    spawn_missing_borders(&state, &existing, screen_w, screen_h, &mut commands);
}

fn should_keep_border(state: &UiState, border: &UiBorder) -> bool {
    state.registry.get(border.0).is_some_and(|f| {
        f.visible
            && f.backdrop
                .as_ref()
                .is_some_and(|b| b.border_color.is_some())
    })
}

fn update_border(
    state: &UiState,
    entity: Entity,
    border: &UiBorder,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    let Some(frame) = state.registry.get(border.0) else {
        return;
    };
    let Some(backdrop) = &frame.backdrop else {
        return;
    };
    let (transform, size, color) = edge_geometry(frame, backdrop, border.1, screen_w, screen_h);
    commands.entity(entity).insert((
        transform,
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
    ));
}

fn spawn_missing_borders(
    state: &UiState,
    existing: &HashSet<(u64, u8)>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    for frame in state.registry.frames_iter() {
        if !frame.visible {
            continue;
        }
        let Some(backdrop) = &frame.backdrop else {
            continue;
        };
        if backdrop.border_color.is_none() {
            continue;
        }
        for edge in 0..4u8 {
            if existing.contains(&(frame.id, edge)) {
                continue;
            }
            spawn_edge(frame, backdrop, edge, screen_w, screen_h, commands);
        }
    }
}

fn spawn_edge(
    frame: &crate::frame::Frame,
    backdrop: &Backdrop,
    edge: u8,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    let (transform, size, color) = edge_geometry(frame, backdrop, edge, screen_w, screen_h);
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        transform,
        RenderLayers::layer(UI_RENDER_LAYER),
        UiBorder(frame.id, edge),
    ));
}

/// Compute transform, size, color for one border edge (0=top, 1=bottom, 2=left, 3=right).
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
    let fw = frame.width;
    let fh = frame.height;

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

/// Syncs CSS-style borders (4 edge sprites per frame that has `border: Some(_)`).
pub fn sync_css_borders(
    state: Res<UiState>,
    mut commands: Commands,
    parts: Query<(Entity, &UiBorderPart)>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;

    let mut existing: HashSet<(u64, u8)> = HashSet::new();
    for (entity, part) in &parts {
        if should_keep_css_border(&state, part) {
            existing.insert((part.0, part.1));
            update_css_border(&state, entity, part, screen_w, screen_h, &mut commands);
        } else {
            commands.entity(entity).despawn();
        }
    }

    spawn_missing_css_borders(&state, &existing, screen_w, screen_h, &mut commands);
}

fn should_keep_css_border(state: &UiState, part: &UiBorderPart) -> bool {
    state
        .registry
        .get(part.0)
        .is_some_and(|f| f.visible && f.border.is_some())
}

fn update_css_border(
    state: &UiState,
    entity: Entity,
    part: &UiBorderPart,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    let Some(frame) = state.registry.get(part.0) else { return };
    let Some(border) = &frame.border else { return };
    let (transform, size, color) = css_edge_geometry(frame, border, part.1, screen_w, screen_h);
    commands.entity(entity).insert((transform, Sprite { color, custom_size: Some(size), ..default() }));
}

fn spawn_missing_css_borders(
    state: &UiState,
    existing: &HashSet<(u64, u8)>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
) {
    for frame in state.registry.frames_iter() {
        if !frame.visible { continue; }
        let Some(border) = &frame.border else { continue };
        for side in 0..4u8 {
            if existing.contains(&(frame.id, side)) { continue; }
            let (transform, size, color) = css_edge_geometry(frame, border, side, screen_w, screen_h);
            commands.spawn((
                Sprite { color, custom_size: Some(size), ..default() },
                transform,
                RenderLayers::layer(UI_RENDER_LAYER),
                UiBorderPart(frame.id, side),
            ));
        }
    }
}

/// Compute transform, size, color for one CSS border edge (0=top,1=right,2=bottom,3=left).
fn css_edge_geometry(
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
    let fw = frame.layout_rect.as_ref().map_or(frame.width, |r| r.width);
    let fh = frame.layout_rect.as_ref().map_or(frame.height, |r| r.height);

    // side: 0=top, 1=right, 2=bottom, 3=left
    let (cx, cy, w, h) = match side {
        0 => (fx + fw * 0.5, fy + e * 0.5,        fw,           e), // top
        1 => (fx + fw - e * 0.5, fy + fh * 0.5,   e,            fh), // right
        2 => (fx + fw * 0.5, fy + fh - e * 0.5,   fw,           e), // bottom
        _ => (fx + e * 0.5, fy + fh * 0.5,         e,            fh), // left
    };

    let bx = cx - screen_w * 0.5;
    let by = screen_h * 0.5 - cy;
    let z = frame.frame_level as f32 * 0.001 + 0.0005;
    (Transform::from_xyz(bx, by, z), Vec2::new(w, h), color)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn border_with_border_color_spawns_4_entities() {
        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("BorderFrame", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = 100.0;
            frame.height = 50.0;
            frame.backdrop = Some(Backdrop {
                border_color: Some([1.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            });
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiBorder>>();
        assert_eq!(q.iter(app.world()).count(), 4);
    }

    #[test]
    fn frame_without_border_color_spawns_no_border() {
        let mut app = test_app();
        app.update();
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let id = ui.registry.create_frame("NoBorderFrame", None);
            let frame = ui.registry.get_mut(id).unwrap();
            frame.width = 100.0;
            frame.height = 50.0;
            frame.backdrop = Some(Backdrop {
                bg_color: Some([0.1, 0.1, 0.1, 1.0]),
                border_color: None,
                ..Default::default()
            });
        }
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiBorder>>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }
}
