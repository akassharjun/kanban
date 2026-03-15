mod commands;
pub mod db;
pub mod models;
pub mod orchestration;
mod state;

use state::AppState;
use tauri::Emitter;
use tauri::Manager;

use std::time::{Duration, SystemTime};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let pool = rt.block_on(db::init_db())?;
            app.manage(AppState { pool, rt });

            // Watch the SQLite DB file for external modifications (CLI/MCP writes)
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let db_path = dirs::home_dir()
                    .expect("Failed to resolve home directory")
                    .join(".kanban/data.db");
                let mut last_modified = std::fs::metadata(&db_path)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                loop {
                    std::thread::sleep(Duration::from_secs(2));
                    if let Ok(meta) = std::fs::metadata(&db_path) {
                        if let Ok(modified) = meta.modified() {
                            if modified > last_modified {
                                last_modified = modified;
                                let _ = app_handle.emit("db-changed", ());
                            }
                        }
                    }
                }
            });

            // Timeout recovery thread - reclaims stale tasks every 30 seconds
            let pool_clone = app.state::<AppState>().pool.clone();
            let app_handle2 = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create timeout runtime");
                loop {
                    std::thread::sleep(Duration::from_secs(30));
                    let pool = pool_clone.clone();
                    rt.block_on(async {
                        // Reclaim timed-out tasks
                        if let Ok(reclaimed) = crate::orchestration::timeout::reclaim_timed_out_tasks(&pool).await {
                            if !reclaimed.is_empty() {
                                let _ = app_handle2.emit("db-changed", ());
                            }
                        }
                        // Reclaim offline agents' tasks
                        if let Ok(offline) = crate::orchestration::timeout::reclaim_offline_agents(&pool).await {
                            if !offline.is_empty() {
                                let _ = app_handle2.emit("db-changed", ());
                            }
                        }
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Health
            commands::health::health_check,
            // Projects
            commands::projects::list_projects,
            commands::projects::get_project,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            // Statuses
            commands::statuses::list_statuses,
            commands::statuses::create_status,
            commands::statuses::update_status,
            commands::statuses::delete_status,
            commands::statuses::reorder_statuses,
            // Issues
            commands::issues::create_issue,
            commands::issues::get_issue,
            commands::issues::get_issue_by_identifier,
            commands::issues::list_issues,
            commands::issues::update_issue,
            commands::issues::delete_issue,
            commands::issues::duplicate_issue,
            commands::issues::bulk_update_issues,
            commands::issues::search_issues,
            commands::issues::get_sub_issues,
            commands::issues::set_issue_labels,
            commands::issues::get_activity_log,
            // Members
            commands::members::list_members,
            commands::members::create_member,
            commands::members::update_member,
            commands::members::delete_member,
            // Labels
            commands::labels::list_labels,
            commands::labels::create_label,
            commands::labels::update_label,
            commands::labels::delete_label,
            // Relations
            commands::relations::list_relations,
            commands::relations::create_relation,
            commands::relations::delete_relation,
            // Templates
            commands::templates::list_templates,
            commands::templates::create_template,
            commands::templates::update_template,
            commands::templates::delete_template,
            // Undo/Redo
            commands::undo::undo,
            commands::undo::redo,
            // Notifications
            commands::notifications::list_notifications,
            commands::notifications::unread_notification_count,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_notifications_read,
            commands::notifications::clear_notifications,
            // Comments
            commands::comments::list_comments,
            commands::comments::create_comment,
            commands::comments::update_comment,
            commands::comments::delete_comment,
            commands::comments::comment_count,
            // Custom Fields
            commands::custom_fields::list_custom_fields,
            commands::custom_fields::create_custom_field,
            commands::custom_fields::update_custom_field,
            commands::custom_fields::delete_custom_field,
            commands::custom_fields::get_issue_custom_values,
            commands::custom_fields::set_issue_custom_value,
            // Agents
            commands::agents::register_agent,
            commands::agents::agent_heartbeat,
            commands::agents::deregister_agent,
            commands::agents::list_agents,
            commands::agents::get_agent_stats,
            // Task Contracts
            commands::task_contracts::create_task_contract,
            commands::task_contracts::get_task_contract,
            commands::task_contracts::next_task,
            commands::task_contracts::start_task,
            commands::task_contracts::complete_task,
            commands::task_contracts::fail_task,
            commands::task_contracts::unclaim_task,
            commands::task_contracts::approve_task,
            commands::task_contracts::reject_task,
            // Execution Logs
            commands::execution_logs::log_task_activity,
            commands::execution_logs::task_replay,
            commands::execution_logs::task_attempts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
