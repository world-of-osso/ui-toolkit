use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::Font;
use bevy::text::TextFont;

use crate::font_registry::FontRegistry;
use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::render::{UI_RENDER_LAYER, UiText, build_sorted_visible_frame_ids};
use crate::render_text_fx::{UiTextOutline, UiTextShadow};
use crate::widgets::button::ButtonState;
use crate::widgets::font_string::{GameFont, JustifyH, JustifyV};

/// Syncs text content from the frame registry into Bevy Text2d entities.
pub fn sync_ui_text(
    state: Res<UiState>,
    mut commands: Commands,
    mut font_assets: ResMut<Assets<Font>>,
    mut font_registry: ResMut<FontRegistry>,
    mut texts: Query<
        (
            Entity,
            &UiText,
            &mut Text2d,
            &mut TextFont,
            &mut TextColor,
            &mut Transform,
        ),
        (Without<UiTextShadow>, Without<UiTextOutline>),
    >,
) {
    let screen_w = state.registry.screen_width;
    let screen_h = state.registry.screen_height;
    let sorted_ids = build_sorted_visible_frame_ids(&state);
    let sort_map: std::collections::HashMap<u64, usize> = sorted_ids
        .iter()
        .copied()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();
    let mut existing: std::collections::HashSet<u64> = std::collections::HashSet::new();

    for (entity, ui_text, mut text, mut font, mut color, mut transform) in texts.iter_mut() {
        let Some(frame) = state.registry.get(ui_text.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        if !frame.visible || !has_text(frame) {
            commands.entity(entity).despawn();
            continue;
        }
        let props = extract_text_props(frame);
        existing.insert(ui_text.0);
        *text = Text2d::new(&props.content);
        font.font_size = props.font_size;
        font.font = font_registry.get(props.font, &mut font_assets);
        *color = TextColor(props.color);
        let sort_idx = sort_map[&ui_text.0];
        *transform = text_transform(
            frame,
            screen_w,
            screen_h,
            props.justify_h,
            props.justify_v,
            sort_idx,
        );
        commands
            .entity(entity)
            .insert(text_anchor(props.justify_h, props.justify_v));
    }

    spawn_missing_text(
        &state,
        &sorted_ids,
        &sort_map,
        &existing,
        screen_w,
        screen_h,
        &mut commands,
        &mut font_assets,
        &mut font_registry,
    );
}

fn spawn_missing_text(
    state: &UiState,
    sorted_ids: &[u64],
    sort_map: &std::collections::HashMap<u64, usize>,
    existing: &std::collections::HashSet<u64>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    font_assets: &mut Assets<Font>,
    font_registry: &mut FontRegistry,
) {
    for &frame_id in sorted_ids {
        let Some(frame) = state.registry.get(frame_id) else {
            continue;
        };
        if !frame.visible || existing.contains(&frame.id) || !has_text(frame) {
            continue;
        }
        let props = extract_text_props(frame);
        let transform = text_transform(
            frame,
            screen_w,
            screen_h,
            props.justify_h,
            props.justify_v,
            sort_map[&frame.id],
        );
        let font = font_registry.get(props.font, font_assets);
        commands.spawn((
            Text2d::new(props.content),
            TextFont {
                font,
                font_size: props.font_size,
                ..default()
            },
            TextColor(props.color),
            text_anchor(props.justify_h, props.justify_v),
            transform,
            RenderLayers::layer(UI_RENDER_LAYER),
            UiText(frame.id),
        ));
    }
}

fn has_text(frame: &crate::frame::Frame) -> bool {
    match &frame.widget_data {
        Some(WidgetData::FontString(fs)) => !fs.text.is_empty(),
        Some(WidgetData::EditBox(_)) => true,
        Some(WidgetData::Button(btn)) => !btn.text.is_empty(),
        _ => false,
    }
}

pub(crate) struct TextProps {
    pub content: String,
    pub font: GameFont,
    pub font_size: f32,
    pub color: Color,
    pub justify_h: JustifyH,
    pub justify_v: JustifyV,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            font: GameFont::default(),
            font_size: 12.0,
            color: Color::WHITE,
            justify_h: JustifyH::Center,
            justify_v: JustifyV::Middle,
        }
    }
}

#[cfg(test)]
pub(crate) fn extract_text_props_pub(frame: &crate::frame::Frame) -> TextProps {
    extract_text_props(frame)
}

fn extract_text_props(frame: &crate::frame::Frame) -> TextProps {
    match &frame.widget_data {
        Some(WidgetData::FontString(fs)) => extract_fontstring_text(fs, frame.effective_alpha),
        Some(WidgetData::EditBox(eb)) => extract_editbox_text(eb, frame.effective_alpha),
        Some(WidgetData::Button(btn)) => extract_button_text(btn, frame.effective_alpha),
        _ => TextProps::default(),
    }
}

fn extract_fontstring_text(
    fs: &crate::widgets::font_string::FontStringData,
    alpha: f32,
) -> TextProps {
    let [r, g, b, a] = fs.color;
    TextProps {
        content: fs.text.clone(),
        font: fs.font,
        font_size: fs.font_size,
        color: Color::srgba(r, g, b, a * alpha),
        justify_h: fs.justify_h,
        justify_v: fs.justify_v,
    }
}

fn extract_editbox_text(eb: &crate::widgets::edit_box::EditBoxData, alpha: f32) -> TextProps {
    let display = if eb.password {
        "*".repeat(eb.text.len())
    } else {
        eb.text.clone()
    };
    let [r, g, b, a] = eb.text_color;
    TextProps {
        content: display,
        font: eb.font,
        font_size: eb.font_size,
        color: Color::srgba(r, g, b, a * alpha),
        justify_h: JustifyH::Left,
        justify_v: JustifyV::Middle,
    }
}

pub(crate) fn extract_button_text(
    btn: &crate::widgets::button::ButtonData,
    alpha: f32,
) -> TextProps {
    let (r, g, b) = match btn.state {
        ButtonState::Normal => (1.0, 0.82, 0.0),
        ButtonState::Pushed => (0.8, 0.65, 0.0),
        ButtonState::Disabled => (0.5, 0.5, 0.5),
    };
    TextProps {
        content: btn.text.clone(),
        font: GameFont::default(),
        font_size: btn.font_size,
        color: Color::srgba(r, g, b, alpha),
        justify_h: JustifyH::Center,
        justify_v: JustifyV::Middle,
    }
}

/// Compute the transform for a text entity. Public for use by render_text_fx.
pub fn text_transform(
    frame: &crate::frame::Frame,
    screen_w: f32,
    screen_h: f32,
    justify_h: JustifyH,
    justify_v: JustifyV,
    sort_idx: usize,
) -> Transform {
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let insets = text_insets(frame);
    let x = match justify_h {
        JustifyH::Left => fx + insets[0] - screen_w * 0.5,
        JustifyH::Center => fx + frame.resolved_width() * 0.5 - screen_w * 0.5,
        JustifyH::Right => fx + frame.resolved_width() - insets[1] - screen_w * 0.5,
    };
    let top = fy + insets[2];
    let bottom = fy + frame.resolved_height() - insets[3];
    let y = match justify_v {
        JustifyV::Top => screen_h * 0.5 - top,
        JustifyV::Middle => screen_h * 0.5 - (top + bottom) * 0.5,
        JustifyV::Bottom => screen_h * 0.5 - bottom,
    };
    Transform::from_xyz(x, y, sort_idx as f32 * 0.001 + 0.0007)
}

fn text_anchor(justify_h: JustifyH, justify_v: JustifyV) -> Anchor {
    match (justify_h, justify_v) {
        (JustifyH::Left, JustifyV::Top) => Anchor::TOP_LEFT,
        (JustifyH::Center, JustifyV::Top) => Anchor::TOP_CENTER,
        (JustifyH::Right, JustifyV::Top) => Anchor::TOP_RIGHT,
        (JustifyH::Left, JustifyV::Middle) => Anchor::CENTER_LEFT,
        (JustifyH::Center, JustifyV::Middle) => Anchor::CENTER,
        (JustifyH::Right, JustifyV::Middle) => Anchor::CENTER_RIGHT,
        (JustifyH::Left, JustifyV::Bottom) => Anchor::BOTTOM_LEFT,
        (JustifyH::Center, JustifyV::Bottom) => Anchor::BOTTOM_CENTER,
        (JustifyH::Right, JustifyV::Bottom) => Anchor::BOTTOM_RIGHT,
    }
}

fn text_insets(frame: &crate::frame::Frame) -> [f32; 4] {
    if let Some(WidgetData::EditBox(eb)) = &frame.widget_data {
        if eb.text_insets != [0.0; 4] {
            return eb.text_insets;
        }
        let h = eb.font_size * 0.25;
        return [h, h, 0.0, 0.0];
    }
    [0.0; 4]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Dimension, Frame, WidgetData, WidgetType};
    use crate::layout::LayoutRect;
    use crate::plugin::UiPlugin;
    use crate::widgets::edit_box::EditBoxData;
    use crate::widgets::font_string::FontStringData;

    fn make_edit_box(width: f32, height: f32, insets: [f32; 4]) -> Frame {
        let mut frame = Frame::new(1, Some("EditBox".into()), WidgetType::EditBox);
        frame.width = Dimension::Fixed(width);
        frame.height = Dimension::Fixed(height);
        frame.layout_rect = Some(LayoutRect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        });
        frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
            text_insets: insets,
            ..Default::default()
        }));
        frame
    }

    fn make_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Assets::<Font>::default());
        app.add_plugins(UiPlugin);
        app.update();
        app
    }

    #[test]
    fn text_transform_centers_edit_box_text_between_vertical_insets() {
        let frame = make_edit_box(300.0, 30.0, [12.0, 5.0, 0.0, 5.0]);
        let transform = text_transform(&frame, 800.0, 600.0, JustifyH::Left, JustifyV::Middle, 0);
        assert_eq!(transform.translation.x, -388.0);
        assert_eq!(transform.translation.y, 287.5);
        assert_eq!(transform.translation.z, 0.0007);
    }

    #[test]
    fn text_transform_uses_sort_index_for_z_order() {
        let frame = make_edit_box(100.0, 20.0, [0.0; 4]);
        let lower = text_transform(&frame, 800.0, 600.0, JustifyH::Left, JustifyV::Middle, 2);
        let higher = text_transform(&frame, 800.0, 600.0, JustifyH::Left, JustifyV::Middle, 8);

        assert!(higher.translation.z > lower.translation.z);
        assert_eq!(lower.translation.z, 0.0027);
        assert_eq!(higher.translation.z, 0.0087);
    }

    #[test]
    fn text_anchor_combines_horizontal_and_vertical_justify() {
        assert_eq!(text_anchor(JustifyH::Left, JustifyV::Top), Anchor::TOP_LEFT);
        assert_eq!(
            text_anchor(JustifyH::Center, JustifyV::Middle),
            Anchor::CENTER
        );
        assert_eq!(
            text_anchor(JustifyH::Right, JustifyV::Bottom),
            Anchor::BOTTOM_RIGHT
        );
    }

    #[test]
    fn extract_text_props_uses_edit_box_style_fields() {
        let mut frame = make_edit_box(300.0, 30.0, [12.0, 5.0, 0.0, 5.0]);
        frame.effective_alpha = 0.5;
        frame.widget_data = Some(WidgetData::EditBox(EditBoxData {
            text: "abc".into(),
            font: crate::widgets::font_string::GameFont::ArialNarrow,
            font_size: 16.0,
            text_color: [0.8, 0.7, 0.6, 1.0],
            ..Default::default()
        }));
        let props = extract_text_props(&frame);
        assert_eq!(props.content, "abc");
        assert_eq!(
            props.font,
            crate::widgets::font_string::GameFont::ArialNarrow
        );
        assert_eq!(props.font_size, 16.0);
        let Color::Srgba(srgba) = props.color else {
            panic!("expected srgba")
        };
        assert!((srgba.red - 0.8).abs() < 0.001);
        assert!((srgba.green - 0.7).abs() < 0.001);
        assert!((srgba.blue - 0.6).abs() < 0.001);
        assert!((srgba.alpha - 0.5).abs() < 0.001);
    }

    #[test]
    fn text_only_fontstring_still_spawns_ui_text() {
        let mut app = make_test_app();
        let mut ui = app.world_mut().resource_mut::<crate::plugin::UiState>();
        let id = ui.registry.create_frame("TextOnly", None);
        let frame = ui.registry.get_mut(id).unwrap();
        frame.width = Dimension::Fixed(120.0);
        frame.height = Dimension::Fixed(20.0);
        frame.layout_rect = Some(LayoutRect {
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 20.0,
        });
        frame.widget_data = Some(WidgetData::FontString(FontStringData {
            text: "Hello".into(),
            ..Default::default()
        }));
        drop(ui);

        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<&UiText, Without<crate::render_text_fx::UiTextShadow>>();
        assert!(q.iter(app.world()).any(|t| t.0 == id));
    }
}
