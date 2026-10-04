pub mod support;

use bevy::prelude::*;
use support::*;
use ui_toolkit::frame::{Dimension, NineSlice};
use ui_toolkit::plugin::UiState;
use ui_toolkit::widgets::texture::{DynamicTextureId, TextureSource};

fn fixture() -> (App, u64, DynamicTextureId) {
    let (mut app, _) = native_app();
    let image = add_texture(&mut app);
    let id = create_frame(&mut app, "NineSliceRegression", 120.0, 60.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .nine_slice = Some(NineSlice {
        edge_size: 4.0,
        texture: Some(TextureSource::Dynamic(image.clone())),
        uv_rects: Some([[0.0, 1.0, 0.0, 1.0]; 9]),
        ..default()
    });
    settle(&mut app);
    (app, id, image)
}

fn assert_coverage(app: &mut App, id: u64, width: f32, height: f32, edge: f32) {
    let entities = image_entities(app.world_mut(), id);
    let mut expected = Vec::new();
    for (y, h) in [
        (0.0, edge),
        (edge, height - edge * 2.0),
        (height - edge, edge),
    ] {
        for (x, w) in [
            (0.0, edge),
            (edge, width - edge * 2.0),
            (width - edge, edge),
        ] {
            let entity = image_covering(app.world_mut(), id, Vec2::new(x + w / 2.0, y + h / 2.0));
            assert_eq!(
                logical_rect(app.world(), entity),
                Rect::new(x, y, x + w, y + h)
            );
            expected.push(entity);
        }
    }
    assert_eq!(
        entities, expected,
        "all slice regions cover exactly the authored panel"
    );
}

#[test]
fn settled_nine_parts_have_no_component_changes() {
    let (mut app, id, _) = fixture();
    assert_coverage(&mut app, id, 120.0, 60.0, 4.0);
    app.update();
    assert!(!app.world().resource::<ImageChanges>().0);
}

#[test]
fn geometry_color_image_and_uv_updates_reconcile_existing_parts() {
    let (mut app, id, _) = fixture();
    let original = image_entities(app.world_mut(), id);
    let replacement = add_texture(&mut app);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(100.0);
        let slice = frame.nine_slice.as_mut().unwrap();
        slice.bg_color = [0.2, 0.4, 0.6, 0.8];
        slice.border_color = [0.7, 0.5, 0.3, 1.0];
        slice.texture = Some(TextureSource::Dynamic(replacement.clone()));
        slice.uv_rects = Some([[0.25, 0.75, 0.125, 0.875]; 9]);
    }
    settle(&mut app);
    assert_eq!(image_entities(app.world_mut(), id), original);
    assert_coverage(&mut app, id, 200.0, 100.0, 4.0);
    let center = image_covering(app.world_mut(), id, Vec2::new(100.0, 50.0));
    for entity in original {
        let image = app.world().get::<ImageNode>(entity).unwrap();
        assert_texture(app.world(), &image.image, replacement);
        assert_eq!(image.rect, Some(Rect::new(16.0, 8.0, 48.0, 56.0)));
        let color = if entity == center {
            Color::srgba(0.2, 0.4, 0.6, 0.8)
        } else {
            Color::srgba(0.7, 0.5, 0.3, 1.0)
        };
        assert_eq!(image.color, color);
    }
}

#[test]
fn external_edits_and_missing_components_are_repaired_on_same_entities() {
    let (mut app, id, _) = fixture();
    let original = image_entities(app.world_mut(), id);
    let center = image_covering(app.world_mut(), id, Vec2::new(60.0, 30.0));
    assert_image_repair(&mut app, center);
    assert_eq!(image_entities(app.world_mut(), id), original);
}

#[test]
fn hidden_and_removed_frames_remove_visible_parts() {
    let (mut app, id, _) = fixture();
    let original = image_entities(app.world_mut(), id);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, true);
    settle(&mut app);
    assert!(image_entities(app.world_mut(), id).is_empty());
    for entity in original {
        assert!(app.world().get_entity(entity).is_err());
    }
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, false);
    settle(&mut app);
    assert_coverage(&mut app, id, 120.0, 60.0, 4.0);
    let parent = frame_entity(app.world_mut(), id);
    let restored = image_entities(app.world_mut(), id);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    settle(&mut app);
    assert!(app.world().get_entity(parent).is_err());
    for entity in restored {
        assert!(app.world().get_entity(entity).is_err());
    }
}
