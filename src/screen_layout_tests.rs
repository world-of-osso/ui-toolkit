use super::*;
use crate::widget_def::{Attr, WidgetDef};
use bevy::prelude::Val;

fn named_widget(tag: &'static str, name: &str) -> WidgetDef {
    let mut widget = WidgetDef::new(tag);
    widget.name = Some(name.to_string());
    widget
}

fn sizing_screen(tag: &'static str) -> Screen {
    Screen::new(move |_| {
        let mut widget = named_widget(tag, "Sized");
        widget.attrs = vec![
            Attr::new_static("text", "Hi".to_string()),
            Attr::new_static("font_size", "20".to_string()),
            Attr::new_static("pos_x", "12".to_string()),
        ];
        vec![WidgetChild::Widget(widget)]
    })
}

#[test]
fn screen_text_autosize_updates_dimensions_and_preserves_position() {
    let mut screen = sizing_screen("FontString");
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let label = registry.get_by_name("Sized").unwrap();
    let original_width = registry.get(label).unwrap().width.value();
    let frame = registry.get_mut(label).unwrap();
    frame.width = Dimension::Fixed(0.0);
    let Some(WidgetData::FontString(text)) = &mut frame.widget_data else {
        panic!("expected FontString");
    };
    text.text = "A much longer label".to_string();
    registry.create_frame("UnrelatedDirtyFrame", None);
    screen.sync(&ctx, &mut registry);
    let frame = registry.get(label).unwrap();
    assert!(frame.width.value() > original_width);
    assert_eq!(frame.position.left, Val::Px(12.0));
    registry.render_dirty.clear();
    screen.sync(&ctx, &mut registry);
    assert!(registry.render_dirty.is_empty());
}

#[test]
fn screen_editbox_autosize_updates_dimensions_and_preserves_position() {
    let mut screen = sizing_screen("EditBox");
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let input = registry.get_by_name("Sized").unwrap();
    let frame = registry.get_mut(input).unwrap();
    frame.height = Dimension::Fixed(0.0);
    let Some(WidgetData::EditBox(editbox)) = &mut frame.widget_data else {
        panic!("expected EditBox");
    };
    editbox.font_size = 30.0;
    editbox.text_insets = [1.0, 2.0, 3.0, 3.0];
    registry.create_frame("UnrelatedDirtyFrame", None);
    screen.sync(&ctx, &mut registry);
    let frame = registry.get(input).unwrap();
    assert_eq!(frame.height, Dimension::Fixed(51.0));
    assert_eq!(frame.position.left, Val::Px(12.0));
    registry.render_dirty.clear();
    screen.sync(&ctx, &mut registry);
    assert!(registry.render_dirty.is_empty());
}

#[test]
fn screen_autosize_preserves_explicit_native_position() {
    let mut screen = Screen::new(|_| vec![WidgetChild::Widget(named_widget("EditBox", "Input"))]);
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    let input = registry.get_by_name("Input").unwrap();
    registry.set_pos(input, 73.0, 41.0).unwrap();
    registry.get_mut(input).unwrap().height = Dimension::Fixed(0.0);
    screen.sync(&ctx, &mut registry);
    let frame = registry.get(input).unwrap();
    assert_eq!(frame.position.left, Val::Px(73.0));
    assert_eq!(frame.position.top, Val::Px(41.0));
    assert!(frame.height.value() > 0.0);
}

#[test]
fn screen_preserves_explicit_auto_dimensions_for_native_layout() {
    let mut screen = Screen::new(|_| {
        ["FontString", "EditBox"]
            .into_iter()
            .map(|tag| {
                let mut widget = named_widget(tag, tag);
                widget.attrs = vec![
                    Attr::new_static("text", "Native sizing".to_string()),
                    Attr::new_static("width", "auto".to_string()),
                    Attr::new_static("height", "auto".to_string()),
                ];
                WidgetChild::Widget(widget)
            })
            .collect()
    });
    let ctx = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&ctx, &mut registry);
    screen.sync(&ctx, &mut registry);
    for name in ["FontString", "EditBox"] {
        let frame = registry.get(registry.get_by_name(name).unwrap()).unwrap();
        assert_eq!(frame.width, Dimension::Auto);
        assert_eq!(frame.height, Dimension::Auto);
    }
}

#[test]
fn fontstring_legacy_width_sizing_keeps_explicit_auto_height() {
    let mut screen = Screen::new(|_| {
        let mut widget = named_widget("FontString", "Label");
        widget.attrs = vec![
            Attr::new_static("text", "Measured width".to_string()),
            Attr::new_static("height", "auto".to_string()),
        ];
        vec![WidgetChild::Widget(widget)]
    });
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&SharedContext::new(), &mut registry);
    let frame = registry
        .get(registry.get_by_name("Label").unwrap())
        .unwrap();
    assert!(frame.width.value() > 0.0);
    assert_eq!(frame.height, Dimension::Auto);
}
