mod commands;
pub mod db;
pub mod models;
pub mod orchestration;
mod state;
pub mod cli;
pub mod mcp;

use db::DbBackend;
use state::AppState;
use tauri::Emitter;
use tauri::Manager;

use std::time::Duration;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_gui(None)
}

pub fn run_gui(database_url: Option<String>) {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let (pool, backend) = match rt.block_on(db::init_db(database_url.as_deref())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Database connection failed: {}", e);
                    if database_url.as_deref().map_or(false, |u| u.starts_with("postgres")) {
                        eprintln!("Make sure Docker is running: docker compose up -d");
                        eprintln!("Expected: Postgres on localhost:5433");
                    }
                    return Err(Box::from(format!(
                        "Failed to connect to database.\n\nError: {}",
                        e
                    )));
                }
            };
            app.manage(AppState { pool, backend, rt });

            // Cross-process change detection
            match backend {
                DbBackend::Sqlite => {
                    db::watcher::spawn_wal_watcher(app.handle().clone());
                }
                DbBackend::Postgres => {
                    #[cfg(feature = "redis-sync")]
                    {
                        let app_handle_redis = app.handle().clone();
                        std::thread::spawn(move || {
                            let redis_url = std::env::var("REDIS_URL")
                                .unwrap_or_else(|_| "redis://localhost:6379".to_string());
                            if let Ok(client) = redis::Client::open(redis_url) {
                                if let Ok(mut conn) = client.get_connection() {
                                    let mut pubsub = conn.as_pubsub();
                                    let _ = pubsub.subscribe("kanban:db-changed");
                                    loop {
                                        if let Ok(_msg) = pubsub.get_message() {
                                            let _ = app_handle_redis.emit("db-changed", ());
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            }

            // Timeout recovery thread - reclaims stale tasks every 30 seconds
            let pool_clone = app.state::<AppState>().pool.clone();
            let backend_clone = backend;
            let app_handle2 = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create timeout runtime");
                loop {
                    std::thread::sleep(Duration::from_secs(30));
                    let pool = pool_clone.clone();
                    rt.block_on(async {
                        // Reclaim timed-out tasks
                        if let Ok(reclaimed) = crate::orchestration::timeout::reclaim_timed_out_tasks(&pool, &backend_clone).await {
                            if !reclaimed.is_empty() {
                                let _ = app_handle2.emit("db-changed", ());
                            }
                        }
                        // Reclaim offline agents' tasks
                        if let Ok(offline) = crate::orchestration::timeout::reclaim_offline_agents(&pool, &backend_clone).await {
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
            commands::projects::restore_project,
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
            commands::issues::get_audit_log,
            commands::issues::get_issue_history,
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
            // Mentions
            commands::mentions::list_mentions,
            commands::mentions::search_members_for_mention,
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
            // Hooks
            commands::hooks::list_hooks,
            commands::hooks::create_hook,
            commands::hooks::delete_hook,
            commands::agents::get_project_agent_config,
            commands::agents::update_project_agent_config,
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
            commands::task_contracts::invalidate_task,
            commands::task_contracts::task_graph,
            commands::task_contracts::project_metrics,
            commands::task_contracts::agent_metrics_cmd,
            // Epics
            commands::epics::list_epics,
            commands::epics::get_epic,
            commands::epics::create_epic,
            commands::epics::update_epic,
            commands::epics::delete_epic,
            // Milestones
            commands::milestones::list_milestones,
            commands::milestones::get_milestone,
            commands::milestones::create_milestone,
            commands::milestones::update_milestone,
            commands::milestones::delete_milestone,
            // Execution Logs
            commands::execution_logs::log_task_activity,
            commands::execution_logs::task_replay,
            commands::execution_logs::task_attempts,
            commands::execution_logs::recent_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
