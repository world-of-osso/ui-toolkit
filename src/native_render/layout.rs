//! Translate authored native-compatible layout; never solve registry anchors.
use super::RegistryNode;
use crate::frame::{Dimension, FlexAlign, FlexDirection as RegistryDirection, FlexJustify, Frame};
use crate::layout::LayoutRect;
use crate::plugin::UiState;
use bevy::{math::Affine2, prelude::*};
use ui_toolkit_core::layout_values::{
    PositionType as ModelPositionType, UiRect as ModelUiRect, Val as ModelVal, Val2 as ModelVal2,
};

pub(super) fn node(frame: &Frame) -> Node {
    let mut node = Node {
        position_type: position_type(frame.position_type),
        left: val(frame.position.left),
        right: val(frame.position.right),
        top: val(frame.position.top),
        bottom: val(frame.position.bottom),
        margin: rect(frame.margin),
        width: dimension(frame.width),
        height: dimension(frame.height),
        display: if frame.visible {
            Display::Flex
        } else {
            Display::None
        },
        ..default()
    };
    if let Some(flex) = &frame.flex_layout {
        node.flex_direction = match flex.direction {
            RegistryDirection::Column => FlexDirection::Column,
            RegistryDirection::Row | RegistryDirection::RowWrap => FlexDirection::Row,
        };
        node.flex_wrap = if matches!(flex.direction, RegistryDirection::RowWrap) {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        };
        node.row_gap = px(flex.gap);
        node.column_gap = px(flex.gap);
        node.padding = UiRect::all(px(flex.padding));
        node.justify_content = match flex.justify {
            FlexJustify::Start => JustifyContent::FlexStart,
            FlexJustify::Center => JustifyContent::Center,
            FlexJustify::End => JustifyContent::FlexEnd,
            FlexJustify::SpaceBetween => JustifyContent::SpaceBetween,
        };
        node.align_items = match flex.align {
            FlexAlign::Start => AlignItems::FlexStart,
            FlexAlign::Center => AlignItems::Center,
            FlexAlign::End => AlignItems::FlexEnd,
            FlexAlign::Stretch => AlignItems::Stretch,
        };
    }
    node
}

pub(super) fn val(value: ModelVal) -> Val {
    match value {
        ModelVal::Auto => Val::Auto,
        ModelVal::Px(value) => px(value),
        ModelVal::Percent(value) => percent(value),
    }
}

pub(super) fn rect(value: ModelUiRect) -> UiRect {
    UiRect {
        left: val(value.left),
        right: val(value.right),
        top: val(value.top),
        bottom: val(value.bottom),
    }
}

pub(super) fn translation(value: ModelVal2) -> Val2 {
    Val2 {
        x: val(value.x),
        y: val(value.y),
    }
}

pub(super) fn position_type(value: ModelPositionType) -> PositionType {
    match value {
        ModelPositionType::Relative => PositionType::Relative,
        ModelPositionType::Absolute => PositionType::Absolute,
    }
}

fn dimension(value: Dimension) -> Val {
    match value {
        Dimension::Fixed(value) => px(value),
        Dimension::Fill => percent(100),
        Dimension::Auto => Val::Auto,
    }
}

pub(crate) fn read_bounds(
    mut state: ResMut<UiState>,
    frames: Query<(
        &RegistryNode,
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedUiRenderTargetInfo,
    )>,
) {
    let registry = &mut state.bypass_change_detection().registry;
    for (id, node, transform, target) in &frames {
        let transform = Affine2::from(transform);
        let half = node.size / 2.0;
        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            half,
            Vec2::new(-half.x, half.y),
        ];
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for corner in corners {
            let point = transform.transform_point2(corner) / target.scale_factor();
            min = min.min(point);
            max = max.max(point);
        }
        if let Err(error) = registry.set_computed_layout(
            id.0,
            LayoutRect {
                x: min.x,
                y: min.y,
                width: max.x - min.x,
                height: max.y - min.y,
            },
        ) {
            warn!(
                "native UI bounds for removed registry frame {}: {error}",
                id.0
            );
        }
    }
}
