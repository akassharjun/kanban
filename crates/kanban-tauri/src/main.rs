use kanban_tauri::state::AppState;

// Startup `expect`s are intentional: a workspace that cannot open or a Tauri
// event loop that fails to start are fatal, unrecoverable startup errors.
#[allow(clippy::expect_used)]
fn main() {
    let app_state = AppState::from_default().expect("failed to open kanban workspace");
    let specta = kanban_tauri::specta_builder();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
