use crate::anchor::{Anchor, AnchorPoint};
use crate::registry::FrameRegistry;
use crate::widget_def::AnchorDef;

/// Accumulated anchor attribute state for anchors with dynamic attrs.
/// Static attrs are filled during template instantiation, dynamic attrs arrive via set_attribute.
#[derive(Debug, Clone)]
pub(crate) struct AnchorState {
    point: String,
    relative_to: String,
    relative_point: String,
    x: String,
    y: String,
}

impl AnchorState {
    pub(crate) fn new() -> Self {
        Self {
            point: "CENTER".to_string(),
            relative_to: "$parent".to_string(),
            relative_point: "CENTER".to_string(),
            x: "0".to_string(),
            y: "0".to_string(),
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
    registry.rect_dirty.insert(frame_id);
}

pub(crate) fn apply_anchor_from_def(
    def: &AnchorDef,
    parent_frame_id: u64,
    registry: &mut FrameRegistry,
) -> Option<(u64, String)> {
    let mut state = AnchorState::new();
    state.set("point", &def.point);
    state.set("relative_to", &def.relative_to);
    state.set("relative_point", &def.relative_point);
    state.set("x", &def.x);
    state.set("y", &def.y);
    apply_anchor_state(&state, parent_frame_id, registry)
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
