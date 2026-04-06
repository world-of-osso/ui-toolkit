use std::collections::HashSet;

use crate::frame::{Frame, NineSlice, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::widget_def::{NineSliceDef, WidgetChild, WidgetDef};
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::FontStringData;
use crate::widgets::slider::{SliderData, StatusBarData};
use crate::widgets::texture::TextureData;
use crate::widgets::texture::TextureSource;

pub struct DiffContext {
    /// All frame IDs created by this context.
    pub created_frames: Vec<u64>,
    /// Anchors that couldn't be resolved yet (relative frame not registered).
    pub pending_anchors: Vec<(u64, String)>,
    /// Validated texture/file paths (avoid re-checking).
    pub validated_paths: HashSet<String>,
    /// Missing paths already warned about.
    pub missing_paths: HashSet<String>,
    /// Log attribute changes (enabled during hot-reload).
    pub log_changes: bool,
}

impl DiffContext {
    pub fn new() -> Self {
        Self {
            created_frames: Vec::new(),
            pending_anchors: Vec::new(),
            validated_paths: HashSet::new(),
            missing_paths: HashSet::new(),
            log_changes: false,
        }
    }

    /// Diff a list of WidgetChild against existing children under parent_id.
    /// If parent_id is None, diffs against root-level frames (tracked in created_frames).
    pub fn diff_roots(
        &mut self,
        new_children: &[WidgetChild],
        parent_id: Option<u64>,
        registry: &mut FrameRegistry,
    ) {
        let new_defs = flatten(new_children);

        let existing_fids: Vec<u64> = match parent_id {
            Some(pid) => registry.children_of(pid),
            None => self.created_frames.clone(),
        };

        let mut remaining: Vec<Option<u64>> = existing_fids.into_iter().map(Some).collect();

        let mut matched: Vec<(u64, usize)> = Vec::new();
        let mut unmatched_new: Vec<usize> = Vec::new();

        for (i, def) in new_defs.iter().enumerate() {
            if let Some(fid) = consume_match(def, &mut remaining, registry) {
                matched.push((fid, i));
            } else {
                unmatched_new.push(i);
            }
        }

        // Remove unmatched existing frames
        for slot in remaining.into_iter().flatten() {
            self.remove_subtree(slot, registry);
        }

        // Update matched frames
        for (fid, i) in matched {
            let def = new_defs[i];
            self.apply_def(def, fid, registry);
            self.diff_roots(&def.children, Some(fid), registry);
        }

        // Create new frames for unmatched defs
        for i in unmatched_new {
            let def = new_defs[i];
            let fid = self.create_def(def, parent_id, registry);
            self.diff_roots(&def.children, Some(fid), registry);
        }
    }

    fn apply_def(&mut self, def: &WidgetDef, frame_id: u64, registry: &mut FrameRegistry) {
        // Clear existing anchors (will be re-applied)
        if let Some(frame) = registry.get_mut(frame_id) {
            frame.anchors.clear();
        }
        // Apply name
        if let Some(name) = &def.name {
            registry.set_name(frame_id, name.clone());
        }
        // Apply attrs
        for attr in &def.attrs {
            let attr_name = attr.effective_name();
            let value = attr.value_str();
            if self.log_changes {
                log::info!(
                    "hot-reload: set {}.{} = {}",
                    frame_label(frame_id, registry),
                    attr_name,
                    value
                );
            }
            crate::attrs::apply_attribute(
                registry,
                frame_id,
                attr_name,
                value,
                &mut self.validated_paths,
                &mut self.missing_paths,
            );
        }
        // Apply nine-slice backdrop
        if let Some(ns_def) = &def.nine_slice {
            if let Some(frame) = registry.get_mut(frame_id) {
                frame.nine_slice = Some(nine_slice_from_def(ns_def));
            }
        }
        // Apply anchors
        for anchor in &def.anchors {
            if let Some(pending) =
                crate::anchor_resolve::apply_anchor_from_def(anchor, frame_id, registry)
            {
                self.pending_anchors.push(pending);
            }
        }
    }

    fn create_def(
        &mut self,
        def: &WidgetDef,
        parent_id: Option<u64>,
        registry: &mut FrameRegistry,
    ) -> u64 {
        let tag = def.effective_tag();
        let widget_type = crate::attrs::tag_to_widget_type(tag).unwrap_or(WidgetType::Frame);
        let frame_id = registry.next_id();
        let mut frame = Frame::new(frame_id, None, widget_type);
        frame.widget_data = default_widget_data(widget_type);
        frame.parent_id = parent_id;
        if matches!(
            widget_type,
            WidgetType::Button | WidgetType::CheckButton | WidgetType::EditBox
        ) {
            frame.mouse_enabled = true;
        }
        registry.insert_frame(frame);
        if widget_type == WidgetType::Panel {
            registry.apply_default_panel_style(frame_id);
        }
        if parent_id.is_none() {
            self.created_frames.push(frame_id);
        }
        if self.log_changes {
            let name = def.name.as_deref().unwrap_or(tag);
            log::info!("hot-reload: add <{}> \"{}\" (id={})", tag, name, frame_id);
        }
        self.apply_def(def, frame_id, registry);
        frame_id
    }

    fn remove_subtree(&mut self, frame_id: u64, registry: &mut FrameRegistry) {
        if self.log_changes {
            log::info!(
                "hot-reload: remove {} (id={})",
                frame_label(frame_id, registry),
                frame_id
            );
        }
        let children = registry.children_of(frame_id);
        for child in children {
            self.remove_subtree(child, registry);
        }
        registry.remove_frame(frame_id);
        self.created_frames.retain(|&fid| fid != frame_id);
    }

    /// Patch existing frames by name — find each named widget in the registry
    /// and update its attributes in-place. No frames are created or removed.
    pub fn patch_by_name(&mut self, defs: &[WidgetChild], registry: &mut FrameRegistry) {
        for def in flatten(defs) {
            self.patch_widget(def, registry);
        }
    }

    fn patch_widget(&mut self, def: &WidgetDef, registry: &mut FrameRegistry) {
        let Some(name) = &def.name else { return };
        let Some(frame_id) = registry.get_by_name(name) else {
            return;
        };
        for attr in &def.attrs {
            let attr_name = attr.effective_name();
            let value = attr.value_str();
            let old = if self.log_changes {
                crate::attrs::read_attribute(registry, frame_id, attr_name)
            } else {
                None
            };
            crate::attrs::apply_attribute(
                registry,
                frame_id,
                attr_name,
                value,
                &mut self.validated_paths,
                &mut self.missing_paths,
            );
            if self.log_changes {
                if let Some(old) = &old {
                    if !values_equal(old, value) {
                        log::info!("hot-reload: {}.{}: {} → {}", name, attr_name, old, value);
                    }
                }
            }
        }
        for child_def in flatten(&def.children) {
            self.patch_widget(child_def, registry);
        }
    }
}

impl Default for DiffContext {
    fn default() -> Self {
        Self::new()
    }
}

fn frame_label(frame_id: u64, registry: &FrameRegistry) -> String {
    registry
        .get(frame_id)
        .and_then(|f| f.name.as_deref())
        .map(|n| format!("\"{}\"", n))
        .unwrap_or_else(|| format!("(id={})", frame_id))
}

fn flatten<'a>(children: &'a [WidgetChild]) -> Vec<&'a WidgetDef> {
    let mut out = Vec::new();
    for child in children {
        match child {
            WidgetChild::Widget(def) => out.push(def),
            WidgetChild::Fragment(kids) => out.extend(flatten(kids)),
            WidgetChild::Dynamic => {}
        }
    }
    out
}

/// Try to find and consume a matching existing frame for the given def.
/// Matching prefers name-based lookup, then tag/widget_type.
fn consume_match(
    def: &WidgetDef,
    remaining: &mut Vec<Option<u64>>,
    registry: &FrameRegistry,
) -> Option<u64> {
    if let Some(name) = &def.name {
        if let Some(fid) = registry.get_by_name(name) {
            if let Some(slot) = remaining.iter_mut().find(|s| **s == Some(fid)) {
                *slot = None;
                return Some(fid);
            }
        }
    }

    let wanted_type =
        crate::attrs::tag_to_widget_type(def.effective_tag()).unwrap_or(WidgetType::Frame);
    for slot in remaining.iter_mut() {
        if let Some(fid) = *slot {
            if let Some(frame) = registry.get(fid) {
                if frame.widget_type == wanted_type {
                    *slot = None;
                    return Some(fid);
                }
            }
        }
    }
    None
}

/// Compare attribute values, treating numeric equivalents as equal (e.g. "320" == "320.0").
fn values_equal(old: &str, new: &str) -> bool {
    if old == new {
        return true;
    }
    if let (Ok(a), Ok(b)) = (old.parse::<f32>(), new.parse::<f32>()) {
        return (a - b).abs() < f32::EPSILON;
    }
    false
}

fn nine_slice_from_def(def: &NineSliceDef) -> NineSlice {
    let part_textures = def
        .textures
        .as_ref()
        .map(|paths| std::array::from_fn(|i| TextureSource::File(paths[i].clone())));
    NineSlice {
        edge_size: def.edge_size,
        bg_color: def.bg_color,
        border_color: def.border_color,
        part_textures,
        ..NineSlice::default()
    }
}

fn default_widget_data(widget_type: WidgetType) -> Option<WidgetData> {
    match widget_type {
        WidgetType::Button => Some(WidgetData::Button(ButtonData::default())),
        WidgetType::EditBox => Some(WidgetData::EditBox(EditBoxData::default())),
        WidgetType::FontString => Some(WidgetData::FontString(FontStringData::default())),
        WidgetType::Slider => Some(WidgetData::Slider(SliderData::default())),
        WidgetType::StatusBar => Some(WidgetData::StatusBar(StatusBarData::default())),
        WidgetType::Texture => Some(WidgetData::Texture(TextureData::default())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Dimension;
    use crate::widget_def::*;
    use crate::widgets::texture::TextureSource;

    fn make_registry() -> FrameRegistry {
        FrameRegistry::new(1024.0, 768.0)
    }

    fn slider_def(name: &str, thumb_texture: &str) -> WidgetChild {
        WidgetChild::Widget(WidgetDef {
            tag: "Slider",
            tag_owned: None,
            name: Some(name.to_string()),
            attrs: vec![Attr::new_static("thumb_texture", thumb_texture.to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })
    }

    #[test]
    fn diff_empty_to_empty() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        ctx.diff_roots(&[], None, &mut reg);
        assert!(ctx.created_frames.is_empty());
    }

    #[test]
    fn diff_creates_single_frame() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("TestFrame".to_string()),
            attrs: vec![Attr::new_static("width", "100".to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })];
        ctx.diff_roots(&children, None, &mut reg);
        assert_eq!(ctx.created_frames.len(), 1);
        let fid = ctx.created_frames[0];
        let frame = reg.get(fid).unwrap();
        assert_eq!(frame.name.as_deref(), Some("TestFrame"));
        assert_eq!(frame.width, Dimension::Fixed(100.0));
    }

    #[test]
    fn diff_updates_existing_by_name() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        // First diff: create
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("MyFrame".to_string()),
            attrs: vec![Attr::new_static("width", "100".to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })];
        ctx.diff_roots(&children, None, &mut reg);
        let fid = ctx.created_frames[0];
        assert_eq!(reg.get(fid).unwrap().width, Dimension::Fixed(100.0));

        // Second diff: update width
        let children2 = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("MyFrame".to_string()),
            attrs: vec![Attr::new_static("width", "200".to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })];
        ctx.diff_roots(&children2, None, &mut reg);
        // Same frame ID, updated width
        assert_eq!(reg.get(fid).unwrap().width, Dimension::Fixed(200.0));
        assert_eq!(ctx.created_frames.len(), 1); // no new frames
    }

    #[test]
    fn patch_by_name_can_clear_slider_thumb_texture() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        ctx.diff_roots(
            &[slider_def("MySlider", "data/textures/ui/old_thumb.png")],
            None,
            &mut reg,
        );

        let fid = reg.get_by_name("MySlider").expect("slider frame");
        let before = reg.get(fid).expect("slider frame");
        let Some(WidgetData::Slider(slider)) = &before.widget_data else {
            panic!("expected slider widget data");
        };
        assert!(matches!(slider.thumb_texture, Some(TextureSource::File(_))));

        ctx.patch_by_name(&[slider_def("MySlider", "none")], &mut reg);

        let after = reg.get(fid).expect("slider frame");
        let Some(WidgetData::Slider(slider)) = &after.widget_data else {
            panic!("expected slider widget data");
        };
        assert!(slider.thumb_texture.is_none());
    }

    #[test]
    fn diff_applies_hidden_via_registry_visibility() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Button",
            tag_owned: None,
            name: Some("HiddenButton".to_string()),
            attrs: vec![Attr::new_static("hidden", "true".to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })];

        ctx.diff_roots(&children, None, &mut reg);

        let fid = ctx.created_frames[0];
        let frame = reg.get(fid).unwrap();
        assert!(frame.hidden);
        assert!(!frame.visible);
        assert!((frame.effective_alpha - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn diff_applies_alpha_via_registry_effective_alpha() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("FadedFrame".to_string()),
            attrs: vec![Attr::new_static("alpha", "0.25".to_string())],
            anchors: vec![],
            nine_slice: None,
            children: vec![],
        })];

        ctx.diff_roots(&children, None, &mut reg);

        let fid = ctx.created_frames[0];
        let frame = reg.get(fid).unwrap();
        assert!((frame.alpha - 0.25).abs() < f32::EPSILON);
        assert!((frame.effective_alpha - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn diff_removes_unmatched() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![
            WidgetChild::Widget(WidgetDef::new("Frame")),
            WidgetChild::Widget(WidgetDef::new("Button")),
        ];
        ctx.diff_roots(&children, None, &mut reg);
        assert_eq!(ctx.created_frames.len(), 2);

        // Remove one
        let children2 = vec![WidgetChild::Widget(WidgetDef::new("Frame"))];
        ctx.diff_roots(&children2, None, &mut reg);
        assert_eq!(ctx.created_frames.len(), 1);
    }

    #[test]
    fn diff_applies_nine_slice_def() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("NsFrame".to_string()),
            attrs: vec![],
            anchors: vec![],
            nine_slice: Some(NineSliceDef {
                edge_size: 16.0,
                bg_color: [0.1, 0.2, 0.3, 0.9],
                border_color: [1.0, 0.0, 0.0, 1.0],
                textures: None,
            }),
            children: vec![],
        })];
        ctx.diff_roots(&children, None, &mut reg);
        let fid = ctx.created_frames[0];
        let frame = reg.get(fid).unwrap();
        let ns = frame.nine_slice.as_ref().expect("nine_slice should be set");
        assert!((ns.edge_size - 16.0).abs() < f32::EPSILON);
        assert!((ns.bg_color[0] - 0.1).abs() < 0.001);
        assert!((ns.border_color[0] - 1.0).abs() < f32::EPSILON);
        assert!(ns.part_textures.is_none());
    }
}
