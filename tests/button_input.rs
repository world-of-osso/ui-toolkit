use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::BTreeSet;
use ui_toolkit::button_input::sync_button_input;
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::{Dimension, WidgetData, WidgetType};
use ui_toolkit::layout::LayoutRect;
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::widgets::button::{ButtonData, ButtonState};

#[derive(Resource, Default)]
struct Observed {
    changed: bool,
    dirty: BTreeSet<u64>,
}

// Clear only the test's collected invalidation evidence, without fabricating a
// UiState change before the system under test. Production writes remain tracked.
fn clear_evidence(mut ui: ResMut<UiState>) {
    ui.bypass_change_detection().registry.render_dirty.clear();
}

fn observe(ui: Res<UiState>, mut observed: ResMut<Observed>) {
    observed.changed = ui.is_changed();
    observed.dirty = ui.registry.render_dirty.iter().copied().collect();
}

fn fixture(cursor: Option<Vec2>) -> (App, Entity, [u64; 3]) {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let buttons = std::array::from_fn(|index| {
        let id = registry.create_frame(&format!("Button{index}"), None);
        let frame = registry.get_mut(id).unwrap();
        frame.widget_type = WidgetType::Button;
        frame.width = Dimension::Fixed(100.0);
        frame.height = Dimension::Fixed(40.0);
        frame.mouse_enabled = true;
        frame.layout_rect = Some(LayoutRect {
            x: 100.0 + index as f32 * 150.0,
            y: 100.0,
            width: 100.0,
            height: 40.0,
        });
        let mut button = ButtonData::default();
        if index == 2 {
            button.state = ButtonState::Disabled;
        }
        frame.widget_data = Some(WidgetData::Button(button));
        id
    });
    let mut app = App::new();
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    app.init_resource::<ButtonInput<MouseButton>>();
    let mut window = Window::default();
    window.set_cursor_position(cursor);
    let window = app.world_mut().spawn((window, PrimaryWindow)).id();
    app.add_systems(Update, (clear_evidence, sync_button_input, observe).chain());
    app.update();
    app.update();
    (app, window, buttons)
}

fn set_cursor(app: &mut App, window: Entity, position: Option<Vec2>) {
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(position);
}

fn button(app: &App, id: u64) -> &ButtonData {
    let frame = app.world().resource::<UiState>().registry.get(id).unwrap();
    let Some(WidgetData::Button(button)) = &frame.widget_data else {
        panic!("button fixture lost its widget data");
    };
    button
}

fn assert_dirty(app: &App, ids: &[u64]) {
    assert_eq!(
        app.world().resource::<Observed>().dirty,
        ids.iter().copied().collect()
    );
}

#[test]
fn no_cursor_and_stationary_cursor_do_not_mutate_ui() {
    for cursor in [None, Some(Vec2::new(120.0, 120.0))] {
        let (mut app, _, _) = fixture(cursor);
        app.update();
        assert_dirty(&app, &[]);
        assert!(!app.world().resource::<Observed>().changed);
    }
}

#[test]
fn cursor_enter_move_and_leave_dirty_only_changed_hover() {
    let (mut app, window, [first, second, disabled]) = fixture(None);
    set_cursor(&mut app, window, Some(Vec2::new(120.0, 120.0)));
    app.update();
    assert!(button(&app, first).hovered);
    assert!(!button(&app, second).hovered);
    assert_dirty(&app, &[first]);
    set_cursor(&mut app, window, Some(Vec2::new(270.0, 120.0)));
    app.update();
    assert!(!button(&app, first).hovered);
    assert!(button(&app, second).hovered);
    assert_dirty(&app, &[first, second]);
    set_cursor(&mut app, window, None);
    app.update();
    assert!(!button(&app, second).hovered);
    assert!(!button(&app, disabled).hovered);
    assert_dirty(&app, &[second]);
}

#[test]
fn press_then_release_outside_resets_pushed_button() {
    let (mut app, window, [first, _, _]) = fixture(Some(Vec2::new(120.0, 120.0)));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(button(&app, first).state, ButtonState::Pushed);
    assert_dirty(&app, &[first]);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    set_cursor(&mut app, window, None);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    assert_eq!(button(&app, first).state, ButtonState::Normal);
    assert!(!button(&app, first).hovered);
    assert_dirty(&app, &[first]);
}

#[test]
fn disabled_button_hover_is_retained_but_press_does_not_change_state() {
    let (mut app, _, [_, _, disabled]) = fixture(Some(Vec2::new(420.0, 120.0)));
    assert!(button(&app, disabled).hovered);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(button(&app, disabled).state, ButtonState::Disabled);
    assert!(button(&app, disabled).hovered);
    assert_dirty(&app, &[]);
    assert!(!app.world().resource::<Observed>().changed);
}
