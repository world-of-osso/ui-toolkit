//! Native-only preservation fixtures: observe registry identities after real UI layout.

use super::{RegistryImage, RegistryText, tests::app_with_real_fonts};
use crate::anchor::AnchorTarget;
use crate::frame::{Backdrop, Border, Dimension, NineSlice, ThreeSlice, WidgetData, WidgetType};
use crate::plugin::UiState;
use crate::widgets::font_string::{FontStringData, GameFont, JustifyH, JustifyV, Outline};
use crate::widgets::texture::TextureSource;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::text::{Justify, LineBreak, TextLayoutInfo};
use ui_toolkit_core::layout_values::PositionType;

fn settle(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}

fn frame(app: &mut App, name: &str, width: f32, height: f32) -> u64 {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let id = ui.registry.create_frame(name, None);
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(width);
    frame.height = Dimension::Fixed(height);
    ui.registry
        .set_pos_type(id, PositionType::Absolute)
        .unwrap();
    ui.registry.set_anchor(id, AnchorTarget::Parent).unwrap();
    ui.registry.set_pos(id, 40.0, 50.0).unwrap();
    id
}

fn resize(app: &mut App, id: u64, width: f32, height: f32) {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let frame = ui.registry.get_mut(id).unwrap();
    frame.width = Dimension::Fixed(width);
    frame.height = Dimension::Fixed(height);
    ui.registry.mark_rect_dirty(id);
}

fn image(world: &mut World, id: u64, key: u32) -> Option<Entity> {
    world
        .query::<(Entity, &RegistryImage)>()
        .iter(world)
        .find(|(_, part)| part.frame_id == id && part.key == key)
        .map(|(entity, _)| entity)
}

fn text(world: &mut World, id: u64, key: u32) -> Option<(Entity, Entity)> {
    world
        .query::<(Entity, &RegistryText)>()
        .iter(world)
        .find(|(_, part)| part.frame_id == id && part.key == key)
        .map(|(entity, part)| (entity, part.bounds))
}

fn rect(world: &World, entity: Entity) -> Rect {
    let node = world.get::<ComputedNode>(entity).unwrap();
    let transform = Affine2::from(world.get::<UiGlobalTransform>(entity).unwrap());
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor,
        node.size * node.inverse_scale_factor,
    )
}

fn assert_rect(world: &World, entity: Entity, expected: Rect) {
    let actual = rect(world, entity);
    assert!(
        (actual.min - expected.min).abs().max_element() < 0.6
            && (actual.max - expected.max).abs().max_element() < 0.6,
        "native bounds {actual:?}, expected {expected:?}"
    );
}

fn assert_image(app: &mut App, id: u64, key: u32, bounds: Rect, color: Color, uv: Option<Rect>) {
    let entity = image(app.world_mut(), id, key).expect("native image part");
    assert_rect(app.world(), entity, bounds);
    let node = app.world().get::<ImageNode>(entity).unwrap();
    assert_eq!(node.color, color);
    assert_eq!(node.rect, uv);
}

fn art(app: &mut App, width: u32, height: u32, rgba: [u8; 4]) -> TextureSource {
    let bytes = rgba.repeat((width * height) as usize);
    let id = app
        .world_mut()
        .resource_mut::<UiState>()
        .registry
        .create_dynamic_texture(width, height, bytes)
        .unwrap();
    TextureSource::Dynamic(id)
}

fn assert_pixels(app: &mut App, id: u64, key: u32, width: u32, height: u32, rgba: [u8; 4]) {
    let entity = image(app.world_mut(), id, key).unwrap();
    let handle = &app.world().get::<ImageNode>(entity).unwrap().image;
    let asset = app.world().resource::<Assets<Image>>().get(handle).unwrap();
    assert_eq!((asset.width(), asset.height()), (width, height));
    assert_eq!(
        asset.data.as_deref(),
        Some(rgba.repeat((width * height) as usize).as_slice())
    );
}

fn label(app: &mut App, name: &str, width: f32, height: f32, data: FontStringData) -> u64 {
    let id = frame(app, name, width, height);
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let frame = ui.registry.get_mut(id).unwrap();
    frame.widget_type = WidgetType::FontString;
    frame.widget_data = Some(WidgetData::FontString(data));
    id
}

fn edit_label(app: &mut App, id: u64, edit: impl FnOnce(&mut FontStringData)) {
    let mut ui = app.world_mut().resource_mut::<UiState>();
    let Some(WidgetData::FontString(data)) = &mut ui.registry.get_mut(id).unwrap().widget_data
    else {
        panic!("font string fixture");
    };
    edit(data);
}

#[test]
fn solid_panel_updates_after_idle_and_backgroundless_hidden_removed_panels_leave_no_fill() {
    let mut app = app_with_real_fonts(1.0);
    let panel = frame(&mut app, "SolidPanel", 120.0, 60.0);
    let empty = frame(&mut app, "BackgroundlessPanel", 80.0, 30.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(panel)
        .unwrap()
        .background_color = Some([0.2, 0.4, 0.6, 0.8]);
    settle(&mut app);
    assert_image(
        &mut app,
        panel,
        0,
        Rect::new(40.0, 50.0, 160.0, 110.0),
        Color::srgba(0.2, 0.4, 0.6, 0.8),
        None,
    );
    assert!(image(app.world_mut(), empty, 0).is_none());
    settle(&mut app);
    resize(&mut app, panel, 170.0, 90.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(panel)
        .unwrap()
        .background_color = Some([0.7, 0.3, 0.1, 0.5]);
    settle(&mut app);
    assert_image(
        &mut app,
        panel,
        0,
        Rect::new(40.0, 50.0, 210.0, 140.0),
        Color::srgba(0.7, 0.3, 0.1, 0.5),
        None,
    );
    let painted = image(app.world_mut(), panel, 0).unwrap();
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(panel, true);
    settle(&mut app);
    assert!(image(app.world_mut(), panel, 0).is_none());
    assert!(app.world().get_entity(painted).is_err());
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_hidden(panel, false);
    settle(&mut app);
    assert_image(
        &mut app,
        panel,
        0,
        Rect::new(40.0, 50.0, 210.0, 140.0),
        Color::srgba(0.7, 0.3, 0.1, 0.5),
        None,
    );
    let restored = image(app.world_mut(), panel, 0).unwrap();
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .remove_frame_tree(panel);
    settle(&mut app);
    assert!(image(app.world_mut(), panel, 0).is_none());
    assert!(app.world().get_entity(restored).is_err());
    assert!(
        app.world()
            .resource::<UiState>()
            .registry
            .get(empty)
            .is_some()
    );
}

#[test]
fn css_and_backdrop_edges_keep_inside_outside_bounds_colors_and_resize_after_idle() {
    let mut app = app_with_real_fonts(1.0);
    let panel = frame(&mut app, "BorderPanel", 100.0, 50.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        let frame = ui.registry.get_mut(panel).unwrap();
        frame.border = Some(Border {
            width: 2.0,
            color: [0.2, 0.5, 0.8, 0.6],
        });
        frame.backdrop = Some(Backdrop {
            edge_size: 3.0,
            border_color: Some([0.8, 0.3, 0.1, 0.4]),
            ..default()
        });
        ui.registry.set_alpha(panel, 0.5);
    }
    settle(&mut app);
    for (key, bounds) in [
        (30, Rect::new(40.0, 50.0, 140.0, 52.0)),
        (31, Rect::new(138.0, 50.0, 140.0, 100.0)),
        (32, Rect::new(40.0, 98.0, 140.0, 100.0)),
        (33, Rect::new(40.0, 50.0, 42.0, 100.0)),
    ] {
        assert_image(
            &mut app,
            panel,
            key,
            bounds,
            Color::srgba(0.2, 0.5, 0.8, 0.3),
            None,
        );
    }
    for (key, bounds) in [
        (20, Rect::new(37.0, 47.0, 143.0, 50.0)),
        (21, Rect::new(37.0, 100.0, 143.0, 103.0)),
        (22, Rect::new(37.0, 50.0, 40.0, 100.0)),
        (23, Rect::new(140.0, 50.0, 143.0, 100.0)),
    ] {
        assert_image(
            &mut app,
            panel,
            key,
            bounds,
            Color::srgba(0.8, 0.3, 0.1, 0.2),
            None,
        );
    }
    settle(&mut app);
    resize(&mut app, panel, 150.0, 70.0);
    {
        let mut ui = app.world_mut().resource_mut::<UiState>();
        ui.registry.get_mut(panel).unwrap().border = Some(Border {
            width: 4.0,
            color: [0.1, 0.7, 0.2, 1.0],
        });
    }
    settle(&mut app);
    assert_image(
        &mut app,
        panel,
        31,
        Rect::new(186.0, 50.0, 190.0, 120.0),
        Color::srgba(0.1, 0.7, 0.2, 0.5),
        None,
    );
    assert_image(
        &mut app,
        panel,
        21,
        Rect::new(37.0, 120.0, 193.0, 123.0),
        Color::srgba(0.8, 0.3, 0.1, 0.2),
        None,
    );
}

#[test]
fn nine_slice_asymmetric_uvs_preserve_corner_art_while_center_and_edges_resize() {
    let mut app = app_with_real_fonts(1.0);
    let source = art(&mut app, 32, 24, [180, 80, 20, 255]);
    let panel = frame(&mut app, "NineSlicePanel", 100.0, 60.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(panel)
        .unwrap()
        .nine_slice = Some(NineSlice {
        edge_sizes: Some([5.0, 7.0, 11.0, 13.0]),
        uv_edge_sizes: Some([2.0, 3.0, 4.0, 5.0]),
        texture: Some(source),
        bg_color: [0.3, 0.4, 0.5, 0.7],
        border_color: [0.9, 0.8, 0.6, 1.0],
        ..default()
    });
    settle(&mut app);
    for (index, bounds, uv) in [
        (0, [40.0, 50.0, 45.0, 57.0], [0.0, 0.0, 2.0, 3.0]),
        (1, [45.0, 50.0, 129.0, 57.0], [2.0, 0.0, 28.0, 3.0]),
        (2, [129.0, 50.0, 140.0, 57.0], [28.0, 0.0, 32.0, 3.0]),
        (3, [40.0, 57.0, 45.0, 97.0], [0.0, 3.0, 2.0, 19.0]),
        (4, [45.0, 57.0, 129.0, 97.0], [2.0, 3.0, 28.0, 19.0]),
        (5, [129.0, 57.0, 140.0, 97.0], [28.0, 3.0, 32.0, 19.0]),
        (6, [40.0, 97.0, 45.0, 110.0], [0.0, 19.0, 2.0, 24.0]),
        (7, [45.0, 97.0, 129.0, 110.0], [2.0, 19.0, 28.0, 24.0]),
        (8, [129.0, 97.0, 140.0, 110.0], [28.0, 19.0, 32.0, 24.0]),
    ] {
        let color = if index == 4 {
            Color::srgba(0.3, 0.4, 0.5, 0.7)
        } else {
            Color::srgba(0.9, 0.8, 0.6, 1.0)
        };
        assert_image(
            &mut app,
            panel,
            100 + index,
            Rect::new(bounds[0], bounds[1], bounds[2], bounds[3]),
            color,
            Some(Rect::new(uv[0], uv[1], uv[2], uv[3])),
        );
        assert_pixels(&mut app, panel, 100 + index, 32, 24, [180, 80, 20, 255]);
    }
    settle(&mut app);
    resize(&mut app, panel, 160.0, 90.0);
    settle(&mut app);
    assert_image(
        &mut app,
        panel,
        104,
        Rect::new(45.0, 57.0, 189.0, 127.0),
        Color::srgba(0.3, 0.4, 0.5, 0.7),
        Some(Rect::new(2.0, 3.0, 28.0, 19.0)),
    );
    assert_image(
        &mut app,
        panel,
        108,
        Rect::new(189.0, 127.0, 200.0, 140.0),
        Color::srgba(0.9, 0.8, 0.6, 1.0),
        Some(Rect::new(28.0, 19.0, 32.0, 24.0)),
    );
    assert_pixels(&mut app, panel, 108, 32, 24, [180, 80, 20, 255]);
}

#[test]
fn three_slice_keeps_distinct_cap_art_and_full_uvs_when_width_height_and_caps_change() {
    let mut app = app_with_real_fonts(1.0);
    let left = art(&mut app, 4, 6, [200, 20, 10, 255]);
    let center = art(&mut app, 2, 6, [20, 200, 10, 255]);
    let right = art(&mut app, 4, 6, [20, 10, 200, 255]);
    let panel = frame(&mut app, "ThreeSlicePanel", 100.0, 30.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(panel)
        .unwrap()
        .three_slice = Some(ThreeSlice {
        cap_width: 10.0,
        left,
        center,
        right,
        color: [0.7, 0.8, 0.9, 0.6],
    });
    settle(&mut app);
    for (key, bounds, width, rgba) in [
        (
            200,
            Rect::new(40.0, 50.0, 50.0, 80.0),
            4,
            [200, 20, 10, 255],
        ),
        (
            201,
            Rect::new(50.0, 50.0, 130.0, 80.0),
            2,
            [20, 200, 10, 255],
        ),
        (
            202,
            Rect::new(130.0, 50.0, 140.0, 80.0),
            4,
            [20, 10, 200, 255],
        ),
    ] {
        assert_image(
            &mut app,
            panel,
            key,
            bounds,
            Color::srgba(0.7, 0.8, 0.9, 0.6),
            None,
        );
        assert_pixels(&mut app, panel, key, width, 6, rgba);
    }
    settle(&mut app);
    resize(&mut app, panel, 150.0, 45.0);
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .get_mut(panel)
        .unwrap()
        .three_slice
        .as_mut()
        .unwrap()
        .cap_width = 15.0;
    settle(&mut app);
    for (key, bounds, width, rgba) in [
        (
            200,
            Rect::new(40.0, 50.0, 55.0, 95.0),
            4,
            [200, 20, 10, 255],
        ),
        (
            201,
            Rect::new(55.0, 50.0, 175.0, 95.0),
            2,
            [20, 200, 10, 255],
        ),
        (
            202,
            Rect::new(175.0, 50.0, 190.0, 95.0),
            4,
            [20, 10, 200, 255],
        ),
    ] {
        assert_image(
            &mut app,
            panel,
            key,
            bounds,
            Color::srgba(0.7, 0.8, 0.9, 0.6),
            None,
        );
        assert_pixels(&mut app, panel, key, width, 6, rgba);
    }
}

#[test]
fn text_wrap_and_justification_shape_real_glyphs_and_update_content_after_idle() {
    let mut app = app_with_real_fonts(1.0);
    let id = label(
        &mut app,
        "WrappedLabel",
        100.0,
        100.0,
        FontStringData {
            text: "First second third fourth fifth sixth".into(),
            font: GameFont::ArialNarrow,
            font_size: 18.0,
            word_wrap: true,
            justify_h: JustifyH::Left,
            justify_v: JustifyV::Top,
            ..default()
        },
    );
    settle(&mut app);
    let (entity, bounds) = text(app.world_mut(), id, 0).unwrap();
    assert_rect(app.world(), bounds, Rect::new(40.0, 50.0, 140.0, 150.0));
    let layout = app.world().get::<TextLayout>(entity).unwrap();
    assert_eq!(layout.justify, Justify::Left);
    assert_eq!(layout.linebreak, LineBreak::WordBoundary);
    let shaped = app.world().get::<TextLayoutInfo>(entity).unwrap();
    assert!(
        shaped.glyphs.iter().any(|glyph| glyph.line_index > 0),
        "bounded real text must wrap"
    );
    assert!(shaped.size.y > 18.0, "wrapped ink must span multiple lines");
    assert!((rect(app.world(), entity).width() - 100.0).abs() < 0.6);
    settle(&mut app);
    edit_label(&mut app, id, |data| {
        data.text = "Short".into();
        data.justify_v = JustifyV::Bottom;
    });
    settle(&mut app);
    let (entity, _) = text(app.world_mut(), id, 0).unwrap();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Short");
    let text_rect = rect(app.world(), entity);
    assert!((text_rect.max.y - 150.0).abs() < 0.6);
    let left_x = app.world().get::<TextLayoutInfo>(entity).unwrap().glyphs[0]
        .position
        .x;
    edit_label(&mut app, id, |data| data.justify_h = JustifyH::Center);
    settle(&mut app);
    let centered_x = app.world().get::<TextLayoutInfo>(entity).unwrap().glyphs[0]
        .position
        .x;
    edit_label(&mut app, id, |data| data.justify_h = JustifyH::Right);
    settle(&mut app);
    let right_x = app.world().get::<TextLayoutInfo>(entity).unwrap().glyphs[0]
        .position
        .x;
    assert!(centered_x > left_x + 10.0 && right_x > centered_x + 10.0);
    assert!((right_x - left_x - 2.0 * (centered_x - left_x)).abs() < 1.0);
    assert_eq!(
        app.world().get::<TextLayout>(entity).unwrap().justify,
        Justify::Right
    );
}

#[test]
fn max_lines_clips_native_bounds_and_updates_with_line_limit_and_font_size() {
    let mut app = app_with_real_fonts(1.0);
    let id = label(
        &mut app,
        "LimitedLabel",
        90.0,
        150.0,
        FontStringData {
            text: "One two three four five six seven eight nine ten".into(),
            font: GameFont::ArialNarrow,
            font_size: 20.0,
            justify_v: JustifyV::Top,
            word_wrap: true,
            max_lines: Some(2),
            ..default()
        },
    );
    settle(&mut app);
    let (entity, bounds) = text(app.world_mut(), id, 0).unwrap();
    assert_rect(app.world(), bounds, Rect::new(40.0, 50.0, 130.0, 98.0));
    assert_eq!(
        app.world().get::<Node>(bounds).unwrap().overflow,
        Overflow::clip()
    );
    let shaped = app.world().get::<TextLayoutInfo>(entity).unwrap();
    assert!(
        shaped.size.y > rect(app.world(), bounds).height(),
        "shaped ink {:?} must exceed clipping viewport {:?}",
        shaped.size,
        rect(app.world(), bounds)
    );
    assert!(shaped.glyphs.iter().any(|glyph| glyph.line_index >= 2));
    settle(&mut app);
    edit_label(&mut app, id, |data| {
        data.max_lines = Some(1);
        data.font_size = 30.0;
    });
    settle(&mut app);
    assert_rect(app.world(), bounds, Rect::new(40.0, 50.0, 130.0, 86.0));
    assert_eq!(
        app.world().get::<TextFont>(entity).unwrap().font_size,
        FontSize::Px(30.0)
    );
    edit_label(&mut app, id, |data| data.max_lines = None);
    settle(&mut app);
    assert_rect(app.world(), bounds, Rect::new(40.0, 50.0, 130.0, 200.0));
    assert_eq!(
        app.world().get::<Text>(entity).unwrap().0,
        "One two three four five six seven eight nine ten"
    );
}

fn assert_effect(
    app: &mut App,
    id: u64,
    key: u32,
    offset: Vec2,
    color: Color,
    content: &str,
) -> (Entity, Entity) {
    let (base, base_bounds) = text(app.world_mut(), id, 0).unwrap();
    let (entity, bounds) = text(app.world_mut(), id, key).expect("native effect text");
    let delta = rect(app.world(), bounds).min - rect(app.world(), base_bounds).min;
    assert!(
        (delta - offset).abs().max_element() < 0.6,
        "effect offset {delta:?}, expected {offset:?}"
    );
    assert_eq!(
        rect(app.world(), bounds).size(),
        rect(app.world(), base_bounds).size()
    );
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, content);
    assert_eq!(app.world().get::<TextColor>(entity).unwrap().0, color);
    assert_eq!(
        app.world().get::<TextFont>(entity),
        app.world().get::<TextFont>(base)
    );
    (entity, bounds)
}

#[test]
fn shadow_and_outline_keep_colors_offsets_content_updates_and_remove_without_base_loss() {
    let mut app = app_with_real_fonts(1.0);
    let id = label(
        &mut app,
        "EffectsLabel",
        180.0,
        40.0,
        FontStringData {
            text: "Before".into(),
            font: GameFont::ArialNarrow,
            font_size: 18.0,
            justify_v: JustifyV::Top,
            shadow_color: Some([0.2, 0.4, 0.6, 0.7]),
            shadow_offset: [3.0, 4.0],
            outline: Outline::Outline,
            ..default()
        },
    );
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_alpha(id, 0.5);
    settle(&mut app);
    assert_effect(
        &mut app,
        id,
        1,
        Vec2::new(3.0, 4.0),
        Color::srgba(0.2, 0.4, 0.6, 0.7),
        "Before",
    );
    for (key, offset) in [
        (2, Vec2::new(-1.0, 0.0)),
        (3, Vec2::new(1.0, 0.0)),
        (4, Vec2::new(0.0, 1.0)),
        (5, Vec2::new(0.0, -1.0)),
    ] {
        assert_effect(
            &mut app,
            id,
            key,
            offset,
            Color::srgba(0.0, 0.0, 0.0, 0.5),
            "Before",
        );
    }
    settle(&mut app);
    edit_label(&mut app, id, |data| {
        data.text = "After".into();
        data.font_size = 22.0;
        data.shadow_color = Some([0.8, 0.2, 0.3, 0.4]);
        data.shadow_offset = [-2.0, 5.0];
        data.outline = Outline::ThickOutline;
    });
    app.world_mut()
        .resource_mut::<UiState>()
        .registry
        .set_alpha(id, 0.75);
    settle(&mut app);
    let mut effects = vec![assert_effect(
        &mut app,
        id,
        1,
        Vec2::new(-2.0, 5.0),
        Color::srgba(0.8, 0.2, 0.3, 0.4),
        "After",
    )];
    for (index, offset) in [
        Vec2::new(-2.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(0.0, 2.0),
        Vec2::new(0.0, -2.0),
        Vec2::new(-1.4, 1.4),
        Vec2::new(1.4, 1.4),
        Vec2::new(-1.4, -1.4),
        Vec2::new(1.4, -1.4),
    ]
    .into_iter()
    .enumerate()
    {
        effects.push(assert_effect(
            &mut app,
            id,
            index as u32 + 2,
            offset,
            Color::srgba(0.0, 0.0, 0.0, 0.75),
            "After",
        ));
    }
    edit_label(&mut app, id, |data| {
        data.shadow_color = None;
        data.outline = Outline::None;
    });
    settle(&mut app);
    for (entity, bounds) in effects {
        assert!(app.world().get_entity(entity).is_err());
        assert!(app.world().get_entity(bounds).is_err());
    }
    let (base, bounds) = text(app.world_mut(), id, 0).unwrap();
    assert_eq!(app.world().get::<Text>(base).unwrap().0, "After");
    assert_eq!(
        app.world().get::<TextColor>(base).unwrap().0,
        Color::srgba(1.0, 1.0, 1.0, 0.75)
    );
    assert_rect(app.world(), bounds, Rect::new(40.0, 50.0, 220.0, 90.0));
}
