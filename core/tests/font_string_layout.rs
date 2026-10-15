use ui_toolkit_core as ui_toolkit;
use ui_toolkit_core::{
    attrs::read_attribute,
    frame::WidgetData,
    registry::FrameRegistry,
    rsx,
    screen::{Screen, SharedContext},
    widget_def::Element,
};

type CaptionLayout = (bool, Option<u32>);

fn caption(shared: &SharedContext) -> Element {
    let (wrap, lines) = *shared.get::<CaptionLayout>().unwrap();
    let limit = lines.map_or_else(|| "none".to_owned(), |lines| lines.to_string());
    rsx! { fontstring {
        name: "Caption", text: "A long selected caption", width: 126.0, height: 25.0,
        word_wrap: wrap, max_lines: limit,
    } }
}

#[test]
fn font_string_layout_attributes_reach_render_input_and_update_on_rebuild() {
    let mut shared = SharedContext::new();
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let mut screen = Screen::new(caption);
    for state in [
        (false, Some(1)),
        (true, Some(3)),
        (false, None),
        (true, Some(2)),
        (false, Some(1)),
    ] {
        shared.insert::<CaptionLayout>(state);
        screen.sync(&shared, &mut registry);
        let id = registry.get_by_name("Caption").unwrap();
        let Some(WidgetData::FontString(text)) = &registry.get(id).unwrap().widget_data else {
            panic!("Caption must supply font render input");
        };
        let (wrap, lines) = state;
        assert_eq!((text.word_wrap, text.max_lines), (wrap, lines));
        assert_eq!(
            read_attribute(&registry, id, "word_wrap"),
            Some(wrap.to_string())
        );
        assert_eq!(
            read_attribute(&registry, id, "max_lines"),
            Some(lines.map_or_else(|| "none".to_owned(), |lines| lines.to_string()))
        );
    }
}

#[test]
#[should_panic(expected = "max_lines must be positive or none")]
fn font_string_rejects_zero_line_limit() {
    let mut registry = FrameRegistry::new(100.0, 100.0);
    Screen::new(|_| rsx! { fontstring { name: "InvalidCaption", max_lines: 0 } })
        .sync(&SharedContext::new(), &mut registry);
}
