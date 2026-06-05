#![allow(clippy::unwrap_used)]
use kanban_core::Workspace;
use kanban_tauri::commands::{get_settings_inner, update_settings_inner};
use kanban_tauri::settings::ThemeChoice;

#[test]
fn get_settings_defaults_to_system() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap(); // &Path
    let settings = get_settings_inner(&ws).unwrap();
    assert_eq!(settings.theme, ThemeChoice::System);
}

#[test]
fn update_settings_persists_theme_choice() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let after = update_settings_inner(&ws, ThemeChoice::Dark).unwrap();
    assert_eq!(after.theme, ThemeChoice::Dark);
    let again = get_settings_inner(&ws).unwrap();
    assert_eq!(again.theme, ThemeChoice::Dark);
}
