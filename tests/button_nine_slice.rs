pub mod support;

use bevy::prelude::*;
use support::*;
use ui_toolkit::frame::{Dimension, NineSlice, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::button::{ButtonData, ButtonState};
use ui_toolkit::widgets::texture::{DynamicTextureId, TextureSource};

struct Fixture {
    app: App,
    button: u64,
    other: u64,
    textures: [DynamicTextureId; 4],
}

fn fixture() -> Fixture {
    let (mut app, _) = native_app();
    let textures = std::array::from_fn(|_| add_texture(&mut app));
    let button = create_frame(&mut app, "Button", 120.0, 40.0);
    let other = create_frame(&mut app, "Other", 120.0, 60.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(button).unwrap().mouse_enabled = true;
        ui.registry.get_mut(button).unwrap().widget_data = Some(WidgetData::Button(ButtonData {
            normal_texture: Some(TextureSource::Dynamic(textures[0])),
            pushed_texture: Some(TextureSource::Dynamic(textures[1])),
            disabled_texture: Some(TextureSource::Dynamic(textures[2])),
            highlight_texture: Some(TextureSource::Dynamic(textures[3])),
            ..default()
        }));
        ui.registry.get_mut(other).unwrap().nine_slice = Some(NineSlice {
            edge_size: 13.0,
            ..default()
        });
        ui.registry.set_pos(other, 200.0, 100.0).unwrap();
    }
    settle(&mut app);
    Fixture {
        app,
        button,
        other,
        textures,
    }
}

fn set_hover_cursor(app: &mut App, hovered: bool) {
    let world = app.world_mut();
    let mut windows = world.query_filtered::<&mut Window, With<bevy::window::PrimaryWindow>>();
    windows
        .single_mut(world)
        .unwrap()
        .set_cursor_position(hovered.then_some(Vec2::new(20.0, 20.0)));
}

#[derive(Resource, Default)]
struct Observed(bool);
fn observe(ui: Res<UiState>, mut observed: ResMut<Observed>) {
    observed.0 = ui.is_changed();
}

#[test]
fn settled_buttons_leave_render_dirty_and_resource_ticks_clean() {
    let mut f = fixture();
    f.app.init_resource::<Observed>();
    f.app.add_systems(Last, observe);
    f.app.update();
    f.app.update();
    assert!(!f.app.world().resource::<Observed>().0);
    assert!(!f.app.world().resource::<ImageChanges>().0);
    assert!(
        f.app
            .world()
            .resource::<UiState>()
            .registry
            .render_dirty
            .is_empty()
    );
    let center = image_covering(f.app.world_mut(), f.button, Vec2::new(60.0, 20.0));
    assert_texture(
        f.app.world(),
        &f.app.world().get::<ImageNode>(center).unwrap().image,
        f.textures[0],
    );
}

#[test]
fn button_states_hover_and_resize_update_visible_nine_slice() {
    let mut f = fixture();
    let center = image_covering(f.app.world_mut(), f.button, Vec2::new(60.0, 20.0));
    let other = image_entities(f.app.world_mut(), f.other);
    let other_rects: Vec<_> = other
        .iter()
        .map(|entity| logical_rect(f.app.world(), *entity))
        .collect();
    for (state, hovered, texture) in [
        (ButtonState::Pushed, false, f.textures[1]),
        (ButtonState::Disabled, false, f.textures[2]),
        (ButtonState::Normal, true, f.textures[3]),
        (ButtonState::Normal, false, f.textures[0]),
    ] {
        set_hover_cursor(&mut f.app, hovered);
        {
            let mut ui = f.app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::Button(data)) =
                &mut ui.registry.get_mut(f.button).unwrap().widget_data
            else {
                panic!("button")
            };
            data.state = state;
            data.hovered = hovered;
        }
        settle(&mut f.app);
        assert_eq!(
            image_covering(f.app.world_mut(), f.button, Vec2::new(60.0, 20.0)),
            center
        );
        for entity in image_entities(f.app.world_mut(), f.button) {
            assert_texture(
                f.app.world(),
                &f.app.world().get::<ImageNode>(entity).unwrap().image,
                texture,
            );
        }
    }
    {
        let mut ui = f.app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(f.button).unwrap();
        frame.width = Dimension::Fixed(240.0);
        frame.height = Dimension::Fixed(80.0);
    }
    settle(&mut f.app);
    assert_eq!(
        image_covering(f.app.world_mut(), f.button, Vec2::new(120.0, 40.0)),
        center
    );
    assert_eq!(
        logical_rect(f.app.world(), center),
        Rect::new(4.0, 4.0, 236.0, 76.0)
    );
    assert_eq!(image_entities(f.app.world_mut(), f.other), other);
    assert_eq!(
        other
            .iter()
            .map(|entity| logical_rect(f.app.world(), *entity))
            .collect::<Vec<_>>(),
        other_rects
    );
}

#[test]
fn external_generated_nine_slice_edits_are_replaced_and_nonbuttons_untouched() {
    let mut f = fixture();
    let original = image_entities(f.app.world_mut(), f.button);
    let before: Vec<_> = original
        .iter()
        .map(|entity| logical_rect(f.app.world(), *entity))
        .collect();
    let other = image_entities(f.app.world_mut(), f.other);
    let other_before: Vec<_> = other
        .iter()
        .map(|entity| logical_rect(f.app.world(), *entity))
        .collect();
    {
        let mut ui = f.app.world_mut().resource_mut::<UiState>();
        let nine = ui
            .registry
            .get_mut(f.button)
            .unwrap()
            .nine_slice
            .as_mut()
            .unwrap();
        nine.edge_size = 99.0;
        nine.bg_color = [0.2; 4];
        nine.part_textures = Some(std::array::from_fn(|_| TextureSource::None));
        nine.uv_rects = Some([[0.3; 4]; 9]);
    }
    settle(&mut f.app);
    assert_eq!(image_entities(f.app.world_mut(), f.button), original);
    for (entity, rect) in original.into_iter().zip(before) {
        assert_eq!(logical_rect(f.app.world(), entity), rect);
        let image = f.app.world().get::<ImageNode>(entity).unwrap();
        assert_texture(f.app.world(), &image.image, f.textures[0]);
        assert_eq!(image.color, Color::srgba(1.0, 1.0, 1.0, 1.0));
    }
    assert_eq!(image_entities(f.app.world_mut(), f.other), other);
    assert_eq!(
        other
            .iter()
            .map(|entity| logical_rect(f.app.world(), *entity))
            .collect::<Vec<_>>(),
        other_before
    );
}

#[test]
fn default_skin_states_keep_authored_atlas_and_proportional_edges() {
    let (mut app, _) = native_app();
    let id = create_frame(&mut app, "DefaultSkin", 120.0, 40.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .widget_data = Some(WidgetData::Button(ButtonData::default()));
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .mouse_enabled = true;
    settle(&mut app);
    for (state, hovered, atlas) in [
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
        set_hover_cursor(&mut app, hovered);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::Button(data)) = &mut ui.registry.get_mut(id).unwrap().widget_data
            else {
                panic!("button")
            };
            data.state = state;
            data.hovered = hovered;
        }
        settle(&mut app);
        let ui = app.world().resource::<UiState>();
        assert_eq!(
            ui.registry
                .get(id)
                .unwrap()
                .nine_slice
                .as_ref()
                .unwrap()
                .texture,
            Some(TextureSource::Atlas(atlas.into()))
        );
    }
    let before = app
        .world()
        .resource::<UiState>()
        .registry
        .get(id)
        .unwrap()
        .nine_slice
        .as_ref()
        .unwrap()
        .edge_sizes
        .unwrap();
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(240.0);
        frame.height = Dimension::Fixed(80.0);
    }
    settle(&mut app);
    assert_eq!(
        app.world()
            .resource::<UiState>()
            .registry
            .get(id)
            .unwrap()
            .nine_slice
            .as_ref()
            .unwrap()
            .edge_sizes
            .unwrap(),
        before.map(|edge| edge * 2.0)
    );
}
