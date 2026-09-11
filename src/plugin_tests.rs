use super::*;
use crate::frame::{Dimension, WidgetData};
use crate::native_render::{RegistryNode, RegistryText};
use crate::widgets::font_string::FontStringData;

fn panel(app: &mut App, text: bool) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame("Panel", None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(100.0);
    frame.height = Dimension::Fixed(40.0);
    frame.background_color = Some([1.0, 0.0, 0.0, 1.0]);
    if text {
        frame.widget_data = Some(WidgetData::FontString(FontStringData {
            text: "Hello".into(),
            ..default()
        }));
    }
    id
}

fn image_count(app: &mut App, id: u64) -> usize {
    let frame = app
        .world_mut()
        .query::<(Entity, &RegistryNode)>()
        .iter(app.world())
        .find(|(_, frame)| frame.0 == id)
        .map(|(entity, _)| entity);
    app.world_mut()
        .query::<(&ImageNode, &ChildOf)>()
        .iter(app.world())
        .filter(|(_, parent)| Some(parent.parent()) == frame)
        .count()
}

fn text_count(app: &mut App, id: u64) -> usize {
    app.world_mut()
        .query::<&RegistryText>()
        .iter(app.world())
        .filter(|text| text.frame_id == id && text.key == 0)
        .count()
}

#[test]
fn plugin_adds_registry_and_projection_controls() {
    let mut app = App::new();
    app.add_plugins(UiPlugin);
    assert!(app.world().contains_resource::<UiState>());
    assert!(app.world().resource::<UiRenderEnabled>().0);
    assert!(app.world().resource::<UiTextRenderEnabled>().0);
}

#[test]
fn disabled_ui_processing_pauses_until_reenabled() {
    let mut app = crate::native_render::tests::app_with_real_fonts(1.0);
    app.update();
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        ui.registry.screen_width = 123.0;
        ui.registry.screen_height = 456.0;
    }
    app.world_mut().resource_mut::<UiProcessingEnabled>().0 = false;
    app.update();
    assert_eq!(
        app.world().resource::<UiState>().registry.screen_width,
        123.0
    );
    assert_eq!(
        app.world().resource::<UiState>().registry.screen_height,
        456.0
    );
    app.world_mut().resource_mut::<UiProcessingEnabled>().0 = true;
    app.update();
    assert_eq!(
        app.world().resource::<UiState>().registry.screen_width,
        800.0
    );
    assert_eq!(
        app.world().resource::<UiState>().registry.screen_height,
        600.0
    );
}

#[test]
fn disabled_render_preserves_authored_frame_then_projects_when_enabled() {
    let mut app = crate::native_render::tests::app_with_real_fonts(1.0);
    let id = panel(&mut app, false);
    app.world_mut().resource_mut::<UiRenderEnabled>().0 = false;
    app.update();
    assert!(app.world().resource::<UiState>().registry.get(id).is_some());
    assert_eq!(image_count(&mut app, id), 0);
    app.world_mut().resource_mut::<UiRenderEnabled>().0 = true;
    app.update();
    assert_eq!(image_count(&mut app, id), 1);
}

#[test]
fn disabled_text_preserves_background_then_projects_text_when_enabled() {
    let mut app = crate::native_render::tests::app_with_real_fonts(1.0);
    let id = panel(&mut app, true);
    app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = false;
    app.update();
    assert_eq!(image_count(&mut app, id), 1);
    assert_eq!(text_count(&mut app, id), 0);
    app.world_mut().resource_mut::<UiTextRenderEnabled>().0 = true;
    app.update();
    assert_eq!(text_count(&mut app, id), 1);
}
