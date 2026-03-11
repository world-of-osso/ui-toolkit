use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::mpsc;

use crate::anchor_resolve::apply_anchor_resolved;
use crate::frame::{Dimension, WidgetData};
use crate::hotreload::HotReloadTemplate;
use crate::registry::FrameRegistry;
use crate::text_measure::measure_text;
use crate::widget_def::WidgetChild;
use crate::widget_def_diff::DiffContext;

/// Typed context map for injecting state into build functions.
/// Replaces Dioxus's provide_root_context/use_context.
pub struct ScreenContext {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl ScreenContext {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn insert<T: 'static>(&mut self, val: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(val));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.values.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }
}

impl Default for ScreenContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen: manages a UI component's lifecycle against FrameRegistry.
/// Replaces DioxusScreen.
pub struct Screen {
    build_fn: Box<dyn Fn(&ScreenContext) -> Vec<WidgetChild>>,
    ctx: ScreenContext,
    diff: DiffContext,
    hot_reload_rx: Option<mpsc::Receiver<HotReloadTemplate>>,
    initialized: bool,
    dirty: bool,
}

impl Screen {
    pub fn new<F: Fn(&ScreenContext) -> Vec<WidgetChild> + 'static>(f: F) -> Self {
        Self {
            build_fn: Box::new(f),
            ctx: ScreenContext::new(),
            diff: DiffContext::new(),
            hot_reload_rx: None,
            initialized: false,
            dirty: true,
        }
    }

    /// Set the hot-reload channel receiver.
    pub fn set_hot_reload_rx(&mut self, rx: mpsc::Receiver<HotReloadTemplate>) {
        self.hot_reload_rx = Some(rx);
    }

    /// Access the context to insert values before sync.
    pub fn context_mut(&mut self) -> &mut ScreenContext {
        &mut self.ctx
    }

    /// Mark the screen dirty to force rebuild on next sync.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Sync the widget tree against the registry.
    /// 1. Drain hot-reload channel (apply static-only template updates)
    /// 2. If dirty or first init: call build_fn, diff against registry
    /// 3. Resolve pending anchors
    /// 4. Auto-size fontstrings and editboxes
    pub fn sync(&mut self, registry: &mut FrameRegistry) {
        // 1. Drain hot-reload
        if let Some(rx) = &self.hot_reload_rx {
            self.diff.log_changes = true;
            while let Ok(template) = rx.try_recv() {
                self.diff.patch_by_name(&template.defs, registry);
            }
            self.diff.log_changes = false;
        }

        // 2. Build and diff if needed
        if !self.initialized || self.dirty {
            let tree = (self.build_fn)(&self.ctx);
            self.diff.diff_roots(&tree, None, registry);
            self.initialized = true;
            self.dirty = false;
        }

        // 3. Resolve pending anchors
        self.resolve_pending_anchors(registry);

        // 4. Auto-size
        auto_size_fontstrings(&self.diff, registry);
        auto_size_editboxes(&self.diff, registry);
    }

    fn resolve_pending_anchors(&mut self, registry: &mut FrameRegistry) {
        let pending = std::mem::take(&mut self.diff.pending_anchors);
        for (frame_id, spec) in pending {
            let already_has =
                registry.get(frame_id).is_some_and(|f| !f.anchors.is_empty());
            if !already_has {
                apply_anchor_resolved(registry, frame_id, &spec);
            }
        }
    }

    /// Remove all frames created by this screen (roots + their subtrees).
    pub fn teardown(&mut self, registry: &mut FrameRegistry) {
        for &fid in self.diff.created_frames.iter().rev() {
            registry.remove_frame_tree(fid);
        }
        self.diff = DiffContext::new();
        self.initialized = false;
    }

    /// Get all frame IDs owned by this screen.
    pub fn all_frame_ids(&self) -> &[u64] {
        &self.diff.created_frames
    }
}

fn auto_size_fontstrings(diff: &DiffContext, registry: &mut FrameRegistry) {
    for &fid in &diff.created_frames {
        let Some(frame) = registry.get(fid) else { continue };
        let Some(WidgetData::FontString(fs)) = &frame.widget_data else { continue };
        if frame.width.value() > 0.0 || fs.text.is_empty() { continue }
        let text = fs.text.clone();
        let font = fs.font;
        let font_size = fs.font_size;
        if let Some((w, h)) = measure_text(&text, font, font_size) {
            let frame = registry.get_mut(fid).unwrap();
            frame.width = Dimension::Fixed(w);
            frame.height = Dimension::Fixed(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, WidgetType};
    use crate::hotreload::parser::parse_rsx_blocks;
    use crate::widget_def::WidgetChild;

    fn screen_with_frame(reg: &mut FrameRegistry, name: &str, width: f32) -> u64 {
        let id = reg.next_id();
        let mut frame = Frame::new(id, Some(name.to_string()), WidgetType::Frame);
        frame.width = Dimension::Fixed(width);
        reg.insert_frame(frame);
        id
    }

    fn empty_screen() -> Screen {
        Screen::new(|_ctx| -> Vec<WidgetChild> { vec![] })
    }

    #[test]
    fn hot_reload_patches_existing_frame() {
        let mut screen = empty_screen();
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        screen.sync(&mut reg);

        let fid = screen_with_frame(&mut reg, "TestFrame", 200.0);
        assert_eq!(reg.get(fid).unwrap().width, Dimension::Fixed(200.0));

        let (tx, rx) = mpsc::channel();
        screen.set_hot_reload_rx(rx);

        let t = parse_rsx_blocks(
            r#"fn f() { rsx! { frame { name: "TestFrame", width: 400.0 } } }"#,
            "test.rs",
        );
        tx.send(t.into_iter().next().unwrap()).unwrap();
        screen.sync(&mut reg);

        assert_eq!(reg.get(fid).unwrap().width, Dimension::Fixed(400.0));
    }

    #[test]
    fn hot_reload_ignores_unknown_frames() {
        let mut screen = empty_screen();
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        screen.sync(&mut reg);

        screen_with_frame(&mut reg, "Known", 100.0);
        let frame_count_before = reg.frames_iter().count();

        let (tx, rx) = mpsc::channel();
        screen.set_hot_reload_rx(rx);

        let t = parse_rsx_blocks(
            r#"fn f() { rsx! { frame { name: "Unknown", width: 999.0 } } }"#,
            "test.rs",
        );
        tx.send(t.into_iter().next().unwrap()).unwrap();
        screen.sync(&mut reg);

        assert!(reg.get_by_name("Unknown").is_none());
        assert_eq!(reg.frames_iter().count(), frame_count_before);
    }
}

fn auto_size_editboxes(diff: &DiffContext, registry: &mut FrameRegistry) {
    for &fid in &diff.created_frames {
        let Some(frame) = registry.get(fid) else { continue };
        if frame.height.value() > 0.0 { continue }
        let Some(WidgetData::EditBox(eb)) = &frame.widget_data else { continue };
        let font_size = eb.font_size;
        let v_inset = if eb.text_insets != [0.0; 4] {
            eb.text_insets[2] + eb.text_insets[3]
        } else {
            0.0
        };
        let frame = registry.get_mut(fid).unwrap();
        frame.height = Dimension::Fixed(font_size + font_size * 0.5 + v_inset);
    }
}
