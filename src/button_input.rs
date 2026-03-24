use bevy::input::ButtonInput;
use bevy::prelude::*;

use crate::frame::WidgetData;
use crate::input::{find_frame_at, hit_test};
use crate::plugin::UiState;
use crate::widgets::button::ButtonState;

/// Automatic button hover/pressed state management.
/// Add this system to Update to get button visuals without per-screen code.
pub fn sync_button_input(
    mut ui: ResMut<UiState>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let cursor = cursor_pos(&windows);
    update_hover(&mut ui, cursor);
    if let Some(mouse) = mouse {
        update_pressed(&mut ui, &mouse, cursor);
    }
}

fn cursor_pos(windows: &Query<&Window, With<bevy::window::PrimaryWindow>>) -> Option<(f32, f32)> {
    let window = windows.single().ok()?;
    let pos = window.cursor_position()?;
    Some((pos.x, pos.y))
}

fn update_hover(ui: &mut UiState, cursor: Option<(f32, f32)>) {
    let button_ids: Vec<u64> = ui
        .registry
        .frames_iter()
        .filter(|f| f.visible && matches!(&f.widget_data, Some(WidgetData::Button(_))))
        .map(|f| f.id)
        .collect();
    for id in button_ids {
        let hovered = cursor.is_some_and(|(x, y)| {
            ui.registry
                .get(id)
                .and_then(|f| f.layout_rect.as_ref().map(|r| (r, &f.hit_rect_insets)))
                .is_some_and(|(r, insets)| hit_test(x, y, r, insets))
        });
        if let Some(WidgetData::Button(bd)) = ui
            .registry
            .get_mut(id)
            .and_then(|f| f.widget_data.as_mut())
        {
            bd.hovered = hovered;
        }
    }
}

fn update_pressed(ui: &mut UiState, mouse: &ButtonInput<MouseButton>, cursor: Option<(f32, f32)>) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some((x, y)) = cursor {
            if let Some(id) = find_frame_at(&ui.registry, x, y) {
                set_button_state(&mut ui.registry, id, ButtonState::Pushed);
            }
        }
    }
    if mouse.just_released(MouseButton::Left) {
        // Reset all pushed buttons back to normal
        let pushed: Vec<u64> = ui
            .registry
            .frames_iter()
            .filter(|f| {
                matches!(
                    &f.widget_data,
                    Some(WidgetData::Button(bd)) if bd.state == ButtonState::Pushed
                )
            })
            .map(|f| f.id)
            .collect();
        for id in pushed {
            set_button_state(&mut ui.registry, id, ButtonState::Normal);
        }
    }
}

fn set_button_state(
    registry: &mut crate::registry::FrameRegistry,
    id: u64,
    state: ButtonState,
) {
    if let Some(WidgetData::Button(bd)) = registry.get_mut(id).and_then(|f| f.widget_data.as_mut())
    {
        if bd.state != ButtonState::Disabled {
            bd.state = state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Dimension, WidgetType};
    use crate::layout::LayoutRect;
    use crate::registry::FrameRegistry;
    use crate::widgets::button::ButtonData;

    fn make_button(reg: &mut FrameRegistry, name: &str, x: f32, y: f32) -> u64 {
        let id = reg.create_frame(name, None);
        let f = reg.get_mut(id).unwrap();
        f.widget_type = WidgetType::Button;
        f.width = Dimension::Fixed(100.0);
        f.height = Dimension::Fixed(40.0);
        f.mouse_enabled = true;
        f.widget_data = Some(WidgetData::Button(ButtonData::default()));
        f.layout_rect = Some(LayoutRect { x, y, width: 100.0, height: 40.0 });
        id
    }

    fn get_bd(reg: &FrameRegistry, id: u64) -> &ButtonData {
        match &reg.get(id).unwrap().widget_data {
            Some(WidgetData::Button(bd)) => bd,
            _ => panic!("not a button"),
        }
    }

    #[test]
    fn hover_sets_hovered_flag() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let b1 = make_button(&mut reg, "Btn1", 100.0, 100.0);
        let mut ui = UiState {
            registry: reg,
            event_bus: crate::event::EventBus::new(),
            focused_frame: None,
        };
        update_hover(&mut ui, Some((150.0, 120.0)));
        assert!(get_bd(&ui.registry, b1).hovered);
        update_hover(&mut ui, Some((500.0, 500.0)));
        assert!(!get_bd(&ui.registry, b1).hovered);
    }
}
