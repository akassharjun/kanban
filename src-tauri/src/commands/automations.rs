use crate::models::{AutomationRule, AutomationLogEntry};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateAutomationRuleInput {
    pub project_id: i64,
    pub name: String,
    pub trigger_type: String,
    pub trigger_config: Option<String>,
    pub conditions: Option<String>,
    pub actions: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAutomationRuleInput {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<String>,
    pub conditions: Option<String>,
    pub actions: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AutomationContext {
    pub issue_id: Option<i64>,
    pub project_id: i64,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub actor_name: Option<String>,
    pub agent_name: Option<String>,
    pub task_confidence: Option<f64>,
    // Extra fields from the issue for condition evaluation
    pub issue_title: Option<String>,
    pub issue_identifier: Option<String>,
    pub issue_priority: Option<String>,
    pub issue_status_id: Option<i64>,
    pub issue_assignee_id: Option<i64>,
}

#[tauri::command]
pub fn list_automation_rules(state: State<AppState>, project_id: i64) -> Result<Vec<AutomationRule>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AutomationRule>(
            "SELECT * FROM automation_rules WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_automation_rule(state: State<AppState>, input: CreateAutomationRuleInput) -> Result<AutomationRule, String> {
    if input.name.trim().is_empty() {
        return Err("name cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let trigger_config = input.trigger_config.unwrap_or_else(|| "{}".to_string());
        let conditions = input.conditions.unwrap_or_else(|| "[]".to_string());
        let actions = input.actions.unwrap_or_else(|| "[]".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO automation_rules (project_id, name, trigger_type, trigger_config, conditions, actions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&trigger_config)
        .bind(&conditions)
        .bind(&actions)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, AutomationRule>("SELECT * FROM automation_rules WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_automation_rule(state: State<AppState>, id: i64, input: UpdateAutomationRuleInput) -> Result<AutomationRule, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        let existing = sqlx::query_as::<_, AutomationRule>("SELECT * FROM automation_rules WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
        if existing.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Some(ref name) = input.name {
            sqlx::query("UPDATE automation_rules SET name = $1, updated_at = $2 WHERE id = $3")
                .bind(name).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref trigger_type) = input.trigger_type {
            sqlx::query("UPDATE automation_rules SET trigger_type = $1, updated_at = $2 WHERE id = $3")
                .bind(trigger_type).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref trigger_config) = input.trigger_config {
            sqlx::query("UPDATE automation_rules SET trigger_config = $1, updated_at = $2 WHERE id = $3")
                .bind(trigger_config).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref conditions) = input.conditions {
            sqlx::query("UPDATE automation_rules SET conditions = $1, updated_at = $2 WHERE id = $3")
                .bind(conditions).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref actions) = input.actions {
            sqlx::query("UPDATE automation_rules SET actions = $1, updated_at = $2 WHERE id = $3")
                .bind(actions).bind(&now).bind(id).execute(&state.pool).await?;
        }

        sqlx::query_as::<_, AutomationRule>("SELECT * FROM automation_rules WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_automation_rule(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM automation_rules WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn toggle_automation_rule(state: State<AppState>, id: i64, enabled: bool) -> Result<AutomationRule, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let result = sqlx::query("UPDATE automation_rules SET enabled = $1, updated_at = $2 WHERE id = $3")
            .bind(enabled as i32)
            .bind(&now)
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        sqlx::query_as::<_, AutomationRule>("SELECT * FROM automation_rules WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_automation_log(state: State<AppState>, project_id: i64, limit: Option<i64>) -> Result<Vec<AutomationLogEntry>, String> {
    let limit = limit.unwrap_or(50);
    state.rt.block_on(async {
        sqlx::query_as::<_, AutomationLogEntry>(
            "SELECT al.* FROM automation_log al JOIN automation_rules ar ON al.rule_id = ar.id WHERE ar.project_id = $1 ORDER BY al.executed_at DESC LIMIT $2"
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

/// Evaluate all matching automation rules for a given trigger event.
/// Called internally from issue commands after mutations.
pub async fn evaluate_rules(
    pool: &sqlx::AnyPool,
    project_id: i64,
    trigger_type: &str,
    context: &AutomationContext,
) -> Result<Vec<AutomationLogEntry>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AutomationRule>(
        "SELECT * FROM automation_rules WHERE project_id = $1 AND trigger_type = $2 AND enabled = 1"
    )
    .bind(project_id)
    .bind(trigger_type)
    .fetch_all(pool)
    .await?;

    let mut log_entries = Vec::new();

    for rule in &rules {
        // Evaluate conditions
        if !evaluate_conditions(&rule.conditions, context) {
            continue;
        }

        // Execute actions
        let actions: Vec<Value> = serde_json::from_str(&rule.actions).unwrap_or_default();
        let mut executed_actions: Vec<Value> = Vec::new();
        let mut success = true;
        let mut error_message: Option<String> = None;

        for action in &actions {
            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let config = action.get("config").cloned().unwrap_or(Value::Object(Default::default()));

            match execute_action(pool, action_type, &config, context).await {
                Ok(()) => {
                    executed_actions.push(serde_json::json!({
                        "type": action_type,
                        "config": config,
                        "success": true
                    }));
                }
                Err(e) => {
                    success = false;
                    error_message = Some(e.to_string());
                    executed_actions.push(serde_json::json!({
                        "type": action_type,
                        "config": config,
                        "success": false,
                        "error": e.to_string()
                    }));
                    break; // Stop on first failure
                }
            }
        }

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let actions_json = serde_json::to_string(&executed_actions).unwrap_or_else(|_| "[]".to_string());

        // Log execution
        let log_id: i64 = sqlx::query_scalar(
            "INSERT INTO automation_log (rule_id, issue_id, trigger_type, actions_executed, success, error_message, executed_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(rule.id)
        .bind(context.issue_id)
        .bind(trigger_type)
        .bind(&actions_json)
        .bind(success as i32)
        .bind(&error_message)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        // Update rule execution stats
        sqlx::query(
            "UPDATE automation_rules SET execution_count = execution_count + 1, last_executed_at = $1, updated_at = $1 WHERE id = $2"
        )
        .bind(&now)
        .bind(rule.id)
        .execute(pool)
        .await?;

        let entry = sqlx::query_as::<_, AutomationLogEntry>("SELECT * FROM automation_log WHERE id = $1")
            .bind(log_id)
            .fetch_one(pool)
            .await?;
        log_entries.push(entry);
    }

    Ok(log_entries)
}

/// Evaluate conditions against context. Returns true if all conditions pass.
fn evaluate_conditions(conditions_json: &str, context: &AutomationContext) -> bool {
    let conditions: Vec<Value> = match serde_json::from_str(conditions_json) {
        Ok(c) => c,
        Err(_) => return true, // No valid conditions = pass
    };

    if conditions.is_empty() {
        return true;
    }

    for condition in &conditions {
        let field = condition.get("field").and_then(|v| v.as_str()).unwrap_or("");
        let operator = condition.get("operator").and_then(|v| v.as_str()).unwrap_or("equals");
        let value = condition.get("value").and_then(|v| v.as_str()).unwrap_or("");

        let actual_value = match field {
            "priority" => context.issue_priority.clone().unwrap_or_default(),
            "status_id" => context.issue_status_id.map(|v| v.to_string()).unwrap_or_default(),
            "assignee_id" => context.issue_assignee_id.map(|v| v.to_string()).unwrap_or_default(),
            "title" => context.issue_title.clone().unwrap_or_default(),
            "old_value" => context.old_value.clone().unwrap_or_default(),
            "new_value" => context.new_value.clone().unwrap_or_default(),
            _ => String::new(),
        };

        let matches = match operator {
            "equals" => actual_value == value,
            "not_equals" => actual_value != value,
            "contains" => actual_value.contains(value),
            "not_contains" => !actual_value.contains(value),
            "greater_than" => {
                actual_value.parse::<f64>().unwrap_or(0.0) > value.parse::<f64>().unwrap_or(0.0)
            }
            "less_than" => {
                actual_value.parse::<f64>().unwrap_or(0.0) < value.parse::<f64>().unwrap_or(0.0)
            }
            "is_empty" => actual_value.is_empty(),
            "is_not_empty" => !actual_value.is_empty(),
            _ => true,
        };

        if !matches {
            return false;
        }
    }

    true
}

/// Execute a single automation action.
async fn execute_action(
    pool: &sqlx::AnyPool,
    action_type: &str,
    config: &Value,
    context: &AutomationContext,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    match action_type {
        "change_status" => {
            let status_id = config.get("status_id").and_then(|v| v.as_i64())
                .ok_or_else(|| sqlx::Error::Protocol("missing status_id in change_status action".to_string()))?;
            if let Some(issue_id) = context.issue_id {
                sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(status_id).bind(&now).bind(issue_id)
                    .execute(pool).await?;
                // Log activity
                sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)")
                    .bind(issue_id).bind("status_id")
                    .bind(context.issue_status_id.map(|v| v.to_string()))
                    .bind(Some(status_id.to_string()))
                    .bind(&now)
                    .execute(pool).await?;
            }
        }
        "set_priority" => {
            let priority = config.get("priority").and_then(|v| v.as_str()).unwrap_or("none");
            if let Some(issue_id) = context.issue_id {
                sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                    .bind(priority).bind(&now).bind(issue_id)
                    .execute(pool).await?;
                sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)")
                    .bind(issue_id).bind("priority")
                    .bind(&context.issue_priority)
                    .bind(Some(priority))
                    .bind(&now)
                    .execute(pool).await?;
            }
        }
        "assign_to" => {
            if let Some(issue_id) = context.issue_id {
                let member_id = config.get("member_id").and_then(|v| v.as_i64());
                if let Some(mid) = member_id {
                    sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                        .bind(mid).bind(&now).bind(issue_id)
                        .execute(pool).await?;
                    sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)")
                        .bind(issue_id).bind("assignee_id")
                        .bind(context.issue_assignee_id.map(|v| v.to_string()))
                        .bind(Some(mid.to_string()))
                        .bind(&now)
                        .execute(pool).await?;
                }
            }
        }
        "add_label" => {
            let label_id = config.get("label_id").and_then(|v| v.as_i64());
            if let (Some(issue_id), Some(lid)) = (context.issue_id, label_id) {
                // Ignore duplicate constraint
                let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                    .bind(issue_id).bind(lid)
                    .execute(pool).await;
            }
        }
        "create_issue" => {
            let title_template = config.get("title_template").and_then(|v| v.as_str()).unwrap_or("Auto-created issue");
            let status_id = config.get("status_id").and_then(|v| v.as_i64());
            let priority = config.get("priority").and_then(|v| v.as_str()).unwrap_or("none");

            let title = render_template(title_template, context);

            // Get status_id - use provided or first status in project
            let sid = if let Some(s) = status_id {
                s
            } else {
                sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = $1 ORDER BY position LIMIT 1")
                    .bind(context.project_id)
                    .fetch_one(pool)
                    .await?
            };

            // Increment counter
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            ).bind(context.project_id).fetch_one(pool).await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar("SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2")
                .bind(context.project_id).bind(sid)
                .fetch_one(pool).await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            sqlx::query(
                "INSERT INTO issues (project_id, identifier, title, status_id, priority, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            )
            .bind(context.project_id)
            .bind(&identifier)
            .bind(&title)
            .bind(sid)
            .bind(priority)
            .bind(context.issue_id) // parent_id = triggering issue
            .bind(position)
            .bind(&now)
            .bind(&now)
            .execute(pool).await?;
        }
        "add_comment" => {
            let content_template = config.get("content_template").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(issue_id) = context.issue_id {
                let content = render_template(content_template, context);
                sqlx::query("INSERT INTO comments (issue_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4)")
                    .bind(issue_id).bind(&content).bind(&now).bind(&now)
                    .execute(pool).await?;
            }
        }
        "send_notification" => {
            let message_template = config.get("message_template").and_then(|v| v.as_str()).unwrap_or("");
            let message = render_template(message_template, context);
            sqlx::query("INSERT INTO notifications (type, issue_id, message, created_at) VALUES ($1, $2, $3, $4)")
                .bind("automation")
                .bind(context.issue_id)
                .bind(&message)
                .bind(&now)
                .execute(pool).await?;
        }
        "create_task_contract" => {
            // Only if there's an issue
            if let Some(issue_id) = context.issue_id {
                let task_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("implementation");
                let complexity = config.get("complexity").and_then(|v| v.as_str()).unwrap_or("medium");
                let skills: Vec<String> = config.get("skills")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let skills_json = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());

                let objective = context.issue_title.clone().unwrap_or_default();
                let _ = sqlx::query(
                    "INSERT INTO task_contracts (issue_id, type, objective, required_skills, estimated_complexity) VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(issue_id)
                .bind(task_type)
                .bind(&objective)
                .bind(&skills_json)
                .bind(complexity)
                .execute(pool).await;
            }
        }
        "trigger_webhook" => {
            // Webhook execution is logged but not actually performed in the desktop app.
            // The MCP server or CLI should handle external HTTP calls.
            // We just log it as a no-op success.
        }
        _ => {
            // Unknown action type - skip
        }
    }

    Ok(())
}

/// Replace {{variable}} templates with actual values from context.
fn render_template(template: &str, context: &AutomationContext) -> String {
    let mut result = template.to_string();
    result = result.replace("{{issue.title}}", &context.issue_title.clone().unwrap_or_default());
    result = result.replace("{{issue.identifier}}", &context.issue_identifier.clone().unwrap_or_default());
    result = result.replace("{{issue.priority}}", &context.issue_priority.clone().unwrap_or_default());
    result = result.replace("{{actor.name}}", &context.actor_name.clone().unwrap_or_else(|| "System".to_string()));
    result = result.replace("{{old_value}}", &context.old_value.clone().unwrap_or_default());
    result = result.replace("{{new_value}}", &context.new_value.clone().unwrap_or_default());
    result = result.replace("{{agent.name}}", &context.agent_name.clone().unwrap_or_default());
    result = result.replace("{{task.confidence}}", &context.task_confidence.map(|v| format!("{:.2}", v)).unwrap_or_default());
    result
}
