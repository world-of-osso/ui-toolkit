use super::{NativeAssets, TextPart};
use crate::frame::{Dimension, Frame, WidgetData};
use crate::render_text::{extract_text_props, has_text, text_layout};
use crate::widgets::font_string::JustifyV;
use bevy::prelude::*;

pub(super) fn project_text(frame: &Frame, assets: &mut NativeAssets) -> Vec<TextPart> {
    if !has_text(frame) {
        return Vec::new();
    }
    let props = extract_text_props(frame);
    let insets = match &frame.widget_data {
        Some(WidgetData::EditBox(edit)) => edit.text_insets,
        _ => [0.0; 4],
    };
    let auto = frame.width == Dimension::Auto || frame.height == Dimension::Auto;
    let node = text_node(frame, props.justify_v, insets, auto);
    let font = TextFont {
        font: FontSource::Handle(assets.font(props.font)),
        font_size: FontSize::Px(props.font_size),
        ..default()
    };
    let base = TextPart {
        key: 0,
        node,
        text: props.content.into_owned(),
        font,
        layout: text_layout(frame),
        color: TextColor(props.color),
        z: quantize(assets.frame_z() + 0.0007),
    };
    let mut parts = Vec::new();
    if let Some(WidgetData::FontString(fs)) = &frame.widget_data {
        if let Some([r, g, b, a]) = fs.shadow_color {
            parts.push(offset_part(
                &base,
                1,
                fs.shadow_offset[0],
                fs.shadow_offset[1],
                Color::srgba(r, g, b, a),
                quantize(assets.frame_z() + 0.0006),
            ));
        }
        for (index, &(dx, dy)) in crate::render_text_fx::outline_offsets(fs.outline)
            .iter()
            .enumerate()
        {
            parts.push(offset_part(
                &base,
                index as u32 + 2,
                dx,
                -dy,
                Color::srgba(0.0, 0.0, 0.0, frame.effective_alpha),
                quantize(assets.frame_z() + 0.0005),
            ));
        }
    }
    parts.push(base);
    parts
}

fn text_node(frame: &Frame, justify: JustifyV, insets: [f32; 4], auto: bool) -> Node {
    let justify_content = match justify {
        JustifyV::Top => JustifyContent::FlexStart,
        JustifyV::Middle => JustifyContent::Center,
        JustifyV::Bottom => JustifyContent::FlexEnd,
    };
    let mut node = Node {
        flex_direction: FlexDirection::Column,
        justify_content,
        align_items: AlignItems::Stretch,
        flex_shrink: 0.0,
        ..default()
    };
    if auto {
        node.margin = UiRect {
            left: px(insets[0]),
            right: px(insets[1]),
            top: px(insets[2]),
            bottom: px(insets[3]),
        };
        node.width = if frame.width == Dimension::Auto {
            Val::Auto
        } else {
            percent(100)
        };
        node.height = if frame.height == Dimension::Auto {
            Val::Auto
        } else {
            percent(100)
        };
    } else {
        node.position_type = PositionType::Absolute;
        node.left = px(insets[0]);
        node.right = px(insets[1]);
        node.top = px(insets[2]);
        node.bottom = px(insets[3]);
    }
    if let Some(WidgetData::FontString(fs)) = &frame.widget_data {
        if let Some(lines) = fs.max_lines {
            node.max_height = px(lines as f32 * fs.font_size * 1.2 * fs.text_scale);
            node.overflow = Overflow::clip();
        }
    }
    node
}

fn quantize(z: f32) -> i32 {
    (z * 10000.0).round() as i32
}

fn offset_part(base: &TextPart, key: u32, x: f32, y: f32, color: Color, z: i32) -> TextPart {
    let mut node = base.node.clone();
    node.position_type = PositionType::Absolute;
    node.margin = UiRect::ZERO;
    node.left = px(x);
    node.right = px(-x);
    node.top = px(y);
    node.bottom = px(-y);
    node.width = Val::Auto;
    node.height = Val::Auto;
    TextPart {
        key,
        node,
        text: base.text.clone(),
        font: base.font.clone(),
        layout: base.layout.clone(),
        color: TextColor(color),
        z,
    }
}
