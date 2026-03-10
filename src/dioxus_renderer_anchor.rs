use dioxus_core::{AttributeValue, ElementId};

use crate::dioxus_anchor::{apply_anchor_resolved, apply_anchor_state};
use crate::dioxus_attrs::as_text;
use crate::dioxus_renderer::{GameUiRenderer, NodeKind};
use crate::registry::FrameRegistry;

impl GameUiRenderer {
    /// Resolve anchors that referenced named frames not yet registered at apply time.
    /// Skips if the frame already has an anchor (from a later `set_attribute` that resolved it).
    pub fn resolve_pending_anchors(&mut self, registry: &mut FrameRegistry) {
        let pending = std::mem::take(&mut self.pending_anchors);
        for (frame_id, spec) in pending {
            let already_has = registry
                .get(frame_id)
                .is_some_and(|f| !f.anchors.is_empty());
            if !already_has {
                apply_anchor_resolved(registry, frame_id, &spec);
            }
        }
    }

    /// Handle a `set_attribute` call for an anchor pseudo-element.
    /// Returns `None` if the element is not an anchor, `Some(None)` if applied,
    /// or `Some(Some(pending))` if the relative frame is unresolved.
    pub(crate) fn try_set_anchor_attr(
        &mut self,
        name: &str,
        value: &AttributeValue,
        id: ElementId,
        registry: &mut FrameRegistry,
    ) -> Option<(u64, String)> {
        let parent_frame_id = match self.nodes.get(id.0)? {
            Some(NodeKind::Anchor { parent_frame_id }) => *parent_frame_id,
            _ => return None,
        };
        let text = as_text(value)?;
        let idx = self
            .template_anchor_nodes
            .iter()
            .position(|(_, pfid, _)| *pfid == parent_frame_id)?;
        let state = &mut self.template_anchor_nodes[idx].2;
        state.set(name, text);
        state.remaining_dynamic -= 1;
        if state.remaining_dynamic == 0 {
            let state = self.template_anchor_nodes.remove(idx).2;
            return apply_anchor_state(&state, parent_frame_id, registry);
        }
        None
    }
}
