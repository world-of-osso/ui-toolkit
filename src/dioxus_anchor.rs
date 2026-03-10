use dioxus_core::TemplateNode;

use crate::anchor::{Anchor, AnchorPoint};
use crate::registry::FrameRegistry;

/// Accumulated anchor attribute state for anchors with dynamic attrs.
/// Static attrs are filled during template instantiation, dynamic attrs arrive via set_attribute.
#[derive(Debug, Clone)]
pub(crate) struct AnchorState {
    point: String,
    relative_to: String,
    relative_point: String,
    x: String,
    y: String,
    /// Number of dynamic attrs still expected before we can apply.
    pub(crate) remaining_dynamic: usize,
}

impl AnchorState {
    pub(crate) fn new(remaining_dynamic: usize) -> Self {
        Self {
            point: "CENTER".to_string(),
            relative_to: "$parent".to_string(),
            relative_point: "CENTER".to_string(),
            x: "0".to_string(),
            y: "0".to_string(),
            remaining_dynamic,
        }
    }

    pub(crate) fn set(&mut self, name: &str, value: &str) {
        match name {
            "point" => self.point = value.to_string(),
            "relative_to" => self.relative_to = value.to_string(),
            "relative_point" => self.relative_point = value.to_string(),
            "x" => self.x = value.to_string(),
            "y" => self.y = value.to_string(),
            _ => {}
        }
    }

    fn to_spec(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.point, self.relative_to, self.relative_point, self.x, self.y
        )
    }
}

pub(crate) fn resolve_anchor_relative(
    registry: &FrameRegistry,
    frame_id: u64,
    name: &str,
) -> Option<u64> {
    if name == "$parent" {
        registry.get(frame_id).and_then(|f| f.parent_id)
    } else {
        registry.get_by_name(name)
    }
}

pub(crate) fn apply_anchor_resolved(registry: &mut FrameRegistry, frame_id: u64, s: &str) {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 5 {
        return;
    }
    let point = AnchorPoint::from_str(parts[0].trim()).unwrap_or(AnchorPoint::Center);
    let relative_name = parts[1].trim();
    let relative_point = AnchorPoint::from_str(parts[2].trim()).unwrap_or(AnchorPoint::Center);
    let x_offset: f32 = parts[3].trim().parse().unwrap_or(0.0);
    let y_offset: f32 = parts[4].trim().parse().unwrap_or(0.0);
    let relative_to = resolve_anchor_relative(registry, frame_id, relative_name);
    let anchor = Anchor {
        point,
        relative_to,
        relative_point,
        x_offset,
        y_offset,
    };
    if let Some(frame) = registry.get_mut(frame_id) {
        frame.anchors.push(anchor);
    }
}

/// Apply an all-static `anchor {}` child element to its parent frame.
/// Returns a pending spec if the relative frame isn't registered yet.
pub(crate) fn apply_anchor_element(
    node: &TemplateNode,
    parent_frame_id: u64,
    registry: &mut FrameRegistry,
) -> Option<(u64, String)> {
    let TemplateNode::Element { attrs, .. } = node else {
        return None;
    };
    let mut state = AnchorState::new(0);
    for attr in *attrs {
        if let dioxus_core::TemplateAttribute::Static { name, value, .. } = attr {
            state.set(name, value);
        }
    }
    apply_anchor_state(&state, parent_frame_id, registry)
}

/// Collect static attrs from an anchor template node, leaving defaults for dynamic attrs.
pub(crate) fn collect_anchor_statics(node: &TemplateNode, dynamic_count: usize) -> AnchorState {
    let TemplateNode::Element { attrs, .. } = node else {
        return AnchorState::new(dynamic_count);
    };
    let mut state = AnchorState::new(dynamic_count);
    for attr in *attrs {
        if let dioxus_core::TemplateAttribute::Static { name, value, .. } = attr {
            state.set(name, value);
        }
    }
    state
}

/// Apply an anchor from accumulated state. Returns pending if relative frame not found.
pub(crate) fn apply_anchor_state(
    state: &AnchorState,
    parent_frame_id: u64,
    registry: &mut FrameRegistry,
) -> Option<(u64, String)> {
    let spec = state.to_spec();
    if state.relative_to == "$parent"
        || resolve_anchor_relative(registry, parent_frame_id, &state.relative_to).is_some()
    {
        apply_anchor_resolved(registry, parent_frame_id, &spec);
        None
    } else {
        Some((parent_frame_id, spec))
    }
}
