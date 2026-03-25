//! Text shadow and outline rendering.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::{Font, TextBounds, TextFont, TextLayout};
use std::collections::{HashMap, HashSet};

use crate::font_registry::FontRegistry;
use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::widgets::font_string::{GameFont, JustifyH, JustifyV, Outline};

use super::render::{UI_RENDER_LAYER, UiText, build_sorted_visible_frame_ids};

/// Marker for shadow text entities.
#[derive(Component)]
pub struct UiTextShadow(pub u64);

/// Marker for outline text entities.
#[derive(Component)]
pub struct UiTextOutline(pub u64);

/// Syncs text shadows — a dark copy of text rendered behind the main text.
pub fn sync_ui_text_shadows(
    state: Res<UiState>,
    mut commands: Commands,
    mut font_assets: ResMut<Assets<Font>>,
    mut font_registry: ResMut<FontRegistry>,
    mut shadows: Query<(
        Entity,
        &UiTextShadow,
        &mut Text2d,
        &mut TextLayout,
        &mut TextBounds,
        &mut TextFont,
        &mut TextColor,
        &mut Transform,
        &mut Anchor,
    )>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;
    let sort_map: std::collections::HashMap<u64, usize> = build_sorted_visible_frame_ids(&state)
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();
    let mut existing: HashSet<u64> = HashSet::new();

    for (
        entity,
        shadow,
        mut text,
        mut layout,
        mut bounds,
        mut font,
        mut color,
        mut transform,
        mut anchor,
    ) in shadows.iter_mut()
    {
        let Some(props) = extract_shadow(state.registry.get(shadow.0)) else {
            commands.entity(entity).despawn();
            continue;
        };
        existing.insert(shadow.0);
        if let Some(frame) = state.registry.get(shadow.0) {
            update_shadow_entity(
                frame,
                &props,
                &mut text,
                &mut layout,
                &mut bounds,
                &mut font,
                &mut color,
                &mut font_assets,
                &mut font_registry,
            );
            *anchor = super::render_text::text_anchor_for_frame(frame);
            if let Some(&sort_idx) = sort_map.get(&shadow.0) {
                *transform = shadow_transform(frame, &props, screen_w, screen_h, sort_idx);
            }
        }
    }

    spawn_missing_shadows(
        &state,
        &sort_map,
        &existing,
        screen_w,
        screen_h,
        &mut commands,
        &mut font_assets,
        &mut font_registry,
    );
}

fn spawn_missing_shadows(
    state: &UiState,
    sort_map: &HashMap<u64, usize>,
    existing: &HashSet<u64>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    font_assets: &mut Assets<Font>,
    font_registry: &mut FontRegistry,
) {
    for frame in state.registry.frames_iter() {
        if existing.contains(&frame.id) {
            continue;
        }
        let Some(props) = extract_shadow(Some(frame)) else {
            continue;
        };
        let Some(&sort_idx) = sort_map.get(&frame.id) else {
            continue;
        };
        let transform = shadow_transform(frame, &props, screen_w, screen_h, sort_idx);
        let [r, g, b, a] = props.shadow_color;
        let font = font_registry.get(props.font, font_assets);
        commands.spawn((
            Text2d::new(props.content),
            super::render_text::text_layout(frame),
            super::render_text::text_bounds(frame),
            TextFont {
                font,
                font_size: props.font_size,
                ..default()
            },
            TextColor(Color::srgba(r, g, b, a * frame.effective_alpha)),
            super::render_text::text_anchor_for_frame(frame),
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiText(frame.id),
            UiTextShadow(frame.id),
        ));
    }
}

struct ShadowProps {
    content: String,
    font: GameFont,
    font_size: f32,
    shadow_color: [f32; 4],
    shadow_offset: [f32; 2],
    justify_h: JustifyH,
    justify_v: JustifyV,
}

fn extract_shadow(frame: Option<&crate::frame::Frame>) -> Option<ShadowProps> {
    let frame = frame?;
    if !frame.visible {
        return None;
    }
    let Some(WidgetData::FontString(fs)) = &frame.widget_data else {
        return None;
    };
    if fs.text.is_empty() {
        return None;
    }
    let shadow_color = fs.shadow_color?;
    Some(ShadowProps {
        content: fs.text.clone(),
        font: fs.font,
        font_size: fs.font_size,
        shadow_color,
        shadow_offset: fs.shadow_offset,
        justify_h: fs.justify_h,
        justify_v: fs.justify_v,
    })
}

fn update_shadow_entity(
    frame: &crate::frame::Frame,
    props: &ShadowProps,
    text: &mut Text2d,
    layout: &mut TextLayout,
    bounds: &mut TextBounds,
    font: &mut TextFont,
    color: &mut TextColor,
    font_assets: &mut Assets<Font>,
    font_registry: &mut FontRegistry,
) {
    *text = Text2d::new(&props.content);
    *layout = super::render_text::text_layout(frame);
    *bounds = super::render_text::text_bounds(frame);
    font.font_size = props.font_size;
    font.font = font_registry.get(props.font, font_assets);
    let [r, g, b, a] = props.shadow_color;
    *color = TextColor(Color::srgba(r, g, b, a));
}

fn shadow_transform(
    frame: &crate::frame::Frame,
    props: &ShadowProps,
    screen_w: f32,
    screen_h: f32,
    sort_idx: usize,
) -> Transform {
    let mut t = super::render_text::text_transform(
        frame,
        screen_w,
        screen_h,
        props.justify_h,
        props.justify_v,
        sort_idx,
    );
    t.translation.x += props.shadow_offset[0];
    t.translation.y -= props.shadow_offset[1];
    t.translation.z = sort_idx as f32 * 0.001 + 0.0006;
    t
}

/// Syncs text outlines — dark copies of text at directional offsets.
pub fn sync_ui_text_outlines(
    state: Res<UiState>,
    mut commands: Commands,
    mut font_assets: ResMut<Assets<Font>>,
    mut font_registry: ResMut<FontRegistry>,
    outlines: Query<(Entity, &UiTextOutline)>,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;
    let sort_map: HashMap<u64, usize> = build_sorted_visible_frame_ids(&state)
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();

    let mut existing: HashSet<u64> = HashSet::new();
    for (entity, outline) in &outlines {
        if has_outline_frame(&state, outline.0) {
            existing.insert(outline.0);
        } else {
            commands.entity(entity).despawn();
        }
    }

    for frame in state.registry.frames_iter() {
        if !frame.visible || existing.contains(&frame.id) || !has_outline(frame) {
            continue;
        }
        spawn_outlines(
            frame,
            sort_map[&frame.id],
            screen_w,
            screen_h,
            &mut commands,
            &mut font_assets,
            &mut font_registry,
        );
    }
}

fn has_outline_frame(state: &UiState, id: u64) -> bool {
    state
        .registry
        .get(id)
        .is_some_and(|f| f.visible && has_outline(f))
}

fn has_outline(frame: &crate::frame::Frame) -> bool {
    matches!(
        &frame.widget_data,
        Some(WidgetData::FontString(fs)) if fs.outline != Outline::None && !fs.text.is_empty()
    )
}

fn spawn_outlines(
    frame: &crate::frame::Frame,
    sort_idx: usize,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    font_assets: &mut Assets<Font>,
    font_registry: &mut FontRegistry,
) {
    let Some(WidgetData::FontString(fs)) = &frame.widget_data else {
        return;
    };
    let base = super::render_text::text_transform(
        frame,
        screen_w,
        screen_h,
        fs.justify_h,
        fs.justify_v,
        sort_idx,
    );
    let alpha = frame.effective_alpha;
    let font = font_registry.get(fs.font, font_assets);

    for &(dx, dy) in outline_offsets(fs.outline) {
        let mut transform = base;
        transform.translation.x += dx;
        transform.translation.y += dy;
        transform.translation.z = sort_idx as f32 * 0.001 + 0.0005;
        commands.spawn((
            Text2d::new(&fs.text),
            super::render_text::text_layout(frame),
            super::render_text::text_bounds(frame),
            TextFont {
                font: font.clone(),
                font_size: fs.font_size,
                ..default()
            },
            TextColor(Color::srgba(0.0, 0.0, 0.0, alpha)),
            super::render_text::text_anchor_for_frame(frame),
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiText(frame.id),
            UiTextOutline(frame.id),
        ));
    }
}

fn outline_offsets(outline: Outline) -> &'static [(f32, f32)] {
    match outline {
        Outline::None => &[],
        Outline::Outline => &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)],
        Outline::ThickOutline => &[
            (-2.0, 0.0),
            (2.0, 0.0),
            (0.0, -2.0),
            (0.0, 2.0),
            (-1.4, -1.4),
            (1.4, -1.4),
            (-1.4, 1.4),
            (1.4, 1.4),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Dimension, WidgetData};
    use crate::plugin::UiPlugin;
    use crate::widgets::font_string::{FontStringData, Outline as FsOutline};

    fn make_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Assets::<Font>::default());
        app.add_plugins(UiPlugin);
        app.update();
        app
    }

    fn make_font_string_frame(app: &mut App, name: &str, fs: FontStringData) {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let id = ui.registry.create_frame(name, None);
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(100.0);
        frame.height = Dimension::Fixed(20.0);
        frame.widget_data = Some(WidgetData::FontString(fs));
    }

    #[test]
    fn shadow_color_spawns_shadow_entity() {
        let mut app = make_test_app();
        make_font_string_frame(
            &mut app,
            "ShadowText",
            FontStringData {
                text: "Hello".into(),
                shadow_color: Some([0.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            },
        );
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiTextShadow>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn no_shadow_color_spawns_no_shadow() {
        let mut app = make_test_app();
        make_font_string_frame(
            &mut app,
            "NoShadowText",
            FontStringData {
                text: "Hello".into(),
                shadow_color: None,
                ..Default::default()
            },
        );
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiTextShadow>>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn outline_spawns_4_outline_entities() {
        let mut app = make_test_app();
        make_font_string_frame(
            &mut app,
            "OutlineText",
            FontStringData {
                text: "Hi".into(),
                outline: FsOutline::Outline,
                ..Default::default()
            },
        );
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiTextOutline>>();
        assert_eq!(q.iter(app.world()).count(), 4);
    }

    #[test]
    fn thick_outline_spawns_8_outline_entities() {
        let mut app = make_test_app();
        make_font_string_frame(
            &mut app,
            "ThickOutlineText",
            FontStringData {
                text: "Hi".into(),
                outline: FsOutline::ThickOutline,
                ..Default::default()
            },
        );
        app.update();
        let mut q = app.world_mut().query_filtered::<(), With<UiTextOutline>>();
        assert_eq!(q.iter(app.world()).count(), 8);
    }
}
