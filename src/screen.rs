use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
#[cfg(debug_assertions)]
use std::sync::{Mutex, OnceLock};

use crate::frame::{Dimension, WidgetData};
#[cfg(debug_assertions)]
use crate::hotreload::HotReloadTemplate;
use crate::registry::FrameRegistry;
use crate::text_measure::measure_text;
use crate::widget_def::WidgetChild;
use crate::widget_def_diff::DiffContext;

#[cfg(all(test, debug_assertions))]
thread_local! {
    // Keep deterministic reload tests separate from the process-wide file watcher.
    static TEST_HOT_RELOAD_RX: RefCell<Option<std::sync::mpsc::Receiver<HotReloadTemplate>>> = const { RefCell::new(None) };
}

#[cfg(debug_assertions)]
static GLOBAL_HOT_RELOAD_RX: OnceLock<Mutex<std::sync::mpsc::Receiver<HotReloadTemplate>>> =
    OnceLock::new();

/// Shared reactive context with generation-based dependency tracking.
/// Replaces per-Screen ScreenContext. One instance holds all state;
/// each value has a generation counter that advances on insert.
pub struct SharedContext {
    values: HashMap<TypeId, Box<dyn Any>>,
    generations: HashMap<TypeId, u64>,
    read_tracker: RefCell<HashSet<TypeId>>,
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            generations: HashMap::new(),
            read_tracker: RefCell::new(HashSet::new()),
        }
    }

    /// Store a value, incrementing its generation counter.
    pub fn insert<T: 'static>(&mut self, val: T) {
        let tid = TypeId::of::<T>();
        let g = self.generations.entry(tid).or_insert(0);
        *g += 1;
        self.values.insert(tid, Box::new(val));
    }

    /// Read a value, recording it as a dependency for the current build.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let tid = TypeId::of::<T>();
        self.read_tracker.borrow_mut().insert(tid);
        self.values.get(&tid)?.downcast_ref()
    }

    /// Current generation for a type (0 if never inserted).
    pub fn generation<T: 'static>(&self) -> u64 {
        self.generations
            .get(&TypeId::of::<T>())
            .copied()
            .unwrap_or(0)
    }

    fn generation_of(&self, tid: &TypeId) -> u64 {
        self.generations.get(tid).copied().unwrap_or(0)
    }

    fn start_tracking(&self) {
        self.read_tracker.borrow_mut().clear();
    }

    fn take_reads(&self) -> HashMap<TypeId, u64> {
        let reads = self.read_tracker.borrow();
        reads
            .iter()
            .map(|&tid| (tid, self.generation_of(&tid)))
            .collect()
    }
}

impl Default for SharedContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen: manages a UI component's lifecycle against FrameRegistry.
pub struct Screen {
    build_fn: Box<dyn Fn(&SharedContext) -> Vec<WidgetChild>>,
    deps: HashMap<TypeId, u64>,
    diff: DiffContext,
    initialized: bool,
    parent_frame_name: Option<String>,
}

impl Screen {
    pub fn new<F: Fn(&SharedContext) -> Vec<WidgetChild> + 'static>(f: F) -> Self {
        Self {
            build_fn: Box::new(f),
            deps: HashMap::new(),
            diff: DiffContext::new(),
            initialized: false,
            parent_frame_name: None,
        }
    }

    /// Create a Screen that renders into a named parent frame (created by another Screen).
    pub fn with_parent<F: Fn(&SharedContext) -> Vec<WidgetChild> + 'static>(
        f: F,
        parent_frame_name: &str,
    ) -> Self {
        Self {
            build_fn: Box::new(f),
            deps: HashMap::new(),
            diff: DiffContext::new(),
            initialized: false,
            parent_frame_name: Some(parent_frame_name.to_string()),
        }
    }

    /// Sync the widget tree against the registry using shared context.
    /// Only rebuilds if a dependency's generation has advanced since last render.
    pub fn sync(&mut self, ctx: &SharedContext, registry: &mut FrameRegistry) {
        // 1. Check if rebuild needed
        let needs_rebuild = !self.initialized || self.deps_changed(ctx);
        if needs_rebuild {
            ctx.start_tracking();
            let tree = (self.build_fn)(ctx);
            self.deps = ctx.take_reads();
            let parent_id = self.resolve_parent(registry);
            self.diff.diff_roots(&tree, parent_id, registry);
            self.initialized = true;
        }

        // 2. Auto-size
        let frame_ids = collect_all_frame_ids(&self.diff.created_frames, registry);
        auto_size_fontstrings(&frame_ids, registry);
        auto_size_editboxes(&frame_ids, registry);
    }

    fn deps_changed(&self, ctx: &SharedContext) -> bool {
        self.deps
            .iter()
            .any(|(tid, &last_gen)| ctx.generation_of(tid) > last_gen)
    }

    fn resolve_parent(&self, registry: &FrameRegistry) -> Option<u64> {
        self.parent_frame_name
            .as_ref()
            .and_then(|name| registry.get_by_name(name))
    }

    /// Remove all frames created by this screen (roots + their subtrees).
    pub fn teardown(&mut self, registry: &mut FrameRegistry) {
        for &fid in self.diff.created_frames.iter().rev() {
            registry.remove_frame_tree(fid);
        }
        self.diff = DiffContext::new();
        self.initialized = false;
        self.deps.clear();
    }

    /// Get all frame IDs owned by this screen.
    pub fn all_frame_ids(&self) -> &[u64] {
        &self.diff.created_frames
    }
}

#[cfg(debug_assertions)]
pub fn init_global_hot_reload(watch_dirs: Vec<PathBuf>) {
    let rx = crate::hotreload::watcher::start_watcher(watch_dirs);
    let _ = GLOBAL_HOT_RELOAD_RX.set(Mutex::new(rx));
}

#[cfg(not(debug_assertions))]
pub fn init_global_hot_reload(_watch_dirs: Vec<PathBuf>) {}

/// Apply queued attribute patches independently of normal screen synchronization.
#[cfg(debug_assertions)]
pub(crate) fn poll_hot_reload(
    mut diff: bevy::prelude::Local<DiffContext>,
    mut ui: bevy::prelude::ResMut<crate::plugin::UiState>,
) {
    drain_global_hot_reload(&mut diff, &mut ui.registry);
}

#[cfg(debug_assertions)]
fn drain_global_hot_reload(diff: &mut DiffContext, registry: &mut FrameRegistry) {
    #[cfg(test)]
    if TEST_HOT_RELOAD_RX.with(|slot| {
        let slot = slot.borrow();
        let Some(rx) = slot.as_ref() else {
            return false;
        };
        while let Ok(template) = rx.try_recv() {
            diff.patch_by_name(&template.defs, registry);
        }
        true
    }) {
        return;
    }
    let Some(rx) = GLOBAL_HOT_RELOAD_RX.get() else {
        return;
    };
    let Ok(rx) = rx.lock() else {
        return;
    };
    diff.log_changes = true;
    while let Ok(template) = rx.try_recv() {
        diff.patch_by_name(&template.defs, registry);
    }
    diff.log_changes = false;
}

fn collect_all_frame_ids(roots: &[u64], registry: &FrameRegistry) -> Vec<u64> {
    let mut all = Vec::new();
    let mut stack: Vec<u64> = roots.iter().copied().collect();
    while let Some(fid) = stack.pop() {
        all.push(fid);
        stack.extend(registry.children_of(fid));
    }
    all
}

fn auto_size_fontstrings(frame_ids: &[u64], registry: &mut FrameRegistry) {
    for &fid in frame_ids {
        let Some(frame) = registry.get(fid) else {
            continue;
        };
        let Some(WidgetData::FontString(fs)) = &frame.widget_data else {
            continue;
        };
        if frame.width.value() > 0.0 || fs.text.is_empty() {
            continue;
        }
        let text = fs.text.clone();
        let font = fs.font;
        let font_size = fs.font_size;
        if let Some((w, h)) = measure_text(&text, font, font_size) {
            if frame.width == Dimension::Fixed(w) && frame.height == Dimension::Fixed(h) {
                continue;
            }
            let frame = registry.get_mut(fid).unwrap();
            frame.width = Dimension::Fixed(w);
            frame.height = Dimension::Fixed(h);
            registry.mark_rect_dirty(fid);
        }
    }
}

fn auto_size_editboxes(frame_ids: &[u64], registry: &mut FrameRegistry) {
    for &fid in frame_ids {
        let Some(frame) = registry.get(fid) else {
            continue;
        };
        if frame.height.value() > 0.0 {
            continue;
        }
        let Some(WidgetData::EditBox(eb)) = &frame.widget_data else {
            continue;
        };
        let font_size = eb.font_size;
        let v_inset = if eb.text_insets != [0.0; 4] {
            eb.text_insets[2] + eb.text_insets[3]
        } else {
            0.0
        };
        let height = Dimension::Fixed(font_size + font_size * 0.5 + v_inset);
        if frame.height == height {
            continue;
        }
        registry.get_mut(fid).unwrap().height = height;
        registry.mark_rect_dirty(fid);
    }
}

#[cfg(all(test, debug_assertions))]
#[path = "screen_hot_reload_tests.rs"]
mod hot_reload_tests;

#[cfg(test)]
#[path = "screen_layout_tests.rs"]
mod layout_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_def::WidgetChild;

    fn sizing_widget(tag: &'static str, name: &str) -> crate::widget_def::WidgetDef {
        let mut widget = crate::widget_def::WidgetDef::new(tag);
        widget.name = Some(name.to_string());
        widget
    }

    fn sizing_dimensions(registry: &FrameRegistry, id: u64) -> (f32, f32) {
        let frame = registry.get(id).unwrap();
        (frame.width.value(), frame.height.value())
    }

    #[test]
    fn screen_sizing_preserves_nested_text_and_editbox_updates() {
        let mut screen = Screen::new(|_| {
            let mut nested = sizing_widget("Frame", "nested");
            nested.children = vec![
                WidgetChild::Widget(sizing_widget("FontString", "label")),
                WidgetChild::Widget(sizing_widget("EditBox", "input")),
            ];
            let mut root = sizing_widget("Frame", "root");
            root.children.push(WidgetChild::Widget(nested));
            vec![WidgetChild::Widget(root)]
        });
        let ctx = SharedContext::new();
        let mut registry = FrameRegistry::new(800.0, 600.0);
        screen.sync(&ctx, &mut registry);
        let label = registry.get_by_name("label").unwrap();
        let input = registry.get_by_name("input").unwrap();
        let (font, font_size) = {
            let frame = registry.get_mut(label).unwrap();
            frame.width = Dimension::Fixed(0.0);
            let Some(WidgetData::FontString(text)) = &mut frame.widget_data else {
                panic!("label must be text")
            };
            text.text = "Hi".to_string();
            (text.font, text.font_size)
        };
        {
            let frame = registry.get_mut(input).unwrap();
            frame.width = Dimension::Fixed(120.0);
            frame.height = Dimension::Fixed(0.0);
            let Some(WidgetData::EditBox(editbox)) = &mut frame.widget_data else {
                panic!("input must be an editbox")
            };
            editbox.font_size = 20.0;
            editbox.text_insets = [1.0, 2.0, 3.0, 4.0];
        }
        screen.sync(&ctx, &mut registry);
        let short_size = measure_text("Hi", font, font_size).expect("fixture font must load");
        assert_eq!(sizing_dimensions(&registry, label), short_size);
        assert_eq!(sizing_dimensions(&registry, input), (120.0, 37.0));
        screen.sync(&ctx, &mut registry);
        assert_eq!(sizing_dimensions(&registry, label), short_size);
        assert_eq!(sizing_dimensions(&registry, input), (120.0, 37.0));
        {
            let frame = registry.get_mut(label).unwrap();
            let Some(WidgetData::FontString(text)) = &mut frame.widget_data else {
                unreachable!()
            };
            text.text = "A much longer label".to_string();
        }
        screen.sync(&ctx, &mut registry);
        assert_eq!(sizing_dimensions(&registry, label), short_size);
        registry.get_mut(label).unwrap().width = Dimension::Fixed(0.0);
        registry.get_mut(input).unwrap().height = Dimension::Fixed(0.0);
        if let Some(WidgetData::EditBox(editbox)) =
            &mut registry.get_mut(input).unwrap().widget_data
        {
            editbox.font_size = 24.0;
        }
        screen.sync(&ctx, &mut registry);
        let long_size = measure_text("A much longer label", font, font_size).unwrap();
        assert!(long_size.0 > short_size.0);
        assert_eq!(sizing_dimensions(&registry, label), long_size);
        assert_eq!(sizing_dimensions(&registry, input), (120.0, 43.0));
    }

    #[test]
    fn screen_with_no_deps_never_rebuilds_after_init() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();
        let mut screen = Screen::new(move |_ctx| -> Vec<WidgetChild> {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            vec![]
        });
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        let ctx = SharedContext::new();

        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 1);

        screen.sync(&ctx, &mut reg);
        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn screen_rebuilds_only_when_read_type_generation_advances() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();
        let mut screen = Screen::new(move |ctx| -> Vec<WidgetChild> {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            ctx.get::<String>(); // record dependency on String
            vec![]
        });
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        let mut ctx = SharedContext::new();
        ctx.insert("hello".to_string());

        // First sync: builds
        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 1);

        // No change: no rebuild
        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 1);

        // Insert new value (generation advances): rebuilds
        ctx.insert("world".to_string());
        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 2);

        // Insert unrelated type: no rebuild (screen didn't read u32)
        ctx.insert(42u32);
        screen.sync(&ctx, &mut reg);
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn counting_screen<T: 'static>(counter: &Arc<AtomicU32>) -> Screen {
        let c = counter.clone();
        Screen::new(move |ctx| {
            c.fetch_add(1, Ordering::Relaxed);
            ctx.get::<T>();
            vec![]
        })
    }

    fn assert_builds(counter: &AtomicU32, expected: u32) {
        assert_eq!(counter.load(Ordering::Relaxed), expected);
    }

    #[test]
    fn two_screens_sharing_context_only_affected_one_rebuilds() {
        let counter_a = Arc::new(AtomicU32::new(0));
        let counter_b = Arc::new(AtomicU32::new(0));
        let mut screen_a = counting_screen::<String>(&counter_a);
        let mut screen_b = counting_screen::<u32>(&counter_b);
        let mut reg = FrameRegistry::new(1920.0, 1080.0);
        let mut ctx = SharedContext::new();
        ctx.insert("init".to_string());
        ctx.insert(0u32);

        screen_a.sync(&ctx, &mut reg);
        screen_b.sync(&ctx, &mut reg);
        assert_builds(&counter_a, 1);
        assert_builds(&counter_b, 1);

        // Change only String: screen_a rebuilds, screen_b does not
        ctx.insert("changed".to_string());
        screen_a.sync(&ctx, &mut reg);
        screen_b.sync(&ctx, &mut reg);
        assert_builds(&counter_a, 2);
        assert_builds(&counter_b, 1);

        // Change only u32: screen_b rebuilds, screen_a does not
        ctx.insert(42u32);
        screen_a.sync(&ctx, &mut reg);
        screen_b.sync(&ctx, &mut reg);
        assert_builds(&counter_a, 2);
        assert_builds(&counter_b, 2);

        // No changes: neither rebuilds
        screen_a.sync(&ctx, &mut reg);
        screen_b.sync(&ctx, &mut reg);
        assert_builds(&counter_a, 2);
        assert_builds(&counter_b, 2);
    }
}
