use std::path::PathBuf;
use std::sync::mpsc;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use super::HotReloadTemplate;
use super::parser::parse_rsx_blocks;

/// Start watching directories for .rs file changes.
/// Returns a receiver that yields hot-reloaded templates.
pub fn start_watcher(watch_dirs: Vec<PathBuf>) -> mpsc::Receiver<HotReloadTemplate> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = notify_tx.send(event);
                }
            },
            notify::Config::default(),
        ).expect("failed to create file watcher");

        for dir in &watch_dirs {
            let _ = watcher.watch(dir, RecursiveMode::Recursive);
        }

        for event in notify_rx {
            if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                continue;
            }
            for path in &event.paths {
                if path.extension().is_some_and(|e| e == "rs") {
                    let Ok(source) = std::fs::read_to_string(path) else { continue };
                    let file_str = path.to_string_lossy().to_string();
                    let templates = parse_rsx_blocks(&source, &file_str);
                    for t in templates {
                        let _ = tx.send(t);
                    }
                }
            }
        }
    });
    rx
}
