#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::Workspace;

#[test]
fn theme_setting_defaults_to_system_after_migrations() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let theme = ws.get_setting("theme").unwrap();
    assert_eq!(theme.as_deref(), Some("system"));
}

#[test]
fn set_setting_then_get_setting_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    ws.set_setting("theme", "dark").unwrap();
    assert_eq!(ws.get_setting("theme").unwrap().as_deref(), Some("dark"));
    assert_eq!(ws.get_setting("nonexistent").unwrap(), None);
}

#[test]
fn set_setting_overwrites_existing_value() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    ws.set_setting("theme", "dark").unwrap();
    ws.set_setting("theme", "light").unwrap();
    assert_eq!(ws.get_setting("theme").unwrap().as_deref(), Some("light"));
}
