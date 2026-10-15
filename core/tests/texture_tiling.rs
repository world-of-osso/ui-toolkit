use ui_toolkit_core as ui_toolkit;
use ui_toolkit_core::{
    frame::WidgetData,
    registry::FrameRegistry,
    rsx,
    screen::{Screen, SharedContext},
};

#[test]
fn texture_tile_axes_apply_independently_and_clear_on_rebuild() {
    let mut shared = SharedContext::new();
    shared.insert((false, false));
    let mut registry = FrameRegistry::new(1920.0, 1080.0);
    let mut screen = Screen::new(|shared| {
        let (horizontal, vertical) = *shared.get::<(bool, bool)>().unwrap();
        rsx! { texture {
            name: "TiledRock",
            width: 334.0,
            height: 473.0,
            texture_fdid: 374155_u32,
            horiz_tile: horizontal,
            vert_tile: vertical,
        } }
    });
    screen.sync(&shared, &mut registry);
    let original = registry.get_by_name("TiledRock").unwrap();
    for axes in [
        (false, false),
        (true, false),
        (false, true),
        (true, true),
        (false, false),
    ] {
        shared.insert(axes);
        screen.sync(&shared, &mut registry);
        let id = registry.get_by_name("TiledRock").unwrap();
        assert_eq!(id, original);
        let Some(WidgetData::Texture(texture)) = &registry.get(id).unwrap().widget_data else {
            panic!("texture render input required");
        };
        assert_eq!((texture.horiz_tile, texture.vert_tile), axes);
    }
}
