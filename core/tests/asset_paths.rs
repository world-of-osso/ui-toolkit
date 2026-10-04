use std::process::Command;

use ui_toolkit_core::{
    attrs::apply_attribute,
    frame::WidgetData,
    registry::FrameRegistry,
    widgets::{
        slider::SliderData,
        texture::{TextureData, TextureSource},
    },
};

#[test]
fn authored_paths_do_not_depend_on_host_cwd() {
    if std::env::var_os("UI_CORE_ASSET_PATH_CHILD").is_some() {
        let mut registry = FrameRegistry::new(800.0, 600.0);
        let texture = registry.create_frame("Art", None);
        let slider = registry.create_frame("Slider", None);
        registry.get_mut(texture).unwrap().widget_data =
            Some(WidgetData::Texture(TextureData::default()));
        registry.get_mut(slider).unwrap().widget_data =
            Some(WidgetData::Slider(SliderData::default()));
        for (id, attr, value) in [
            (texture, "texture_file", "data/art/portrait.blp"),
            (texture, "texture_fdid", "123456"),
            (slider, "thumb_texture", "data/art/thumb.blp"),
        ] {
            apply_attribute(&mut registry, id, attr, value);
            match (attr, &registry.get(id).unwrap().widget_data) {
                ("texture_file", Some(WidgetData::Texture(data))) => {
                    assert_eq!(data.source, TextureSource::File(value.into()));
                }
                ("texture_fdid", Some(WidgetData::Texture(data))) => {
                    assert_eq!(data.source, TextureSource::FileDataId(123456));
                }
                ("thumb_texture", Some(WidgetData::Slider(data))) => {
                    assert_eq!(data.thumb_texture, Some(TextureSource::File(value.into())));
                }
                _ => panic!("unexpected widget data for {attr}"),
            }
        }
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let output = Command::new(exe)
        .args([
            "--exact",
            "authored_paths_do_not_depend_on_host_cwd",
            "--nocapture",
        ])
        .env("UI_CORE_ASSET_PATH_CHILD", "1")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
