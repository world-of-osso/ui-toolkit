pub mod parser;
pub mod watcher;

use crate::widget_def::WidgetChild;

/// A hot-reloaded RSX template identified by source location.
pub struct HotReloadTemplate {
    /// Source location key: (file_path, line, column).
    pub key: (String, u32, u32),
    /// Parsed widget tree (static values only).
    pub defs: Vec<WidgetChild>,
}
