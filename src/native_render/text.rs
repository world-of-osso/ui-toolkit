use super::{NativeAssets, TextPart};
use crate::frame::{Frame, WidgetData};
use crate::render_text::{extract_text_props, has_text, text_bounds, text_layout, text_transform};
use bevy::prelude::*;

pub(super) fn project_text(frame: &Frame, assets: &mut NativeAssets) -> Vec<TextPart> {
    if !has_text(frame) {
        return Vec::new();
    }
    let props = extract_text_props(frame);
    let origin = text_transform(frame, 0.0, 0.0, props.justify_h, props.justify_v, 0);
    let rect = frame.layout_rect.as_ref();
    let x = origin.translation.x - rect.map_or(0.0, |r| r.x);
    let y = -origin.translation.y - rect.map_or(0.0, |r| r.y);
    let bounds = text_bounds(frame);
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(x),
        top: px(y),
        width: px(bounds.width.unwrap_or(frame.resolved_width())),
        height: bounds.height.map_or(Val::Auto, px),
        align_items: AlignItems::FlexStart,
        ..default()
    };
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

fn quantize(z: f32) -> i32 {
    (z * 10000.0).round() as i32
}

fn offset_part(base: &TextPart, key: u32, x: f32, y: f32, color: Color, z: i32) -> TextPart {
    let mut node = base.node.clone();
    if let Val::Px(left) = &mut node.left {
        *left += x;
    }
    if let Val::Px(top) = &mut node.top {
        *top += y;
    }
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
