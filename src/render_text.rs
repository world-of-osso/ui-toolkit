use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::Font;
use bevy::text::TextFont;

use crate::font_registry::FontRegistry;
use crate::frame::WidgetData;
use crate::plugin::UiState;
use crate::render::UI_RENDER_LAYER;
use crate::render::UiText;
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
        *transform = text_transform(frame, screen_w, screen_h, props.justify_h, props.justify_v);
        commands
            .entity(entity)
            .insert(text_anchor(props.justify_h, props.justify_v));
    }

    spawn_missing_text(
        &state,
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
    existing: &std::collections::HashSet<u64>,
    screen_w: f32,
    screen_h: f32,
    commands: &mut Commands,
    font_assets: &mut Assets<Font>,
    font_registry: &mut FontRegistry,
) {
    for frame in state.registry.frames_iter() {
        if !frame.visible || existing.contains(&frame.id) || !has_text(frame) {
            continue;
        }
        let props = extract_text_props(frame);
        let transform = text_transform(frame, screen_w, screen_h, props.justify_h, props.justify_v);
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

pub(crate) fn extract_text_props_pub(frame: &crate::frame::Frame) -> TextProps {
    extract_text_props(frame)
}

fn extract_text_props(frame: &crate::frame::Frame) -> TextProps {
    match &frame.widget_data {
        Some(WidgetData::FontString(fs)) => {
            let [r, g, b, a] = fs.color;
            TextProps {
                content: fs.text.clone(),
                font: fs.font,
                font_size: fs.font_size,
                color: Color::srgba(r, g, b, a * frame.effective_alpha),
                justify_h: fs.justify_h,
                justify_v: fs.justify_v,
            }
        }
        Some(WidgetData::EditBox(eb)) => {
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
                color: Color::srgba(r, g, b, a * frame.effective_alpha),
                justify_h: JustifyH::Left,
                justify_v: JustifyV::Middle,
            }
        }
        Some(WidgetData::Button(btn)) => extract_button_text(btn, frame.effective_alpha),
        _ => TextProps {
            content: String::new(),
            font: GameFont::default(),
            font_size: 12.0,
            color: Color::WHITE,
            justify_h: JustifyH::Center,
            justify_v: JustifyV::Middle,
        },
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
) -> Transform {
    let rect = frame.layout_rect.as_ref();
    let fx = rect.map_or(0.0, |r| r.x);
    let fy = rect.map_or(0.0, |r| r.y);
    let insets = text_insets(frame);
    let x = match justify_h {
        JustifyH::Left => fx + insets[0] - screen_w * 0.5,
        JustifyH::Center => fx + frame.width * 0.5 - screen_w * 0.5,
        JustifyH::Right => fx + frame.width - insets[1] - screen_w * 0.5,
    };
    let top = fy + insets[2];
    let bottom = fy + frame.height - insets[3];
    let y = match justify_v {
        JustifyV::Top => screen_h * 0.5 - top,
        JustifyV::Middle => screen_h * 0.5 - (top + bottom) * 0.5,
        JustifyV::Bottom => screen_h * 0.5 - bottom,
    };
    Transform::from_xyz(x, y, 10.0)
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
    use crate::frame::{Frame, WidgetData, WidgetType};
    use crate::layout::LayoutRect;
    use crate::widgets::edit_box::EditBoxData;

    fn make_edit_box(width: f32, height: f32, insets: [f32; 4]) -> Frame {
        let mut frame = Frame::new(1, Some("EditBox".into()), WidgetType::EditBox);
        frame.width = width;
        frame.height = height;
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

    #[test]
    fn text_transform_centers_edit_box_text_between_vertical_insets() {
        let frame = make_edit_box(300.0, 30.0, [12.0, 5.0, 0.0, 5.0]);
        let transform = text_transform(&frame, 800.0, 600.0, JustifyH::Left, JustifyV::Middle);
        assert_eq!(transform.translation.x, -388.0);
        assert_eq!(transform.translation.y, 287.5);
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
}
