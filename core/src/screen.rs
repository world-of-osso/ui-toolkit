use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::registry::FrameRegistry;
use crate::widget_def::WidgetChild;
use crate::widget_def_diff::DiffContext;

/// Shared reactive context with generation-based dependency tracking.
/// Replaces per-Screen ScreenContext. One instance holds all state;
/// each value has a generation counter that advances on insert.
pub struct SharedContext {
    values: HashMap<TypeId, Box<dyn Any>>,
    generations: HashMap<TypeId, u64>,
    read_tracker: RefCell<HashSet<TypeId>>,
    /// Scroll list positions loaded from the registry for the current build.
    scroll_rows: RefCell<HashMap<String, usize>>,
    scroll_reads: RefCell<HashSet<String>>,
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            generations: HashMap::new(),
            read_tracker: RefCell::new(HashSet::new()),
            scroll_rows: RefCell::new(HashMap::new()),
            scroll_reads: RefCell::new(HashSet::new()),
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

    /// First visible row of the named scroll list, recorded as a build dependency.
    pub fn scroll_first_row(&self, name: &str) -> usize {
        self.scroll_reads.borrow_mut().insert(name.to_string());
        self.scroll_rows.borrow().get(name).copied().unwrap_or(0)
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

    fn start_tracking(&self, registry: &FrameRegistry) {
        self.read_tracker.borrow_mut().clear();
        self.scroll_reads.borrow_mut().clear();
        *self.scroll_rows.borrow_mut() = registry.scroll_lists.first_rows();
    }

    fn take_scroll_reads(&self, registry: &FrameRegistry) -> HashMap<String, u64> {
        self.scroll_reads
            .borrow_mut()
            .drain()
            .map(|name| {
                let generation = registry.scroll_lists.generation(&name);
                (name, generation)
            })
            .collect()
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
    scroll_deps: HashMap<String, u64>,
    diff: DiffContext,
    initialized: bool,
    parent_frame_name: Option<String>,
}

impl Screen {
    pub fn new<F: Fn(&SharedContext) -> Vec<WidgetChild> + 'static>(f: F) -> Self {
        Self {
            build_fn: Box::new(f),
            deps: HashMap::new(),
            scroll_deps: HashMap::new(),
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
            scroll_deps: HashMap::new(),
            diff: DiffContext::new(),
            initialized: false,
            parent_frame_name: Some(parent_frame_name.to_string()),
        }
    }

    /// Sync the widget tree against the registry using shared context.
    /// Only rebuilds if a dependency's generation has advanced since last render.
    pub fn sync(&mut self, ctx: &SharedContext, registry: &mut FrameRegistry) {
        // 1. Check if rebuild needed
        let needs_rebuild =
            !self.initialized || self.deps_changed(ctx) || self.scroll_deps_changed(registry);
        if needs_rebuild {
            ctx.start_tracking(registry);
            let tree = (self.build_fn)(ctx);
            self.deps = ctx.take_reads();
            self.scroll_deps = ctx.take_scroll_reads(registry);
            let parent_id = self.resolve_parent(registry);
            self.diff.diff_roots(&tree, parent_id, registry);
            self.initialized = true;
        }
    }

    fn scroll_deps_changed(&self, registry: &FrameRegistry) -> bool {
        self.scroll_deps
            .iter()
            .any(|(name, &last_gen)| registry.scroll_lists.generation(name) != last_gen)
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
        self.scroll_deps.clear();
    }

    /// Get all frame IDs owned by this screen.
    pub fn all_frame_ids(&self) -> &[u64] {
        &self.diff.created_frames
    }
}

#[cfg(test)]
#[path = "screen_layout_tests.rs"]
mod layout_tests;

#[cfg(test)]
mod tests {
    use super::*;
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
