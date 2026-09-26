pub use ui_toolkit_core::screen::*;

#[cfg(debug_assertions)]
use crate::hotreload::HotReloadTemplate;
use crate::registry::FrameRegistry;
use crate::widget_def_diff::DiffContext;
#[cfg(all(test, debug_assertions))]
use std::cell::RefCell;
use std::path::PathBuf;
#[cfg(debug_assertions)]
use std::sync::{Mutex, OnceLock};
#[cfg(all(test, debug_assertions))]
thread_local! {
    // Keep deterministic reload tests separate from the process-wide file watcher.
    static TEST_HOT_RELOAD_RX: RefCell<Option<std::sync::mpsc::Receiver<HotReloadTemplate>>> = const { RefCell::new(None) };
}

#[cfg(debug_assertions)]
static GLOBAL_HOT_RELOAD_RX: OnceLock<Mutex<std::sync::mpsc::Receiver<HotReloadTemplate>>> =
    OnceLock::new();

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

#[cfg(all(test, debug_assertions))]
#[path = "screen_hot_reload_tests.rs"]
mod hot_reload_tests;
