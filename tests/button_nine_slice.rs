use bevy::prelude::*;
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::{Dimension, NineSlice, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render_button::sync_button_nine_slices;
use ui_toolkit::widgets::button::{ButtonData, ButtonState};
use ui_toolkit::widgets::texture::TextureSource;

#[derive(Resource, Default)]
struct Observed(bool);

fn observe(state: Res<UiState>, mut observed: ResMut<Observed>) {
    observed.0 = state.is_changed();
}

fn fixture() -> (App, u64, u64) {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let button = registry.create_frame("Button", None);
    let frame = registry.get_mut(button).unwrap();
    frame.width = Dimension::Fixed(120.0);
    frame.height = Dimension::Fixed(40.0);
    frame.widget_data = Some(WidgetData::Button(ButtonData::default()));
    let other = registry.create_frame("Other", None);
    registry.get_mut(other).unwrap().nine_slice = Some(NineSlice {
        edge_size: 13.0,
        ..default()
    });
    let mut app = App::new();
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    app.add_systems(Update, (sync_button_nine_slices, observe).chain());
    app.update();
    settle(&mut app);
    (app, button, other)
}

fn settle(app: &mut App) {
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .render_dirty
        .clear();
    app.world_mut().clear_trackers();
}

fn slice(app: &App, id: u64) -> &NineSlice {
    app.world()
        .resource::<UiState>()
        .registry
        .get(id)
        .unwrap()
        .nine_slice
        .as_ref()
        .unwrap()
}

#[test]
fn settled_buttons_leave_render_dirty_and_resource_ticks_clean() {
    let (mut app, _, _) = fixture();
    app.update();
    let state = app.world().resource::<UiState>();
    assert_eq!(
        (
            state.registry.render_dirty.is_empty(),
            app.world().resource::<Observed>().0
        ),
        (true, false),
        "settled synchronization must preserve render dirtiness and UiState change ticks"
    );
}

#[test]
fn button_states_hover_and_resize_update_nine_slice() {
    let (mut app, button, other) = fixture();
    for (state, hovered, texture) in [
        (
            ButtonState::Pushed,
            false,
            "defaultbutton-nineslice-pressed",
        ),
        (
            ButtonState::Disabled,
            false,
            "defaultbutton-nineslice-disabled",
        ),
        (
            ButtonState::Normal,
            true,
            "defaultbutton-nineslice-highlight",
        ),
        (ButtonState::Normal, false, "defaultbutton-nineslice-up"),
    ] {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let Some(WidgetData::Button(data)) = &mut ui.registry.get_mut(button).unwrap().widget_data
        else {
            panic!("button fixture")
        };
        data.state = state;
        data.hovered = hovered;
        settle(&mut app);
        app.update();
        assert!(
            matches!(&slice(&app, button).texture, Some(TextureSource::Atlas(name)) if name == texture)
        );
        assert_eq!(
            app.world()
                .resource::<UiState>()
                .registry
                .render_dirty
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![button]
        );
        assert!(app.world().resource::<Observed>().0);
    }
    let before = slice(&app, button).edge_sizes.unwrap();
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(button).unwrap();
        frame.width = Dimension::Fixed(240.0);
        frame.height = Dimension::Fixed(80.0);
    }
    settle(&mut app);
    app.update();
    assert_eq!(
        slice(&app, button).edge_sizes.unwrap(),
        before.map(|value| value * 2.0)
    );
    assert_eq!(slice(&app, other).edge_size, 13.0);
}

#[test]
fn external_nine_slice_edits_are_replaced_and_nonbuttons_untouched() {
    let (mut app, button, other) = fixture();
    let expected = slice(&app, button).clone();
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let nine = state
            .registry
            .get_mut(button)
            .unwrap()
            .nine_slice
            .as_mut()
            .unwrap();
        nine.edge_size = 99.0;
        nine.bg_color = [0.2; 4];
        nine.part_textures = Some(std::array::from_fn(|_| TextureSource::None));
        nine.uv_rects = Some([[0.3; 4]; 9]);
    }
    settle(&mut app);
    app.update();
    let repaired = slice(&app, button);
    assert_eq!(repaired.edge_size, expected.edge_size);
    assert_eq!(repaired.bg_color, expected.bg_color);
    assert!(
        matches!(&repaired.texture, Some(TextureSource::Atlas(name)) if name == "defaultbutton-nineslice-up")
    );
    assert!(repaired.part_textures.is_none());
    assert!(repaired.uv_rects.is_none());
    assert_eq!(slice(&app, other).edge_size, 13.0);
    assert_eq!(
        app.world()
            .resource::<UiState>()
            .registry
            .render_dirty
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![button]
    );
    assert!(app.world().resource::<Observed>().0);
}
