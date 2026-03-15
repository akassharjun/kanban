use crate::models::agent::{Agent, ProjectAgentConfig, TaskContract};
use crate::orchestration::routing::{build_full_contract, FullTaskContract};
use crate::orchestration::state_machine::{task_state_to_status_category, TaskState};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateTaskContractInput {
    pub project_id: i64,
    pub title: String,
    pub objective: String,
    pub status_id: i64,
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub skills: Option<Vec<String>>,
    pub complexity: Option<String>,
    pub constraints: Option<Vec<String>>,
    pub success_criteria: Option<serde_json::Value>,
    pub context_files: Option<Vec<String>>,
    pub timeout_minutes: Option<i64>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct CompleteTaskInput {
    pub identifier: String,
    pub agent_id: String,
    pub confidence: f64,
    pub summary: String,
    pub artifacts: Option<serde_json::Value>,
}

/// Helper: sync issues.status_id based on task state category
async fn sync_issue_status(
    pool: &sqlx::SqlitePool,
    issue_id: i64,
    task_state: TaskState,
    now: &str,
) -> Result<(), sqlx::Error> {
    let category = task_state_to_status_category(task_state);
    sqlx::query(
        "UPDATE issues SET status_id = (
            SELECT s.id FROM statuses s
            WHERE s.project_id = issues.project_id AND s.category = ?
            ORDER BY s.position ASC LIMIT 1
         ), updated_at = ?
         WHERE id = ?",
    )
    .bind(category)
    .bind(now)
    .bind(issue_id)
    .execute(pool)
    .await?;
    Ok(())
}

#[tauri::command]
pub fn create_task_contract(
    state: State<AppState>,
    input: CreateTaskContractInput,
) -> Result<FullTaskContract, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let priority = input.priority.unwrap_or_else(|| "none".to_string());
            let task_type = input.r#type.unwrap_or_else(|| "task".to_string());
            let skills = input.skills.unwrap_or_default();
            let constraints = input.constraints.unwrap_or_default();
            let success_criteria = input
                .success_criteria
                .unwrap_or_else(|| serde_json::Value::Array(vec![]));
            let context_files = input.context_files.unwrap_or_default();
            let timeout_minutes = input.timeout_minutes.unwrap_or(30);
            let depends_on = input.depends_on.unwrap_or_default();

            let mut tx = state.pool.begin().await?;

            // 1. Create the issue (same pattern as issues.rs create_issue)
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix",
            )
            .bind(input.project_id)
            .fetch_one(&mut *tx)
            .await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?",
            )
            .bind(input.project_id)
            .bind(input.status_id)
            .fetch_one(&mut *tx)
            .await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let result = sqlx::query(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(input.project_id)
            .bind(&identifier)
            .bind(&input.title)
            .bind(&input.description)
            .bind(input.status_id)
            .bind(&priority)
            .bind(input.assignee_id)
            .bind(input.parent_id)
            .bind(position)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            let issue_id = result.last_insert_rowid();

            // 2. Build context JSON
            let context = serde_json::json!({
                "files": context_files,
                "related_tasks": [],
                "prior_attempts": []
            });

            // 3. Insert task_contracts row
            let skills_json =
                serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());
            let constraints_json =
                serde_json::to_string(&constraints).unwrap_or_else(|_| "[]".to_string());
            let success_criteria_json =
                serde_json::to_string(&success_criteria).unwrap_or_else(|_| "[]".to_string());
            let context_json =
                serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

            sqlx::query(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES (?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?, 0)",
            )
            .bind(issue_id)
            .bind(&task_type)
            .bind(&input.objective)
            .bind(&context_json)
            .bind(&constraints_json)
            .bind(&success_criteria_json)
            .bind(&skills_json)
            .bind(&input.complexity)
            .bind(timeout_minutes)
            .execute(&mut *tx)
            .await?;

            // 4. If depends_on specified: resolve each identifier to issue_id, insert issue_relations
            for dep_identifier in &depends_on {
                let dep_issue_id: i64 = sqlx::query_scalar(
                    "SELECT id FROM issues WHERE identifier = ?",
                )
                .bind(dep_identifier)
                .fetch_one(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocks')",
                )
                .bind(dep_issue_id)
                .bind(issue_id)
                .execute(&mut *tx)
                .await?;
            }

            tx.commit().await?;

            // Check if this task needs decomposition
            if let Ok(true) = crate::orchestration::decomposition::check_decomposition_needed(&state.pool, issue_id).await {
                let _ = crate::orchestration::decomposition::create_decomposition_task(&state.pool, issue_id).await;
            }

            // 5. Return full contract
            let contract = build_full_contract(&state.pool, issue_id).await?;
            contract.ok_or_else(|| sqlx::Error::RowNotFound)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_task_contract(
    state: State<AppState>,
    identifier: String,
) -> Result<FullTaskContract, String> {
    state
        .rt
        .block_on(async {
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = build_full_contract(&state.pool, issue_id).await?;
            contract.ok_or_else(|| sqlx::Error::RowNotFound)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn next_task(
    state: State<AppState>,
    agent_id: String,
    skills_override: Option<Vec<String>>,
) -> Result<Option<FullTaskContract>, String> {
    state
        .rt
        .block_on(async {
            let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
                .bind(&agent_id)
                .fetch_optional(&state.pool)
                .await?
                .ok_or_else(|| sqlx::Error::Protocol("AGENT_NOT_REGISTERED".to_string()))?;

            let skills: Vec<String> = if let Some(override_skills) = skills_override {
                override_skills
            } else {
                serde_json::from_str(&agent.skills).unwrap_or_default()
            };

            crate::orchestration::routing::next_task(
                &state.pool,
                &agent_id,
                &skills,
                &agent.max_complexity,
                agent.max_concurrent,
            )
            .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn start_task(
    state: State<AppState>,
    agent_id: String,
    identifier: String,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Verify state transition
            let current_state = TaskState::from_str(&contract.task_state)
                .map_err(|e| sqlx::Error::Protocol(e))?;
            if !current_state.can_transition_to(TaskState::Executing) {
                return Err(sqlx::Error::Protocol(format!(
                    "INVALID_TRANSITION: cannot transition from '{}' to 'executing'",
                    contract.task_state
                )));
            }

            // Verify claimed_by matches agent_id
            match &contract.claimed_by {
                Some(claimed) if claimed == &agent_id => {}
                _ => {
                    return Err(sqlx::Error::Protocol(
                        "TASK_NOT_CLAIMED_BY_AGENT".to_string(),
                    ));
                }
            }

            // Update task_state to executing
            sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = ?")
                .bind(issue_id)
                .execute(&state.pool)
                .await?;

            // Log in execution_logs
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'start', 'Task execution started', ?)",
            )
            .bind(issue_id)
            .bind(&agent_id)
            .bind(contract.attempt_count)
            .bind(&now)
            .execute(&state.pool)
            .await?;

            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn complete_task(
    state: State<AppState>,
    input: CompleteTaskInput,
) -> Result<serde_json::Value, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&input.identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Verify claimed_by matches agent_id
            match &contract.claimed_by {
                Some(claimed) if claimed == &input.agent_id => {}
                _ => {
                    return Err(sqlx::Error::Protocol(
                        "TASK_NOT_CLAIMED_BY_AGENT".to_string(),
                    ));
                }
            }

            // Verify state can transition
            let current_state = TaskState::from_str(&contract.task_state)
                .map_err(|e| sqlx::Error::Protocol(e))?;
            // completing can go to validating or completed (both valid from executing)
            if !current_state.can_transition_to(TaskState::Validating)
                && !current_state.can_transition_to(TaskState::Completed)
                && !current_state.can_transition_to(TaskState::Queued)
            {
                return Err(sqlx::Error::Protocol(format!(
                    "INVALID_TRANSITION: cannot complete from state '{}'",
                    contract.task_state
                )));
            }

            // Get full issue for project config and review task creation
            let issue = sqlx::query_as::<_, crate::models::Issue>(
                "SELECT * FROM issues WHERE id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_configs WHERE project_id = ?",
            )
            .bind(issue.project_id)
            .fetch_optional(&state.pool)
            .await?;

            let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.85);
            let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.50);

            // Build result JSON
            let result_json = serde_json::json!({
                "confidence": input.confidence,
                "summary": input.summary,
                "artifacts": input.artifacts,
            });
            let result_str = serde_json::to_string(&result_json).unwrap_or_else(|_| "{}".to_string());

            // Decide new_state based on confidence thresholds
            let (new_state, accepted) = if input.confidence >= auto_accept {
                (TaskState::Completed, true)
            } else if input.confidence >= human_review {
                (TaskState::Validating, false)
            } else {
                // Auto-reject: requeue
                (TaskState::Queued, false)
            };

            let new_state_str = new_state.as_str();

            if new_state == TaskState::Queued {
                // Auto-reject: clear claimed_by/claimed_at, increment attempt_count
                sqlx::query(
                    "UPDATE task_contracts SET task_state = 'queued', result = ?, claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = ?",
                )
                .bind(&result_str)
                .bind(issue_id)
                .execute(&state.pool)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE task_contracts SET task_state = ?, result = ? WHERE issue_id = ?",
                )
                .bind(new_state_str)
                .bind(&result_str)
                .bind(issue_id)
                .execute(&state.pool)
                .await?;
            }

            // Sync issues.status_id
            sync_issue_status(&state.pool, issue_id, new_state, &now).await?;

            // Auto-create review task when entering validating state
            if new_state == TaskState::Validating {
                let review_title = format!("Review: {}", &input.identifier);
                let review_objective = format!(
                    "Verify completion of {} (confidence: {:.2}). Check the execution log and validate the work.",
                    &input.identifier, input.confidence
                );
                let review_context = serde_json::json!({
                    "files": [],
                    "related_tasks": [&input.identifier],
                    "original_task_result": &result_json,
                    "prior_attempts": []
                });

                // Create review issue
                let (counter, prefix): (i64, String) = sqlx::query_as(
                    "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix",
                )
                .bind(issue.project_id)
                .fetch_one(&state.pool)
                .await?;
                let review_identifier = format!("{}-{}", prefix, counter);

                let review_result = sqlx::query(
                    "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, NULL, 0.0, ?, ?)",
                )
                .bind(issue.project_id)
                .bind(&review_identifier)
                .bind(&review_title)
                .bind(&review_objective)
                .bind(issue.status_id)
                .bind(&issue.priority)
                .bind(&now)
                .bind(&now)
                .execute(&state.pool)
                .await?;

                let review_issue_id = review_result.last_insert_rowid();

                // Find an unstarted status for the review task
                let unstarted_sid: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1",
                )
                .bind(issue.project_id)
                .fetch_optional(&state.pool)
                .await?;
                if let Some(sid) = unstarted_sid {
                    sqlx::query("UPDATE issues SET status_id = ? WHERE id = ?")
                        .bind(sid)
                        .bind(review_issue_id)
                        .execute(&state.pool)
                        .await?;
                }

                // Create review task contract
                sqlx::query(
                    "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, required_skills, estimated_complexity, timeout_minutes) VALUES (?, 'review', 'queued', ?, ?, '[\"review\"]', 'small', 30)",
                )
                .bind(review_issue_id)
                .bind(&review_objective)
                .bind(review_context.to_string())
                .execute(&state.pool)
                .await?;

                // Create notification
                sqlx::query(
                    "INSERT INTO notifications (type, issue_id, message, read, created_at) VALUES ('low_confidence', ?, ?, 0, ?)",
                )
                .bind(issue_id)
                .bind(format!(
                    "{} completed with {:.2} confidence, review task {} created",
                    &input.identifier, input.confidence, &review_identifier
                ))
                .bind(&now)
                .execute(&state.pool)
                .await?;
            }

            // Log result in execution_logs
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES (?, ?, ?, 'complete', ?, ?, ?)",
            )
            .bind(issue_id)
            .bind(&input.agent_id)
            .bind(contract.attempt_count)
            .bind(&input.summary)
            .bind(&result_str)
            .bind(&now)
            .execute(&state.pool)
            .await?;

            // If accepted: update agent_stats
            if accepted {
                sqlx::query(
                    "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + ? WHERE agent_id = ?",
                )
                .bind(input.confidence)
                .bind(&input.agent_id)
                .execute(&state.pool)
                .await?;
            }

            // Auto-unblock downstream tasks when completed
            if new_state == TaskState::Completed {
                let _ = crate::orchestration::dependency::resolve_downstream(&state.pool, issue_id).await;
            }

            Ok(serde_json::json!({
                "accepted": accepted,
                "new_state": new_state_str,
            }))
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn fail_task(
    state: State<AppState>,
    agent_id: String,
    identifier: String,
    reason: String,
) -> Result<serde_json::Value, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue = sqlx::query_as::<_, crate::models::Issue>(
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(&identifier)
            .fetch_one(&state.pool)
            .await?;
            let issue_id = issue.id;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            let new_attempt_count = contract.attempt_count + 1;

            // Parse context JSON, append to prior_attempts
            let mut context: serde_json::Value =
                serde_json::from_str(&contract.context).unwrap_or_else(|_| serde_json::json!({}));
            let attempt_entry = serde_json::json!({
                "agent": agent_id,
                "attempt_number": new_attempt_count,
                "result": "failed",
                "reason": reason,
            });
            if let Some(obj) = context.as_object_mut() {
                let arr = obj.entry("prior_attempts").or_insert(serde_json::json!([]));
                if let Some(a) = arr.as_array_mut() {
                    a.push(attempt_entry);
                }
            }
            let context_str = serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

            // Update task_contracts: requeue, clear claimed_by/claimed_at, update attempt_count and context
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = ?, context = ? WHERE issue_id = ?",
            )
            .bind(new_attempt_count)
            .bind(&context_str)
            .bind(issue_id)
            .execute(&state.pool)
            .await?;

            // Check escalation: if attempt_count >= max_attempts, block instead of requeue
            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_configs WHERE project_id = ?",
            )
            .bind(issue.project_id)
            .fetch_optional(&state.pool)
            .await?;

            let max_attempts = config.as_ref().map(|c| c.max_attempts).unwrap_or(3);

            if new_attempt_count >= max_attempts {
                // Escalate: block the task instead of requeuing
                sqlx::query("UPDATE task_contracts SET task_state = 'blocked' WHERE issue_id = ?")
                    .bind(issue_id)
                    .execute(&state.pool)
                    .await?;

                // Sync to blocked status
                let blocked_status: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM statuses WHERE project_id = ? AND category = 'blocked' ORDER BY position LIMIT 1",
                )
                .bind(issue.project_id)
                .fetch_optional(&state.pool)
                .await?;
                if let Some(sid) = blocked_status {
                    sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                        .bind(sid)
                        .bind(&now)
                        .bind(issue_id)
                        .execute(&state.pool)
                        .await?;
                }

                // Create escalation notification
                sqlx::query(
                    "INSERT INTO notifications (type, issue_id, message, read, created_at) VALUES ('escalation', ?, ?, 0, ?)",
                )
                .bind(issue_id)
                .bind(format!(
                    "{} has failed {} times. Human intervention required.",
                    &identifier, new_attempt_count
                ))
                .bind(&now)
                .execute(&state.pool)
                .await?;
            }

            // Log failure in execution_logs
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'fail', ?, ?)",
            )
            .bind(issue_id)
            .bind(&agent_id)
            .bind(new_attempt_count)
            .bind(&reason)
            .bind(&now)
            .execute(&state.pool)
            .await?;

            // Update agent_stats (tasks_failed + 1)
            sqlx::query(
                "UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = ?",
            )
            .bind(&agent_id)
            .execute(&state.pool)
            .await?;

            // Sync issues.status_id to 'unstarted' category
            sync_issue_status(&state.pool, issue_id, TaskState::Queued, &now).await?;

            Ok(serde_json::json!({
                "requeued": true,
                "attempt_number": new_attempt_count,
            }))
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn unclaim_task(
    state: State<AppState>,
    agent_id: String,
    identifier: String,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Verify claimed_by matches agent_id
            match &contract.claimed_by {
                Some(claimed) if claimed == &agent_id => {}
                _ => {
                    return Err(sqlx::Error::Protocol(
                        "TASK_NOT_CLAIMED_BY_AGENT".to_string(),
                    ));
                }
            }

            // Update: task_state='queued', clear claimed_by/claimed_at
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?",
            )
            .bind(issue_id)
            .execute(&state.pool)
            .await?;

            // Sync issues.status_id to 'unstarted'
            sync_issue_status(&state.pool, issue_id, TaskState::Queued, &now).await?;

            // Log in execution_logs
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'unclaim', 'Task unclaimed by agent', ?)",
            )
            .bind(issue_id)
            .bind(&agent_id)
            .bind(contract.attempt_count)
            .bind(&now)
            .execute(&state.pool)
            .await?;

            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn approve_task(state: State<AppState>, identifier: String) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Verify task_state == "validating"
            if contract.task_state != "validating" {
                return Err(sqlx::Error::Protocol(format!(
                    "TASK_WRONG_STATE: expected 'validating', got '{}'",
                    contract.task_state
                )));
            }

            // Update task_state to 'completed'
            sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = ?")
                .bind(issue_id)
                .execute(&state.pool)
                .await?;

            // Sync issues.status_id to 'completed' category
            sync_issue_status(&state.pool, issue_id, TaskState::Completed, &now).await?;

            // If claimed_by exists: update agent_stats tasks_completed + 1
            if let Some(ref claimed_by) = contract.claimed_by {
                sqlx::query(
                    "UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = ?",
                )
                .bind(claimed_by)
                .execute(&state.pool)
                .await?;
            }

            // Auto-unblock downstream tasks
            let _ = crate::orchestration::dependency::resolve_downstream(&state.pool, issue_id).await;

            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn reject_task(state: State<AppState>, identifier: String) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
                    .bind(&identifier)
                    .fetch_one(&state.pool)
                    .await?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Verify task_state == "validating"
            if contract.task_state != "validating" {
                return Err(sqlx::Error::Protocol(format!(
                    "TASK_WRONG_STATE: expected 'validating', got '{}'",
                    contract.task_state
                )));
            }

            // Update: task_state='queued', clear claimed_by/claimed_at, increment attempt_count
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = ?",
            )
            .bind(issue_id)
            .execute(&state.pool)
            .await?;

            // Sync issues.status_id to 'unstarted' category
            sync_issue_status(&state.pool, issue_id, TaskState::Queued, &now).await?;

            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}
