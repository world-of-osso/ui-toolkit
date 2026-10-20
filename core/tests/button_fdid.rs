use ui_toolkit_core as ui_toolkit;
use ui_toolkit_core::{
    frame::WidgetData,
    registry::FrameRegistry,
    rsx,
    screen::{Screen, SharedContext},
    widgets::texture::TextureSource,
};

#[test]
fn button_state_fdids_survive_screen_rebuilds_without_losing_actions() {
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let mut context = SharedContext::new();
    context.insert(130828_u32);
    let mut screen = Screen::new(|context| {
        let normal = *context.get::<u32>().unwrap();
        rsx! { button {
            name: "SourceButton",
            button_fdid_up: normal,
            button_fdid_pressed: 130825_u32,
            button_fdid_highlight: 130826_u32,
            button_fdid_disabled: 130824_u32,
            onclick: "accept",
        } }
    });
    screen.sync(&context, &mut registry);
    let original = registry.get_by_name("SourceButton").unwrap();
    for normal in [130828_u32, 4294967294] {
        context.insert(normal);
        screen.sync(&context, &mut registry);
        let id = registry.get_by_name("SourceButton").unwrap();
        assert_eq!(id, original);
        let Some(WidgetData::Button(button)) = &registry.get(id).unwrap().widget_data else {
            panic!("expected button");
        };
        assert_eq!(
            button
                .normal_texture
                .as_ref()
                .map(|texture| &texture.source),
            Some(&TextureSource::FileDataId(normal))
        );
        assert_eq!(
            button
                .pushed_texture
                .as_ref()
                .map(|texture| &texture.source),
            Some(&TextureSource::FileDataId(130825))
        );
        assert_eq!(
            button
                .highlight_texture
                .as_ref()
                .map(|texture| &texture.source),
            Some(&TextureSource::FileDataId(130826))
        );
        assert_eq!(
            button
                .disabled_texture
                .as_ref()
                .map(|texture| &texture.source),
            Some(&TextureSource::FileDataId(130824))
        );
        assert_eq!(registry.click_frame(id).as_deref(), Some("accept"));
    }
}
