use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use super::HotReloadTemplate;
use super::parser::parse_rsx_blocks;

const DEBOUNCE: Duration = Duration::from_millis(50);

fn is_rs_change(event: &Event) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

fn collect_changed_rs_paths(event: &Event) -> impl Iterator<Item = &PathBuf> {
    event.paths.iter().filter(|p| p.extension().is_some_and(|e| e == "rs"))
}

/// Drain pending events for DEBOUNCE duration, collecting unique .rs paths.
fn drain_debounce(notify_rx: &mpsc::Receiver<Event>, paths: &mut HashSet<PathBuf>) {
    while let Ok(event) = notify_rx.recv_timeout(DEBOUNCE) {
        if is_rs_change(&event) {
            paths.extend(collect_changed_rs_paths(&event).cloned());
        }
    }
}

fn process_changed_paths(paths: &HashSet<PathBuf>, tx: &mpsc::Sender<HotReloadTemplate>) {
    for path in paths {
        let Ok(source) = std::fs::read_to_string(path) else { continue };
        let file_str = path.to_string_lossy().to_string();
        let templates = parse_rsx_blocks(&source, &file_str);
        log::info!("hot-reload: {} — {} rsx blocks", path.display(), templates.len());
        for t in templates {
            let _ = tx.send(t);
        }
    }
}

/// Start watching directories for .rs file changes.
/// Returns a receiver that yields hot-reloaded templates.
pub fn start_watcher(watch_dirs: Vec<PathBuf>) -> mpsc::Receiver<HotReloadTemplate> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || run_watcher_loop(watch_dirs, tx));
    rx
}

fn run_watcher_loop(watch_dirs: Vec<PathBuf>, tx: mpsc::Sender<HotReloadTemplate>) {
    let (notify_tx, notify_rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res { let _ = notify_tx.send(event); }
        },
        notify::Config::default(),
    ).expect("failed to create file watcher");

    for dir in &watch_dirs {
        log::info!("hot-reload: watching {}", dir.display());
        let _ = watcher.watch(dir, RecursiveMode::Recursive);
    }

    for event in &notify_rx {
        if !is_rs_change(&event) { continue; }
        let mut paths: HashSet<PathBuf> = HashSet::new();
        paths.extend(collect_changed_rs_paths(&event).cloned());
        drain_debounce(&notify_rx, &mut paths);
        process_changed_paths(&paths, &tx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ui_toolkit_watcher_{}_{}", name, std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn watcher_emits_templates_on_rs_file_change() {
        let dir = test_dir("emit");
        let rx = start_watcher(vec![dir.clone()]);

        // Give the watcher time to register
        std::thread::sleep(Duration::from_millis(200));

        let source = r#"fn build() { rsx! { frame { name: "Hot", width: 100.0 } } }"#;
        fs::write(dir.join("ui.rs"), source).unwrap();

        let template = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher should emit template for .rs change");
        assert_eq!(template.defs.len(), 1);
        assert!(template.key.0.contains("ui.rs"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn watcher_ignores_non_rs_files() {
        let dir = test_dir("ignore");
        let rx = start_watcher(vec![dir.clone()]);

        std::thread::sleep(Duration::from_millis(200));

        fs::write(dir.join("data.txt"), "not rust").unwrap();

        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());

        fs::remove_dir_all(&dir).ok();
    }
}
