//! Native Bevy rendering projection of the authoritative FrameRegistry.

use crate::font_registry::FontRegistry;
use crate::frame::{Frame, WidgetData};
use crate::plugin::UiState;
use crate::render::{LoadedTexture, UiCamera, UiFrameOrder};
use crate::render_texture::{BlpLoaderRes, load_texture_source};
use crate::widgets::{font_string::GameFont, texture::TextureSource};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub mod caret;
#[cfg(test)]
mod tests;

mod images;
pub(crate) mod layout;
mod text;

/// Stable logical identity; all authored properties remain in FrameRegistry.
#[derive(Component, Clone, Copy, Debug)]
pub struct RegistryNode(pub u64);

#[derive(Component)]
struct Canvas;
#[derive(Component)]
struct RegistryImage {
    frame_id: u64,
    key: u32,
}
#[derive(Component)]
pub struct RegistryText {
    pub frame_id: u64,
    pub key: u32,
    bounds: Entity,
}

pub(crate) struct ImagePart {
    pub key: u32,
    pub node: Node,
    pub image: ImageNode,
    pub transform: UiTransform,
    pub z: i32,
}

pub(crate) struct TextPart {
    pub key: u32,
    pub node: Node,
    pub text: String,
    pub font: TextFont,
    pub layout: TextLayout,
    pub color: TextColor,
    pub z: i32,
}

#[derive(SystemParam)]
pub(crate) struct NativeAssets<'w, 's> {
    images: Option<ResMut<'w, Assets<Image>>>,
    fonts: ResMut<'w, Assets<Font>>,
    font_registry: ResMut<'w, FontRegistry>,
    loader: Option<Res<'w, BlpLoaderRes>>,
    textures: Local<'s, HashMap<u32, Handle<Image>>>,
    files: Local<'s, HashMap<String, Handle<Image>>>,
    missing_textures: Local<'s, HashSet<u32>>,
    missing_files: Local<'s, HashSet<String>>,
    frame_z: Local<'s, f32>,
}

impl NativeAssets<'_, '_> {
    pub(crate) fn frame_z(&self) -> f32 {
        *self.frame_z
    }
    pub(crate) fn load(&mut self, source: &TextureSource) -> Option<LoadedTexture> {
        load_texture_source(
            source,
            &mut self.images,
            &mut self.textures,
            &mut self.files,
            &mut self.missing_textures,
            &mut self.missing_files,
            self.loader.as_deref(),
        )
    }
    pub(crate) fn font(&mut self, font: GameFont) -> Handle<Font> {
        self.font_registry.get(font, &mut self.fonts)
    }
    pub(crate) fn image_size(&self, handle: &Handle<Image>) -> Option<Vec2> {
        self.images
            .as_ref()?
            .get(handle)
            .map(|image| Vec2::new(image.width() as f32, image.height() as f32))
    }
}

#[derive(SystemParam)]
pub(crate) struct ProjectionQueries<'w, 's> {
    canvas: Query<'w, 's, (Entity, &'static UiTransform), With<Canvas>>,
    frames: Query<
        'w,
        's,
        (
            Entity,
            &'static RegistryNode,
            &'static Node,
            &'static ChildOf,
            &'static GlobalZIndex,
            Option<&'static Name>,
        ),
    >,
    images: Query<
        'w,
        's,
        (
            Entity,
            &'static RegistryImage,
            &'static Node,
            &'static ImageNode,
            &'static UiTransform,
            &'static GlobalZIndex,
        ),
    >,
    texts: Query<
        'w,
        's,
        (
            Entity,
            &'static RegistryText,
            &'static Text,
            &'static TextFont,
            &'static TextLayout,
            &'static TextColor,
        ),
    >,
    nodes: Query<'w, 's, &'static Node>,
    layers: Query<'w, 's, &'static GlobalZIndex>,
    transforms: Query<'w, 's, &'static UiTransform>,
    computed: Query<'w, 's, &'static ComputedNode>,
    children: Query<'w, 's, &'static Children>,
}

pub(crate) fn sync_registry(
    mut state: ResMut<UiState>,
    order: Res<UiFrameOrder>,
    text_enabled: Res<crate::plugin::UiTextRenderEnabled>,
    cameras: Query<(Entity, &Projection), With<UiCamera>>,
    mut commands: Commands,
    mut assets: NativeAssets,
    mut ui_scale: ResMut<UiScale>,
    query: ProjectionQueries,
) {
    let Ok((camera, projection)) = cameras.single() else {
        return;
    };
    if let Projection::Orthographic(projection) = projection {
        ui_scale.set_if_neq(UiScale(projection.scale.recip()));
    }
    let canvas = sync_canvas(&mut commands, &query, camera);
    let frames = sync_frames(&state, &order, canvas, &mut commands, &query);
    sync_images(&state, &order, &frames, &mut assets, &mut commands, &query);
    sync_texts(
        &state,
        &order,
        &frames,
        text_enabled.0,
        &mut assets,
        &mut commands,
        &query,
    );
    state
        .bypass_change_detection()
        .registry
        .render_dirty
        .clear();
    state.bypass_change_detection().registry.rect_dirty.clear();
}

fn sync_canvas(commands: &mut Commands, query: &ProjectionQueries, camera: Entity) -> Entity {
    let transform = UiTransform::default();
    if let Ok((entity, old)) = query.canvas.single() {
        if *old != transform {
            commands.entity(entity).insert(transform);
        }
        return entity;
    }
    commands
        .spawn((
            Canvas,
            Name::new("RegistryUiCanvas"),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            transform,
            UiTargetCamera(camera),
            bevy::picking::Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .id()
}

fn sync_frames(
    state: &UiState,
    order: &UiFrameOrder,
    canvas: Entity,
    commands: &mut Commands,
    query: &ProjectionQueries,
) -> HashMap<u64, Entity> {
    let mut entities: HashMap<_, _> = query.frames.iter().map(|(e, id, ..)| (id.0, e)).collect();
    let removed: HashSet<_> = query
        .frames
        .iter()
        .filter(|(_, id, ..)| state.registry.get(id.0).is_none())
        .map(|(e, ..)| e)
        .collect();
    for (entity, id, ..) in &query.frames {
        if removed.contains(&entity) {
            entities.remove(&id.0);
        }
    }
    let mut ordered_frames: Vec<_> = state.registry.frames_iter().collect();
    ordered_frames.sort_by_key(|frame| frame.id);
    for frame in &ordered_frames {
        entities.entry(frame.id).or_insert_with(|| {
            commands
                .spawn((
                    RegistryNode(frame.id),
                    Node::default(),
                    ChildOf(canvas),
                    GlobalZIndex(0),
                    bevy::picking::Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .id()
        });
    }
    for frame in &ordered_frames {
        let entity = entities[&frame.id];
        let parent = match frame.anchor {
            crate::anchor::AnchorTarget::Screen => canvas,
            crate::anchor::AnchorTarget::Parent => {
                frame.parent_id.map_or(canvas, |id| entities[&id])
            }
        };
        let node = layout::node(frame);
        let transform = UiTransform {
            translation: frame.translation,
            scale: Vec2::splat(frame.scale),
            ..default()
        };
        if query
            .transforms
            .get(entity)
            .map_or(true, |old| *old != transform)
        {
            commands.entity(entity).insert(transform);
        };
        let z = GlobalZIndex(order.indices.get(&frame.id).copied().unwrap_or(0) as i32);
        if let Ok((_, _, old_node, old_parent, old_z, old_name)) = query.frames.get(entity) {
            if *old_node != node {
                commands.entity(entity).insert(node);
            }
            if old_parent.parent() != parent {
                commands.entity(entity).insert(ChildOf(parent));
            }
            if *old_z != z {
                commands.entity(entity).insert(z);
            }
            if old_name.map(Name::as_str) != frame.name.as_deref() {
                set_frame_name(commands, entity, frame.name.as_deref());
            }
        } else {
            commands.entity(entity).insert((node, ChildOf(parent), z));
            set_frame_name(commands, entity, frame.name.as_deref());
        }
    }
    synchronize_child_order(state, canvas, &entities, commands, query);
    for (entity, _, _, parent, ..) in &query.frames {
        if removed.contains(&entity) && !removed.contains(&parent.parent()) {
            commands.entity(entity).despawn();
        }
    }
    entities
}

fn synchronize_child_order(
    state: &UiState,
    canvas: Entity,
    frames: &HashMap<u64, Entity>,
    commands: &mut Commands,
    query: &ProjectionQueries,
) {
    let frame_entities: HashSet<_> = frames.values().copied().collect();
    let mut groups: HashMap<Entity, Vec<u64>> = HashMap::new();
    for frame in state.registry.frames_iter() {
        let parent = match frame.anchor {
            crate::anchor::AnchorTarget::Screen => canvas,
            crate::anchor::AnchorTarget::Parent => frame.parent_id.map_or(canvas, |id| frames[&id]),
        };
        groups.entry(parent).or_default().push(frame.id);
    }
    for (parent, mut ids) in groups {
        if parent == canvas {
            ids.sort_unstable();
        } else if let Some(frame) = state
            .registry
            .frames_iter()
            .find(|f| frames[&f.id] == parent)
        {
            ids.sort_by_key(|id| {
                frame
                    .children
                    .iter()
                    .position(|child| child == id)
                    .unwrap_or(usize::MAX)
            });
        }
        let mut desired: Vec<_> = ids.into_iter().map(|id| frames[&id]).collect();
        if let Ok(children) = query.children.get(parent) {
            desired.extend(
                children
                    .iter()
                    .filter(|child| !frame_entities.contains(child)),
            );
            if children.iter().eq(desired.iter().copied()) {
                continue;
            }
        }
        commands.entity(parent).replace_children(&desired);
    }
}

fn projection_frame(frame: &Frame, entity: Entity, query: &ProjectionQueries) -> Frame {
    let mut projected = frame.clone();
    let size = query
        .computed
        .get(entity)
        .map(|node| node.size * node.inverse_scale_factor)
        .unwrap_or(Vec2::new(frame.width.value(), frame.height.value()));
    projected.layout_rect = Some(crate::layout::LayoutRect {
        x: 0.0,
        y: 0.0,
        width: size.x,
        height: size.y,
    });
    projected
}

fn set_frame_name(commands: &mut Commands, entity: Entity, name: Option<&str>) {
    if let Some(name) = name {
        commands.entity(entity).insert(Name::new(name.to_owned()));
    } else {
        commands.entity(entity).remove::<Name>();
    }
}

fn sync_images(
    state: &UiState,
    order: &UiFrameOrder,
    frames: &HashMap<u64, Entity>,
    assets: &mut NativeAssets,
    commands: &mut Commands,
    query: &ProjectionQueries,
) {
    let existing: HashMap<_, _> = query
        .images
        .iter()
        .map(|(entity, part, ..)| ((part.frame_id, part.key), entity))
        .collect();
    let mut seen = HashSet::new();
    for frame in state.registry.frames_iter().filter(|f| f.visible) {
        *assets.frame_z = order.indices.get(&frame.id).copied().unwrap_or(0) as f32 * 0.001;
        let projected = projection_frame(frame, frames[&frame.id], query);
        for part in images::project_images(&projected, assets) {
            let key = (frame.id, part.key);
            seen.insert(key);
            if let Some(&entity) = existing.get(&key) {
                let (_, _, node, image, transform, z) = query.images.get(entity).unwrap();
                if *node != part.node {
                    commands.entity(entity).insert(part.node);
                }
                if !images_equal(image, &part.image) {
                    commands.entity(entity).insert(part.image);
                }
                if *transform != part.transform {
                    commands.entity(entity).insert(part.transform);
                }
                if z.0 != part.z {
                    commands.entity(entity).insert(GlobalZIndex(part.z));
                }
            } else {
                commands.spawn((
                    RegistryImage {
                        frame_id: frame.id,
                        key: part.key,
                    },
                    part.node,
                    part.image,
                    part.transform,
                    GlobalZIndex(part.z),
                    ChildOf(frames[&frame.id]),
                    bevy::picking::Pickable::IGNORE,
                ));
            }
        }
    }
    for (key, entity) in existing {
        if !seen.contains(&key) && frames.contains_key(&key.0) {
            commands.entity(entity).despawn();
        }
    }
}

fn images_equal(a: &ImageNode, b: &ImageNode) -> bool {
    a.image == b.image
        && a.color == b.color
        && a.rect == b.rect
        && a.flip_x == b.flip_x
        && a.flip_y == b.flip_y
        && a.image_mode == b.image_mode
        && a.visual_box == b.visual_box
}

fn text_content_node(frame: &Frame, part: &TextPart) -> Node {
    Node {
        width: if frame.width == crate::frame::Dimension::Auto {
            Val::Auto
        } else {
            percent(100)
        },
        min_height: match part.font.font_size {
            FontSize::Px(size) => px(size),
            _ => Val::Auto,
        },
        flex_shrink: 0.0,
        ..default()
    }
}

fn sync_texts(
    state: &UiState,
    order: &UiFrameOrder,
    frames: &HashMap<u64, Entity>,
    enabled: bool,
    assets: &mut NativeAssets,
    commands: &mut Commands,
    query: &ProjectionQueries,
) {
    let existing: HashMap<_, _> = query
        .texts
        .iter()
        .map(|(entity, part, ..)| ((part.frame_id, part.key), (entity, part.bounds)))
        .collect();
    let mut seen = HashSet::new();
    for frame in state
        .registry
        .frames_iter()
        .filter(|f| enabled && f.visible)
    {
        *assets.frame_z = order.indices.get(&frame.id).copied().unwrap_or(0) as f32 * 0.001;
        let projected = projection_frame(frame, frames[&frame.id], query);
        for part in text::project_text(&projected, assets) {
            let content_node = text_content_node(frame, &part);
            let key = (frame.id, part.key);
            seen.insert(key);
            if let Some(&(entity, bounds)) = existing.get(&key) {
                let (_, _, old_text, font, layout, color) = query.texts.get(entity).unwrap();
                if *query.nodes.get(entity).unwrap() != content_node {
                    commands.entity(entity).insert(content_node);
                }
                if *query.nodes.get(bounds).unwrap() != part.node {
                    commands.entity(bounds).insert(part.node);
                }
                if query.layers.get(bounds).unwrap().0 != part.z {
                    commands.entity(bounds).insert(GlobalZIndex(part.z));
                }
                if old_text.0 != part.text {
                    commands.entity(entity).insert(Text::new(part.text));
                }
                if *font != part.font {
                    commands.entity(entity).insert(part.font);
                }
                if layout.justify != part.layout.justify
                    || layout.linebreak != part.layout.linebreak
                {
                    commands.entity(entity).insert(part.layout);
                }
                if *color != part.color {
                    commands.entity(entity).insert(part.color);
                }
            } else {
                let bounds = commands
                    .spawn((
                        part.node,
                        GlobalZIndex(part.z),
                        ChildOf(frames[&frame.id]),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                let entity = commands
                    .spawn((
                        RegistryText {
                            frame_id: frame.id,
                            key: part.key,
                            bounds,
                        },
                        Text::new(part.text),
                        part.font,
                        part.layout,
                        part.color,
                        content_node,
                        ChildOf(bounds),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                if part.key == 0 && matches!(frame.widget_data, Some(WidgetData::EditBox(_))) {
                    caret::spawn_caret(commands, entity, bounds, frame.id);
                }
            }
        }
    }
    for (key, (_, bounds)) in existing {
        if !seen.contains(&key) && frames.contains_key(&key.0) {
            commands.entity(bounds).despawn();
        }
    }
}
