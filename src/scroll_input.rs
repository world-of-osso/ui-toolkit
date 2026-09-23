use bevy::input::ButtonInput;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use crate::input::find_frame_at;
use crate::plugin::UiState;
use crate::registry::FrameRegistry;
use crate::widgets::scroll_list::{thumb_name, track_name};

/// One frame of input relevant to scroll lists.
#[derive(Debug, Default, Clone)]
pub struct ScrollInput {
    pub cursor: Option<(f32, f32)>,
    pub pressed: bool,
    pub released: bool,
    /// Wheel movement in rows; positive scrolls toward later rows.
    pub wheel_rows: f32,
    pub wheel_pixels: f32,
    pub keys: Vec<KeyCode>,
}

impl ScrollInput {
    fn is_idle(&self, ui: &UiState) -> bool {
        !self.pressed
            && !self.released
            && self.wheel_rows == 0.0
            && self.wheel_pixels == 0.0
            && self.keys.is_empty()
            && ui.registry.scroll_lists.dragging().next().is_none()
    }
}

/// Wheel scrolling over a hovered list, thumb dragging, and PgUp/PgDn/Home/End
/// for the focused list. Positions live in `FrameRegistry::scroll_lists`.
pub fn sync_scroll_list_input(
    mut ui: ResMut<UiState>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    wheel: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let input = collect_input(
        mouse.as_deref(),
        keys.as_deref(),
        wheel.as_deref(),
        &windows,
    );
    if !input.is_idle(&ui) {
        apply_scroll_input(&mut ui, &input);
    }
}

fn collect_input(
    mouse: Option<&ButtonInput<MouseButton>>,
    keys: Option<&ButtonInput<KeyCode>>,
    wheel: Option<&AccumulatedMouseScroll>,
    windows: &Query<&Window, With<bevy::window::PrimaryWindow>>,
) -> ScrollInput {
    let cursor = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .map(|pos| (pos.x, pos.y));
    let (wheel_rows, wheel_pixels) = match wheel {
        Some(w) if w.unit == MouseScrollUnit::Line => (-w.delta.y, 0.0),
        Some(w) => (0.0, -w.delta.y),
        None => (0.0, 0.0),
    };
    let scroll_keys = [
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Home,
        KeyCode::End,
    ];
    ScrollInput {
        cursor,
        pressed: mouse.is_some_and(|m| m.just_pressed(MouseButton::Left)),
        released: mouse.is_some_and(|m| m.just_released(MouseButton::Left)),
        wheel_rows,
        wheel_pixels,
        keys: keys.map_or_else(Vec::new, |k| {
            scroll_keys
                .into_iter()
                .filter(|key| k.just_pressed(*key))
                .collect()
        }),
    }
}

pub fn apply_scroll_input(ui: &mut UiState, input: &ScrollInput) {
    let hovered = input
        .cursor
        .and_then(|(x, y)| hovered_list(&ui.registry, x, y));
    if input.released {
        end_drags(&mut ui.registry);
    }
    if input.pressed {
        press(ui, input.cursor, hovered.as_ref());
    }
    if let Some((_, y)) = input.cursor {
        drag_thumbs(&mut ui.registry, y);
    }
    if let Some((_, name)) = &hovered {
        wheel(&mut ui.registry, name, input);
    }
    if let Some(name) = focused_list(ui) {
        for key in &input.keys {
            apply_key(&mut ui.registry, &name, *key);
        }
    }
}

/// Topmost scroll list under the cursor: the hit frame or its nearest list ancestor.
fn hovered_list(registry: &FrameRegistry, x: f32, y: f32) -> Option<(u64, String)> {
    let mut id = find_frame_at(registry, x, y)?;
    loop {
        let frame = registry.get(id)?;
        if let Some(name) = &frame.name
            && registry.scroll_lists.get(name).is_some()
        {
            return Some((id, name.clone()));
        }
        id = frame.parent_id?;
    }
}

fn focused_list(ui: &UiState) -> Option<String> {
    let frame = ui.registry.get(ui.focused_frame?)?;
    let name = frame.name.as_ref()?;
    ui.registry.scroll_lists.get(name).map(|_| name.clone())
}

fn press(ui: &mut UiState, cursor: Option<(f32, f32)>, hovered: Option<&(u64, String)>) {
    let Some((list_id, name)) = hovered else {
        if focused_list(ui).is_some() {
            ui.focused_frame = None;
        }
        return;
    };
    ui.focused_frame = Some(*list_id);
    let Some((x, y)) = cursor else { return };
    let thumb = ui.registry.get_by_name(&thumb_name(name));
    if thumb.is_some() && thumb == find_frame_at(&ui.registry, x, y) {
        let thumb_y = thumb
            .and_then(|id| ui.registry.get(id))
            .and_then(|frame| frame.layout_rect.as_ref())
            .map_or(y, |rect| rect.y);
        ui.registry.scroll_lists.set_drag(name, Some(y - thumb_y));
    }
}

fn end_drags(registry: &mut FrameRegistry) {
    let names: Vec<String> = registry
        .scroll_lists
        .dragging()
        .map(|(name, _)| name.to_string())
        .collect();
    for name in names {
        registry.scroll_lists.set_drag(&name, None);
    }
}

fn drag_thumbs(registry: &mut FrameRegistry, cursor_y: f32) {
    let targets: Vec<(String, usize)> = registry
        .scroll_lists
        .dragging()
        .filter_map(|(name, state)| {
            let track = registry.get(registry.get_by_name(&track_name(name))?)?;
            let track_y = track.layout_rect.as_ref()?.y;
            let thumb_top = cursor_y - state.drag_grab? - track_y;
            Some((name.to_string(), state.geometry.row_at_thumb_top(thumb_top)))
        })
        .collect();
    for (name, row) in targets {
        registry.scroll_lists.scroll_to(&name, row);
    }
}

fn wheel(registry: &mut FrameRegistry, name: &str, input: &ScrollInput) {
    let Some(state) = registry.scroll_lists.get(name) else {
        return;
    };
    let row_height = state.geometry.row_height;
    let pixel_rows = if row_height > 0.0 {
        input.wheel_pixels / row_height
    } else {
        0.0
    };
    let rows = (input.wheel_rows + pixel_rows).round() as isize;
    if rows != 0 {
        registry.scroll_lists.scroll_by(name, rows);
    }
}

fn apply_key(registry: &mut FrameRegistry, name: &str, key: KeyCode) {
    let Some(state) = registry.scroll_lists.get(name) else {
        return;
    };
    let page = state.geometry.visible_rows.max(1) as isize;
    match key {
        KeyCode::PageUp => registry.scroll_lists.scroll_by(name, -page),
        KeyCode::PageDown => registry.scroll_lists.scroll_by(name, page),
        KeyCode::Home => registry.scroll_lists.scroll_to(name, 0),
        KeyCode::End => registry.scroll_lists.scroll_to(name, usize::MAX),
        _ => false,
    };
}
