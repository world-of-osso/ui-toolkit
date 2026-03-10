use std::collections::{HashMap, HashSet};

use std::sync::atomic::{AtomicU8, Ordering};

use dioxus_core::internal::TemplateGlobalKey;
use dioxus_core::{AttributeValue, ElementId, Template, TemplateNode, WriteMutations};

/// Returns true when `DX_TRACE=1` is set. Cached after first check.
pub(crate) fn trace_enabled() -> bool {
    static STATE: AtomicU8 = AtomicU8::new(0);
    match STATE.load(Ordering::Relaxed) {
        2 => true,
        1 => false,
        _ => {
            let on = std::env::var("DX_TRACE").is_ok_and(|v| v == "1");
            STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            on
        }
    }
}

use crate::dioxus_anchor::{
    AnchorState, apply_anchor_element, collect_anchor_statics,
};
use crate::dioxus_attrs::{apply_attribute, apply_static_attribute};
use crate::dioxus_elements::tag_to_widget_type;
use crate::frame::{Frame, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::FontStringData;
use crate::widgets::texture::TextureData;

/// A node in the renderer's internal tree (mirrors Dioxus virtual DOM).
#[derive(Debug)]
pub(crate) enum NodeKind {
    Element {
        frame_id: u64,
    },
    Text {
        frame_id: u64,
    },
    /// Anchor pseudo-element with dynamic attrs, tracking parent frame.
    Anchor {
        parent_frame_id: u64,
    },
    Placeholder {
        parent_frame_id: Option<u64>,
    },
}

pub struct GameUiRenderer {
    pub(crate) nodes: Vec<Option<NodeKind>>,
    stack: Vec<ElementId>,
    /// All frame IDs created by this renderer (including static template children).
    pub(crate) created_frames: Vec<u64>,
    /// Frame IDs for template child nodes, keyed by path bytes from the last `load_template`.
    /// Used by `assign_node_id` to map dynamic ElementIds to template children.
    template_child_frames: Vec<(Vec<u8>, u64)>,
    /// Anchor pseudo-elements with dynamic attrs, keyed by path bytes.
    /// Used by `assign_node_id` to create `NodeKind::Anchor` entries.
    pub(crate) template_anchor_nodes: Vec<(Vec<u8>, u64, AnchorState)>,
    /// Anchors whose relative frame couldn't be resolved by name at apply time.
    /// Resolved after all mutations are applied (cross-component name references).
    pub(crate) pending_anchors: Vec<(u64, String)>,
    pub(crate) validated_paths: HashSet<String>,
    pub(crate) missing_paths: HashSet<String>,
    /// Persistent map from template key to (roots, index→frame_id).
    /// Used to apply hotreload diffs directly without Dioxus teardown.
    pub(crate) templates_by_key: HashMap<TemplateGlobalKey, (&'static [TemplateNode], HashMap<usize, u64>)>,
    /// Frame IDs reused by `reuse_frame_for_tag` in this render cycle.
    /// `replace_node_with` skips destruction for these, clearing each entry on use.
    pub(crate) reused_frame_ids: HashSet<u64>,
    /// Cache of last-applied dynamic attr values per (frame_id, attr_name).
    /// Used to skip redundant hotreload applications.
    pub(crate) dynamic_attr_cache: HashMap<(u64, String), String>,
}

impl GameUiRenderer {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            stack: Vec::new(),
            created_frames: Vec::new(),
            template_child_frames: Vec::new(),
            template_anchor_nodes: Vec::new(),
            pending_anchors: Vec::new(),
            validated_paths: HashSet::new(),
            missing_paths: HashSet::new(),
            templates_by_key: HashMap::new(),
            reused_frame_ids: HashSet::new(),
            dynamic_attr_cache: HashMap::new(),
        }
    }

    pub fn frame_id(&self, id: ElementId) -> Option<u64> {
        self.nodes.get(id.0).and_then(|n| match n {
            Some(NodeKind::Element { frame_id }) | Some(NodeKind::Text { frame_id }) => {
                Some(*frame_id)
            }
            _ => None,
        })
    }

    pub fn apply_to_registry(&mut self, _registry: &mut FrameRegistry) {}

    pub fn all_frame_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.created_frames.iter().copied()
    }

    /// Remove a frame and all its children from the registry, and purge from created_frames/nodes.
    fn ensure_slot(&mut self, id: ElementId) {
        if id.0 >= self.nodes.len() {
            self.nodes.resize_with(id.0 + 1, || None);
        }
    }

    pub(crate) fn create_frame_for_tag(
        &mut self,
        tag: &str,
        id: ElementId,
        registry: &mut FrameRegistry,
    ) -> u64 {
        let widget_type = tag_to_widget_type(tag).unwrap_or(WidgetType::Frame);
        let frame_id = registry.next_id();
        let mut frame = Frame::new(frame_id, None, widget_type);
        frame.widget_data = default_widget_data(widget_type);
        registry.insert_frame(frame);
        self.ensure_slot(id);
        self.nodes[id.0] = Some(NodeKind::Element { frame_id });
        self.created_frames.push(frame_id);
        frame_id
    }

    fn template_root_tag(template: &Template, index: usize) -> &'static str {
        if let Some(TemplateNode::Element { tag, .. }) = template.roots.get(index) {
            tag
        } else {
            "Frame"
        }
    }

    fn apply_node_attributes(
        node: &TemplateNode,
        registry: &mut FrameRegistry,
        frame_id: u64,
        pending: &mut Vec<(u64, String)>,
        validated_paths: &mut HashSet<String>,
        missing_paths: &mut HashSet<String>,
    ) {
        let TemplateNode::Element { attrs, .. } = node else {
            return;
        };
        for attr in *attrs {
            if let dioxus_core::TemplateAttribute::Static {
                name,
                value,
                namespace,
            } = attr
            {
                apply_static_attribute(
                    registry,
                    frame_id,
                    name,
                    *namespace,
                    value,
                    pending,
                    validated_paths,
                    missing_paths,
                );
            }
        }
    }

    fn handle_anchor_child(
        &mut self,
        child: &TemplateNode,
        parent_frame_id: u64,
        registry: &mut FrameRegistry,
        path: &[u8],
    ) {
        let TemplateNode::Element { attrs, .. } = child else {
            return;
        };
        let dynamic_count = attrs
            .iter()
            .filter(|a| matches!(a, dioxus_core::TemplateAttribute::Dynamic { .. }))
            .count();
        if dynamic_count == 0 {
            if let Some(pending) = apply_anchor_element(child, parent_frame_id, registry) {
                self.pending_anchors.push(pending);
            }
        } else {
            let state = collect_anchor_statics(child, dynamic_count);
            self.template_anchor_nodes
                .push((path.to_vec(), parent_frame_id, state));
        }
    }

    fn handle_element_child(
        &mut self,
        child: &TemplateNode,
        tag: &str,
        parent_frame_id: u64,
        registry: &mut FrameRegistry,
        path: &mut Vec<u8>,
    ) {
        let child_fid = instantiate_element(tag, parent_frame_id, registry);
        self.created_frames.push(child_fid);
        self.template_child_frames.push((path.clone(), child_fid));
        Self::apply_node_attributes(
            child,
            registry,
            child_fid,
            &mut self.pending_anchors,
            &mut self.validated_paths,
            &mut self.missing_paths,
        );
        self.instantiate_template_children(child, child_fid, registry, path);
    }

    fn template_root_name(template: &Template, index: usize) -> Option<&'static str> {
        let TemplateNode::Element { attrs, .. } = template.roots.get(index)? else {
            return None;
        };
        attrs.iter().find_map(|attr| match attr {
            dioxus_core::TemplateAttribute::Static {
                name: "name",
                value,
                ..
            } => Some(*value),
            _ => None,
        })
    }

    /// Look up an existing frame ID to reuse for this template root index.
    fn find_reusable_frame_by_name(
        &self,
        template: &Template,
        index: usize,
        registry: &FrameRegistry,
    ) -> Option<u64> {
        let name = Self::template_root_name(template, index)?;
        let fid = registry.get_by_name(name)?;
        let expected = tag_to_widget_type(Self::template_root_tag(template, index))
            .unwrap_or(WidgetType::Frame);
        let frame = registry.get(fid)?;
        if frame.widget_type != expected {
            return None;
        }
        if trace_enabled() {
            eprintln!("[dx] reusing frame {fid} for index={index}");
        }
        Some(fid)
    }

    /// Reuse an existing frame by ID: preserve parent/position, remove old children,
    /// and mark as reused so `replace_node_with` skips destruction.
    fn reuse_frame_for_tag(
        &mut self,
        tag: &str,
        id: ElementId,
        reuse_fid: u64,
        registry: &mut FrameRegistry,
    ) -> u64 {
        let old_parent_id = registry.get(reuse_fid).and_then(|f| f.parent_id);
        // Remove old children (template will recreate them)
        for cfid in registry.children_of(reuse_fid) {
            self.remove_frame_tree(cfid, registry);
        }
        self.reused_frame_ids.insert(reuse_fid);
        let widget_type = tag_to_widget_type(tag).unwrap_or(WidgetType::Frame);
        let mut frame = Frame::new(reuse_fid, None, widget_type);
        frame.widget_data = default_widget_data(widget_type);
        frame.parent_id = old_parent_id;
        registry.insert_frame(frame);
        self.ensure_slot(id);
        self.nodes[id.0] = Some(NodeKind::Element {
            frame_id: reuse_fid,
        });
        self.created_frames.push(reuse_fid);
        reuse_fid
    }

    /// Create root frame and instantiate static children for a template.
    fn create_or_reuse_root(
        &mut self,
        template: &Template,
        index: usize,
        id: ElementId,
        reuse_fid: Option<u64>,
        registry: &mut FrameRegistry,
    ) -> u64 {
        let tag = Self::template_root_tag(template, index);
        if let Some(old_fid) = reuse_fid {
            self.reuse_frame_for_tag(tag, id, old_fid, registry)
        } else {
            self.create_frame_for_tag(tag, id, registry)
        }
    }

    /// Apply static attributes and instantiate children for a template root.
    fn apply_template_children(
        &mut self,
        template: &Template,
        index: usize,
        frame_id: u64,
        registry: &mut FrameRegistry,
    ) {
        self.template_child_frames.clear();
        self.template_anchor_nodes.clear();
        if let Some(root_node) = template.roots.get(index) {
            Self::apply_node_attributes(
                root_node,
                registry,
                frame_id,
                &mut self.pending_anchors,
                &mut self.validated_paths,
                &mut self.missing_paths,
            );
            let mut path = Vec::new();
            self.instantiate_template_children(root_node, frame_id, registry, &mut path);
        }
    }

    fn instantiate_template_children(
        &mut self,
        node: &TemplateNode,
        parent_frame_id: u64,
        registry: &mut FrameRegistry,
        path: &mut Vec<u8>,
    ) {
        let TemplateNode::Element { children, .. } = node else {
            return;
        };
        for (i, child) in children.iter().enumerate() {
            path.push(i as u8);
            match child {
                TemplateNode::Element { tag, .. } if *tag == "Anchor" => {
                    self.handle_anchor_child(child, parent_frame_id, registry, path);
                }
                TemplateNode::Element { tag, .. } => {
                    self.handle_element_child(child, tag, parent_frame_id, registry, path);
                }
                TemplateNode::Text { text } => {
                    let child_fid = instantiate_text(text, parent_frame_id, registry);
                    self.created_frames.push(child_fid);
                    self.template_child_frames.push((path.clone(), child_fid));
                }
                TemplateNode::Dynamic { .. } => {}
            }
            path.pop();
        }
    }
}

fn instantiate_element(tag: &str, parent_fid: u64, registry: &mut FrameRegistry) -> u64 {
    let widget_type = tag_to_widget_type(tag).unwrap_or(WidgetType::Frame);
    let child_fid = registry.next_id();
    let mut frame = Frame::new(child_fid, None, widget_type);
    frame.widget_data = default_widget_data(widget_type);
    registry.insert_frame(frame);
    wire_parent_child(registry, parent_fid, child_fid);
    child_fid
}

fn instantiate_text(text: &str, parent_fid: u64, registry: &mut FrameRegistry) -> u64 {
    let child_fid = registry.next_id();
    let mut frame = Frame::new(child_fid, None, WidgetType::FontString);
    frame.name = Some(text.to_string());
    registry.insert_frame(frame);
    wire_parent_child(registry, parent_fid, child_fid);
    child_fid
}

fn find_template_frame(frames: &[(Vec<u8>, u64)], path: &[u8]) -> Option<u64> {
    frames
        .iter()
        .find(|(p, _)| p.as_slice() == path)
        .map(|(_, fid)| *fid)
}

fn default_widget_data(widget_type: WidgetType) -> Option<WidgetData> {
    match widget_type {
        WidgetType::Button => Some(WidgetData::Button(ButtonData::default())),
        WidgetType::EditBox => Some(WidgetData::EditBox(EditBoxData::default())),
        WidgetType::FontString => Some(WidgetData::FontString(FontStringData::default())),
        WidgetType::Texture => Some(WidgetData::Texture(TextureData::default())),
        _ => None,
    }
}

pub struct MutationApplier<'a> {
    pub renderer: &'a mut GameUiRenderer,
    pub registry: &'a mut FrameRegistry,
}

impl<'a> MutationApplier<'a> {
    pub fn new(renderer: &'a mut GameUiRenderer, registry: &'a mut FrameRegistry) -> Self {
        Self { renderer, registry }
    }

    fn reparent_nodes(&mut self, parent_fid: Option<u64>, children: &[ElementId]) {
        let Some(pfid) = parent_fid else { return };
        for &child_eid in children {
            if let Some(cfid) = self.renderer.frame_id(child_eid) {
                self.insert_frame_child(pfid, cfid, None);
            }
        }
    }

    fn insert_frame_child(&mut self, parent_fid: u64, child_fid: u64, index: Option<usize>) {
        let old_parent_id = self
            .registry
            .get(child_fid)
            .and_then(|frame| frame.parent_id);
        if let Some(old_parent_id) = old_parent_id
            && let Some(old_parent) = self.registry.get_mut(old_parent_id)
        {
            old_parent
                .children
                .retain(|&existing| existing != child_fid);
        }
        if let Some(child) = self.registry.get_mut(child_fid) {
            child.parent_id = Some(parent_fid);
        }
        if let Some(parent) = self.registry.get_mut(parent_fid) {
            parent.children.retain(|&existing| existing != child_fid);
            let insert_at = index
                .unwrap_or(parent.children.len())
                .min(parent.children.len());
            parent.children.insert(insert_at, child_fid);
        }
    }

    fn drain_stack_nodes(&mut self, m: usize) -> Vec<ElementId> {
        let start = self.renderer.stack.len().saturating_sub(m);
        self.renderer.stack.drain(start..).collect()
    }

    fn parent_frame_id_for_node(&self, id: ElementId) -> Option<u64> {
        match self.renderer.nodes.get(id.0) {
            Some(Some(NodeKind::Element { frame_id }))
            | Some(Some(NodeKind::Text { frame_id })) => self
                .registry
                .get(*frame_id)
                .and_then(|frame| frame.parent_id),
            Some(Some(NodeKind::Anchor { parent_frame_id })) => Some(*parent_frame_id),
            Some(Some(NodeKind::Placeholder { parent_frame_id })) => *parent_frame_id,
            _ => None,
        }
    }

    fn container_frame_id_for_node(&self, id: ElementId) -> Option<u64> {
        match self.renderer.nodes.get(id.0) {
            Some(Some(NodeKind::Element { frame_id }))
            | Some(Some(NodeKind::Text { frame_id })) => Some(*frame_id),
            Some(Some(NodeKind::Anchor { parent_frame_id })) => Some(*parent_frame_id),
            Some(Some(NodeKind::Placeholder { parent_frame_id })) => *parent_frame_id,
            _ => None,
        }
    }
}

impl WriteMutations for MutationApplier<'_> {
    fn append_children(&mut self, id: ElementId, m: usize) {
        if trace_enabled() {
            eprintln!(
                "[dx] append_children id={id:?} m={m} stack={:?}",
                &self.renderer.stack[self.renderer.stack.len().saturating_sub(m)..]
            );
        }
        let parent_fid = self.renderer.frame_id(id);
        let stack_len = self.renderer.stack.len();
        let start = stack_len.saturating_sub(m);
        let children: Vec<ElementId> = self.renderer.stack.drain(start..).collect();
        if let Some(pfid) = parent_fid {
            for child_eid in children {
                if let Some(cfid) = self.renderer.frame_id(child_eid) {
                    wire_parent_child(self.registry, pfid, cfid);
                }
            }
        }
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        if trace_enabled() {
            eprintln!("[dx] assign_node_id path={path:?} id={id:?}");
        }
        self.renderer.ensure_slot(id);
        if let Some(frame_id) = find_template_frame(&self.renderer.template_child_frames, path) {
            self.renderer.nodes[id.0] = Some(NodeKind::Element { frame_id });
            return;
        }
        if let Some(idx) = self
            .renderer
            .template_anchor_nodes
            .iter()
            .position(|(p, _, _)| p.as_slice() == path)
        {
            let (_, parent_frame_id, _) = &self.renderer.template_anchor_nodes[idx];
            self.renderer.nodes[id.0] = Some(NodeKind::Anchor {
                parent_frame_id: *parent_frame_id,
            });
        }
    }

    fn create_placeholder(&mut self, id: ElementId) {
        let parent_frame_id = self
            .renderer
            .stack
            .last()
            .and_then(|parent_id| self.container_frame_id_for_node(*parent_id));
        self.renderer.ensure_slot(id);
        self.renderer.nodes[id.0] = Some(NodeKind::Placeholder { parent_frame_id });
        self.renderer.stack.push(id);
    }

    fn create_text_node(&mut self, _value: &str, id: ElementId) {
        let frame_id = self.registry.next_id();
        let frame = Frame::new(frame_id, None, WidgetType::FontString);
        self.registry.insert_frame(frame);
        self.renderer.ensure_slot(id);
        self.renderer.nodes[id.0] = Some(NodeKind::Text { frame_id });
        self.renderer.created_frames.push(frame_id);
        self.renderer.stack.push(id);
    }

    fn load_template(&mut self, template: Template, index: usize, id: ElementId, template_key: Option<&dioxus_core::internal::TemplateGlobalKey>) {
        if trace_enabled() {
            eprintln!(
                "[dx] load_template index={index} id={id:?} roots={}",
                template.roots.len()
            );
        }
        let reuse_fid = self
            .renderer
            .find_reusable_frame_by_name(&template, index, self.registry);
        let frame_id =
            self.renderer
                .create_or_reuse_root(&template, index, id, reuse_fid, self.registry);
        self.renderer
            .apply_template_children(&template, index, frame_id, self.registry);
        if let Some(key) = template_key {
            self.renderer
                .templates_by_key
                .entry(key.clone())
                .and_modify(|(roots, fids)| {
                    *roots = template.roots;
                    fids.insert(index, frame_id);
                })
                .or_insert_with(|| {
                    let mut fids = HashMap::new();
                    fids.insert(index, frame_id);
                    (template.roots, fids)
                });
        }
        self.renderer.stack.push(id);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        if trace_enabled() {
            let fid = self.renderer.frame_id(id);
            let name = fid
                .and_then(|f| self.registry.get(f))
                .and_then(|f| f.name.clone());
            eprintln!("[dx] replace_node_with id={id:?} m={m} fid={fid:?} name={name:?}");
        }
        let parent_fid = self.parent_frame_id_for_node(id);
        let children = self.drain_stack_nodes(m);
        if let Some(fid) = self.renderer.frame_id(id) {
            if self.renderer.reused_frame_ids.remove(&fid) {
                // Reused frame — children already cleaned up, skip destruction
            } else {
                self.renderer.remove_frame_tree(fid, self.registry);
            }
        }
        self.renderer.nodes[id.0] = None;
        self.reparent_nodes(parent_fid, &children);
    }

    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        let parent_fid = if path.len() <= 1 {
            let root_index = self.renderer.stack.len().saturating_sub(m + 1);
            self.renderer
                .stack
                .get(root_index)
                .and_then(|id| self.renderer.frame_id(*id))
        } else {
            find_template_frame(
                &self.renderer.template_child_frames,
                &path[..path.len() - 1],
            )
        };
        let children = self.drain_stack_nodes(m);
        self.reparent_nodes(parent_fid, &children);
    }

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        let Some(parent_fid) = self.parent_frame_id_for_node(id) else {
            let _ = self.drain_stack_nodes(m);
            return;
        };
        let Some(target_fid) = self.renderer.frame_id(id) else {
            let _ = self.drain_stack_nodes(m);
            return;
        };
        let mut insert_at = self
            .registry
            .get(parent_fid)
            .and_then(|parent| {
                parent
                    .children
                    .iter()
                    .position(|&child| child == target_fid)
            })
            .map(|idx| idx + 1)
            .unwrap_or_else(|| {
                self.registry
                    .get(parent_fid)
                    .map_or(0, |parent| parent.children.len())
            });
        let children = self.drain_stack_nodes(m);
        for child in children {
            if let Some(child_fid) = self.renderer.frame_id(child) {
                self.insert_frame_child(parent_fid, child_fid, Some(insert_at));
                insert_at += 1;
            }
        }
    }

    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
        let Some(parent_fid) = self.parent_frame_id_for_node(id) else {
            let _ = self.drain_stack_nodes(m);
            return;
        };
        let Some(target_fid) = self.renderer.frame_id(id) else {
            let _ = self.drain_stack_nodes(m);
            return;
        };
        let mut insert_at = self
            .registry
            .get(parent_fid)
            .and_then(|parent| {
                parent
                    .children
                    .iter()
                    .position(|&child| child == target_fid)
            })
            .unwrap_or(0);
        let children = self.drain_stack_nodes(m);
        for child in children {
            if let Some(child_fid) = self.renderer.frame_id(child) {
                self.insert_frame_child(parent_fid, child_fid, Some(insert_at));
                insert_at += 1;
            }
        }
    }

    fn set_attribute(
        &mut self,
        name: &'static str,
        _ns: Option<&'static str>,
        value: &AttributeValue,
        id: ElementId,
    ) {
        // if trace_enabled() {
        //     let fid = self.renderer.frame_id(id);
        //     eprintln!("[dx] set_attribute name={name:?} id={id:?} fid={fid:?} value={value:?}");
        // }
        if let Some(pending) = self
            .renderer
            .try_set_anchor_attr(name, value, id, self.registry)
        {
            self.renderer.pending_anchors.push(pending);
            return;
        }
        let Some(fid) = self.renderer.frame_id(id) else {
            return;
        };
        let (vp, mp) = (
            &mut self.renderer.validated_paths,
            &mut self.renderer.missing_paths,
        );
        if let Some(pending) = apply_attribute(self.registry, fid, name, value, vp, mp) {
            self.renderer.pending_anchors.push(pending);
        }
        let cache_val = match value {
            AttributeValue::Text(s) => Some(s.clone()),
            AttributeValue::Bool(b) => Some(b.to_string()),
            AttributeValue::Int(i) => Some(i.to_string()),
            AttributeValue::Float(f) => Some(f.to_string()),
            _ => None,
        };
        if let Some(v) = cache_val {
            self.renderer.dynamic_attr_cache.insert((fid, name.to_string()), v);
        }
    }

    fn set_node_text(&mut self, _value: &str, id: ElementId) {
        let _ = self.renderer.frame_id(id);
    }

    fn create_event_listener(&mut self, _name: &'static str, _id: ElementId) {}
    fn remove_event_listener(&mut self, _name: &'static str, _id: ElementId) {}

    fn remove_node(&mut self, id: ElementId) {
        if trace_enabled() {
            let fid = self.renderer.frame_id(id);
            let name = fid
                .and_then(|f| self.registry.get(f))
                .and_then(|f| f.name.clone());
            eprintln!("[dx] remove_node id={id:?} fid={fid:?} name={name:?}");
        }
        if let Some(fid) = self.renderer.frame_id(id) {
            self.renderer.remove_frame_tree(fid, self.registry);
        }
        if let Some(slot) = self.renderer.nodes.get_mut(id.0) {
            *slot = None;
        }
    }

    fn push_root(&mut self, id: ElementId) {
        self.renderer.stack.push(id);
    }
}

pub(crate) fn wire_parent_child(registry: &mut FrameRegistry, parent_id: u64, child_id: u64) {
    if let Some(child) = registry.get_mut(child_id) {
        child.parent_id = Some(parent_id);
    }
    if let Some(parent) = registry.get_mut(parent_id)
        && !parent.children.contains(&child_id)
    {
        parent.children.push(child_id);
    }
}

#[cfg(test)]
#[path = "dioxus_renderer_tests.rs"]
mod tests;
