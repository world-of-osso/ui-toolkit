use std::collections::HashSet;

use crate::dioxus_renderer::{GameUiRenderer, NodeKind};
use crate::registry::FrameRegistry;

impl GameUiRenderer {
    pub(crate) fn remove_frame_tree(&mut self, fid: u64, registry: &mut FrameRegistry) {
        let preserved_reused = Self::collect_subtree(registry, fid)
            .into_iter()
            .filter(|id| {
                *id != fid && self.reused_frame_ids.contains(id) && self.is_frame_referenced(*id)
            })
            .collect::<HashSet<_>>();
        let subtree = Self::collect_subtree_excluding(registry, fid, &preserved_reused);
        for &preserved_fid in &preserved_reused {
            self.detach_from_removed_parent(preserved_fid, &subtree, registry);
        }
        for &id in &subtree {
            registry.remove_frame(id);
        }
        self.created_frames.retain(|id| !subtree.contains(id));
        self.clear_stale_nodes(&subtree);
        self.reused_frame_ids.retain(|id| !subtree.contains(id));
    }

    fn collect_subtree(registry: &FrameRegistry, root: u64) -> HashSet<u64> {
        Self::collect_subtree_excluding(registry, root, &HashSet::new())
    }

    fn collect_subtree_excluding(
        registry: &FrameRegistry,
        root: u64,
        exclude: &HashSet<u64>,
    ) -> HashSet<u64> {
        let mut result = HashSet::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if id != root && exclude.contains(&id) {
                continue;
            }
            result.insert(id);
            if let Some(frame) = registry.get(id) {
                stack.extend_from_slice(&frame.children);
            }
        }
        result
    }

    /// Clear node slots that reference any of the given frame IDs.
    fn clear_stale_nodes(&mut self, removed: &HashSet<u64>) {
        for slot in &mut self.nodes {
            let stale = matches!(
                slot,
                Some(NodeKind::Element { frame_id }) | Some(NodeKind::Text { frame_id })
                    if removed.contains(frame_id)
            );
            if stale {
                *slot = None;
            }
        }
    }

    fn is_frame_referenced(&self, fid: u64) -> bool {
        self.nodes.iter().any(|slot| {
            matches!(
                slot,
                Some(NodeKind::Element { frame_id }) | Some(NodeKind::Text { frame_id })
                    if *frame_id == fid
            )
        })
    }

    fn detach_from_removed_parent(
        &self,
        fid: u64,
        removed: &HashSet<u64>,
        registry: &mut FrameRegistry,
    ) {
        let parent_id = registry.get(fid).and_then(|frame| frame.parent_id);
        let Some(parent_id) = parent_id else {
            return;
        };
        if !removed.contains(&parent_id) {
            return;
        }
        if let Some(parent) = registry.get_mut(parent_id) {
            parent.children.retain(|&child_id| child_id != fid);
        }
        if let Some(frame) = registry.get_mut(fid) {
            frame.parent_id = None;
        }
    }
}
