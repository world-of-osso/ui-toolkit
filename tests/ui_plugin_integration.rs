use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::sprite::Anchor as TextAnchor;
use bevy::text::TextBounds;
use bevy::window::PrimaryWindow;
use ui_toolkit::anchor::{Anchor, AnchorPoint};
use ui_toolkit::button_input::sync_button_input;
use ui_toolkit::frame::{Dimension, NineSlice, WidgetData};
use ui_toolkit::plugin::{UiPlugin, UiState};
use ui_toolkit::render::UiText;
use ui_toolkit::render_nine_slice::UiNineSlicePart;
use ui_toolkit::widgets::button::ButtonData;
use ui_toolkit::widgets::font_string::FontStringData;
use ui_toolkit::widgets::texture::TextureSource;

#[derive(Resource, Default)]
struct Observed {
    ui_changed: bool,
    sprite_changes: usize,
    text_changes: usize,
}

fn observe(
    ui: Res<UiState>,
    sprites: Query<(Ref<Transform>, Ref<Sprite>), With<UiNineSlicePart>>,
    texts: Query<
        (
            Ref<Text2d>,
            Ref<TextLayout>,
            Ref<TextBounds>,
            Ref<TextFont>,
            Ref<TextColor>,
            Ref<Transform>,
            Ref<TextAnchor>,
        ),
        With<UiText>,
    >,
    mut observed: ResMut<Observed>,
) {
    observed.ui_changed = ui.is_changed();
    observed.sprite_changes = sprites
        .iter()
        .filter(|(transform, sprite)| transform.is_changed() || sprite.is_changed())
        .count();
    observed.text_changes = texts
        .iter()
        .filter(|(text, layout, bounds, font, color, transform, anchor)| {
            text.is_changed()
                || layout.is_changed()
                || bounds.is_changed()
                || font.is_changed()
                || color.is_changed()
                || transform.is_changed()
                || anchor.is_changed()
        })
        .count();
}

struct Fixture {
    app: App,
    window: Entity,
    button: u64,
    panel: u64,
    normal: Handle<Image>,
    hover: Handle<Image>,
}

fn place(ui: &mut UiState, id: u64, x: f32, y: f32) {
    ui.registry
        .set_point(
            id,
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_point: AnchorPoint::TopLeft,
                x_offset: x,
                y_offset: y,
            },
        )
        .unwrap();
}

fn fixture() -> Fixture {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>();
    app.init_asset::<Font>();
    app.add_plugins(UiPlugin);
    app.init_resource::<Observed>();
    app.add_systems(Update, observe.after(sync_button_input));
    let window = app
        .world_mut()
        .spawn((
            Window {
                resolution: (800, 600).into(),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let normal = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let hover = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
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
    for _ in 0..4 {
        app.update();
    }
    assert_eq!(
        app.world_mut()
            .query::<&UiNineSlicePart>()
            .iter(app.world())
            .count(),
        18
    );
    assert_eq!(
        app.world_mut().query::<&UiText>().iter(app.world()).count(),
        1
    );
    Fixture {
        app,
        window,
        button,
        panel,
        normal,
        hover,
    }
}

fn center(app: &mut App, frame: u64) -> Entity {
    app.world_mut()
        .query::<(Entity, &UiNineSlicePart)>()
        .iter(app.world())
        .find(|(_, part)| part.0 == frame && part.1 == 4)
        .unwrap()
        .0
}

#[test]
fn real_plugin_settled_components_and_resource_are_clean() {
    let mut fixture = fixture();
    fixture.app.update();
    let observed = fixture.app.world().resource::<Observed>();
    assert_eq!(observed.sprite_changes, 0);
    assert_eq!(observed.text_changes, 0);
    let ui = fixture.app.world().resource::<UiState>();
    assert!(ui.registry.rect_dirty.is_empty());
    assert!(ui.registry.render_dirty.is_empty());
    assert!(
        !observed.ui_changed,
        "actual UiPlugin chain changed settled UiState"
    );
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
    assert_eq!(
        fixture.app.world().get::<Sprite>(entity).unwrap().image,
        fixture.normal
    );
    fixture.app.update();
    assert_eq!(center(&mut fixture.app, fixture.button), entity);
    assert_eq!(
        fixture.app.world().get::<Sprite>(entity).unwrap().image,
        fixture.hover
    );
}

#[test]
fn real_plugin_layout_and_missing_component_repair_preserve_entities() {
    let mut fixture = fixture();
    let entity = center(&mut fixture.app, fixture.panel);
    let before = fixture
        .app
        .world()
        .get::<Sprite>(entity)
        .unwrap()
        .custom_size
        .unwrap();
    {
        let mut ui = fixture.app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(fixture.panel).unwrap().width = Dimension::Fixed(180.0);
        ui.registry.mark_rect_dirty(fixture.panel);
    }
    fixture.app.update();
    let after = fixture
        .app
        .world()
        .get::<Sprite>(entity)
        .unwrap()
        .custom_size
        .unwrap();
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
    let expected = *fixture.app.world().get::<Transform>(entity).unwrap();
    fixture
        .app
        .world_mut()
        .entity_mut(entity)
        .remove::<Transform>();
    fixture.app.update();
    assert_eq!(center(&mut fixture.app, fixture.panel), entity);
    assert_eq!(
        *fixture.app.world().get::<Transform>(entity).unwrap(),
        expected
    );
    fixture
        .app
        .world_mut()
        .entity_mut(entity)
        .remove::<Sprite>();
    fixture.app.update();
    assert_eq!(center(&mut fixture.app, fixture.panel), entity);
    assert_eq!(
        fixture
            .app
            .world()
            .get::<Sprite>(entity)
            .unwrap()
            .custom_size,
        Some(after)
    );
}
