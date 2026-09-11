//! Insertion caret projected from registry-owned EditBoxData.
use crate::{frame::WidgetData, plugin::UiState};
use bevy::{math::Affine2, prelude::*, text::ComputedTextBlock};
use parley::{Affinity, editing::Cursor};
use std::hash::{Hash, Hasher};

/// The host sets this while a modal blocks edit-box input.
#[derive(Resource, Default)]
pub struct UiCaretBlocked(pub bool);

#[derive(Component)]
struct Caret {
    frame: u64,
    text: Entity,
    bounds: Entity,
    snapshot: Option<(bool, usize, u64)>,
    started: f64,
}

pub(super) fn spawn_caret(commands: &mut Commands, text: Entity, bounds: Entity, frame: u64) {
    commands.spawn((
        Caret {
            frame,
            text,
            bounds,
            snapshot: None,
            started: 0.0,
        },
        Node {
            position_type: PositionType::Absolute,
            width: px(2),
            height: px(20),
            ..default()
        },
        BackgroundColor(Color::NONE),
        ZIndex(1),
        ChildOf(bounds),
        bevy::picking::Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
}

pub fn sync_carets(world: &mut World) {
    let now = world.resource::<Time>().elapsed_secs_f64();
    let blocked = world.get_resource::<UiCaretBlocked>().is_some_and(|b| b.0);
    let carets: Vec<_> = world
        .query::<(Entity, &Caret)>()
        .iter(world)
        .map(|(entity, caret)| (entity, caret.frame, caret.text, caret.bounds))
        .collect();
    for (entity, frame, text, bounds) in carets {
        let Some((snapshot, speed, insets)) = edit_state(world, frame, blocked) else {
            world.get_mut::<BackgroundColor>(entity).unwrap().0 = Color::NONE;
            continue;
        };
        let visible = {
            let mut caret = world.get_mut::<Caret>(entity).unwrap();
            if caret.snapshot != Some(snapshot) {
                caret.snapshot = Some(snapshot);
                caret.started = now;
            }
            snapshot.0
                && (speed <= 0.0
                    || (now - caret.started).rem_euclid(f64::from(speed) * 2.0) < f64::from(speed))
        };
        let geometry = geometry(world, text, snapshot.1);
        let color = world.get::<TextColor>(text).map(|c| c.0);
        world.get_mut::<BackgroundColor>(entity).unwrap().0 =
            match (visible, geometry.is_some(), color) {
                (true, true, Some(color)) => color,
                _ => Color::NONE,
            };
        if let Some((transform, size, inverse_scale)) = geometry {
            *world.get_mut::<UiGlobalTransform>(entity).unwrap() = transform;
            let mut node = world.get_mut::<ComputedNode>(entity).unwrap();
            node.size = size;
            node.unrounded_size = size;
            node.inverse_scale_factor = inverse_scale;
            if let Some(clip) = edit_clip(world, bounds, insets) {
                world.entity_mut(entity).insert(CalculatedClip { clip });
            }
        }
    }
}

fn edit_state(
    world: &World,
    id: u64,
    blocked: bool,
) -> Option<((bool, usize, u64), f32, [f32; 4])> {
    let state = world.get_resource::<UiState>()?;
    let frame = state.registry.get(id)?;
    let WidgetData::EditBox(edit) = frame.widget_data.as_ref()? else {
        return None;
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    edit.text.hash(&mut hash);
    Some((
        (
            frame.visible && !blocked && state.focused_frame == Some(id),
            edit.cursor_position,
            hash.finish(),
        ),
        edit.blink_speed,
        edit.text_insets,
    ))
}

fn geometry(world: &World, text: Entity, cursor: usize) -> Option<(UiGlobalTransform, Vec2, f32)> {
    let node = world.get::<ComputedNode>(text)?;
    let transform = world.get::<UiGlobalTransform>(text)?;
    let scale = node.inverse_scale_factor.recip();
    let width = 2.0 * scale;
    let rect = if world.get::<Text>(text)?.0.is_empty() {
        let FontSize::Px(size) = world.get::<TextFont>(text)?.font_size else {
            return None;
        };
        Rect::from_corners(Vec2::ZERO, Vec2::new(width, size * scale))
    } else {
        let layout = world.get::<ComputedTextBlock>(text)?.buffer();
        if layout.is_empty() {
            return None;
        }
        let rect =
            Cursor::from_byte_index(layout, cursor, Affinity::Downstream).geometry(layout, width);
        Rect::from_corners(
            Vec2::new(rect.x0 as f32, rect.y0 as f32),
            Vec2::new(rect.x1 as f32, rect.y1 as f32),
        )
    };
    Some((
        UiGlobalTransform::from(
            Affine2::from(transform) * Affine2::from_translation(rect.center() - node.size / 2.0),
        ),
        rect.size(),
        node.inverse_scale_factor,
    ))
}

fn edit_clip(world: &World, bounds: Entity, insets: [f32; 4]) -> Option<Rect> {
    let frame = world.get::<ChildOf>(bounds)?.parent();
    let node = world.get::<ComputedNode>(frame)?;
    let transform = Affine2::from(world.get::<UiGlobalTransform>(frame)?);
    let scale = node.inverse_scale_factor.recip();
    let min =
        transform.transform_point2(-node.size / 2.0 + Vec2::new(insets[0], insets[2]) * scale);
    let max = transform.transform_point2(node.size / 2.0 - Vec2::new(insets[1], insets[3]) * scale);
    let mut clip = Rect::from_corners(min, max);
    if let Some(parent_clip) = world.get::<CalculatedClip>(bounds) {
        clip = clip.intersect(parent_clip.clip);
    }
    Some(clip)
}
