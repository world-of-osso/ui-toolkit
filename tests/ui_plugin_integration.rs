pub mod support;
use bevy::math::Affine2;
use bevy::prelude::*;
use ui_toolkit::anchor::AnchorTarget;
use ui_toolkit::frame::{Dimension, NineSlice, WidgetData};
use ui_toolkit::native_render::{RegistryNode, RegistryText};
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::button::ButtonData;
use ui_toolkit::widgets::font_string::FontStringData;
use ui_toolkit::widgets::texture::{DynamicTextureId, TextureSource};
use ui_toolkit_core::layout_values::{PositionType, UiRect, Val};

#[derive(Resource, Default)]
struct Observed {
    ui_changed: bool,
    image_changes: usize,
    text_changes: usize,
}

fn observe(
    ui: Res<UiState>,
    images: Query<(
        Ref<Node>,
        Ref<ImageNode>,
        Ref<UiTransform>,
        Ref<GlobalZIndex>,
    )>,
    texts: Query<(Ref<Text>, Ref<TextLayout>, Ref<TextFont>, Ref<TextColor>), With<RegistryText>>,
    mut observed: ResMut<Observed>,
) {
    observed.ui_changed = ui.is_changed();
    observed.image_changes = images
        .iter()
        .filter(|(node, image, transform, z)| {
            node.is_changed() || image.is_changed() || transform.is_changed() || z.is_changed()
        })
        .count();
    observed.text_changes = texts
        .iter()
        .filter(|(text, layout, font, color)| {
            text.is_changed() || layout.is_changed() || font.is_changed() || color.is_changed()
        })
        .count();
}

fn native_app() -> (App, Entity) {
    let (mut app, window) = support::native_app();
    app.init_resource::<Observed>();
    app.add_systems(Last, observe);
    (app, window)
}

struct Fixture {
    app: App,
    window: Entity,
    button: u64,
    panel: u64,
    normal: DynamicTextureId,
    hover: DynamicTextureId,
}

fn place(ui: &mut UiState, id: u64, x: f32, y: f32) {
    ui.registry.set_anchor(id, AnchorTarget::Parent).unwrap();
    ui.registry
        .set_pos_type(id, PositionType::Absolute)
        .unwrap();
    ui.registry.set_pos(id, x, y).unwrap();
}

fn settle(app: &mut App) {
    for _ in 0..4 {
        app.update();
    }
}

fn fixture() -> Fixture {
    let (mut app, window) = native_app();
    let normal = support::add_texture(&mut app);
    let hover = support::add_texture(&mut app);
    let (button, panel) = {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let button = ui.registry.create_frame("PluginButton", None);
        let frame = ui.registry.get_mut(button).unwrap();
        frame.width = Dimension::Fixed(120.0);
        frame.height = Dimension::Fixed(40.0);
        frame.mouse_enabled = true;
        frame.widget_data = Some(WidgetData::Button(ButtonData {
            normal_texture: Some(TextureSource::Dynamic(normal.clone())),
            highlight_texture: Some(TextureSource::Dynamic(hover.clone())),
            ..default()
        }));
        place(&mut ui, button, 20.0, 20.0);
        let label = ui.registry.create_frame("PluginLabel", None);
        let frame = ui.registry.get_mut(label).unwrap();
        frame.width = Dimension::Fixed(160.0);
        frame.height = Dimension::Fixed(30.0);
        frame.widget_data = Some(WidgetData::FontString(FontStringData {
            text: "Static plugin text".into(),
            ..default()
        }));
        place(&mut ui, label, 200.0, 20.0);
        let panel = ui.registry.create_frame("PluginPanel", None);
        let frame = ui.registry.get_mut(panel).unwrap();
        frame.width = Dimension::Fixed(100.0);
        frame.height = Dimension::Fixed(60.0);
        frame.nine_slice = Some(NineSlice {
            texture: Some(TextureSource::Dynamic(normal.clone())),
            ..default()
        });
        place(&mut ui, panel, 200.0, 100.0);
        (button, panel)
    };
    settle(&mut app);
    let label = app
        .world_mut()
        .query::<(Entity, &RegistryText)>()
        .iter(app.world())
        .find(|(_, text)| text.key == 0)
        .unwrap()
        .0;
    assert_eq!(
        app.world().get::<Text>(label).unwrap().0,
        "Static plugin text"
    );
    assert!(rect(app.world(), label).width() > 0.0);
    Fixture {
        app,
        window,
        button,
        panel,
        normal,
        hover,
    }
}

fn frame_entity(world: &mut World, id: u64) -> Entity {
    world
        .query::<(Entity, &RegistryNode)>()
        .iter(world)
        .find(|(_, frame)| frame.0 == id)
        .unwrap()
        .0
}

fn rect(world: &World, entity: Entity) -> Rect {
    let node = world.get::<ComputedNode>(entity).unwrap();
    let transform = Affine2::from(world.get::<UiGlobalTransform>(entity).unwrap());
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor,
        node.size * node.inverse_scale_factor,
    )
}

fn center(app: &mut App, id: u64) -> Entity {
    let frame = frame_entity(app.world_mut(), id);
    let midpoint = rect(app.world(), frame).center();
    let parts: Vec<_> = app
        .world()
        .get::<Children>(frame)
        .unwrap()
        .iter()
        .filter(|entity| app.world().get::<ImageNode>(*entity).is_some())
        .collect();
    let matches: Vec<_> = parts
        .into_iter()
        .filter(|entity| rect(app.world(), *entity).contains(midpoint))
        .collect();
    assert_eq!(matches.len(), 1, "one slice covers the frame center");
    matches[0]
}

fn resize_fixture() -> Fixture {
    let mut fixture = fixture();
    {
        let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
        let panel = ui.registry.get_mut(fixture.panel).unwrap();
        panel.position = UiRect {
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..UiRect::AUTO
        };
    }
    settle(&mut fixture.app);
    fixture
}

fn set_window_quarter_pixels(fixture: &mut Fixture, width: u32, height: u32) {
    let mut window = fixture
        .app
        .world_mut()
        .get_mut::<Window>(fixture.window)
        .unwrap();
    window.resolution.set_scale_factor_override(Some(4.0));
    window.resolution.set_physical_resolution(width, height);
    assert_eq!(window.width(), width as f32 / 4.0);
    assert_eq!(window.height(), height as f32 / 4.0);
}

fn assert_panel_rect(fixture: &Fixture, width: f32, height: f32, tolerance: f32) {
    let ui = fixture.app.world().resource::<UiState>();
    let actual = ui
        .registry
        .get(fixture.panel)
        .unwrap()
        .layout_rect
        .as_ref()
        .unwrap();
    for (actual, expected) in [
        (actual.x, width - 120.0),
        (actual.y, height - 80.0),
        (actual.width, 100.0),
        (actual.height, 60.0),
    ] {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected}"
        );
    }
}

#[test]
fn real_native_layout_reads_subpixel_resize_without_authored_size_changes() {
    let mut fixture = resize_fixture();
    let entity = center(&mut fixture.app, fixture.panel);
    for (width, height) in [(3201, 2400), (3202, 2400), (3200, 2401), (3200, 2402)] {
        set_window_quarter_pixels(&mut fixture, width, height);
        settle(&mut fixture.app);
        // The registry's window-change threshold is not a second layout solver:
        // Bevy still positions against the actual render-target dimensions.
        assert_panel_rect(&fixture, width as f32 / 4.0, height as f32 / 4.0, 0.25);
        assert_eq!(center(&mut fixture.app, fixture.panel), entity);
        let ui = fixture.app.world().resource::<UiState>();
        let panel = ui.registry.get(fixture.panel).unwrap();
        assert_eq!(
            (panel.width, panel.height),
            (Dimension::Fixed(100.0), Dimension::Fixed(60.0))
        );
        assert_eq!(panel.position.right, Val::Px(20.0));
        assert_eq!(panel.position.bottom, Val::Px(20.0));
        assert!(ui.registry.rect_dirty.is_empty());
    }
}

#[test]
fn real_plugin_resize_above_half_pixel_updates_registry_and_native_layout() {
    for (width, height) in [(3203, 2400), (3200, 2403), (3203, 2401), (3197, 2399)] {
        let mut fixture = resize_fixture();
        let entity = center(&mut fixture.app, fixture.panel);
        let before = rect(fixture.app.world(), entity);
        set_window_quarter_pixels(&mut fixture, width, height);
        settle(&mut fixture.app);
        let (w, h) = (width as f32 / 4.0, height as f32 / 4.0);
        let ui = fixture.app.world().resource::<UiState>();
        assert_eq!(
            (ui.registry.screen_width, ui.registry.screen_height),
            (w, h)
        );
        assert!(ui.registry.rect_dirty.is_empty());
        assert_panel_rect(&fixture, w, h, 0.25);
        assert_eq!(center(&mut fixture.app, fixture.panel), entity);
        let after = rect(fixture.app.world(), entity);
        assert!(
            (after.min - before.min - Vec2::new(w - 800.0, h - 600.0))
                .abs()
                .max_element()
                <= 0.25
        );
    }
}

#[test]
fn real_plugin_settled_components_and_resource_are_clean() {
    let mut fixture = fixture();
    fixture.app.update();
    let observed = fixture.app.world().resource::<Observed>();
    assert_eq!(observed.image_changes, 0);
    assert_eq!(observed.text_changes, 0);
    assert!(
        !observed.ui_changed,
        "native projection changed settled UiState"
    );
    let ui = fixture.app.world().resource::<UiState>();
    assert!(ui.registry.rect_dirty.is_empty());
    assert!(ui.registry.render_dirty.is_empty());
}

#[test]
fn real_plugin_input_last_updates_hover_visuals_on_next_frame() {
    let mut fixture = fixture();
    let entity = center(&mut fixture.app, fixture.button);
    fixture
        .app
        .world_mut()
        .get_mut::<Window>(fixture.window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(40.0, 40.0)));
    fixture.app.update();
    let ui = fixture.app.world().resource::<UiState>();
    let Some(WidgetData::Button(button)) = &ui.registry.get(fixture.button).unwrap().widget_data
    else {
        panic!("button")
    };
    assert!(button.hovered);
    support::assert_texture(
        fixture.app.world(),
        &fixture.app.world().get::<ImageNode>(entity).unwrap().image,
        fixture.normal,
    );
    fixture.app.update();
    assert_eq!(center(&mut fixture.app, fixture.button), entity);
    support::assert_texture(
        fixture.app.world(),
        &fixture.app.world().get::<ImageNode>(entity).unwrap().image,
        fixture.hover,
    );
}

#[test]
fn real_plugin_layout_and_missing_component_repair_preserve_entities() {
    let mut fixture = fixture();
    let entity = center(&mut fixture.app, fixture.panel);
    let before = rect(fixture.app.world(), entity).size();
    {
        let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(fixture.panel).unwrap().width = Dimension::Fixed(180.0);
        ui.registry.mark_rect_dirty(fixture.panel);
    }
    settle(&mut fixture.app);
    let after = rect(fixture.app.world(), entity).size();
    assert_eq!(after.x - before.x, 80.0);
    assert_eq!(after.y, before.y);
    assert_eq!(
        fixture
            .app
            .world()
            .resource::<UiState>()
            .registry
            .get(fixture.panel)
            .unwrap()
            .layout_rect
            .as_ref()
            .unwrap()
            .width,
        180.0
    );
    let expected = *fixture.app.world().get::<UiTransform>(entity).unwrap();
    fixture
        .app
        .world_mut()
        .entity_mut(entity)
        .remove::<UiTransform>();
    settle(&mut fixture.app);
    assert_eq!(center(&mut fixture.app, fixture.panel), entity);
    assert_eq!(
        *fixture.app.world().get::<UiTransform>(entity).unwrap(),
        expected
    );
    fixture
        .app
        .world_mut()
        .entity_mut(entity)
        .remove::<ImageNode>();
    settle(&mut fixture.app);
    assert_eq!(center(&mut fixture.app, fixture.panel), entity);
    assert_eq!(rect(fixture.app.world(), entity).size(), after);
    support::assert_texture(
        fixture.app.world(),
        &fixture.app.world().get::<ImageNode>(entity).unwrap().image,
        fixture.normal,
    );
}
