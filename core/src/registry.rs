use std::collections::{HashMap, HashSet};

use crate::layout_values::{PositionType, UiRect, Val};

use crate::anchor::AnchorTarget;
use crate::frame::{Frame, NineSlice, ThreeSlice, WidgetData, WidgetType};
use crate::layout::LayoutRect;
use crate::widgets::scroll_list::{ScrollGeometry, ScrollLists};
use crate::widgets::texture::{DynamicTexture, DynamicTextureId, TextureSource};

/// Central registry owning all UI frames, keyed by ID.
pub struct FrameRegistry {
    frames: HashMap<u64, Frame>,
    names: HashMap<String, u64>,
    next_id: u64,
    next_dynamic_texture_id: u64,
    dynamic_textures: HashMap<DynamicTextureId, DynamicTexture>,
    /// Screen size in UI units: window logical size divided by [`Self::ui_scale`].
    pub screen_width: f32,
    pub screen_height: f32,
    /// Logical pixels per UI unit, derived from the UI camera projection.
    pub ui_scale: f32,
    pub render_dirty: HashSet<u64>,
    /// Deferred [`Self::get_mut`] write windows pending diff at the next boundary.
    ///
    /// Each entry records the frame state at access and whether that access created
    /// the `render_dirty` mark, so resolution retracts only marks created by write
    /// windows that left the frame unchanged.
    pub(crate) pending_writes: HashMap<u64, (Frame, bool)>,
    pub rect_dirty: HashSet<u64>,
    /// Frames removed since the last [`Self::drain_removed_frames`]; the host
    /// drops their event listeners and stale focus ownership.
    pub removed_frames: Vec<u64>,
    /// Parents whose child order changed after their children existed; the host
    /// reorders its own child nodes to match, then clears the set.
    pub child_order_dirty: HashSet<u64>,
    pub focused_frame: Option<u64>,
    pub(crate) panel_styles: HashMap<String, NineSlice>,
    pub(crate) three_slice_styles: HashMap<String, ThreeSlice>,
    pub scroll_lists: ScrollLists,
    /// Loading fontstrings and their base text, animated by `animate_loading_texts`.
    pub loading_texts: HashMap<u64, String>,
}

impl FrameRegistry {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            frames: HashMap::new(),
            names: HashMap::new(),
            next_id: 1,
            next_dynamic_texture_id: 1,
            dynamic_textures: HashMap::new(),
            screen_width,
            screen_height,
            ui_scale: 1.0,
            render_dirty: HashSet::new(),
            pending_writes: HashMap::new(),
            rect_dirty: HashSet::new(),
            removed_frames: Vec::new(),
            child_order_dirty: HashSet::new(),
            focused_frame: None,
            panel_styles: HashMap::new(),
            three_slice_styles: HashMap::new(),
            scroll_lists: ScrollLists::default(),
            loading_texts: HashMap::new(),
        }
    }

    /// Create a runtime image whose ID remains stable through pixel updates.
    pub fn create_dynamic_texture(
        &mut self,
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
    ) -> Result<DynamicTextureId, &'static str> {
        let texture = DynamicTexture::new(width, height, rgba8)?;
        let id = DynamicTextureId(self.next_dynamic_texture_id);
        self.next_dynamic_texture_id = self
            .next_dynamic_texture_id
            .checked_add(1)
            .ok_or("dynamic texture IDs exhausted")?;
        self.dynamic_textures.insert(id, texture);
        Ok(id)
    }

    pub fn dynamic_texture(&self, id: DynamicTextureId) -> Option<&DynamicTexture> {
        self.dynamic_textures.get(&id)
    }

    /// Replace pixel data without changing the texture source stored on a frame.
    pub fn update_dynamic_texture(
        &mut self,
        id: DynamicTextureId,
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
    ) -> Result<(), &'static str> {
        let replacement = DynamicTexture::new(width, height, rgba8)?;
        let texture = self
            .dynamic_textures
            .get_mut(&id)
            .ok_or("unknown dynamic texture ID")?;
        if *texture != replacement {
            *texture = replacement;
            self.resolve_pending_writes();
            self.mark_dynamic_texture_frames_dirty(id);
        }
        Ok(())
    }

    fn mark_dynamic_texture_frames_dirty(&mut self, id: DynamicTextureId) {
        self.render_dirty.extend(
            self.frames
                .iter()
                .filter(|(_, frame)| frame_uses_dynamic_texture(frame, id))
                .map(|(&frame_id, _)| frame_id),
        );
    }

    pub fn remove_dynamic_texture(&mut self, id: DynamicTextureId) -> Option<DynamicTexture> {
        let removed = self.dynamic_textures.remove(&id)?;
        self.resolve_pending_writes();
        self.mark_dynamic_texture_frames_dirty(id);
        Some(removed)
    }

    pub fn screen_rect(&self) -> LayoutRect {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: self.screen_width,
            height: self.screen_height,
        }
    }

    /// Focus a frame by click. Editboxes get focused and select-all; returns the onclick action otherwise.
    pub fn click_frame(&mut self, id: u64) -> Option<String> {
        self.resolve_pending_writes();
        let frame = self.frames.get(&id)?;
        if frame.is_editbox() {
            self.focused_frame = Some(id);
            self.select_all_editbox(id);
            None
        } else {
            frame.onclick.clone()
        }
    }

    pub fn select_all_editbox(&mut self, id: u64) {
        self.resolve_pending_writes();
        if let Some(WidgetData::EditBox(eb)) = self.get_mut(id).and_then(|f| f.widget_data.as_mut())
        {
            eb.cursor_position = eb.text.len();
        }
    }

    pub fn mark_all_rects_dirty(&mut self) {
        self.resolve_pending_writes();
        self.rect_dirty.extend(self.frames.keys().copied());
    }

    /// Allocate an ID without creating a frame (for external creation).
    pub fn next_id(&mut self) -> u64 {
        self.resolve_pending_writes();
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Insert a pre-built frame into the registry and wire up parent-child.
    pub fn insert_frame(&mut self, mut frame: Frame) {
        self.resolve_pending_writes();
        let id = frame.id;
        let parent_id = frame.parent_id;

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get(&pid)
        {
            frame.strata = parent.strata;
            frame.visible = parent.visible && !frame.hidden;
            frame.effective_alpha = parent.effective_alpha * frame.alpha;
            frame.effective_scale = parent.effective_scale * frame.scale;
            frame.frame_level = parent.frame_level + 1;
        }

        if let Some(n) = &frame.name {
            self.names.insert(n.clone(), id);
        }
        self.render_dirty.insert(id);
        self.frames.insert(id, frame);

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get_mut(&pid)
        {
            parent.children.push(id);
        }
        self.mark_rect_dirty(id);
    }

    /// Remove a frame and unlink it from its parent.
    pub fn remove_frame(&mut self, id: u64) {
        self.resolve_pending_writes();
        if let Some(frame) = self.frames.remove(&id) {
            if self.focused_frame == Some(id) {
                self.focused_frame = None;
            }
            self.removed_frames.push(id);
            self.loading_texts.remove(&id);
            if let Some(name) = &frame.name {
                self.names.remove(name);
            }
            if let Some(pid) = frame.parent_id
                && let Some(parent) = self.frames.get_mut(&pid)
            {
                parent.children.retain(|&c| c != id);
            }
            if let Some(parent_id) = frame.parent_id {
                self.mark_rect_dirty(parent_id);
            }
            self.render_dirty.remove(&id);
            self.rect_dirty.remove(&id);
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
        self.resolve_pending_writes();
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
            frame.strata = parent.strata;
            frame.visible = parent.visible && !frame.hidden;
            frame.effective_alpha = parent.effective_alpha * frame.alpha;
            frame.effective_scale = parent.effective_scale * frame.scale;
            frame.frame_level = parent.frame_level + 1;
        }

        if let Some(n) = &frame.name {
            self.names.insert(n.clone(), id);
        }

        self.render_dirty.insert(id);

        // Must insert frame before mutating parent
        self.frames.insert(id, frame);

        if let Some(pid) = parent_id
            && let Some(parent) = self.frames.get_mut(&pid)
        {
            parent.children.push(id);
        }
        self.mark_rect_dirty(id);

        id
    }

    pub fn get(&self, id: u64) -> Option<&Frame> {
        self.frames.get(&id)
    }

    /// Take the IDs removed since the last drain.
    pub fn drain_removed_frames(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.removed_frames)
    }

    /// Mutate a frame with deferred change detection.
    ///
    /// Access marks the frame dirty immediately (existing publication semantics);
    /// the mark is retracted at the next [`Self::resolve_pending_writes`] boundary
    /// when the write window left the frame unchanged, so unchanged-value writes
    /// end up publishing nothing.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Frame> {
        self.resolve_pending_writes();
        let frame = self.frames.get_mut(&id)?;
        let fresh_mark = self.render_dirty.insert(id);
        self.pending_writes.insert(id, (frame.clone(), fresh_mark));
        Some(frame)
    }

    /// Close deferred [`Self::get_mut`] write windows: frames left unchanged lose
    /// the dirty mark their access created; real changes keep publishing. Runs at
    /// every mutating boundary and before render consumers read dirty state.
    pub fn resolve_pending_writes(&mut self) {
        for (id, (snapshot, fresh_mark)) in std::mem::take(&mut self.pending_writes) {
            if fresh_mark && self.frames.get(&id) == Some(&snapshot) {
                self.render_dirty.remove(&id);
            }
        }
    }

    /// Record a named frame's `scroll_list` geometry; see [`ScrollGeometry::parse_attr`].
    pub(crate) fn configure_scroll_list(&mut self, id: u64, value: &str) {
        let geometry = ScrollGeometry::parse_attr(value);
        let Some(name) = self.frames.get(&id).and_then(|f| f.name.as_deref()) else {
            panic!("scroll_list requires a named frame (id={id})");
        };
        self.scroll_lists.configure(name, geometry);
    }

    /// Mark a fontstring as a loading indicator with `text` as its base; empty clears it.
    pub(crate) fn set_loading_text(&mut self, id: u64, text: &str) {
        if text.is_empty() {
            self.loading_texts.remove(&id);
        } else {
            self.loading_texts.insert(id, text.to_string());
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<u64> {
        self.names.get(name).copied()
    }

    pub fn frames_iter(&self) -> impl Iterator<Item = &Frame> {
        self.frames.values()
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Whether a native projection built from `projected_frames` frames is stale:
    /// every mutation publishes a dirty mark except removals, which change the count.
    pub fn projection_outdated(&self, projected_frames: usize) -> bool {
        !self.render_dirty.is_empty()
            || !self.rect_dirty.is_empty()
            || self.frames.len() != projected_frames
    }

    /// Set parent/screen-space offsets without changing position type or logical parent.
    pub fn set_pos(&mut self, id: u64, x: f32, y: f32) -> Result<(), &'static str> {
        self.resolve_pending_writes();
        if !x.is_finite() || !y.is_finite() {
            return Err("position must be finite");
        }
        let frame = self.frames.get_mut(&id).ok_or("frame not found")?;
        let position = UiRect {
            left: Val::Px(x),
            top: Val::Px(y),
            right: Val::Auto,
            bottom: Val::Auto,
        };
        if frame.position != position {
            frame.position = position;
            self.mark_rect_dirty(id);
        }
        Ok(())
    }

    pub fn set_pos_type(
        &mut self,
        id: u64,
        position_type: PositionType,
    ) -> Result<(), &'static str> {
        self.resolve_pending_writes();
        let frame = self.frames.get_mut(&id).ok_or("frame not found")?;
        if frame.position_type != position_type {
            frame.position_type = position_type;
            self.mark_rect_dirty(id);
        }
        Ok(())
    }

    /// Select a layout reference space without reparenting the logical frame.
    pub fn set_anchor(&mut self, id: u64, anchor: AnchorTarget) -> Result<(), &'static str> {
        self.resolve_pending_writes();
        let frame = self.frames.get_mut(&id).ok_or("frame not found")?;
        if frame.anchor != anchor {
            frame.anchor = anchor;
            self.mark_rect_dirty(id);
        }
        Ok(())
    }

    /// Store layout readback without invalidating authored layout or its relatives.
    pub fn set_computed_layout(&mut self, id: u64, rect: LayoutRect) -> Result<(), &'static str> {
        self.resolve_pending_writes();
        let frame = self.frames.get_mut(&id).ok_or("frame not found")?;
        if frame.layout_rect.as_ref() != Some(&rect) {
            frame.layout_rect = Some(rect);
            self.render_dirty.insert(id);
        }
        Ok(())
    }

    /// Set a frame's alpha and propagate effective_alpha down the subtree.
    /// Set a frame's name and update the name index.
    pub fn set_name(&mut self, id: u64, name: String) {
        self.resolve_pending_writes();
        // Remove old name from index.
        if let Some(frame) = self.frames.get(&id) {
            if let Some(old_name) = &frame.name {
                self.names.remove(old_name);
            }
        }
        self.names.insert(name.clone(), id);
        if let Some(frame) = self.frames.get_mut(&id)
            && frame.name.as_ref() != Some(&name)
        {
            frame.name = Some(name);
            self.render_dirty.insert(id);
        }
    }

    pub fn set_alpha(&mut self, id: u64, alpha: f32) {
        self.resolve_pending_writes();
        let parent_effective = self.parent_effective_alpha(id);
        if let Some(frame) = self.frames.get_mut(&id) {
            let new_effective = if frame.visible {
                parent_effective * alpha
            } else {
                0.0
            };
            let mut changed = false;
            if frame.alpha != alpha {
                frame.alpha = alpha;
                changed = true;
            }
            if frame.effective_alpha != new_effective {
                frame.effective_alpha = new_effective;
                changed = true;
            }
            if changed {
                self.render_dirty.insert(id);
            }
        }
        let children = self.child_ids(id);
        for child_id in children {
            self.propagate_alpha(child_id);
        }
    }

    /// Set a frame's hidden state and propagate visibility + alpha down the subtree.
    pub fn set_hidden(&mut self, id: u64, hidden: bool) {
        self.resolve_pending_writes();
        let parent_visible = self.parent_visible(id);
        let parent_effective_alpha = self.parent_effective_alpha(id);
        if let Some(frame) = self.frames.get_mut(&id) {
            let visible = parent_visible && !hidden;
            let effective_alpha = if visible {
                parent_effective_alpha * frame.alpha
            } else {
                0.0
            };
            let mut changed = false;
            if frame.hidden != hidden {
                frame.hidden = hidden;
                changed = true;
            }
            if frame.visible != visible {
                frame.visible = visible;
                changed = true;
            }
            if frame.effective_alpha != effective_alpha {
                frame.effective_alpha = effective_alpha;
                changed = true;
            }
            if changed {
                self.render_dirty.insert(id);
            }
        }
        let children = self.child_ids(id);
        for child_id in children {
            self.propagate_visibility_and_alpha(child_id);
        }
    }

    /// Set a frame's scale and propagate effective_scale down the subtree.
    pub fn set_scale(&mut self, id: u64, scale: f32) {
        self.resolve_pending_writes();
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

    /// Replace `parent`'s children with `order`, the same frames in a new order.
    /// An unchanged order is a no-op; a changed one is marked for layout and host
    /// node reordering ([`Self::child_order_dirty`]).
    pub fn set_child_order(&mut self, parent: u64, order: Vec<u64>) {
        self.resolve_pending_writes();
        let frame = self
            .frames
            .get_mut(&parent)
            .expect("set_child_order parent exists");
        if frame.children == order {
            return;
        }
        debug_assert_eq!(frame.children.len(), order.len());
        frame.children = order;
        self.child_order_dirty.insert(parent);
        self.mark_rect_dirty(parent);
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

    fn propagate_visibility_and_alpha(&mut self, id: u64) {
        let parent_visible = self.parent_visible(id);
        let parent_effective = self.parent_effective_alpha(id);
        let children = if let Some(frame) = self.frames.get_mut(&id) {
            let visible = parent_visible && !frame.hidden;
            let effective_alpha = if visible {
                parent_effective * frame.alpha
            } else {
                0.0
            };
            if frame.visible != visible {
                frame.visible = visible;
                self.render_dirty.insert(id);
            }
            if frame.effective_alpha != effective_alpha {
                frame.effective_alpha = effective_alpha;
                self.render_dirty.insert(id);
            }
            frame.children.clone()
        } else {
            return;
        };
        for child_id in children {
            self.propagate_visibility_and_alpha(child_id);
        }
    }

    fn propagate_alpha(&mut self, id: u64) {
        let parent_effective = self.parent_effective_alpha(id);
        let children = if let Some(frame) = self.frames.get_mut(&id) {
            let effective_alpha = if frame.visible {
                parent_effective * frame.alpha
            } else {
                0.0
            };
            if frame.effective_alpha != effective_alpha {
                frame.effective_alpha = effective_alpha;
                self.render_dirty.insert(id);
            }
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

    /// Invalidate authored layout for the logical subtree and flex parent.
    /// Computed rectangles are retained until Bevy publishes new observations.
    pub fn mark_rect_dirty(&mut self, id: u64) {
        self.resolve_pending_writes();
        if !self.rect_dirty.insert(id) {
            return;
        }
        self.render_dirty.insert(id);
        if let Some(parent_id) = self.frames.get(&id).and_then(|frame| frame.parent_id)
            && self
                .frames
                .get(&parent_id)
                .is_some_and(|parent| parent.flex_layout.is_some())
        {
            self.mark_rect_dirty(parent_id);
        }
        for child_id in self.child_ids(id) {
            self.mark_rect_dirty(child_id);
        }
    }
}

fn frame_uses_dynamic_texture(frame: &Frame, id: DynamicTextureId) -> bool {
    let uses = |source: &TextureSource| *source == TextureSource::Dynamic(id);
    let optional = |source: &Option<TextureSource>| source.as_ref().is_some_and(uses);
    let widget_uses = match &frame.widget_data {
        Some(WidgetData::Texture(texture)) => uses(&texture.source),
        Some(WidgetData::Button(button)) => {
            optional(&button.normal_texture)
                || optional(&button.pushed_texture)
                || optional(&button.highlight_texture)
                || optional(&button.disabled_texture)
        }
        Some(WidgetData::Slider(slider)) => optional(&slider.thumb_texture),
        Some(WidgetData::StatusBar(bar)) => optional(&bar.texture),
        _ => false,
    };
    widget_uses
        || frame.nine_slice.as_ref().is_some_and(|slice| {
            optional(&slice.texture)
                || slice
                    .part_textures
                    .as_ref()
                    .is_some_and(|parts| parts.iter().any(uses))
        })
        || frame
            .three_slice
            .as_ref()
            .is_some_and(|slice| uses(&slice.left) || uses(&slice.center) || uses(&slice.right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strata::FrameStrata;
    use crate::widgets::texture::{TextureData, TextureSource};

    #[test]
    fn dynamic_texture_update_preserves_identity_and_invalidates_referencing_frame() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let first = reg
            .create_dynamic_texture(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255])
            .unwrap();
        let second = reg
            .create_dynamic_texture(1, 1, vec![0, 255, 0, 255])
            .unwrap();
        assert_ne!(first, second);
        let frame = reg.create_frame("icon", None);
        reg.get_mut(frame).unwrap().widget_data = Some(WidgetData::Texture(TextureData {
            source: TextureSource::Dynamic(first),
            ..Default::default()
        }));
        let unrelated = reg.create_frame("unrelated", None);
        reg.resolve_pending_writes();
        reg.render_dirty.clear();

        reg.update_dynamic_texture(first, 1, 1, vec![24, 32, 48, 255])
            .unwrap();
        assert_eq!(reg.dynamic_texture(first).unwrap().rgba8, [24, 32, 48, 255]);
        assert_eq!(
            (
                reg.dynamic_texture(first).unwrap().width,
                reg.dynamic_texture(first).unwrap().height
            ),
            (1, 1)
        );
        assert_eq!(reg.dynamic_texture(second).unwrap().rgba8, [0, 255, 0, 255]);
        assert_eq!(
            reg.get(frame)
                .unwrap()
                .widget_data
                .as_ref()
                .and_then(|data| match data {
                    WidgetData::Texture(texture) => Some(&texture.source),
                    _ => None,
                }),
            Some(&TextureSource::Dynamic(first))
        );
        assert_eq!(reg.render_dirty, HashSet::from([frame]));
        assert!(!reg.render_dirty.contains(&unrelated));
    }

    #[test]
    fn invalid_dynamic_texture_data_does_not_replace_existing_image() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        assert!(reg.create_dynamic_texture(2, 1, vec![0; 4]).is_err());
        let id = reg.create_dynamic_texture(1, 1, vec![1, 2, 3, 4]).unwrap();
        assert!(reg.update_dynamic_texture(id, 2, 1, vec![0; 4]).is_err());
        assert_eq!(reg.dynamic_texture(id).unwrap().rgba8, [1, 2, 3, 4]);
        assert!(reg.update_dynamic_texture(id, 0, 1, vec![]).is_err());
        assert!(
            reg.update_dynamic_texture(super::DynamicTextureId(999), 1, 1, vec![0; 4])
                .is_err()
        );
    }

    #[test]
    fn set_pos_replaces_edges_and_preserves_position_mode() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let id = reg.create_frame("Panel", None);
        reg.get_mut(id).unwrap().position = UiRect::all(Val::Px(8.0));
        reg.rect_dirty.clear();
        reg.render_dirty.clear();
        reg.set_pos(id, 12.0, -7.0).unwrap();
        let frame = reg.get(id).unwrap();
        assert_eq!(
            frame.position,
            UiRect {
                left: Val::Px(12.0),
                top: Val::Px(-7.0),
                right: Val::Auto,
                bottom: Val::Auto,
            }
        );
        assert_eq!(frame.position_type, PositionType::Relative);
        assert_eq!(reg.rect_dirty, HashSet::from([id]));
        assert_eq!(reg.render_dirty, HashSet::from([id]));
    }

    #[test]
    fn set_pos_rejects_nonfinite_values_without_mutation() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let id = reg.create_frame("Panel", None);
        reg.set_pos(id, 12.0, 9.0).unwrap();
        reg.rect_dirty.clear();
        reg.render_dirty.clear();
        let before = reg.get(id).unwrap().position;
        for (x, y) in [
            (f32::NAN, 0.0),
            (0.0, f32::NAN),
            (f32::INFINITY, 0.0),
            (0.0, f32::NEG_INFINITY),
        ] {
            assert_eq!(reg.set_pos(id, x, y), Err("position must be finite"));
            assert_eq!(reg.get(id).unwrap().position, before);
        }
        assert!(reg.rect_dirty.is_empty());
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn authored_setters_reject_missing_frames() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        assert_eq!(reg.set_pos(42, 1.0, 2.0), Err("frame not found"));
        assert_eq!(
            reg.set_pos_type(42, PositionType::Absolute),
            Err("frame not found")
        );
        assert_eq!(
            reg.set_anchor(42, AnchorTarget::Screen),
            Err("frame not found")
        );
        assert_eq!(
            reg.set_computed_layout(42, reg.screen_rect()),
            Err("frame not found")
        );
        assert!(reg.rect_dirty.is_empty());
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn screen_target_and_absolute_position_preserve_logical_parentage() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let root = reg.create_frame("Root", None);
        let child = reg.create_frame("Child", Some(root));
        reg.set_pos(child, 30.0, 40.0).unwrap();
        reg.set_pos_type(child, PositionType::Absolute).unwrap();
        reg.set_anchor(child, AnchorTarget::Screen).unwrap();
        let frame = reg.get(child).unwrap();
        assert_eq!(frame.anchor, AnchorTarget::Screen);
        assert_eq!(frame.position_type, PositionType::Absolute);
        assert_eq!(frame.parent_id, Some(root));
        assert_eq!(reg.children_of(root), vec![child]);
        assert_eq!(reg.get_by_name("Child"), Some(child));
        reg.set_anchor(child, AnchorTarget::Parent).unwrap();
        reg.set_pos_type(child, PositionType::Relative).unwrap();
        assert_eq!(reg.get(child).unwrap().position.left, Val::Px(30.0));
        assert_eq!(reg.parent_of(child), Some(root));
    }

    #[test]
    fn repeated_authored_values_leave_dirty_sets_empty() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let id = reg.create_frame("Panel", None);
        reg.set_pos(id, 12.0, 7.0).unwrap();
        reg.set_pos_type(id, PositionType::Absolute).unwrap();
        reg.set_anchor(id, AnchorTarget::Screen).unwrap();
        reg.rect_dirty.clear();
        reg.render_dirty.clear();
        reg.set_pos(id, 12.0, 7.0).unwrap();
        reg.set_pos_type(id, PositionType::Absolute).unwrap();
        reg.set_anchor(id, AnchorTarget::Screen).unwrap();
        assert!(reg.rect_dirty.is_empty());
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn authored_geometry_invalidates_subtree_and_flex_parent() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let root = reg.create_frame("Row", None);
        reg.get_mut(root).unwrap().flex_layout = Some(crate::frame::FlexLayout::default());
        let child = reg.create_frame("Child", Some(root));
        let sibling = reg.create_frame("Sibling", Some(root));
        let unrelated = reg.create_frame("Other", None);
        reg.rect_dirty.clear();
        reg.render_dirty.clear();
        reg.set_pos(child, 5.0, 6.0).unwrap();
        assert_eq!(reg.rect_dirty, HashSet::from([root, child, sibling]));
        assert!(!reg.render_dirty.contains(&unrelated));
    }

    #[test]
    fn computed_layout_updates_observation_without_authored_invalidation() {
        use crate::frame::Dimension;
        use crate::layout_values::Val2;
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let root = reg.create_frame("Root", None);
        let child = reg.create_frame("Child", Some(root));
        reg.set_pos(child, 10.0, 20.0).unwrap();
        reg.set_anchor(child, AnchorTarget::Screen).unwrap();
        reg.set_pos_type(child, PositionType::Absolute).unwrap();
        let frame = reg.get_mut(child).unwrap();
        frame.width = Dimension::Auto;
        frame.height = Dimension::Fill;
        frame.translation = Val2::percent(-50.0, -50.0);
        frame.margin = UiRect::all(Val::Px(3.0));
        reg.rect_dirty.clear();
        reg.render_dirty.clear();
        let rect = LayoutRect {
            x: 41.0,
            y: 52.0,
            width: 123.0,
            height: 234.0,
        };
        reg.set_computed_layout(child, rect.clone()).unwrap();
        let frame = reg.get(child).unwrap();
        assert_eq!(frame.layout_rect, Some(rect.clone()));
        assert_eq!(
            (frame.resolved_width(), frame.resolved_height()),
            (123.0, 234.0)
        );
        assert_eq!(
            (frame.width, frame.height),
            (Dimension::Auto, Dimension::Fill)
        );
        assert_eq!(frame.position.left, Val::Px(10.0));
        assert_eq!(frame.position.top, Val::Px(20.0));
        assert_eq!(frame.anchor, AnchorTarget::Screen);
        assert_eq!(frame.position_type, PositionType::Absolute);
        assert_eq!(frame.translation, Val2::percent(-50.0, -50.0));
        assert_eq!(frame.margin, UiRect::all(Val::Px(3.0)));
        assert_eq!(frame.parent_id, Some(root));
        assert!(reg.rect_dirty.is_empty());
        assert_eq!(reg.render_dirty, HashSet::from([child]));
        reg.render_dirty.clear();
        reg.set_computed_layout(child, rect).unwrap();
        assert!(reg.rect_dirty.is_empty());
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn computed_layout_does_not_clear_existing_authored_invalidation() {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let id = reg.create_frame("Panel", None);
        reg.set_pos(id, 8.0, 9.0).unwrap();
        let dirty = reg.rect_dirty.clone();
        reg.set_computed_layout(id, reg.screen_rect()).unwrap();
        assert_eq!(reg.rect_dirty, dirty);
    }

    fn visibility_alpha_tree() -> (FrameRegistry, [u64; 3]) {
        let mut reg = FrameRegistry::new(800.0, 600.0);
        let root = reg.create_frame("Root", None);
        let child = reg.create_frame("Child", Some(root));
        let grandchild = reg.create_frame("Grandchild", Some(child));
        for id in [root, child, grandchild] {
            reg.set_alpha(id, 0.5);
        }
        reg.render_dirty.clear();
        reg.rect_dirty.clear();
        (reg, [root, child, grandchild])
    }

    fn assert_visibility_alpha(
        reg: &FrameRegistry,
        ids: [u64; 3],
        visible: [bool; 3],
        effective: [f32; 3],
    ) {
        for (index, id) in ids.into_iter().enumerate() {
            let frame = reg.get(id).unwrap();
            assert_eq!(frame.visible, visible[index]);
            assert_eq!(frame.effective_alpha, effective[index]);
            assert_eq!(frame.effective_scale, 1.0);
        }
        assert!(reg.rect_dirty.is_empty());
    }

    #[test]
    fn visibility_alpha_repeated_values_leave_subtree_clean() {
        let (mut reg, ids) = visibility_alpha_tree();
        for id in ids {
            reg.set_hidden(id, false);
            reg.set_alpha(id, 0.5);
        }
        assert_visibility_alpha(&reg, ids, [true; 3], [0.5, 0.25, 0.125]);
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn visibility_alpha_hide_show_marks_only_changed_frames() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, child, _] = ids;
        reg.set_hidden(root, true);
        assert_visibility_alpha(&reg, ids, [false; 3], [0.0; 3]);
        assert_eq!(reg.render_dirty, HashSet::from(ids));
        reg.render_dirty.clear();
        reg.set_hidden(root, true);
        assert!(reg.render_dirty.is_empty());
        reg.set_hidden(child, true);
        assert_eq!(reg.render_dirty, HashSet::from([child]));
        reg.render_dirty.clear();
        reg.set_hidden(root, false);
        assert_visibility_alpha(&reg, ids, [true, false, false], [0.5, 0.0, 0.0]);
        assert_eq!(reg.render_dirty, HashSet::from([root]));
    }

    #[test]
    fn visibility_alpha_changes_keep_raw_values_and_products() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, child, _] = ids;
        reg.set_alpha(root, 2.0);
        assert_eq!(reg.get(root).unwrap().alpha, 2.0);
        assert_visibility_alpha(&reg, ids, [true; 3], [2.0, 1.0, 0.5]);
        assert_eq!(reg.render_dirty, HashSet::from(ids));
        reg.set_alpha(child, -0.5);
        assert_eq!(reg.get(child).unwrap().alpha, -0.5);
        assert_visibility_alpha(&reg, ids, [true; 3], [2.0, -1.0, -0.5]);
        reg.render_dirty.clear();
        reg.set_alpha(root, 2.0);
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn visibility_alpha_hidden_child_keeps_effective_alpha_zero() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, child, _] = ids;
        reg.set_hidden(child, true);
        reg.render_dirty.clear();
        reg.set_alpha(root, 0.25);
        assert_visibility_alpha(&reg, ids, [true, false, false], [0.25, 0.0, 0.0]);
        assert_eq!(reg.render_dirty, HashSet::from([root]));
        reg.render_dirty.clear();
        reg.set_alpha(child, 0.75);
        assert_eq!(reg.get(child).unwrap().alpha, 0.75);
        assert_eq!(reg.render_dirty, HashSet::from([child]));
        reg.render_dirty.clear();
        reg.set_alpha(child, 0.75);
        assert!(reg.render_dirty.is_empty());
    }

    #[test]
    fn visibility_alpha_same_hidden_repairs_stale_grandchild() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, _, grandchild] = ids;
        let stale = reg.get_mut(grandchild).unwrap();
        stale.visible = false;
        stale.effective_alpha = 0.0;
        reg.render_dirty.clear();
        reg.set_hidden(root, false);
        assert_visibility_alpha(&reg, ids, [true; 3], [0.5, 0.25, 0.125]);
        assert_eq!(reg.render_dirty, HashSet::from([grandchild]));
    }

    #[test]
    fn visibility_single_pass_preserves_branched_hidden_and_stale_state() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, child, grandchild] = ids;
        let hidden = reg.create_frame("HiddenBranch", Some(root));
        let hidden_leaf = reg.create_frame("HiddenLeaf", Some(hidden));
        reg.set_alpha(hidden, 0.25);
        reg.set_alpha(hidden_leaf, 0.75);
        reg.set_hidden(hidden, true);
        for id in [grandchild, hidden_leaf] {
            let frame = reg.get_mut(id).unwrap();
            frame.visible = id == hidden_leaf;
            frame.effective_alpha = 9.0;
        }
        reg.render_dirty.clear();
        reg.rect_dirty.clear();

        reg.set_hidden(root, false);
        assert_visibility_alpha(&reg, ids, [true; 3], [0.5, 0.25, 0.125]);
        for id in [hidden, hidden_leaf] {
            let frame = reg.get(id).unwrap();
            assert!(!frame.visible);
            assert_eq!(frame.effective_alpha, 0.0);
        }
        assert_eq!(reg.get(hidden).unwrap().alpha, 0.25);
        assert_eq!(reg.get(hidden_leaf).unwrap().alpha, 0.75);
        assert_eq!(reg.render_dirty, HashSet::from([grandchild, hidden_leaf]));

        reg.render_dirty.clear();
        reg.set_hidden(root, true);
        assert_visibility_alpha(&reg, ids, [false; 3], [0.0; 3]);
        assert_eq!(reg.render_dirty, HashSet::from([root, child, grandchild]));
        reg.render_dirty.clear();
        reg.set_hidden(root, false);
        assert_visibility_alpha(&reg, ids, [true; 3], [0.5, 0.25, 0.125]);
        assert_eq!(reg.render_dirty, HashSet::from([root, child, grandchild]));
        reg.render_dirty.clear();
        reg.set_hidden(root, false);
        assert!(reg.render_dirty.is_empty());
        assert!(!reg.get(hidden_leaf).unwrap().visible);
        assert_eq!(reg.get(hidden_leaf).unwrap().effective_alpha, 0.0);
    }

    #[test]
    fn visibility_alpha_same_alpha_repairs_stale_descendants() {
        let (mut reg, ids) = visibility_alpha_tree();
        let [root, child, grandchild] = ids;
        reg.get_mut(child).unwrap().effective_alpha = 0.75;
        reg.get_mut(grandchild).unwrap().effective_alpha = 0.375;
        reg.render_dirty.clear();
        reg.set_alpha(root, 0.5);
        assert_visibility_alpha(&reg, ids, [true; 3], [0.5, 0.25, 0.125]);
        assert_eq!(reg.render_dirty, HashSet::from([child, grandchild]));
    }

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
        reg.set_hidden(parent, true);

        let parent_frame = reg.get(parent).unwrap();
        assert!(parent_frame.hidden);
        assert!(!parent_frame.visible);

        let child_frame = reg.get(child).unwrap();
        // Child's hidden stays false, but visible becomes false
        assert!(!child_frame.hidden);
        assert!(!child_frame.visible);

        // Show parent again
        reg.set_hidden(parent, false);
        let child_frame = reg.get(child).unwrap();
        assert!(child_frame.visible);
    }

    #[test]
    fn hidden_frame_effective_alpha_zero() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let parent = reg.create_frame("Parent", None);
        let child = reg.create_frame("Child", Some(parent));

        reg.set_alpha(child, 0.8);
        reg.set_hidden(parent, true);

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
    fn strata_inheritance() {
        let mut reg = FrameRegistry::new(1024.0, 768.0);
        let root = reg.create_frame("Root", None);
        reg.get_mut(root).unwrap().strata = FrameStrata::Dialog;

        let child = reg.create_frame("Child", Some(root));

        assert_eq!(reg.get(child).unwrap().strata, FrameStrata::Dialog);
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
}
