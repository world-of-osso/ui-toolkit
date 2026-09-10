use bevy::prelude::*;
use std::path::{Path, PathBuf};
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::{Dimension, WidgetData};
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render_texture::{BlpLoader, BlpLoaderRes};
use ui_toolkit::render_tiled::{UiTile, sync_ui_tiled_textures};
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
            Some("fixture-101.blp" | "fixture-202.blp") => Ok(Image::default()),
            _ => Err(format!("unknown fixture image: {}", path.display())),
        }
    }
}

#[derive(Resource, Default)]
struct Observed(Vec<(u32, bool, bool)>);

fn observe(tiles: Query<(&UiTile, Ref<Transform>, Ref<Sprite>)>, mut seen: ResMut<Observed>) {
    seen.0 = tiles
        .iter()
        .map(|(tile, transform, sprite)| (tile.1, transform.is_changed(), sprite.is_changed()))
        .collect();
    seen.0.sort_by_key(|entry| entry.0);
}

fn fixture() -> (App, u64) {
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("TiledRegression", None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(128.0);
    frame.height = Dimension::Fixed(128.0);
    frame.widget_data = Some(WidgetData::Texture(TextureData {
        source: TextureSource::FileDataId(101),
        horiz_tile: true,
        vert_tile: true,
        ..default()
    }));
    let mut app = App::new();
    app.init_resource::<Assets<Image>>();
    app.insert_resource(BlpLoaderRes(Box::new(FixtureLoader)));
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    app.add_systems(Update, (sync_ui_tiled_textures, observe).chain());
    app.update();
    assert_eq!(entities(&mut app).len(), 4);
    (app, id)
}

fn entities(app: &mut App) -> Vec<Entity> {
    let mut tiles: Vec<_> = app
        .world_mut()
        .query::<(Entity, &UiTile)>()
        .iter(app.world())
        .map(|(entity, tile)| (tile.1, entity))
        .collect();
    tiles.sort_by_key(|entry| entry.0);
    tiles.into_iter().map(|entry| entry.1).collect()
}

#[test]
fn settled_tiles_keep_component_ticks_clean() {
    let (mut app, _) = fixture();
    app.update();
    assert_eq!(
        app.world().resource::<Observed>().0,
        (0..4)
            .map(|index| (index, false, false))
            .collect::<Vec<_>>()
    );
}

#[test]
fn geometry_tint_and_source_changes_update_existing_tiles() {
    let (mut app, id) = fixture();
    let original = entities(&mut app);
    let old_image = app
        .world()
        .get::<Sprite>(original[0])
        .unwrap()
        .image
        .clone();
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        state.registry.screen_width = 1000.0;
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(96.0);
        let Some(WidgetData::Texture(texture)) = &mut frame.widget_data else {
            panic!("texture fixture")
        };
        texture.source = TextureSource::FileDataId(202);
        texture.vertex_color = [0.2, 0.4, 0.6, 0.8];
    }
    app.update();
    assert_eq!(entities(&mut app), original);
    for (index, entity) in original.iter().enumerate() {
        assert_eq!(
            app.world().get::<Transform>(*entity).unwrap().translation,
            Vec3::new(
                -468.0 + (index % 2) as f32 * 64.0,
                268.0 - (index / 2) as f32 * 64.0,
                0.001
            )
        );
        let sprite = app.world().get::<Sprite>(*entity).unwrap();
        assert_eq!(sprite.custom_size, Some(Vec2::splat(64.0)));
        assert_eq!(sprite.color, Color::srgba(0.2, 0.4, 0.6, 0.8));
        assert_ne!(sprite.image, old_image);
        assert!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&sprite.image)
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
    let (mut app, _) = fixture();
    let original = entities(&mut app);
    let entity = original[0];
    let transform = *app.world().get::<Transform>(entity).unwrap();
    let sprite = app.world().get::<Sprite>(entity).unwrap().clone();
    app.world_mut().entity_mut(entity).insert((
        Transform::from_xyz(1.0, 2.0, 3.0),
        Sprite {
            color: Color::BLACK,
            flip_x: true,
            rect: Some(Rect::new(0.0, 0.0, 0.5, 0.5)),
            ..default()
        },
    ));
    app.update();
    assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &sprite);
    app.world_mut().entity_mut(entity).remove::<Sprite>();
    app.update();
    assert_eq!(entities(&mut app), original);
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &sprite);
    app.world_mut().entity_mut(entity).remove::<Transform>();
    app.update();
    assert_eq!(entities(&mut app), original);
    assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
}

fn assert_sprite(actual: &Sprite, expected: &Sprite) {
    assert_eq!(actual.image, expected.image);
    assert_eq!(actual.color, expected.color);
    assert_eq!(actual.custom_size, expected.custom_size);
    assert_eq!(actual.rect, expected.rect);
    assert_eq!(actual.flip_x, expected.flip_x);
    assert_eq!(actual.flip_y, expected.flip_y);
}

#[test]
fn shrinking_and_hiding_frames_removes_stale_tiles() {
    let (mut app, id) = fixture();
    let original = entities(&mut app);
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(64.0);
        frame.height = Dimension::Fixed(64.0);
    }
    app.update();
    assert_eq!(entities(&mut app), vec![original[0]]);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, true);
    app.update();
    assert!(entities(&mut app).is_empty());
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(id, false);
    app.update();
    assert_eq!(entities(&mut app).len(), 1);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    app.update();
    assert!(entities(&mut app).is_empty());
}
