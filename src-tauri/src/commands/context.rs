use crate::models::*;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorAttempt {
    pub agent_name: String,
    pub attempt_number: i64,
    pub result: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub issue: Issue,
    pub labels: Vec<Label>,
    pub parent_issue: Option<Issue>,
    pub sub_issues: Vec<Issue>,
    pub related_issues: Vec<IssueRelation>,
    pub blocking_issues: Vec<Issue>,
    pub blocked_issues: Vec<Issue>,
    pub comments: Vec<Comment>,
    pub activity_log: Vec<ActivityLogEntry>,
    pub prior_attempts: Vec<PriorAttempt>,
    pub similar_completed_issues: Vec<Issue>,
    pub project_path: Option<String>,
    pub context_files: Vec<String>,
}

#[tauri::command]
pub fn get_task_context(state: State<AppState>, identifier: String) -> Result<TaskContext, String> {
    state.rt.block_on(async {
        get_task_context_async(&state.pool, &identifier).await
    })
}

pub async fn get_task_context_async(
    pool: &sqlx::AnyPool,
    identifier: &str,
) -> Result<TaskContext, String> {
    // Get the issue
    let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
        .bind(identifier)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Issue not found: {}", e))?;

    // Get labels
    let labels = sqlx::query_as::<_, Label>(
        "SELECT l.* FROM labels l JOIN issue_labels il ON il.label_id = l.id WHERE il.issue_id = $1",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get parent issue
    let parent_issue = if let Some(pid) = issue.parent_id {
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(pid)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        None
    };

    // Get sub-issues
    let sub_issues = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE parent_id = $1")
        .bind(issue.id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Get related issues (all relations involving this issue)
    let related_issues = sqlx::query_as::<_, IssueRelation>(
        "SELECT * FROM issue_relations WHERE source_issue_id = $1 OR target_issue_id = $2",
    )
    .bind(issue.id)
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get blocking issues (what blocks this issue)
    let blocking_issues = sqlx::query_as::<_, Issue>(
        "SELECT i.* FROM issues i JOIN issue_relations r ON r.source_issue_id = i.id WHERE r.target_issue_id = $1 AND r.relation_type = 'blocks'",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get blocked issues (what this issue blocks)
    let blocked_issues = sqlx::query_as::<_, Issue>(
        "SELECT i.* FROM issues i JOIN issue_relations r ON r.target_issue_id = i.id WHERE r.source_issue_id = $1 AND r.relation_type = 'blocks'",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get comments
    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE issue_id = $1 ORDER BY created_at ASC",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get activity log
    let activity_log = sqlx::query_as::<_, ActivityLogEntry>(
        "SELECT * FROM activity_log WHERE issue_id = $1 ORDER BY timestamp DESC LIMIT 50",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Get prior attempts from execution logs
    let prior_attempts: Vec<PriorAttempt> = sqlx::query_as::<_, (String, i64, String, Option<String>)>(
        "SELECT el.agent_id, el.attempt_number, el.entry_type, el.message FROM execution_logs el WHERE el.issue_id = $1 AND el.entry_type IN ('complete', 'fail') ORDER BY el.attempt_number ASC",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(|(agent_id, attempt_number, entry_type, message)| PriorAttempt {
        agent_name: agent_id,
        attempt_number,
        result: entry_type,
        reason: message,
    })
    .collect();

    // Get similar completed issues (same labels, completed)
    let similar_completed_issues = get_similar_issues_async(pool, issue.project_id, issue.id, 5).await?;

    // Get project path
    let project_path: Option<String> =
        sqlx::query_scalar("SELECT path FROM projects WHERE id = $1")
            .bind(issue.project_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

    // Get context_files from task contract if exists
    let context_files: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT context FROM task_contracts WHERE issue_id = $1",
    )
    .bind(issue.id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .map(|ctx| {
        let v: serde_json::Value = serde_json::from_str(&ctx).unwrap_or_default();
        v.get("context_files")
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    })
    .unwrap_or_default();

    Ok(TaskContext {
        issue,
        labels,
        parent_issue,
        sub_issues,
        related_issues,
        blocking_issues,
        blocked_issues,
        comments,
        activity_log,
        prior_attempts,
        similar_completed_issues,
        project_path,
        context_files,
    })
}

#[tauri::command]
pub fn get_similar_issues(
    state: State<AppState>,
    project_id: i64,
    issue_id: i64,
    limit: i32,
) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        get_similar_issues_async(&state.pool, project_id, issue_id, limit).await
    })
}

pub async fn get_similar_issues_async(
    pool: &sqlx::AnyPool,
    project_id: i64,
    issue_id: i64,
    limit: i32,
) -> Result<Vec<Issue>, String> {
    // Find completed issues that share labels with the given issue
    let issues = sqlx::query_as::<_, Issue>(
        "SELECT DISTINCT i.* FROM issues i
         JOIN issue_labels il ON il.issue_id = i.id
         JOIN statuses s ON s.id = i.status_id
         WHERE i.project_id = $1
           AND i.id != $2
           AND s.category = 'completed'
           AND il.label_id IN (SELECT label_id FROM issue_labels WHERE issue_id = $3)
         ORDER BY i.updated_at DESC
         LIMIT $4",
    )
    .bind(project_id)
    .bind(issue_id)
    .bind(issue_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(issues)
}
