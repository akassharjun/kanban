mod error;

// FIXME(spec2-phase2): scaffold entry point; replaced by real error handling in Phase 2.
#[allow(clippy::expect_used)]
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
