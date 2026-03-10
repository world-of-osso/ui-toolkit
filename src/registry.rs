use std::collections::{HashMap, HashSet};

use crate::frame::{Frame, WidgetType};
use crate::layout::LayoutRect;

/// Central registry owning all UI frames, keyed by ID.
pub struct FrameRegistry {
    frames: HashMap<u64, Frame>,
    names: HashMap<String, u64>,
    next_id: u64,
    pub screen_width: f32,
    pub screen_height: f32,
    pub render_dirty: HashSet<u64>,
    pub rect_dirty: HashSet<u64>,
    pub anchor_dependents: HashMap<u64, HashSet<u64>>,
}

impl FrameRegistry {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            frames: HashMap::new(),
            names: HashMap::new(),
            next_id: 1,
            screen_width,
            screen_height,
            render_dirty: HashSet::new(),
            rect_dirty: HashSet::new(),
            anchor_dependents: HashMap::new(),
        }
    }

    pub fn screen_rect(&self) -> LayoutRect {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: self.screen_width,
            height: self.screen_height,
        }
    }

    pub fn mark_all_rects_dirty(&mut self) {
        self.rect_dirty.extend(self.frames.keys().copied());
    }

    /// Allocate an ID without creating a frame (for external creation).
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert a pre-built frame into the registry and wire up parent-child.
    pub fn insert_frame(&mut self, frame: Frame) {
        let id = frame.id;
        let parent_id = frame.parent_id;

        if let Some(n) = &frame.name {
            self.names.insert(n.clone(), id);
        }
        self.render_dirty.insert(id);
        self.rect_dirty.insert(id);
        self.frames.insert(id, frame);

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get_mut(&pid)
        {
            if !parent.children.contains(&id) {
                parent.children.push(id);
            }
        }
    }

    /// Remove a frame and unlink it from its parent.
    pub fn remove_frame(&mut self, id: u64) {
        if let Some(frame) = self.frames.remove(&id) {
            if let Some(name) = &frame.name {
                self.names.remove(name);
            }
            if let Some(pid) = frame.parent_id
                && let Some(parent) = self.frames.get_mut(&pid)
            {
                parent.children.retain(|&c| c != id);
            }
            self.render_dirty.remove(&id);
            self.rect_dirty.remove(&id);
            self.anchor_dependents.remove(&id);
        }
    }

    /// Remove a frame and its entire subtree.
    pub fn remove_frame_tree(&mut self, id: u64) {
        let children = self.get(id).map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_frame_tree(child_id);
        }
        self.remove_frame(id);
    }

    /// Create a new frame, inheriting effective properties from parent.
    pub fn create_frame(&mut self, name: &str, parent_id: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut frame = Frame::new(
            id,
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
            WidgetType::Frame,
        );
        frame.parent_id = parent_id;

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get(&pid)
        {
            frame.visible = parent.visible && frame.shown;
            frame.effective_alpha = parent.effective_alpha * frame.alpha;
            frame.effective_scale = parent.effective_scale * frame.scale;
            frame.frame_level = parent.frame_level + 1;
        }

        if let Some(n) = &frame.name {
            self.names.insert(n.clone(), id);
        }

        self.render_dirty.insert(id);
        self.rect_dirty.insert(id);

        // Must insert frame before mutating parent
        self.frames.insert(id, frame);

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get_mut(&pid)
        {
            parent.children.push(id);
        }

        id
    }

    pub fn get(&self, id: u64) -> Option<&Frame> {
        self.frames.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Frame> {
        self.render_dirty.insert(id);
        self.frames.get_mut(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<u64> {
        self.names.get(name).copied()
    }

    pub fn frames_iter(&self) -> impl Iterator<Item = &Frame> {
        self.frames.values()
    }

    pub fn set_point(
        &mut self,
        id: u64,
        anchor: crate::anchor::Anchor,
    ) -> Result<(), &'static str> {
        if !self.frames.contains_key(&id) {
            return Err("frame not found");
        }
        if anchor.relative_to == Some(id) {
            return Err("frame cannot anchor to itself");
        }
        if let Some(target_id) = anchor.relative_to
            && self.depends_on(target_id, id)
        {
            return Err("anchor cycle detected");
        }

        let previous_anchor = {
            let frame = self.frames.get_mut(&id).expect("checked above");
            if let Some(existing) = frame
                .anchors
                .iter_mut()
                .find(|existing| existing.point == anchor.point)
            {
                let previous = *existing;
                *existing = anchor;
                Some(previous)
            } else {
                frame.anchors.push(anchor);
                None
            }
        };

        if let Some(previous) = previous_anchor {
            self.unregister_anchor_dependency(id, previous);
        }
        self.register_anchor_dependency(id, anchor);
        self.mark_rect_dirty(id);
        Ok(())
    }

    pub fn clear_all_points(&mut self, id: u64) {
        let anchors = self
            .frames
            .get(&id)
            .map(|frame| frame.anchors.clone())
            .unwrap_or_default();
        for anchor in anchors {
            self.unregister_anchor_dependency(id, anchor);
        }
        if let Some(frame) = self.frames.get_mut(&id) {
            frame.anchors.clear();
        }
        self.mark_rect_dirty(id);
    }

    pub fn stretch_to_fill(
        &mut self,
        id: u64,
        relative_to: Option<u64>,
    ) -> Result<(), &'static str> {
        self.clear_all_points(id);
        self.set_point(
            id,
            crate::anchor::Anchor {
                point: crate::anchor::AnchorPoint::TopLeft,
                relative_to,
                relative_point: crate::anchor::AnchorPoint::TopLeft,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        )?;
        self.set_point(
            id,
            crate::anchor::Anchor {
                point: crate::anchor::AnchorPoint::BottomRight,
                relative_to,
                relative_point: crate::anchor::AnchorPoint::BottomRight,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        )
    }

    /// Set a frame's alpha and propagate effective_alpha down the subtree.
    /// Set a frame's name and update the name index.
    pub fn set_name(&mut self, id: u64, name: String) {
        // Remove old name from index.
        if let Some(frame) = self.frames.get(&id) {
            if let Some(old_name) = &frame.name {
                self.names.remove(old_name);
            }
        }
        self.names.insert(name.clone(), id);
        if let Some(frame) = self.frames.get_mut(&id) {
            frame.name = Some(name);
        }
    }

    pub fn set_alpha(&mut self, id: u64, alpha: f32) {
        let parent_effective = self.parent_effective_alpha(id);
        if let Some(frame) = self.frames.get_mut(&id) {
            frame.alpha = alpha;
            let new_effective = if frame.visible {
                parent_effective * alpha
            } else {
                0.0
            };
            frame.effective_alpha = new_effective;
            self.render_dirty.insert(id);
        }
        let children = self.child_ids(id);
        for child_id in children {
            self.propagate_alpha(child_id);
        }
    }

    /// Set a frame's shown state and propagate visibility + alpha down the subtree.
    pub fn set_shown(&mut self, id: u64, shown: bool) {
        let parent_visible = self.parent_visible(id);
        let parent_effective_alpha = self.parent_effective_alpha(id);
        if let Some(frame) = self.frames.get_mut(&id) {
            frame.shown = shown;
            frame.visible = parent_visible && shown;
            frame.effective_alpha = if frame.visible {
                parent_effective_alpha * frame.alpha
            } else {
                0.0
            };
            self.render_dirty.insert(id);
        }
        let children = self.child_ids(id);
        for child_id in children {
            self.propagate_visibility(child_id);
            self.propagate_alpha(child_id);
        }
    }

    /// Set a frame's scale and propagate effective_scale down the subtree.
    pub fn set_scale(&mut self, id: u64, scale: f32) {
        let parent_effective = self.parent_effective_scale(id);
        if let Some(frame) = self.frames.get_mut(&id) {
            frame.scale = scale;
            frame.effective_scale = parent_effective * scale;
            self.render_dirty.insert(id);
        }
        let children = self.child_ids(id);
        for child_id in children {
            self.propagate_scale(child_id);
        }
    }

    /// Return ordered child frame IDs for a given frame.
    pub fn children_of(&self, id: u64) -> Vec<u64> {
        self.frames
            .get(&id)
            .map(|f| f.children.clone())
            .unwrap_or_default()
    }

    pub fn parent_of(&self, id: u64) -> Option<u64> {
        self.frames.get(&id)?.parent_id
    }

    // --- helpers ---

    fn child_ids(&self, id: u64) -> Vec<u64> {
        self.frames
            .get(&id)
            .map(|f| f.children.clone())
            .unwrap_or_default()
    }

    fn parent_visible(&self, id: u64) -> bool {
        self.frames
            .get(&id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| self.frames.get(&pid))
            .is_none_or(|p| p.visible)
    }

    fn parent_effective_alpha(&self, id: u64) -> f32 {
        self.frames
            .get(&id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| self.frames.get(&pid))
            .map_or(1.0, |p| p.effective_alpha)
    }

    fn parent_effective_scale(&self, id: u64) -> f32 {
        self.frames
            .get(&id)
            .and_then(|f| f.parent_id)
            .and_then(|pid| self.frames.get(&pid))
            .map_or(1.0, |p| p.effective_scale)
    }

    fn propagate_visibility(&mut self, id: u64) {
        let parent_visible = self.parent_visible(id);
        let children = if let Some(frame) = self.frames.get_mut(&id) {
            frame.visible = parent_visible && frame.shown;
            self.render_dirty.insert(id);
            frame.children.clone()
        } else {
            return;
        };
        for child_id in children {
            self.propagate_visibility(child_id);
        }
    }

    fn propagate_alpha(&mut self, id: u64) {
        let parent_effective = self.parent_effective_alpha(id);
        let children = if let Some(frame) = self.frames.get_mut(&id) {
            frame.effective_alpha = if frame.visible {
                parent_effective * frame.alpha
            } else {
                0.0
            };
            self.render_dirty.insert(id);
            frame.children.clone()
        } else {
            return;
        };
        for child_id in children {
            self.propagate_alpha(child_id);
        }
    }

    fn propagate_scale(&mut self, id: u64) {
        let parent_effective = self.parent_effective_scale(id);
        let children = if let Some(frame) = self.frames.get_mut(&id) {
            frame.effective_scale = parent_effective * frame.scale;
            self.render_dirty.insert(id);
            frame.children.clone()
        } else {
            return;
        };
        for child_id in children {
            self.propagate_scale(child_id);
        }
    }

    fn register_anchor_dependency(&mut self, frame_id: u64, anchor: crate::anchor::Anchor) {
        if let Some(target_id) = anchor.relative_to {
            self.anchor_dependents
                .entry(target_id)
                .or_default()
                .insert(frame_id);
        }
    }

    fn unregister_anchor_dependency(&mut self, frame_id: u64, anchor: crate::anchor::Anchor) {
        let Some(target_id) = anchor.relative_to else {
            return;
        };
        let remove_entry = if let Some(dependents) = self.anchor_dependents.get_mut(&target_id) {
            dependents.remove(&frame_id);
            dependents.is_empty()
        } else {
            false
        };
        if remove_entry {
            self.anchor_dependents.remove(&target_id);
        }
    }

    fn mark_rect_dirty(&mut self, id: u64) {
        if !self.rect_dirty.insert(id) {
            return;
        }

        let mut dependents = self
            .anchor_dependents
            .get(&id)
            .map(|items| items.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        dependents.extend(self.child_ids(id));
        for dependent_id in dependents {
            self.mark_rect_dirty(dependent_id);
        }
    }

    fn depends_on(&self, start_id: u64, target_id: u64) -> bool {
        if start_id == target_id {
            return true;
        }

        let mut stack = vec![start_id];
        let mut visited = HashSet::new();
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            if current == target_id {
                return true;
            }
            if let Some(frame) = self.frames.get(&current) {
                for anchor in &frame.anchors {
                    if let Some(next) = anchor.relative_to {
                        stack.push(next);
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{Anchor, AnchorPoint};

    #[test]
    fn create_and_lookup_by_id() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let id = reg.create_frame("TestFrame", None);
        let frame = reg.get(id).unwrap();
        assert_eq!(frame.id, id);
        assert_eq!(frame.name.as_deref(), Some("TestFrame"));
        assert_eq!(frame.widget_type, WidgetType::Frame);
    }

    #[test]
    fn lookup_by_name() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let id = reg.create_frame("MyFrame", None);
        assert_eq!(reg.get_by_name("MyFrame"), Some(id));
        assert_eq!(reg.get_by_name("NoSuchFrame"), None);
    }

    #[test]
    fn parent_child_relationship() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        let child_frame = reg.get(child).unwrap();
        assert_eq!(child_frame.parent_id, Some(parent));

        let parent_frame = reg.get(parent).unwrap();
        assert!(parent_frame.children.contains(&child));
    }

    #[test]
    fn effective_alpha_propagation() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        reg.set_alpha(parent, 0.5);
        reg.set_alpha(child, 0.5);

        let child_frame = reg.get(child).unwrap();
        assert!((child_frame.effective_alpha - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn visibility_propagation() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        // Hide parent
        reg.set_shown(parent, false);

        let parent_frame = reg.get(parent).unwrap();
        assert!(!parent_frame.shown);
        assert!(!parent_frame.visible);

        let child_frame = reg.get(child).unwrap();
        // Child's shown stays true, but visible becomes false
        assert!(child_frame.shown);
        assert!(!child_frame.visible);

        // Show parent again
        reg.set_shown(parent, true);
        let child_frame = reg.get(child).unwrap();
        assert!(child_frame.visible);
    }

    #[test]
    fn hidden_frame_effective_alpha_zero() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        reg.set_alpha(child, 0.8);
        reg.set_shown(parent, false);

        let child_frame = reg.get(child).unwrap();
        assert!((child_frame.effective_alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_propagation() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        reg.set_scale(parent, 2.0);
        reg.set_scale(child, 0.5);

        let child_frame = reg.get(child).unwrap();
        assert!((child_frame.effective_scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn frame_level_inheritance() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let root = reg.create_frame("Root", None);
        let mid = reg.create_frame("Mid", Some(root));
        let leaf = reg.create_frame("Leaf", Some(mid));

        assert_eq!(reg.get(root).unwrap().frame_level, 0);
        assert_eq!(reg.get(mid).unwrap().frame_level, 1);
        assert_eq!(reg.get(leaf).unwrap().frame_level, 2);
    }

    #[test]
    fn screen_rect() {
        let reg = FrameRegistry::new(1920.0, 1080.0);
        let rect = reg.screen_rect();
        assert!((rect.x - 0.0).abs() < f32::EPSILON);
        assert!((rect.y - 0.0).abs() < f32::EPSILON);
        assert!((rect.width - 1920.0).abs() < f32::EPSILON);
        assert!((rect.height - 1080.0).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_name_not_registered() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let id = reg.create_frame("", None);
        let frame = reg.get(id).unwrap();
        assert!(frame.name.is_none());
        assert_eq!(reg.get_by_name(""), None);
    }

    fn test_anchor(
        point: AnchorPoint,
        relative_to: Option<u64>,
        relative_point: AnchorPoint,
    ) -> Anchor {
        Anchor {
            point,
            relative_to,
            relative_point,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    #[test]
    fn set_point_tracks_dependents_and_marks_rect_dirty() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let target = reg.create_frame("Target", None);
        let child = reg.create_frame("Child", None);

        reg.set_point(
            child,
            test_anchor(AnchorPoint::TopLeft, Some(target), AnchorPoint::BottomRight),
        )
        .unwrap();

        let frame = reg.get(child).unwrap();
        assert_eq!(frame.anchors.len(), 1);
        assert_eq!(frame.anchors[0].point, AnchorPoint::TopLeft);
        assert_eq!(frame.anchors[0].relative_to, Some(target));
        assert!(reg.rect_dirty.contains(&child));
        assert!(reg.anchor_dependents[&target].contains(&child));
    }

    #[test]
    fn set_point_replaces_existing_point_and_updates_dependents() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let first = reg.create_frame("First", None);
        let second = reg.create_frame("Second", None);
        let child = reg.create_frame("Child", None);

        reg.set_point(
            child,
            test_anchor(AnchorPoint::Center, Some(first), AnchorPoint::Center),
        )
        .unwrap();
        reg.set_point(
            child,
            test_anchor(AnchorPoint::Center, Some(second), AnchorPoint::TopLeft),
        )
        .unwrap();

        let frame = reg.get(child).unwrap();
        assert_eq!(frame.anchors.len(), 1);
        assert_eq!(frame.anchors[0].relative_to, Some(second));
        assert!(
            reg.anchor_dependents
                .get(&first)
                .is_none_or(|dependents| !dependents.contains(&child))
        );
        assert!(reg.anchor_dependents[&second].contains(&child));
    }

    #[test]
    fn clear_all_points_removes_dependencies() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let target = reg.create_frame("Target", None);
        let child = reg.create_frame("Child", None);

        reg.set_point(
            child,
            test_anchor(AnchorPoint::TopLeft, Some(target), AnchorPoint::TopLeft),
        )
        .unwrap();

        reg.clear_all_points(child);

        let frame = reg.get(child).unwrap();
        assert!(frame.anchors.is_empty());
        assert!(reg.rect_dirty.contains(&child));
        assert!(
            reg.anchor_dependents
                .get(&target)
                .is_none_or(|dependents| !dependents.contains(&child))
        );
    }

    #[test]
    fn stretch_to_fill_creates_stretch_anchors() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let target = reg.create_frame("Target", None);
        let child = reg.create_frame("Child", None);

        reg.stretch_to_fill(child, Some(target)).unwrap();

        let frame = reg.get(child).unwrap();
        assert_eq!(frame.anchors.len(), 2);
        assert_eq!(frame.anchors[0].point, AnchorPoint::TopLeft);
        assert_eq!(frame.anchors[1].point, AnchorPoint::BottomRight);
        assert_eq!(frame.anchors[0].relative_to, Some(target));
        assert_eq!(frame.anchors[1].relative_to, Some(target));
        assert!(reg.anchor_dependents[&target].contains(&child));
    }
}
