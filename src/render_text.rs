use std::borrow::Cow;

use bevy::prelude::*;
use bevy::text::{Justify, LineBreak, TextLayout};

use crate::frame::WidgetData;
use crate::widgets::button::ButtonState;
use crate::widgets::font_string::{GameFont, JustifyH, JustifyV};

pub(crate) fn has_text(frame: &crate::frame::Frame) -> bool {
    match &frame.widget_data {
        Some(WidgetData::FontString(fs)) => !fs.text.is_empty(),
        Some(WidgetData::EditBox(_)) => true,
        Some(WidgetData::Button(btn)) => !btn.text.is_empty(),
        _ => false,
    }
}

pub(crate) struct TextProps<'a> {
    pub content: Cow<'a, str>,
    pub font: GameFont,
    pub font_size: f32,
    pub color: Color,
    pub justify_h: JustifyH,
    pub justify_v: JustifyV,
}

impl Default for TextProps<'_> {
    fn default() -> Self {
        Self {
            content: Cow::Borrowed(""),
            font: GameFont::default(),
            font_size: 12.0,
            color: Color::WHITE,
            justify_h: JustifyH::Center,
            justify_v: JustifyV::Middle,
        }
    }
}

#[cfg(test)]
pub(crate) fn extract_text_props_pub(frame: &crate::frame::Frame) -> TextProps<'_> {
    extract_text_props(frame)
}

pub(crate) fn extract_text_props(frame: &crate::frame::Frame) -> TextProps<'_> {
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
) -> TextProps<'_> {
    let [r, g, b, a] = fs.color;
    TextProps {
        content: Cow::Borrowed(&fs.text),
        font: fs.font,
        font_size: fs.font_size,
        color: Color::srgba(r, g, b, a * alpha),
        justify_h: fs.justify_h,
        justify_v: fs.justify_v,
    }
}

fn extract_editbox_text(eb: &crate::widgets::edit_box::EditBoxData, alpha: f32) -> TextProps<'_> {
    let display = if eb.password {
        Cow::Owned("*".repeat(eb.text.len()))
    } else {
        Cow::Borrowed(eb.text.as_str())
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
) -> TextProps<'_> {
    let (r, g, b) = match btn.state {
        ButtonState::Normal => (1.0, 0.82, 0.0),
        ButtonState::Pushed => (0.8, 0.65, 0.0),
        ButtonState::Disabled => (0.5, 0.5, 0.5),
    };
    TextProps {
        content: Cow::Borrowed(&btn.text),
        font: GameFont::default(),
        font_size: btn.font_size,
        color: Color::srgba(r, g, b, alpha),
        justify_h: JustifyH::Center,
        justify_v: JustifyV::Middle,
    }
}

pub(crate) fn text_layout(frame: &crate::frame::Frame) -> TextLayout {
    TextLayout::new(text_justify(frame), text_linebreak(frame))
}

fn text_justify(frame: &crate::frame::Frame) -> Justify {
    match text_justify_h(frame) {
        JustifyH::Left => Justify::Left,
        JustifyH::Center => Justify::Center,
        JustifyH::Right => Justify::Right,
    }
}

fn text_linebreak(_frame: &crate::frame::Frame) -> LineBreak {
    // Use WordBoundary for all bounded text so cosmic-text respects Justify
    // alignment within the bounds. Single-line text that fits won't actually wrap.
    LineBreak::WordBoundary
}

fn text_justify_h(frame: &crate::frame::Frame) -> JustifyH {
    match &frame.widget_data {
        Some(WidgetData::FontString(fs)) => fs.justify_h,
        Some(WidgetData::EditBox(_)) => JustifyH::Left,
        Some(WidgetData::Button(_)) => JustifyH::Center,
        _ => JustifyH::Center,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Dimension, Frame, WidgetData, WidgetType};
    use crate::layout::LayoutRect;
    use crate::widgets::edit_box::EditBoxData;

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
