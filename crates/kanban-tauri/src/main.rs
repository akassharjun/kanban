use kanban_tauri::state::AppState;

// Startup `expect`s are intentional: a workspace that cannot open or a Tauri
// event loop that fails to start are fatal, unrecoverable startup errors.
#[allow(clippy::expect_used)]
fn main() {
    let app_state = AppState::from_default().expect("failed to open kanban workspace");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
