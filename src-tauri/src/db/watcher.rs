use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Poll the SQLite WAL file for changes and emit Tauri events.
/// Checks the mtime of `~/.kanban/data.db-wal` every 500ms.
pub fn spawn_wal_watcher(app_handle: tauri::AppHandle) {
    let wal_path = dirs::home_dir()
        .map(|h| h.join(".kanban").join("data.db-wal"))
        .unwrap_or_else(|| PathBuf::from("data.db-wal"));

    std::thread::spawn(move || {
        let mut last_mtime: Option<SystemTime> = None;
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if let Ok(metadata) = std::fs::metadata(&wal_path) {
                if let Ok(mtime) = metadata.modified() {
                    if last_mtime.map_or(true, |prev| mtime != prev) {
                        if last_mtime.is_some() {
                            let _ = tauri::Emitter::emit(&app_handle, "db-changed", ());
                        }
                        last_mtime = Some(mtime);
                    }
                }
            }
        }
    });
}
