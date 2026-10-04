//! Native migration coverage map (legacy baseline: 4062a98).
//!
//! - text_components/shadow_components: Text2d, Anchor and TextBounds checks map
//!   to real Text shaping, parent ComputedNode bounds, native offsets, style,
//!   unicode/password content, updates, idle writes and external repair.
//! - nine_slice_components/tiled_components/remaining_sprites: Sprite geometry,
//!   UVs, tint, image source and ownership map to ImageNode pixels and real native
//!   rectangles, stacking, stable identity, component repair and stale cleanup.
//!   Tiles assert the visible repeating overlay above its native flat background;
//!   hover-only buttons drive real cursor input instead of isolated highlights.
//! - button_nine_slice: state selection, resize and generated-skin authority now
//!   assert native slice pixels/rectangles and preserve unrelated panel output.
//!   Default atlas derivation remains supplemental registry coverage: project
//!   KTX2 artwork is not generated or substituted by this fixture.
//! - render_dirty_clear: duplicate quads/tiled pipeline-only dirty-clear cases
//!   map to the single native projection's empty-tick and publish/drain cases.
//! - shared_frame_order/ui_plugin_integration: retain native order, pause/resume,
//!   input timing, resize, layout readback and missing-component contracts; share
//!   explicit font assets and camera/time setup instead of machine-local fonts.
//! - button_input/scroll_list_input: retained unchanged; neither calls a retired
//!   projection entrypoint. They remain focused input/registry behavior suites.
//!
//! No legacy marker or renderer is retained for compatibility. Component counts
//! alone are not evidence: region coverage, content, pixels and ownership are.

use bevy::asset::{AssetApp, AssetPlugin, RenderAssetUsages};
use bevy::camera::{CameraPlugin, CameraUpdateSystems, ComputedCameraValues, RenderTargetInfo};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use ui_toolkit::font_registry::FontRegistry;
use ui_toolkit::frame::Dimension;
use ui_toolkit::native_render::{RegistryNode, RegistryText};
use ui_toolkit::plugin::{UiPlugin, UiState};
use ui_toolkit::render::UiCamera;
use ui_toolkit::widgets::font_string::GameFont;

#[derive(Resource)]
struct FontDirectory(PathBuf);

impl FontDirectory {
    fn write_fixture() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ui-native-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("create isolated font directory");
        for font in [GameFont::FrizQuadrata, GameFont::ArialNarrow] {
            std::fs::write(path.join(font.file_name()), bevy::text::DEFAULT_FONT_DATA)
                .expect("write valid embedded font fixture");
        }
        Self(path)
    }
}

impl Drop for FontDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove isolated font directory");
    }
}

#[derive(Resource, Default)]
pub struct ImageChanges(pub bool);

fn observe_images(
    images: Query<(Ref<Node>, Ref<ImageNode>, Ref<UiTransform>)>,
    mut seen: ResMut<ImageChanges>,
) {
    seen.0 = images.iter().any(|(node, image, transform)| {
        node.is_changed() || image.is_changed() || transform.is_changed()
    });
}

pub fn native_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::image::ImagePlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::window::WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        },
        bevy::input::InputPlugin,
        bevy::transform::TransformPlugin,
        CameraPlugin,
        bevy::text::TextPlugin,
        bevy::picking::DefaultPickingPlugins,
        bevy::ui::UiPlugin,
        UiPlugin,
    ));
    let directory = FontDirectory::write_fixture();
    app.insert_resource(FontRegistry::with_directory(&directory.0));
    app.insert_resource(directory);
    app.init_asset::<TextureAtlasLayout>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    let window = app
        .world_mut()
        .spawn((
            Window {
                resolution: (800, 600).into(),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    app.add_systems(
        PostUpdate,
        update_camera_target
            .after(CameraUpdateSystems)
            .before(bevy::ui::UiSystems::Prepare),
    );
    app.init_resource::<ImageChanges>();
    app.add_systems(Last, observe_images);
    app.finish();
    app.cleanup();
    (app, window)
}

// Only render-target metadata is supplied by the fixture. Layout, shaping,
// projection, input, transforms and readback run their production systems.
fn update_camera_target(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<UiCamera>>,
) {
    let window = windows.single().unwrap();
    for mut camera in &mut cameras {
        camera.computed = ComputedCameraValues {
            target_info: Some(RenderTargetInfo {
                physical_size: UVec2::new(window.physical_width(), window.physical_height()),
                scale_factor: window.scale_factor(),
            }),
            ..default()
        };
    }
}

pub fn settle(app: &mut App) {
    for _ in 0..4 {
        app.update();
    }
}

pub fn create_frame(app: &mut App, name: &str, width: f32, height: f32) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    ui.registry
        .set_pos_type(id, ui_toolkit_core::layout_values::PositionType::Absolute)
        .unwrap();
    ui.registry.set_pos(id, 0.0, 0.0).unwrap();
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(width);
    frame.height = Dimension::Fixed(height);
    id
}

pub fn frame_entity(world: &mut World, id: u64) -> Entity {
    world
        .query::<(Entity, &RegistryNode)>()
        .iter(world)
        .find(|(_, frame)| frame.0 == id)
        .expect("projected frame")
        .0
}

pub fn text_entity(world: &mut World, id: u64, key: u32) -> Entity {
    world
        .query::<(Entity, &RegistryText)>()
        .iter(world)
        .find(|(_, text)| text.frame_id == id && text.key == key)
        .expect("projected text")
        .0
}

pub fn logical_rect(world: &World, entity: Entity) -> Rect {
    let node = world
        .get::<ComputedNode>(entity)
        .expect("native computed layout");
    let transform = Affine2::from(world.get::<UiGlobalTransform>(entity).unwrap());
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor,
        node.size * node.inverse_scale_factor,
    )
}

pub fn image_entities(world: &mut World, id: u64) -> Vec<Entity> {
    let parent = frame_entity(world, id);
    let mut entities: Vec<_> = world
        .get::<Children>(parent)
        .into_iter()
        .flat_map(|children| children.iter())
        .filter(|child| world.get::<ImageNode>(*child).is_some())
        .collect();
    entities.sort_by(|a, b| {
        let a = logical_rect(world, *a).min;
        let b = logical_rect(world, *b).min;
        a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x))
    });
    entities
}

pub fn image_covering(world: &mut World, id: u64, point: Vec2) -> Entity {
    let matches: Vec<_> = image_entities(world, id)
        .into_iter()
        .filter(|entity| logical_rect(world, *entity).contains(point))
        .collect();
    assert_eq!(matches.len(), 1, "exactly one image covers {point:?}");
    matches[0]
}

pub fn image() -> Image {
    Image::new_fill(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub fn add_texture(app: &mut App) -> ui_toolkit::widgets::texture::DynamicTextureId {
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    let color = NEXT.fetch_add(1, Ordering::Relaxed) as u8;
    let pixels = [color, 127, 255, 255].repeat(64 * 64);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .create_dynamic_texture(64, 64, pixels)
        .unwrap()
}

pub fn assert_texture(
    world: &World,
    handle: &Handle<Image>,
    id: ui_toolkit::widgets::texture::DynamicTextureId,
) {
    let ui = world.resource::<UiState>();
    let source = ui.registry.dynamic_texture(id).unwrap();
    let projected = world
        .resource::<Assets<Image>>()
        .get(handle)
        .expect("loaded image asset");
    assert_eq!(projected.width(), source.width);
    assert_eq!(projected.height(), source.height);
    assert_eq!(projected.data.as_deref(), Some(source.rgba8.as_slice()));
}

pub fn assert_image(actual: &ImageNode, expected: &ImageNode) {
    assert_eq!(actual.image, expected.image);
    assert_eq!(actual.color, expected.color);
    assert_eq!(actual.rect, expected.rect);
    assert_eq!(actual.flip_x, expected.flip_x);
    assert_eq!(actual.flip_y, expected.flip_y);
    assert_eq!(actual.image_mode, expected.image_mode);
}

pub fn assert_image_repair(app: &mut App, entity: Entity) {
    let expected_image = app.world().get::<ImageNode>(entity).unwrap().clone();
    let expected_node = app.world().get::<Node>(entity).unwrap().clone();
    let expected_transform = *app.world().get::<UiTransform>(entity).unwrap();
    let expected_rect = logical_rect(app.world(), entity);
    app.world_mut().entity_mut(entity).insert((
        ImageNode {
            color: Color::BLACK,
            flip_x: true,
            flip_y: true,
            ..default()
        },
        Node {
            width: px(1),
            height: px(1),
            ..default()
        },
        UiTransform::from_rotation(Rot2::radians(1.0)),
    ));
    settle(app);
    assert_image(
        app.world().get::<ImageNode>(entity).unwrap(),
        &expected_image,
    );
    assert_eq!(*app.world().get::<Node>(entity).unwrap(), expected_node);
    assert_eq!(
        *app.world().get::<UiTransform>(entity).unwrap(),
        expected_transform
    );
    assert_eq!(logical_rect(app.world(), entity), expected_rect);
    app.world_mut().entity_mut(entity).remove::<ImageNode>();
    settle(app);
    assert_image(
        app.world().get::<ImageNode>(entity).unwrap(),
        &expected_image,
    );
    app.world_mut().entity_mut(entity).remove::<UiTransform>();
    settle(app);
    assert_eq!(
        *app.world().get::<UiTransform>(entity).unwrap(),
        expected_transform
    );
    assert_eq!(logical_rect(app.world(), entity), expected_rect);
}
