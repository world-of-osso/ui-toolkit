pub mod support;

use bevy::prelude::*;
use bevy::text::{FontWeight, Justify, LineBreak};
use support::{create_frame, logical_rect, native_app, settle, text_entity};
use ui_toolkit::frame::{Dimension, WidgetData};
use ui_toolkit::native_render::RegistryText;
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::font_string::{FontStringData, GameFont, JustifyH};

fn fixture() -> (App, Entity, u64) {
    let (mut app, _) = native_app();
    let id = create_frame(&mut app, "RegressionShadow", 180.0, 40.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .widget_data = Some(WidgetData::FontString(FontStringData {
        text: "Before".into(),
        shadow_color: Some([0.1, 0.2, 0.3, 0.7]),
        shadow_offset: [2.0, 3.0],
        ..default()
    }));
    app.init_resource::<Observed>();
    app.add_systems(Last, observe);
    settle(&mut app);
    let entity = text_entity(app.world_mut(), id, 1);
    (app, entity, id)
}

#[derive(Resource, Default)]
struct Observed(bool);

fn observe(
    texts: Query<(
        &RegistryText,
        Ref<Text>,
        Ref<TextFont>,
        Ref<TextLayout>,
        Ref<TextColor>,
    )>,
    mut observed: ResMut<Observed>,
) {
    observed.0 = texts.iter().any(|(part, text, font, layout, color)| {
        part.key == 1
            && (text.is_changed() || font.is_changed() || layout.is_changed() || color.is_changed())
    });
}

#[test]
fn unchanged_shadow_components_keep_change_ticks_clean() {
    let (mut app, shadow, id) = fixture();
    let foreground = text_entity(app.world_mut(), id, 0);
    let before = logical_rect(app.world(), shadow);
    app.update();
    assert!(!app.world().resource::<Observed>().0);
    assert_eq!(logical_rect(app.world(), shadow), before);
    assert_eq!(
        before.min - logical_rect(app.world(), foreground).min,
        Vec2::new(2.0, 3.0)
    );
    let shadow_bounds = app.world().get::<ChildOf>(shadow).unwrap().parent();
    let foreground_bounds = app.world().get::<ChildOf>(foreground).unwrap().parent();
    assert!(
        app.world().get::<GlobalZIndex>(shadow_bounds).unwrap().0
            < app
                .world()
                .get::<GlobalZIndex>(foreground_bounds)
                .unwrap()
                .0
    );
}

#[test]
fn source_updates_change_owned_shadow_components() {
    let (mut app, entity, id) = fixture();
    app.world_mut().get_mut::<TextFont>(entity).unwrap().weight = FontWeight::BOLD;
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(250.0);
        frame.height = Dimension::Fixed(70.0);
        let Some(WidgetData::FontString(data)) = &mut frame.widget_data else {
            panic!("fontstring")
        };
        data.text = "After".into();
        data.font = GameFont::ArialNarrow;
        data.font_size = 24.0;
        data.justify_h = JustifyH::Right;
        data.shadow_color = Some([0.2, 0.4, 0.6, 0.8]);
        data.shadow_offset = [4.0, 5.0];
    }
    settle(&mut app);
    assert_eq!(text_entity(app.world_mut(), id, 1), entity);
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "After");
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().justify,
        Justify::Right
    );
    assert_eq!(
        app.world().get::<TextFont>(entity).unwrap().font_size,
        FontSize::Px(24.0)
    );
    assert_eq!(
        app.world().get::<TextFont>(entity).unwrap().weight,
        FontWeight::BOLD
    );
    assert_eq!(
        app.world().get::<TextColor>(entity).unwrap().0,
        Color::srgba(0.2, 0.4, 0.6, 0.8)
    );
    let foreground = text_entity(app.world_mut(), id, 0);
    assert_eq!(
        logical_rect(app.world(), entity).min - logical_rect(app.world(), foreground).min,
        Vec2::new(4.0, 5.0)
    );
    let bounds = app.world().get::<ChildOf>(entity).unwrap().parent();
    assert_eq!(
        logical_rect(app.world(), bounds).size(),
        Vec2::new(250.0, 70.0)
    );
}

#[test]
fn external_component_edits_are_repaired_without_resetting_unowned_font_fields() {
    let (mut app, entity, _) = fixture();
    let expected_font = app.world().get::<TextFont>(entity).unwrap().clone();
    let expected_layout = *app.world().get::<TextLayout>(entity).unwrap();
    let expected_color = *app.world().get::<TextColor>(entity).unwrap();
    let expected_rect = logical_rect(app.world(), entity);
    app.world_mut().entity_mut(entity).insert((
        Text::new("External"),
        TextLayout::new(Justify::Right, LineBreak::NoWrap),
        TextColor(Color::BLACK),
    ));
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
    let font = app.world().get::<TextFont>(entity).unwrap();
    assert_eq!(font.font, expected_font.font);
    assert_eq!(font.font_size, expected_font.font_size);
    assert_eq!(font.weight, FontWeight::BOLD);
    assert_eq!(
        logical_rect(app.world(), entity).size(),
        expected_rect.size()
    );
}
