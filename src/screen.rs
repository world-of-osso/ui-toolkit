use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::anchor_resolve::apply_anchor_resolved;
use crate::frame::{Dimension, WidgetData};
use crate::hotreload::HotReloadTemplate;
use crate::registry::FrameRegistry;
use crate::text_measure::measure_text;
use crate::widget_def::WidgetChild;
use crate::widget_def_diff::DiffContext;

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
        drain_global_hot_reload(&mut self.diff, registry);

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

        // 2. Resolve pending anchors
        self.resolve_pending_anchors(registry);

        // 3. Auto-size
        auto_size_fontstrings(&self.diff, registry);
        auto_size_editboxes(&self.diff, registry);
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

    fn resolve_pending_anchors(&mut self, registry: &mut FrameRegistry) {
        let pending = std::mem::take(&mut self.diff.pending_anchors);
        for (frame_id, spec) in pending {
            let already_has = registry
                .get(frame_id)
                .is_some_and(|f| !f.anchors.is_empty());
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

#[cfg(debug_assertions)]
fn drain_global_hot_reload(diff: &mut DiffContext, registry: &mut FrameRegistry) {
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

#[cfg(not(debug_assertions))]
fn drain_global_hot_reload(_diff: &mut DiffContext, _registry: &mut FrameRegistry) {}

fn collect_all_frame_ids(roots: &[u64], registry: &FrameRegistry) -> Vec<u64> {
    let mut all = Vec::new();
    let mut stack: Vec<u64> = roots.iter().copied().collect();
    while let Some(fid) = stack.pop() {
        all.push(fid);
        stack.extend(registry.children_of(fid));
    }
    all
}

fn auto_size_fontstrings(diff: &DiffContext, registry: &mut FrameRegistry) {
    let all_ids = collect_all_frame_ids(&diff.created_frames, registry);
    for fid in all_ids {
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
            let frame = registry.get_mut(fid).unwrap();
            frame.width = Dimension::Fixed(w);
            frame.height = Dimension::Fixed(h);
        }
    }
}

fn auto_size_editboxes(diff: &DiffContext, registry: &mut FrameRegistry) {
    let all_ids = collect_all_frame_ids(&diff.created_frames, registry);
    for fid in all_ids {
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
        let frame = registry.get_mut(fid).unwrap();
        frame.height = Dimension::Fixed(font_size + font_size * 0.5 + v_inset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, WidgetType};
    use crate::widget_def::WidgetChild;

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
