use crate::frame::{Frame, NineSlice, WidgetData, WidgetType};
use crate::registry::FrameRegistry;
use crate::widget_def::{Attr, NineSliceDef, WidgetChild, WidgetDef};
use crate::widgets::button::ButtonData;
use crate::widgets::edit_box::EditBoxData;
use crate::widgets::font_string::FontStringData;
use crate::widgets::slider::{SliderData, StatusBarData};
use crate::widgets::texture::TextureData;
use crate::widgets::texture::TextureSource;

pub struct DiffContext {
    /// All frame IDs created by this context.
    pub created_frames: Vec<u64>,
    /// Log attribute changes (enabled during hot-reload).
    pub log_changes: bool,
}

impl DiffContext {
    pub fn new() -> Self {
        Self {
            created_frames: Vec::new(),
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

        let mut matched: Vec<(u64, usize, Reuse)> = Vec::new();
        let mut unmatched_new: Vec<usize> = Vec::new();

        for (i, def) in new_defs.iter().enumerate() {
            if let Some((fid, reuse)) = consume_match(def, &mut remaining, registry) {
                matched.push((fid, i, reuse));
            } else {
                unmatched_new.push(i);
            }
        }

        // Remove unmatched existing frames
        for slot in remaining.into_iter().flatten() {
            self.remove_subtree(slot, registry);
        }

        // Update matched frames
        for (fid, i, reuse) in matched {
            let def = new_defs[i];
            if reuse == Reuse::OtherFrame {
                reset_visibility_state(fid, registry);
            }
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
        self.clear_reapplied_frame_state(def, frame_id, registry);
        self.apply_def_name(def, frame_id, registry);
        self.apply_def_attrs(def, frame_id, registry);
        self.apply_def_nine_slice(def, frame_id, registry);
    }

    fn clear_reapplied_frame_state(
        &self,
        def: &WidgetDef,
        frame_id: u64,
        registry: &mut FrameRegistry,
    ) {
        let Some(frame) = registry.get_mut(frame_id) else {
            return;
        };
        frame.position = crate::layout_values::UiRect::AUTO;
        frame.position_type = crate::layout_values::PositionType::Relative;
        frame.anchor = crate::anchor::AnchorTarget::Parent;
        frame.translation = crate::layout_values::Val2::ZERO;
        frame.margin = crate::layout_values::UiRect::ZERO;
        if !has_background_color_attr(def) {
            frame.background_color = None;
        }
    }

    fn apply_def_name(&self, def: &WidgetDef, frame_id: u64, registry: &mut FrameRegistry) {
        if let Some(name) = &def.name {
            registry.set_name(frame_id, name.clone());
        }
    }

    fn apply_def_attrs(&mut self, def: &WidgetDef, frame_id: u64, registry: &mut FrameRegistry) {
        for attr in &def.attrs {
            let attr_name = attr.effective_name();
            let value = attr.value_str();
            self.log_attr_change(frame_id, attr_name, value, registry);
            crate::attrs::apply_attribute(registry, frame_id, attr_name, value);
        }
    }

    fn apply_def_nine_slice(&self, def: &WidgetDef, frame_id: u64, registry: &mut FrameRegistry) {
        if let Some(ns_def) = &def.nine_slice
            && let Some(frame) = registry.get_mut(frame_id)
        {
            frame.nine_slice = Some(nine_slice_from_def(ns_def));
        }
    }

    fn log_attr_change(
        &self,
        frame_id: u64,
        attr_name: &str,
        value: &str,
        registry: &FrameRegistry,
    ) {
        if self.log_changes {
            log::info!(
                "hot-reload: set {}.{} = {}",
                frame_label(frame_id, registry),
                attr_name,
                value
            );
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
        let Some((name, frame_id)) = self.patch_target(def, registry) else {
            return;
        };
        self.patch_widget_attrs(name, frame_id, &def.attrs, registry);
        self.patch_widget_children(def, registry);
    }

    fn patch_target<'a>(
        &self,
        def: &'a WidgetDef,
        registry: &FrameRegistry,
    ) -> Option<(&'a str, u64)> {
        let name = def.name.as_deref()?;
        let frame_id = registry.get_by_name(name)?;
        Some((name, frame_id))
    }

    fn patch_widget_attrs(
        &mut self,
        name: &str,
        frame_id: u64,
        attrs: &[Attr],
        registry: &mut FrameRegistry,
    ) {
        for attr in attrs {
            self.patch_widget_attr(name, frame_id, attr, registry);
        }
    }

    fn patch_widget_attr(
        &mut self,
        name: &str,
        frame_id: u64,
        attr: &Attr,
        registry: &mut FrameRegistry,
    ) {
        let attr_name = attr.effective_name();
        let value = attr.value_str();
        let old = self.patch_old_value(registry, frame_id, attr_name);
        crate::attrs::apply_attribute(registry, frame_id, attr_name, value);
        self.log_patch_attr_change(name, attr_name, value, old.as_deref());
    }

    fn patch_old_value(
        &self,
        registry: &FrameRegistry,
        frame_id: u64,
        attr_name: &str,
    ) -> Option<String> {
        if !self.log_changes {
            return None;
        }
        crate::attrs::read_attribute(registry, frame_id, attr_name)
    }

    fn log_patch_attr_change(&self, name: &str, attr_name: &str, value: &str, old: Option<&str>) {
        let Some(old) = old else {
            return;
        };
        if values_equal(old, value) {
            return;
        }
        log::info!("hot-reload: {}.{}: {} → {}", name, attr_name, old, value);
    }

    fn patch_widget_children(&mut self, def: &WidgetDef, registry: &mut FrameRegistry) {
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
/// How a new def found its existing frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reuse {
    /// The frame this def built before, found by name.
    SameFrame,
    /// An unnamed leftover frame of the same widget type.
    OtherFrame,
}

/// Matching prefers name-based lookup, then an unnamed frame of the same widget type.
fn consume_match(
    def: &WidgetDef,
    remaining: &mut [Option<u64>],
    registry: &FrameRegistry,
) -> Option<(u64, Reuse)> {
    consume_name_match(def, remaining, registry)
        .map(|fid| (fid, Reuse::SameFrame))
        .or_else(|| {
            consume_type_match(def, remaining, registry).map(|fid| (fid, Reuse::OtherFrame))
        })
}

/// A frame taken over by a different def starts visible and opaque, as a new frame would;
/// its def re-applies `hidden`/`alpha` when it declares them. Frames matched by name keep
/// runtime visibility their owners set.
fn reset_visibility_state(frame_id: u64, registry: &mut FrameRegistry) {
    registry.set_hidden(frame_id, false);
    registry.set_alpha(frame_id, 1.0);
}

fn consume_name_match(
    def: &WidgetDef,
    remaining: &mut [Option<u64>],
    registry: &FrameRegistry,
) -> Option<u64> {
    let name = def.name.as_deref()?;
    let fid = registry.get_by_name(name)?;
    consume_frame_id(remaining, fid)
}

fn consume_type_match(
    def: &WidgetDef,
    remaining: &mut [Option<u64>],
    registry: &FrameRegistry,
) -> Option<u64> {
    let wanted_type =
        crate::attrs::tag_to_widget_type(def.effective_tag()).unwrap_or(WidgetType::Frame);
    let index = remaining.iter().position(|slot| {
        let Some(fid) = *slot else {
            return false;
        };
        // A named frame belongs to the def with that name; never hand it to another def.
        registry
            .get(fid)
            .is_some_and(|frame| frame.widget_type == wanted_type && frame.name.is_none())
    })?;
    remaining[index].take()
}

fn consume_frame_id(remaining: &mut [Option<u64>], fid: u64) -> Option<u64> {
    let slot = remaining.iter_mut().find(|slot| **slot == Some(fid))?;
    slot.take()
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

fn has_background_color_attr(def: &WidgetDef) -> bool {
    def.attrs
        .iter()
        .any(|attr| attr.effective_name() == "background_color")
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
    fn diff_button_disabled_false_stays_normal() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let children = vec![WidgetChild::Widget(WidgetDef {
            tag: "Button",
            tag_owned: None,
            name: Some("TestButton".to_string()),
            attrs: vec![Attr::new_static("disabled", "false".to_string())],
            nine_slice: None,
            children: vec![],
        })];

        ctx.diff_roots(&children, None, &mut reg);

        let fid = ctx.created_frames[0];
        let frame = reg.get(fid).unwrap();
        let Some(WidgetData::Button(bd)) = &frame.widget_data else {
            panic!("expected button widget data");
        };
        assert_eq!(bd.state, crate::widgets::button::ButtonState::Normal);
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
    fn diff_clears_background_color_when_attr_removed() {
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let with_bg = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("BgFrame".to_string()),
            attrs: vec![Attr::new_static(
                "background_color",
                "0.1,0.2,0.3,1.0".to_string(),
            )],
            nine_slice: None,
            children: vec![],
        })];

        ctx.diff_roots(&with_bg, None, &mut reg);

        let without_bg = vec![WidgetChild::Widget(WidgetDef {
            tag: "Frame",
            tag_owned: None,
            name: Some("BgFrame".to_string()),
            attrs: vec![],
            nine_slice: None,
            children: vec![],
        })];

        ctx.diff_roots(&without_bg, None, &mut reg);

        let fid = reg.get_by_name("BgFrame").expect("frame should exist");
        let frame = reg.get(fid).expect("frame should exist");
        assert!(
            frame.background_color.is_none(),
            "background_color should be cleared when the attr is removed"
        );
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

    #[test]
    fn diff_reapplies_native_layout_attributes() {
        use crate::anchor::AnchorTarget;
        use crate::layout_values::{PositionType, Val};
        let mut reg = make_registry();
        let mut ctx = DiffContext::new();
        let mut widget = WidgetDef::new("Frame");
        widget.name = Some("Positioned".to_string());
        widget.attrs = vec![
            Attr::new_static("pos_x", "10".to_string()),
            Attr::new_static("anchor", "screen".to_string()),
        ];
        ctx.diff_roots(&[WidgetChild::Widget(widget)], None, &mut reg);
        let fid = reg.get_by_name("Positioned").unwrap();
        assert_eq!(reg.get(fid).unwrap().position.left, Val::Px(10.0));
        assert_eq!(reg.get(fid).unwrap().anchor, AnchorTarget::Screen);
        let mut widget = WidgetDef::new("Frame");
        widget.name = Some("Positioned".to_string());
        widget.attrs = vec![
            Attr::new_dynamic("pos_x", "20".to_string()),
            Attr::new_static("pos_type", "absolute".to_string()),
        ];
        ctx.diff_roots(&[WidgetChild::Widget(widget)], None, &mut reg);
        assert_eq!(reg.get_by_name("Positioned"), Some(fid));
        let frame = reg.get(fid).unwrap();
        assert_eq!(frame.position.left, Val::Px(20.0));
        assert_eq!(frame.position_type, PositionType::Absolute);
        assert_eq!(frame.anchor, AnchorTarget::Parent);
    }
}

#[cfg(test)]
mod named_reuse_tests {
    use super::*;
    use crate::widget_def::AttrValue;

    fn frame_def(name: Option<&str>, attrs: &[(&'static str, &str)]) -> WidgetDef {
        let mut def = WidgetDef::new("Frame");
        def.name = name.map(str::to_owned);
        def.attrs = attrs
            .iter()
            .map(|&(name, value)| Attr {
                name,
                name_owned: None,
                value: AttrValue::Static(value.to_owned()),
            })
            .collect();
        def
    }

    fn list(children: Vec<WidgetDef>) -> Vec<WidgetChild> {
        let mut root = frame_def(Some("List"), &[]);
        root.children = children.into_iter().map(WidgetChild::Widget).collect();
        vec![WidgetChild::Widget(root)]
    }

    fn frame<'a>(registry: &'a FrameRegistry, name: &str) -> &'a Frame {
        registry
            .get(registry.get_by_name(name).expect(name))
            .expect(name)
    }

    #[test]
    fn new_named_row_does_not_take_over_a_hidden_named_sibling() {
        let mut registry = FrameRegistry::new(1920.0, 1080.0);
        let mut diff = DiffContext::new();
        let track = || frame_def(Some("Track"), &[("hidden", "true")]);
        diff.diff_roots(&list(vec![track()]), None, &mut registry);
        let track_id = registry.get_by_name("Track").unwrap();

        diff.diff_roots(
            &list(vec![frame_def(Some("Row0"), &[]), track()]),
            None,
            &mut registry,
        );

        let row = frame(&registry, "Row0");
        assert!(
            row.visible && !row.hidden,
            "new row inherited the track's hidden state"
        );
        assert_ne!(registry.get_by_name("Row0"), Some(track_id));
        assert_eq!(registry.get_by_name("Track"), Some(track_id));
        assert!(frame(&registry, "Track").hidden);
    }

    #[test]
    fn unnamed_frame_reused_by_another_def_starts_visible_and_opaque() {
        let mut registry = FrameRegistry::new(1920.0, 1080.0);
        let mut diff = DiffContext::new();
        diff.diff_roots(
            &list(vec![frame_def(None, &[("hidden", "true")])]),
            None,
            &mut registry,
        );
        let list_id = registry.get_by_name("List").unwrap();
        let old = registry.children_of(list_id)[0];
        registry.set_alpha(old, 0.2);

        diff.diff_roots(
            &list(vec![frame_def(Some("Row0"), &[])]),
            None,
            &mut registry,
        );

        assert_eq!(
            registry.get_by_name("Row0"),
            Some(old),
            "unnamed frame reused"
        );
        let row = frame(&registry, "Row0");
        assert!(row.visible && !row.hidden);
        assert_eq!(row.alpha, 1.0);
    }

    #[test]
    fn frame_matched_by_name_keeps_runtime_visibility() {
        let mut registry = FrameRegistry::new(1920.0, 1080.0);
        let mut diff = DiffContext::new();
        let panel = || list(vec![frame_def(Some("Panel"), &[])]);
        diff.diff_roots(&panel(), None, &mut registry);
        let id = registry.get_by_name("Panel").unwrap();
        registry.set_hidden(id, true);

        diff.diff_roots(&panel(), None, &mut registry);

        assert!(frame(&registry, "Panel").hidden);
    }
}
