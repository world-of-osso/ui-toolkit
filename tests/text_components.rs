use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::{FontWeight, Justify, LineBreak, TextBounds};
use ui_toolkit::event::EventBus;
use ui_toolkit::font_registry::FontRegistry;
use ui_toolkit::frame::{Dimension, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render::UiText;
use ui_toolkit::render_text::sync_ui_text;
use ui_toolkit::widgets::font_string::{FontStringData, GameFont, JustifyH};

fn fixture() -> (App, Entity, u64) {
    let mut app = App::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("RegressionText", None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(180.0);
    frame.height = Dimension::Fixed(40.0);
    frame.widget_data = Some(WidgetData::FontString(FontStringData {
        text: "Before".into(),
        ..default()
    }));
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Assets<Font>>();
    app.init_resource::<FontRegistry>();
    app.add_systems(Update, sync_ui_text);
    app.update();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<UiText>>()
        .single(app.world())
        .unwrap();
    (app, entity, id)
}

fn changed<T: Component>(app: &mut App) -> bool {
    app.world_mut()
        .query_filtered::<Entity, (With<UiText>, Changed<T>)>()
        .iter(app.world())
        .next()
        .is_some()
}

fn ticks(app: &mut App) -> [bool; 7] {
    [
        changed::<Text2d>(app),
        changed::<TextLayout>(app),
        changed::<TextBounds>(app),
        changed::<TextFont>(app),
        changed::<TextColor>(app),
        changed::<Transform>(app),
        changed::<Anchor>(app),
    ]
}

#[test]
fn unchanged_text_components_keep_change_ticks_clean() {
    let (mut app, _, _) = fixture();
    app.world_mut().clear_trackers();
    app.update();
    assert_eq!(
        ticks(&mut app),
        [false; 7],
        "Text2d/layout/bounds/font/color/Transform/Anchor"
    );
}

#[test]
fn source_updates_change_owned_text_components() {
    let (mut app, entity, id) = fixture();
    app.world_mut().get_mut::<TextFont>(entity).unwrap().weight = FontWeight::BOLD;
    app.world_mut().clear_trackers();
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(250.0);
        frame.height = Dimension::Fixed(70.0);
        let Some(WidgetData::FontString(data)) = &mut frame.widget_data else {
            panic!("fontstring fixture")
        };
        data.text = "After".into();
        data.font = GameFont::ArialNarrow;
        data.font_size = 24.0;
        data.justify_h = JustifyH::Right;
        data.color = [0.2, 0.4, 0.6, 0.8];
    }
    app.update();
    assert_eq!(app.world().get::<Text2d>(entity).unwrap().0, "After");
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().justify,
        Justify::Right
    );
    assert_eq!(
        app.world().get::<TextBounds>(entity).unwrap().width,
        Some(250.0)
    );
    let font = app.world().get::<TextFont>(entity).unwrap();
    assert_eq!(font.font_size, FontSize::Px(24.0));
    assert_eq!(font.weight, FontWeight::BOLD);
    assert_eq!(
        app.world().get::<TextColor>(entity).unwrap().0,
        Color::srgba(0.2, 0.4, 0.6, 0.8)
    );
    assert_eq!(ticks(&mut app), [true, true, true, true, true, true, false]);
}

#[test]
fn external_component_edits_are_repaired_without_resetting_unowned_font_fields() {
    let (mut app, entity, _) = fixture();
    let expected_font = app.world().get::<TextFont>(entity).unwrap().clone();
    let expected_layout = *app.world().get::<TextLayout>(entity).unwrap();
    let expected_bounds = *app.world().get::<TextBounds>(entity).unwrap();
    let expected_color = *app.world().get::<TextColor>(entity).unwrap();
    let expected_transform = *app.world().get::<Transform>(entity).unwrap();
    app.world_mut().entity_mut(entity).insert((
        Text2d::new("External"),
        TextLayout::new(Justify::Right, LineBreak::NoWrap),
        TextBounds {
            width: Some(1.0),
            height: Some(1.0),
        },
        TextColor(Color::BLACK),
        Transform::from_xyz(1.0, 2.0, 3.0),
        Anchor::BOTTOM_RIGHT,
    ));
    {
        let mut font = app.world_mut().get_mut::<TextFont>(entity).unwrap();
        font.font = FontSource::Handle(Handle::default());
        font.font_size = FontSize::Px(1.0);
        font.weight = FontWeight::BOLD;
    }
    app.world_mut().clear_trackers();
    app.update();
    assert_eq!(app.world().get::<Text2d>(entity).unwrap().0, "Before");
    let layout = app.world().get::<TextLayout>(entity).unwrap();
    assert_eq!(layout.justify, expected_layout.justify);
    assert_eq!(layout.linebreak, expected_layout.linebreak);
    let bounds = app.world().get::<TextBounds>(entity).unwrap();
    assert_eq!(bounds.width, expected_bounds.width);
    assert_eq!(bounds.height, expected_bounds.height);
    assert_eq!(
        *app.world().get::<TextColor>(entity).unwrap(),
        expected_color
    );
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        expected_transform
    );
    assert_eq!(
        *app.world().get::<Anchor>(entity).unwrap(),
        Anchor::TOP_LEFT
    );
    let font = app.world().get::<TextFont>(entity).unwrap();
    assert_eq!(font.font, expected_font.font);
    assert_eq!(font.font_size, expected_font.font_size);
    assert_eq!(font.weight, FontWeight::BOLD);
    assert_eq!(ticks(&mut app), [true; 7]);
}
