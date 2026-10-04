use ui_toolkit_core::{
    frame::{Frame, WidgetData, WidgetType},
    layout_values::{PositionType, UiRect, Val, Val2},
    registry::FrameRegistry,
    screen::{Screen, SharedContext},
    widget_def::{Attr, WidgetChild, WidgetDef},
    widgets::texture::{TextureData, TextureSource},
};

#[test]
fn registry_preserves_layout_and_mutation_publication() {
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let parent = registry.create_frame("parent", None);
    let child = registry.create_frame("child", Some(parent));
    registry.render_dirty.clear();
    registry.get_mut(child).unwrap();
    registry.resolve_pending_writes();
    assert!(
        registry.render_dirty.is_empty(),
        "unchanged access must not publish"
    );
    registry
        .set_pos_type(child, PositionType::Absolute)
        .unwrap();
    registry.set_pos(child, 25.0, 40.0).unwrap();
    let frame = registry.get_mut(child).unwrap();
    frame.margin = UiRect::all(Val::Px(4.0));
    frame.translation = Val2::percent(-50.0, -50.0);
    registry.resolve_pending_writes();
    assert!(
        registry.render_dirty.contains(&child),
        "changed frame must publish"
    );
    let frame = registry.get(child).unwrap();
    assert_eq!(
        frame.position,
        UiRect {
            left: Val::Px(25.0),
            top: Val::Px(40.0),
            right: Val::Auto,
            bottom: Val::Auto
        }
    );
    assert_eq!(frame.position_type, PositionType::Absolute);
    assert_eq!(frame.margin, UiRect::all(Val::Px(4.0)));
    assert_eq!(frame.translation, Val2::percent(-50.0, -50.0));
    assert_eq!(registry.get(parent).unwrap().children, vec![child]);
}

#[test]
fn screen_diff_keeps_widget_data_and_reuses_named_frame() {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let mut context = SharedContext::new();
    context.insert("first".to_owned());
    let mut screen = Screen::new(|context| {
        let mut widget = WidgetDef::new("texture");
        widget.name = Some("icon".into());
        widget.attrs.push(Attr::new_dynamic(
            "texture_file",
            context.get::<String>().unwrap().clone(),
        ));
        vec![WidgetChild::Widget(widget)]
    });
    screen.sync(&context, &mut registry);
    let id = registry.get_by_name("icon").unwrap();
    assert_eq!(
        registry.get(id).unwrap().widget_data,
        Some(WidgetData::Texture(TextureData {
            source: TextureSource::File("first".into()),
            ..Default::default()
        }))
    );
    context.insert("second".to_owned());
    screen.sync(&context, &mut registry);
    assert_eq!(registry.get_by_name("icon"), Some(id));
    assert_eq!(
        registry.get(id).unwrap().widget_data,
        Some(WidgetData::Texture(TextureData {
            source: TextureSource::File("second".into()),
            ..Default::default()
        }))
    );
    screen.teardown(&mut registry);
    assert_eq!(registry.get_by_name("icon"), None);
    assert_eq!(
        Frame::new(1, None, WidgetType::Frame).position,
        UiRect::AUTO
    );
}
