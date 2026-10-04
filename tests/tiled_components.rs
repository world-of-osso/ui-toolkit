pub mod support;

use bevy::prelude::*;
use std::path::{Path, PathBuf};
use support::*;
use ui_toolkit::frame::{Dimension, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::render_texture::{BlpLoader, BlpLoaderRes};
use ui_toolkit::widgets::texture::{TextureData, TextureSource};

struct FixtureLoader;

impl BlpLoader for FixtureLoader {
    fn ensure_texture(&self, fdid: u32) -> Option<PathBuf> {
        match fdid {
            101 | 202 => Some(PathBuf::from(format!("fixture-{fdid}.blp"))),
            _ => None,
        }
    }
    fn load_blp_to_image(&self, path: &Path) -> Result<Image, String> {
        self.load_blp_gpu_image(path)
    }
    fn load_blp_gpu_image(&self, path: &Path) -> Result<Image, String> {
        match path.to_str() {
            Some("fixture-101.blp" | "fixture-202.blp") => Ok(image()),
            _ => Err(format!("unknown fixture image: {}", path.display())),
        }
    }
}

fn fixture() -> (App, u64) {
    let (mut app, _) = native_app();
    app.insert_resource(BlpLoaderRes(Box::new(FixtureLoader)));
    let id = create_frame(&mut app, "TiledRegression", 128.0, 128.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(id)
        .unwrap()
        .widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::FileDataId(101),
        horiz_tile: true,
        vert_tile: true,
        ..default()
    }));
    settle(&mut app);
    (app, id)
}

// Native UiPlugin also owns the underlying flat image. Assert the visible
// repeating overlay, selected by its real stack order rather than private markers.
fn tile_entities(world: &mut World, id: u64) -> Vec<Entity> {
    let images = image_entities(world, id);
    let top = images
        .iter()
        .map(|entity| world.get::<GlobalZIndex>(*entity).unwrap().0)
        .max();
    images
        .into_iter()
        .filter(|entity| Some(world.get::<GlobalZIndex>(*entity).unwrap().0) == top)
        .collect()
}

fn tile_covering(world: &mut World, id: u64, point: Vec2) -> Entity {
    let matches: Vec<_> = tile_entities(world, id)
        .into_iter()
        .filter(|entity| logical_rect(world, *entity).contains(point))
        .collect();
    assert_eq!(matches.len(), 1, "one topmost tile covers {point:?}");
    matches[0]
}

fn assert_tiles(app: &mut App, id: u64, width: f32, height: f32) {
    let actual = tile_entities(app.world_mut(), id);
    let mut expected = Vec::new();
    for row in 0..(height / 64.0).ceil() as u32 {
        for column in 0..(width / 64.0).ceil() as u32 {
            let min = Vec2::new(column as f32 * 64.0, row as f32 * 64.0);
            let entity = tile_covering(app.world_mut(), id, min + Vec2::splat(32.0));
            assert_eq!(
                logical_rect(app.world(), entity),
                Rect::from_corners(min, min + Vec2::splat(64.0))
            );
            expected.push(entity);
        }
    }
    assert_eq!(
        actual, expected,
        "tiles repeat at 64px without gaps or stale regions"
    );
}

#[test]
fn settled_tiles_keep_component_ticks_clean() {
    let (mut app, id) = fixture();
    assert_tiles(&mut app, id, 128.0, 128.0);
    app.update();
    assert!(!app.world().resource::<ImageChanges>().0);
}

#[test]
fn geometry_tint_and_source_changes_update_existing_tiles() {
    let (mut app, id) = fixture();
    let original = tile_entities(app.world_mut(), id);
    let old_image = app
        .world()
        .get::<ImageNode>(original[0])
        .unwrap()
        .image
        .clone();
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(96.0);
        let Some(WidgetData::Texture(texture)) = &mut frame.widget_data else {
            panic!("texture")
        };
        texture.source = TextureSource::FileDataId(202);
        texture.vertex_color = [0.2, 0.4, 0.6, 0.8];
    }
    settle(&mut app);
    assert_eq!(tile_entities(app.world_mut(), id), original);
    assert_tiles(&mut app, id, 96.0, 128.0);
    for entity in original {
        let image = app.world().get::<ImageNode>(entity).unwrap();
        assert_eq!(image.color, Color::srgba(0.2, 0.4, 0.6, 0.8));
        assert_ne!(image.image, old_image);
        assert!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&image.image)
                .is_some()
        );
    }
    assert!(
        app.world()
            .resource::<UiState>()
            .registry
            .render_dirty
            .is_empty()
    );
}

#[test]
fn external_and_missing_components_are_repaired_on_same_tiles() {
    let (mut app, id) = fixture();
    let original = tile_entities(app.world_mut(), id);
    assert_image_repair(&mut app, original[0]);
    assert_eq!(tile_entities(app.world_mut(), id), original);
}

#[test]
fn shrinking_and_hiding_frames_removes_stale_tiles() {
    let (mut app, id) = fixture();
    let original = tile_entities(app.world_mut(), id);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(64.0);
        frame.height = Dimension::Fixed(64.0);
    }
    settle(&mut app);
    assert_tiles(&mut app, id, 64.0, 64.0);
    assert_eq!(tile_entities(app.world_mut(), id), vec![original[0]]);
    for &stale in &original[1..] {
        assert!(app.world().get_entity(stale).is_err());
    }
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, true);
    settle(&mut app);
    assert!(tile_entities(app.world_mut(), id).is_empty());
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, false);
    settle(&mut app);
    assert_tiles(&mut app, id, 64.0, 64.0);
    let restored = tile_entities(app.world_mut(), id);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    settle(&mut app);
    for entity in restored {
        assert!(app.world().get_entity(entity).is_err());
    }
}
