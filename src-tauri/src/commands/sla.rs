use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SlaPolicy {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub target_type: String,
    pub priority_filter: Option<String>,
    pub warning_minutes: i64,
    pub breach_minutes: i64,
    pub escalation_action: String,
    pub enabled: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaStatus {
    pub issue_id: i64,
    pub issue_identifier: String,
    pub issue_title: String,
    pub policy_id: i64,
    pub policy_name: String,
    pub status: String,
    pub elapsed_minutes: f64,
    pub remaining_minutes: f64,
    pub breach_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SlaEvent {
    pub id: i64,
    pub sla_policy_id: i64,
    pub issue_id: i64,
    pub event_type: String,
    pub message: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDashboard {
    pub total_tracked: i64,
    pub total_ok: i64,
    pub total_warning: i64,
    pub total_breached: i64,
    pub policies: Vec<SlaPolicy>,
    pub statuses: Vec<SlaStatus>,
    pub recent_events: Vec<SlaEvent>,
}

#[derive(Deserialize)]
pub struct CreateSlaPolicyInput {
    pub project_id: i64,
    pub name: String,
    pub target_type: String,
    pub priority_filter: Option<String>,
    pub warning_minutes: i64,
    pub breach_minutes: i64,
    pub escalation_action: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSlaPolicyInput {
    pub name: Option<String>,
    pub target_type: Option<String>,
    pub priority_filter: Option<String>,
    pub warning_minutes: Option<i64>,
    pub breach_minutes: Option<i64>,
    pub escalation_action: Option<String>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn list_sla_policies(state: State<AppState>, project_id: i64) -> Result<Vec<SlaPolicy>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, SlaPolicy>(
            "SELECT * FROM sla_policies WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_sla_policy(state: State<AppState>, input: CreateSlaPolicyInput) -> Result<SlaPolicy, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let escalation = input.escalation_action.unwrap_or_else(|| "{}".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO sla_policies (project_id, name, target_type, priority_filter, warning_minutes, breach_minutes, escalation_action, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.name)
        .bind(&input.target_type)
        .bind(&input.priority_filter)
        .bind(input.warning_minutes)
        .bind(input.breach_minutes)
        .bind(&escalation)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, SlaPolicy>("SELECT * FROM sla_policies WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_sla_policy(state: State<AppState>, id: i64, input: UpdateSlaPolicyInput) -> Result<SlaPolicy, String> {
    state.rt.block_on(async {
        if let Some(name) = &input.name {
            sqlx::query("UPDATE sla_policies SET name = $1 WHERE id = $2")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(tt) = &input.target_type {
            sqlx::query("UPDATE sla_policies SET target_type = $1 WHERE id = $2")
                .bind(tt).bind(id).execute(&state.pool).await?;
        }
        if let Some(pf) = &input.priority_filter {
            sqlx::query("UPDATE sla_policies SET priority_filter = $1 WHERE id = $2")
                .bind(pf).bind(id).execute(&state.pool).await?;
        }
        if let Some(wm) = input.warning_minutes {
            sqlx::query("UPDATE sla_policies SET warning_minutes = $1 WHERE id = $2")
                .bind(wm).bind(id).execute(&state.pool).await?;
        }
        if let Some(bm) = input.breach_minutes {
            sqlx::query("UPDATE sla_policies SET breach_minutes = $1 WHERE id = $2")
                .bind(bm).bind(id).execute(&state.pool).await?;
        }
        if let Some(ea) = &input.escalation_action {
            sqlx::query("UPDATE sla_policies SET escalation_action = $1 WHERE id = $2")
                .bind(ea).bind(id).execute(&state.pool).await?;
        }
        if let Some(en) = input.enabled {
            let val: i64 = if en { 1 } else { 0 };
            sqlx::query("UPDATE sla_policies SET enabled = $1 WHERE id = $2")
                .bind(val).bind(id).execute(&state.pool).await?;
        }

        sqlx::query_as::<_, SlaPolicy>("SELECT * FROM sla_policies WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_sla_policy(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM sla_policies WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

/// Calculate SLA status for in-progress issues against applicable policies.
async fn compute_sla_compliance(pool: &sqlx::AnyPool, project_id: i64) -> Result<Vec<SlaStatus>, sqlx::Error> {
    let policies = sqlx::query_as::<_, SlaPolicy>(
        "SELECT * FROM sla_policies WHERE project_id = $1 AND enabled = 1"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    if policies.is_empty() {
        return Ok(Vec::new());
    }

    // Get all in-progress issues (started category)
    let issues = sqlx::query_as::<_, crate::models::Issue>(
        "SELECT i.* FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.project_id = $1 AND s.category = 'started'"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let now = chrono::Utc::now();
    let mut results = Vec::new();

    for issue in &issues {
        for policy in &policies {
            // Check priority filter
            if let Some(ref pf) = policy.priority_filter {
                if !pf.is_empty() && pf != &issue.priority {
                    continue;
                }
            }

            // Calculate elapsed time since issue was last updated to started status
            // We use the activity_log to find when the issue entered 'started' category
            let started_at_str: Option<String> = sqlx::query_scalar(
                "SELECT MIN(al.timestamp) FROM activity_log al WHERE al.issue_id = $1 AND al.field_changed = 'status_id'"
            )
            .bind(issue.id)
            .fetch_optional(pool)
            .await?
            .flatten();

            // Fallback to created_at if no activity log
            let start_time_str = started_at_str.unwrap_or_else(|| issue.created_at.clone());
            let start_time = chrono::NaiveDateTime::parse_from_str(&start_time_str, "%Y-%m-%d %H:%M:%SZ")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&start_time_str, "%Y-%m-%dT%H:%M:%S%.fZ"))
                .unwrap_or_else(|_| now.naive_utc());

            let elapsed = now.naive_utc().signed_duration_since(start_time);
            let elapsed_minutes = elapsed.num_minutes() as f64;
            let remaining = policy.breach_minutes as f64 - elapsed_minutes;

            let breach_at = start_time + chrono::Duration::minutes(policy.breach_minutes);
            let breach_at_str = breach_at.format("%Y-%m-%d %H:%M:%SZ").to_string();

            let status = if elapsed_minutes >= policy.breach_minutes as f64 {
                "breached"
            } else if elapsed_minutes >= (policy.breach_minutes - policy.warning_minutes) as f64 {
                "warning"
            } else {
                "ok"
            };

            results.push(SlaStatus {
                issue_id: issue.id,
                issue_identifier: issue.identifier.clone(),
                issue_title: issue.title.clone(),
                policy_id: policy.id,
                policy_name: policy.name.clone(),
                status: status.to_string(),
                elapsed_minutes,
                remaining_minutes: remaining.max(0.0),
                breach_at: breach_at_str,
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn check_sla_compliance(state: State<AppState>, project_id: i64) -> Result<Vec<SlaStatus>, String> {
    state.rt.block_on(async {
        compute_sla_compliance(&state.pool, project_id).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn enforce_sla(state: State<AppState>, project_id: i64) -> Result<Vec<SlaEvent>, String> {
    state.rt.block_on(async {
        enforce_sla_async(&state.pool, project_id).await
    }).map_err(|e| e.to_string())
}

/// Enforce SLA across all active projects (called from background thread).
pub async fn enforce_sla_all_projects(pool: &sqlx::AnyPool) -> Result<Vec<SlaEvent>, String> {
    let project_ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM projects WHERE status = 'active' AND deleted_at IS NULL"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut all_events = Vec::new();
    for (project_id,) in project_ids {
        if let Ok(events) = enforce_sla_async(pool, project_id).await {
            all_events.extend(events);
        }
    }
    Ok(all_events)
}

pub async fn enforce_sla_async(pool: &sqlx::AnyPool, project_id: i64) -> Result<Vec<SlaEvent>, String> {
    let statuses = compute_sla_compliance(pool, project_id).await.map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    let mut events = Vec::new();

    for s in &statuses {
        if s.status == "ok" {
            continue;
        }

        let event_type = if s.status == "breached" { "breach" } else { "warning" };

        // Check if we already recorded this event recently (within 5 minutes)
        let recent: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sla_events WHERE sla_policy_id = $1 AND issue_id = $2 AND event_type = $3 AND created_at >= datetime('now', '-5 minutes')"
        )
        .bind(s.policy_id)
        .bind(s.issue_id)
        .bind(event_type)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        if recent.unwrap_or(0) > 0 {
            continue;
        }

        let message = format!(
            "SLA {} for {} ({}): elapsed {:.0}m, policy '{}'",
            event_type, s.issue_identifier, s.issue_title, s.elapsed_minutes, s.policy_name
        );

        let eid: i64 = sqlx::query_scalar(
            "INSERT INTO sla_events (sla_policy_id, issue_id, event_type, message, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(s.policy_id)
        .bind(s.issue_id)
        .bind(event_type)
        .bind(&message)
        .bind("{}")
        .bind(&now)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Execute escalation for breached SLAs
        if s.status == "breached" {
            // Look up policy escalation action
            let policy: SlaPolicy = sqlx::query_as(
                "SELECT * FROM sla_policies WHERE id = $1"
            )
            .bind(s.policy_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Ok(action) = serde_json::from_str::<serde_json::Value>(&policy.escalation_action) {
                if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                    match action_type {
                        "change_priority" => {
                            // Escalate priority
                            let current_priority: String = sqlx::query_scalar(
                                "SELECT priority FROM issues WHERE id = $1"
                            )
                            .bind(s.issue_id)
                            .fetch_one(pool)
                            .await
                            .map_err(|e| e.to_string())?;

                            let new_priority = match current_priority.as_str() {
                                "low" => "medium",
                                "medium" => "high",
                                "high" | "none" => "urgent",
                                _ => "urgent",
                            };

                            sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                                .bind(new_priority)
                                .bind(&now)
                                .bind(s.issue_id)
                                .execute(pool)
                                .await
                                .map_err(|e| e.to_string())?;

                            // Log escalation event
                            let _: i64 = sqlx::query_scalar(
                                "INSERT INTO sla_events (sla_policy_id, issue_id, event_type, message, metadata, created_at) VALUES ($1, $2, 'escalated', $3, $4, $5) RETURNING id"
                            )
                            .bind(s.policy_id)
                            .bind(s.issue_id)
                            .bind(format!("Priority escalated from {} to {}", current_priority, new_priority))
                            .bind(format!("{{\"from\":\"{}\",\"to\":\"{}\"}}", current_priority, new_priority))
                            .bind(&now)
                            .fetch_one(pool)
                            .await
                            .map_err(|e| e.to_string())?;
                        }
                        "reassign" => {
                            // Unclaim the task to make it available
                            let _ = sqlx::query(
                                "UPDATE task_contracts SET claimed_by = NULL, claimed_at = NULL, task_state = 'queued' WHERE issue_id = $1 AND task_state IN ('claimed', 'executing')"
                            )
                            .bind(s.issue_id)
                            .execute(pool)
                            .await;
                        }
                        "notify" => {
                            // Create a notification
                            sqlx::query(
                                "INSERT INTO notifications (type, issue_id, message, created_at) VALUES ('sla_breach', $1, $2, $3)"
                            )
                            .bind(s.issue_id)
                            .bind(&message)
                            .bind(&now)
                            .execute(pool)
                            .await
                            .map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                }
            }
        }

        let event = sqlx::query_as::<_, SlaEvent>("SELECT * FROM sla_events WHERE id = $1")
            .bind(eid)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
        events.push(event);
    }

    Ok(events)
}

#[tauri::command]
pub fn get_sla_events(state: State<AppState>, issue_id: i64) -> Result<Vec<SlaEvent>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, SlaEvent>(
            "SELECT * FROM sla_events WHERE issue_id = $1 ORDER BY created_at DESC"
        )
        .bind(issue_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sla_dashboard(state: State<AppState>, project_id: i64) -> Result<SlaDashboard, String> {
    state.rt.block_on(async {
        let policies = sqlx::query_as::<_, SlaPolicy>(
            "SELECT * FROM sla_policies WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let statuses = compute_sla_compliance(&state.pool, project_id).await.map_err(|e| e.to_string())?;

        let recent_events = sqlx::query_as::<_, SlaEvent>(
            "SELECT se.* FROM sla_events se JOIN sla_policies sp ON se.sla_policy_id = sp.id WHERE sp.project_id = $1 ORDER BY se.created_at DESC LIMIT 50"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_tracked = statuses.len() as i64;
        let total_ok = statuses.iter().filter(|s| s.status == "ok").count() as i64;
        let total_warning = statuses.iter().filter(|s| s.status == "warning").count() as i64;
        let total_breached = statuses.iter().filter(|s| s.status == "breached").count() as i64;

        Ok(SlaDashboard {
            total_tracked,
            total_ok,
            total_warning,
            total_breached,
            policies,
            statuses,
            recent_events,
        })
    })
}
