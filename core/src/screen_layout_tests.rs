use super::*;
use crate::frame::{Dimension, WidgetData};
use crate::layout_values::Val;
use crate::widget_def::{Attr, WidgetDef};

struct TextState {
    text: String,
    font_size: f32,
}

fn named_widget(tag: &'static str, name: &str) -> WidgetDef {
    let mut widget = WidgetDef::new(tag);
    widget.name = Some(name.to_owned());
    widget
}

fn sizing_screen(width: &'static str, height: &'static str) -> Screen {
    Screen::new(move |context| {
        let state = context.get::<TextState>().unwrap();
        let mut parent = named_widget("Frame", "Parent");
        parent.children = ["FontString", "EditBox"]
            .into_iter()
            .map(|tag| {
                let mut widget = named_widget(tag, tag);
                widget.attrs = vec![
                    Attr::new_dynamic("text", state.text.clone()),
                    Attr::new_dynamic("font_size", state.font_size.to_string()),
                    Attr::new_static("width", width.to_owned()),
                    Attr::new_static("height", height.to_owned()),
                    Attr::new_static("pos_x", "12".to_owned()),
                ];
                WidgetChild::Widget(widget)
            })
            .collect();
        vec![WidgetChild::Widget(parent)]
    })
}

fn text_value(frame: &crate::frame::Frame) -> &str {
    match frame.widget_data.as_ref().unwrap() {
        WidgetData::FontString(text) => &text.text,
        WidgetData::EditBox(editbox) => &editbox.text,
        other => panic!("expected text widget, got {other:?}"),
    }
}

fn assert_authored_sizes_survive_updates(
    width: &'static str,
    height: &'static str,
    expected: (Dimension, Dimension),
) {
    let mut screen = sizing_screen(width, height);
    let mut context = SharedContext::new();
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let mut original_ids = None;
    for (text, font_size) in [("Hi", 20.0), ("A much longer label", 32.0)] {
        context.insert(TextState {
            text: text.into(),
            font_size,
        });
        screen.sync(&context, &mut registry);
        let ids = ["FontString", "EditBox"].map(|name| registry.get_by_name(name).unwrap());
        if let Some(original) = original_ids {
            assert_eq!(ids, original);
        } else {
            original_ids = Some(ids);
        }
        for id in ids {
            let frame = registry.get(id).unwrap();
            assert_eq!((frame.width, frame.height), expected);
            assert_eq!(frame.position.left, Val::Px(12.0));
            assert_eq!(text_value(frame), text);
            match frame.widget_data.as_ref().unwrap() {
                WidgetData::FontString(text) => assert_eq!(text.font_size, font_size),
                WidgetData::EditBox(editbox) => assert_eq!(editbox.font_size, font_size),
                _ => unreachable!(),
            }
            assert!(
                frame.layout_rect.is_none(),
                "Screen must not compute layout"
            );
        }
        registry.render_dirty.clear();
        registry.rect_dirty.clear();
        screen.sync(&context, &mut registry);
        assert!(registry.render_dirty.is_empty());
        assert!(registry.rect_dirty.is_empty());
    }
}

#[test]
fn screen_leaves_auto_dimensions_authored_through_nested_text_updates() {
    assert_authored_sizes_survive_updates("auto", "auto", (Dimension::Auto, Dimension::Auto));
}

#[test]
fn screen_preserves_explicit_dimensions_including_zero_through_text_updates() {
    for (width, height, expected) in [
        ("0", "0", (Dimension::Fixed(0.0), Dimension::Fixed(0.0))),
        ("120", "0", (Dimension::Fixed(120.0), Dimension::Fixed(0.0))),
        (
            "120",
            "42",
            (Dimension::Fixed(120.0), Dimension::Fixed(42.0)),
        ),
        ("0", "auto", (Dimension::Fixed(0.0), Dimension::Auto)),
    ] {
        assert_authored_sizes_survive_updates(width, height, expected);
    }
}

#[test]
fn idle_screen_sync_preserves_registry_text_edits_sizes_and_position() {
    let mut screen = sizing_screen("auto", "auto");
    let mut context = SharedContext::new();
    context.insert(TextState {
        text: "Initial".into(),
        font_size: 20.0,
    });
    let mut registry = FrameRegistry::new(800.0, 600.0);
    screen.sync(&context, &mut registry);
    for name in ["FontString", "EditBox"] {
        let id = registry.get_by_name(name).unwrap();
        registry.set_pos(id, 73.0, 41.0).unwrap();
        let frame = registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(0.0);
        frame.height = Dimension::Fixed(0.0);
        match frame.widget_data.as_mut().unwrap() {
            WidgetData::FontString(text) => text.text = "Edited directly".into(),
            WidgetData::EditBox(editbox) => {
                editbox.text = "Edited directly".into();
                editbox.font_size = 30.0;
                editbox.text_insets = [1.0, 2.0, 3.0, 4.0];
            }
            _ => unreachable!(),
        }
    }
    registry.render_dirty.clear();
    registry.rect_dirty.clear();
    screen.sync(&context, &mut registry);
    for name in ["FontString", "EditBox"] {
        let frame = registry.get(registry.get_by_name(name).unwrap()).unwrap();
        assert_eq!(text_value(frame), "Edited directly");
        assert_eq!(
            (frame.width, frame.height),
            (Dimension::Fixed(0.0), Dimension::Fixed(0.0))
        );
        assert_eq!(frame.position.left, Val::Px(73.0));
        assert_eq!(frame.position.top, Val::Px(41.0));
    }
    assert!(registry.render_dirty.is_empty());
    assert!(registry.rect_dirty.is_empty());
}
