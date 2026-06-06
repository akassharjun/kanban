pub mod commands;
pub mod dto;
pub mod error;
pub mod settings;
pub mod state;

use tauri_specta::{Builder, collect_commands, collect_events};

/// Build the tauri-specta command/event registry. Shared by `main` (invoke
/// handler) and the binding-export test so the registered commands and the
/// generated TypeScript bindings never drift.
#[must_use]
pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::list_projects,
            commands::get_project,
            commands::list_issues,
            commands::get_issue,
            commands::list_statuses,
            commands::list_labels,
            commands::list_members,
            commands::get_settings,
            commands::update_settings,
            commands::apply,
            commands::undo,
            commands::redo,
        ])
        .events(collect_events![])
}
