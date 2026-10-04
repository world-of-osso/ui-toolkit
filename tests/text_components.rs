pub mod support;

use bevy::prelude::*;
use bevy::text::{FontWeight, Justify, LineBreak};
use support::{create_frame, logical_rect, native_app, settle, text_entity};
use ui_toolkit::frame::{Dimension, WidgetData};
use ui_toolkit::native_render::RegistryText;
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::button::ButtonData;
use ui_toolkit::widgets::edit_box::EditBoxData;
use ui_toolkit::widgets::font_string::{FontStringData, GameFont, JustifyH};

fn fixture(widget: WidgetData) -> (App, Entity, u64) {
    let (mut app, _) = native_app();
    let id = create_frame(&mut app, "RegressionText", 180.0, 40.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .widget_data = Some(widget);
    app.init_resource::<ObservedTicks>();
    app.add_systems(Last, observe_ticks);
    settle(&mut app);
    let entity = text_entity(app.world_mut(), id, 0);
    (app, entity, id)
}

#[derive(Resource, Default)]
struct ObservedTicks([bool; 4]);

fn observe_ticks(
    texts: Query<(Ref<Text>, Ref<TextLayout>, Ref<TextFont>, Ref<TextColor>), With<RegistryText>>,
    mut observed: ResMut<ObservedTicks>,
) {
    observed.0 = texts
        .iter()
        .fold([false; 4], |old, (text, layout, font, color)| {
            [
                old[0] || text.is_changed(),
                old[1] || layout.is_changed(),
                old[2] || font.is_changed(),
                old[3] || color.is_changed(),
            ]
        });
}

fn replace_source_text(app: &mut App, id: u64, text: &str) {
    let mut state = app.world_mut().resource_mut::<UiState>();
    let widget = state
        .registry
        .get_mut(id)
        .unwrap()
        .widget_data
        .as_mut()
        .unwrap();
    let content = match widget {
        WidgetData::FontString(data) => &mut data.text,
        WidgetData::Button(data) => &mut data.text,
        WidgetData::EditBox(data) => &mut data.text,
        _ => panic!("text fixture"),
    };
    *content = text.to_owned();
}

fn assert_content_and_external_repair(app: &mut App, entity: Entity, expected: &str) {
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, expected);
    app.world_mut().get_mut::<Text>(entity).unwrap().0 = "External overwrite".into();
    settle(app);
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, expected);
    assert_eq!(app.world().resource::<ObservedTicks>().0, [false; 4]);
}

#[test]
fn unicode_content_survives_spawn_updates_and_external_repair() {
    let original = "Élan 世界 🦊";
    for widget in [
        WidgetData::FontString(FontStringData {
            text: original.into(),
            ..default()
        }),
        WidgetData::Button(ButtonData {
            text: original.into(),
            ..default()
        }),
        WidgetData::EditBox(EditBoxData {
            text: original.into(),
            ..default()
        }),
    ] {
        let (mut app, entity, id) = fixture(widget);
        assert_content_and_external_repair(&mut app, entity, original);
        replace_source_text(&mut app, id, "Après 🌙 — 秘密");
        settle(&mut app);
        assert_content_and_external_repair(&mut app, entity, "Après 🌙 — 秘密");
    }
}

#[test]
fn password_content_preserves_byte_count_through_mode_and_text_changes() {
    let (mut app, entity, id) = fixture(WidgetData::EditBox(EditBoxData {
        text: "é猫🦊".into(),
        password: true,
        ..default()
    }));
    // Existing contract is UTF-8 byte count, not grapheme or scalar count.
    assert_content_and_external_repair(&mut app, entity, "*********");
    for (password, text, expected) in [
        (false, "é猫🦊", "é猫🦊"),
        (true, "é猫🦊", "*********"),
        (true, "aé", "***"),
        (true, "", ""),
        (true, "猫", "***"),
        (false, "猫", "猫"),
    ] {
        replace_source_text(&mut app, id, text);
        {
            let mut ui = app.world_mut().resource_mut::<UiState>();
            let Some(WidgetData::EditBox(data)) = &mut ui.registry.get_mut(id).unwrap().widget_data
            else {
                panic!("editbox")
            };
            data.password = password;
        }
        settle(&mut app);
        let entity = text_entity(app.world_mut(), id, 0);
        assert_content_and_external_repair(&mut app, entity, expected);
    }
}

#[test]
fn unchanged_text_components_keep_change_ticks_clean() {
    let (mut app, entity, _) = fixture(WidgetData::FontString(FontStringData {
        text: "Before".into(),
        ..default()
    }));
    let before = logical_rect(app.world(), entity);
    app.update();
    assert_eq!(app.world().resource::<ObservedTicks>().0, [false; 4]);
    assert_eq!(logical_rect(app.world(), entity), before);
    assert!(
        before.width() > 0.0 && before.height() > 0.0,
        "real font shaping and layout"
    );
}

#[test]
fn source_updates_change_owned_text_components() {
    let (mut app, entity, id) = fixture(WidgetData::FontString(FontStringData {
        text: "Before".into(),
        ..default()
    }));
    let before = logical_rect(app.world(), entity);
    app.world_mut().get_mut::<TextFont>(entity).unwrap().weight = FontWeight::BOLD;
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(250.0);
        frame.height = Dimension::Fixed(70.0);
        let Some(WidgetData::FontString(data)) = &mut frame.widget_data else {
            panic!("fontstring")
        };
        data.text = "After".into();
        data.font = GameFont::ArialNarrow;
        data.font_size = 24.0;
        data.justify_h = JustifyH::Right;
        data.color = [0.2, 0.4, 0.6, 0.8];
    }
    settle(&mut app);
    assert_eq!(text_entity(app.world_mut(), id, 0), entity);
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "After");
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().justify,
        Justify::Right
    );
    let font = app.world().get::<TextFont>(entity).unwrap();
    assert_eq!(font.font_size, FontSize::Px(24.0));
    assert_eq!(font.weight, FontWeight::BOLD);
    assert_eq!(
        app.world().get::<TextColor>(entity).unwrap().0,
        Color::srgba(0.2, 0.4, 0.6, 0.8)
    );
    let bounds = app.world().get::<ChildOf>(entity).unwrap().parent();
    assert_eq!(
        logical_rect(app.world(), bounds).size(),
        Vec2::new(250.0, 70.0)
    );
    assert!(logical_rect(app.world(), entity).height() > before.height());
}

#[test]
fn external_component_edits_are_repaired_without_resetting_unowned_font_fields() {
    let (mut app, entity, _) = fixture(WidgetData::FontString(FontStringData {
        text: "Before".into(),
        ..default()
    }));
    let expected_font = app.world().get::<TextFont>(entity).unwrap().clone();
    let expected_layout = *app.world().get::<TextLayout>(entity).unwrap();
    let expected_color = *app.world().get::<TextColor>(entity).unwrap();
    let bounds = app.world().get::<ChildOf>(entity).unwrap().parent();
    let expected_node = app.world().get::<Node>(bounds).unwrap().clone();
    app.world_mut().entity_mut(entity).insert((
        Text::new("External"),
        TextLayout::new(Justify::Right, LineBreak::NoWrap),
        TextColor(Color::BLACK),
    ));
    app.world_mut().entity_mut(bounds).insert(Node {
        width: px(1),
        height: px(1),
        ..default()
    });
    {
        let mut font = app.world_mut().get_mut::<TextFont>(entity).unwrap();
        font.font = FontSource::Handle(Handle::default());
        font.font_size = FontSize::Px(1.0);
        font.weight = FontWeight::BOLD;
    }
    settle(&mut app);
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Before");
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().justify,
        expected_layout.justify
    );
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().linebreak,
        expected_layout.linebreak
    );
    assert_eq!(
        *app.world().get::<TextColor>(entity).unwrap(),
        expected_color
    );
    assert_eq!(*app.world().get::<Node>(bounds).unwrap(), expected_node);
    let font = app.world().get::<TextFont>(entity).unwrap();
    assert_eq!(font.font, expected_font.font);
    assert_eq!(font.font_size, expected_font.font_size);
    assert_eq!(font.weight, FontWeight::BOLD);
}
