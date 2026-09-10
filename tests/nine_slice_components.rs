use bevy::prelude::*;
use ui_toolkit::event::EventBus;
use ui_toolkit::frame::{Dimension, NineSlice};
use ui_toolkit::plugin::UiState;
use ui_toolkit::registry::FrameRegistry;
use ui_toolkit::render_nine_slice::{UiNineSlicePart, sync_ui_nine_slices};
use ui_toolkit::widgets::texture::TextureSource;

#[derive(Resource, Default)]
struct Observed(Vec<(u8, bool, bool)>);

fn observe(
    parts: Query<(&UiNineSlicePart, Ref<Transform>, Ref<Sprite>)>,
    mut observed: ResMut<Observed>,
) {
    observed.0 = parts
        .iter()
        .map(|(part, transform, sprite)| (part.1, transform.is_changed(), sprite.is_changed()))
        .collect();
    observed.0.sort_by_key(|entry| entry.0);
}

fn fixture() -> (App, u64, Handle<Image>) {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>();
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let mut registry = FrameRegistry::new(800.0, 600.0);
    let id = registry.create_frame("NineSliceRegression", None);
    let frame = registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(120.0);
    frame.height = Dimension::Fixed(60.0);
    frame.nine_slice = Some(NineSlice {
        edge_size: 4.0,
        texture: Some(TextureSource::Dynamic(image.clone())),
        uv_rects: Some([[0.0, 1.0, 0.0, 1.0]; 9]),
        ..default()
    });
    app.insert_resource(UiState {
        registry,
        event_bus: EventBus::new(),
        focused_frame: None,
    });
    app.init_resource::<Observed>();
    app.add_systems(Update, (sync_ui_nine_slices, observe).chain());
    app.update();
    assert_eq!(entities(&mut app).len(), 9);
    (app, id, image)
}

fn entities(app: &mut App) -> Vec<Entity> {
    let mut parts: Vec<_> = app
        .world_mut()
        .query::<(Entity, &UiNineSlicePart)>()
        .iter(app.world())
        .map(|(entity, part)| (part.1, entity))
        .collect();
    parts.sort_by_key(|entry| entry.0);
    parts.into_iter().map(|entry| entry.1).collect()
}

#[test]
fn settled_nine_parts_have_no_component_changes() {
    let (mut app, _, _) = fixture();
    app.update();
    assert_eq!(
        app.world().resource::<Observed>().0,
        (0..9).map(|part| (part, false, false)).collect::<Vec<_>>()
    );
}

#[test]
fn geometry_color_image_and_uv_updates_reconcile_existing_parts() {
    let (mut app, id, _) = fixture();
    let original_entities = entities(&mut app);
    let replacement = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    {
        let mut state = app.world_mut().resource_mut::<UiState>();
        let frame = state.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(200.0);
        frame.height = Dimension::Fixed(100.0);
        let slice = frame.nine_slice.as_mut().unwrap();
        slice.bg_color = [0.2, 0.4, 0.6, 0.8];
        slice.border_color = [0.7, 0.5, 0.3, 1.0];
        slice.texture = Some(TextureSource::Dynamic(replacement.clone()));
        slice.uv_rects = Some([[0.25, 0.75, 0.125, 0.875]; 9]);
    }
    app.update();
    assert_eq!(entities(&mut app), original_entities);
    let center = original_entities[4];
    assert_eq!(
        app.world().get::<Transform>(center).unwrap().translation,
        Vec3::new(-300.0, 250.0, 0.0)
    );
    assert_eq!(
        app.world().get::<Sprite>(center).unwrap().custom_size,
        Some(Vec2::new(192.0, 92.0))
    );
    for (part, entity) in original_entities.iter().enumerate() {
        let sprite = app.world().get::<Sprite>(*entity).unwrap();
        assert_eq!(sprite.image, replacement);
        assert_eq!(sprite.rect, Some(Rect::new(0.25, 0.125, 0.75, 0.875)));
        let expected = if part == 4 {
            Color::srgba(0.2, 0.4, 0.6, 0.8)
        } else {
            Color::srgba(0.7, 0.5, 0.3, 1.0)
        };
        assert_eq!(sprite.color, expected);
    }
}

#[test]
fn external_edits_and_missing_components_are_repaired_on_same_entities() {
    let (mut app, _, _) = fixture();
    let original_entities = entities(&mut app);
    let entity = original_entities[4];
    let expected_transform = *app.world().get::<Transform>(entity).unwrap();
    let expected_sprite = app.world().get::<Sprite>(entity).unwrap().clone();
    app.world_mut().entity_mut(entity).insert((
        Transform::from_xyz(1.0, 2.0, 3.0),
        Sprite {
            color: Color::BLACK,
            flip_x: true,
            ..default()
        },
    ));
    app.update();
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        expected_transform
    );
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &expected_sprite);
    app.world_mut().entity_mut(entity).remove::<Sprite>();
    app.update();
    assert_eq!(entities(&mut app), original_entities);
    assert_sprite(app.world().get::<Sprite>(entity).unwrap(), &expected_sprite);
    app.world_mut().entity_mut(entity).remove::<Transform>();
    app.update();
    assert_eq!(entities(&mut app), original_entities);
    assert_eq!(
        *app.world().get::<Transform>(entity).unwrap(),
        expected_transform
    );
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
fn hidden_and_removed_frames_despawn_all_parts() {
    let (mut app, id, _) = fixture();
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
    assert_eq!(entities(&mut app).len(), 9);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame(id);
    app.update();
    assert!(entities(&mut app).is_empty());
}
