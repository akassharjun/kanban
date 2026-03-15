use kanban_lib::models::*;
use kanban_lib::orchestration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::io::{self, BufRead, Write};
use uuid::Uuid;

fn notify_change() {
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        if let Ok(client) = redis::Client::open(redis_url) {
            if let Ok(mut conn) = client.get_connection() {
                let _: Result<(), _> = redis::cmd("PUBLISH").arg("kanban:db-changed").arg("1").query(&mut conn);
            }
        }
    }
}

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn success(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(result),
        error: None,
    }
}

fn error(id: Value, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(json!({ "code": code, "message": message })),
    }
}

fn tool_def(name: &str, description: &str, properties: Value, required: Vec<&str>) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    })
}

fn prop(type_str: &str, description: &str) -> Value {
    json!({ "type": type_str, "description": description })
}

fn tools_list() -> Vec<Value> {
    vec![
        tool_def("list_projects", "List all projects", json!({}), vec![]),
        tool_def(
            "create_issue",
            "Create a new issue",
            json!({
                "project_id": prop("number", "Project ID"),
                "title": prop("string", "Issue title"),
                "status_id": prop("number", "Status ID"),
                "priority": prop("string", "Priority: none, urgent, high, medium, low"),
                "description": prop("string", "Issue description (Markdown)"),
                "assignee_id": prop("number", "Assignee member ID"),
                "parent_id": prop("number", "Parent issue ID"),
                "template": prop("string", "Template name to pre-fill fields"),
            }),
            vec!["project_id", "title", "status_id"],
        ),
        tool_def(
            "update_issue",
            "Update an existing issue",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
                "title": prop("string", "New title"),
                "description": prop("string", "New description"),
                "status_id": prop("number", "New status ID"),
                "priority": prop("string", "New priority"),
                "assignee_id": prop("number", "New assignee ID (0 to unassign)"),
            }),
            vec!["identifier"],
        ),
        tool_def(
            "get_issue",
            "Get issue details by identifier",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
            }),
            vec!["identifier"],
        ),
        tool_def(
            "list_issues",
            "List issues with optional filters",
            json!({
                "project_id": prop("number", "Project ID"),
                "status_id": prop("number", "Filter by status ID"),
                "priority": prop("string", "Filter by priority"),
                "assignee_id": prop("number", "Filter by assignee ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "search_issues",
            "Search issues by text",
            json!({
                "project_id": prop("number", "Project ID"),
                "query": prop("string", "Search query"),
            }),
            vec!["project_id", "query"],
        ),
        tool_def(
            "move_issue",
            "Change issue status or parent",
            json!({
                "identifier": prop("string", "Issue identifier"),
                "status_id": prop("number", "New status ID"),
                "parent_identifier": prop("string", "New parent issue identifier"),
            }),
            vec!["identifier"],
        ),
        tool_def(
            "bulk_update",
            "Update multiple issues at once",
            json!({
                "identifiers": { "type": "array", "items": { "type": "string" }, "description": "Issue identifiers" },
                "status_id": prop("number", "New status ID"),
                "priority": prop("string", "New priority"),
                "assignee_id": prop("number", "New assignee ID"),
            }),
            vec!["identifiers"],
        ),
        tool_def(
            "get_board",
            "Get kanban board state for a project",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "create_label",
            "Create a project label",
            json!({
                "project_id": prop("number", "Project ID"),
                "name": prop("string", "Label name"),
                "color": prop("string", "Label color hex"),
            }),
            vec!["project_id", "name", "color"],
        ),
        tool_def(
            "add_blocker",
            "Mark an issue as blocked by another",
            json!({
                "identifier": prop("string", "Issue that is blocked"),
                "blocker_identifier": prop("string", "Issue that blocks"),
            }),
            vec!["identifier", "blocker_identifier"],
        ),
        tool_def(
            "list_members",
            "List workspace members",
            json!({}),
            vec![],
        ),
        tool_def(
            "add_member",
            "Add a team member",
            json!({
                "name": prop("string", "Member name"),
                "email": prop("string", "Email address"),
                "display_name": prop("string", "Display name"),
            }),
            vec!["name"],
        ),
        tool_def("list_comments", "List comments on an issue", json!({
            "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
        }), vec!["identifier"]),
        tool_def("add_comment", "Add a comment to an issue", json!({
            "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
            "content": prop("string", "Comment content (Markdown)"),
            "member_id": prop("number", "Member ID of the commenter"),
        }), vec!["identifier", "content"]),
        // Agent lifecycle
        tool_def("register_agent", "Register a new AI agent", json!({
            "name": prop("string", "Agent name (unique)"),
            "skills": {"type": "array", "items": {"type": "string"}, "description": "Skills the agent has"},
            "task_types": {"type": "array", "items": {"type": "string"}, "description": "Task types agent can handle"},
            "max_concurrent": prop("number", "Max concurrent tasks (default: 1)"),
            "max_complexity": prop("string", "Max complexity: small, medium, large (default: large)")
        }), vec!["name", "skills"]),
        tool_def("agent_heartbeat", "Send agent heartbeat", json!({
            "agent_id": prop("string", "Agent ID")
        }), vec!["agent_id"]),
        tool_def("deregister_agent", "Deregister an agent and reclaim its tasks", json!({
            "agent_id": prop("string", "Agent ID")
        }), vec!["agent_id"]),
        // Task work loop
        tool_def("next_task", "Get next available task and atomically claim it", json!({
            "agent_id": prop("string", "Agent ID"),
            "skills_override": {"type": "array", "items": {"type": "string"}, "description": "Override skills for this query"}
        }), vec!["agent_id"]),
        tool_def("start_task", "Mark a claimed task as executing", json!({
            "agent_id": prop("string", "Agent ID"),
            "identifier": prop("string", "Task identifier (e.g. KAN-42)")
        }), vec!["agent_id", "identifier"]),
        tool_def("complete_task", "Complete a task with confidence score and summary", json!({
            "agent_id": prop("string", "Agent ID"),
            "identifier": prop("string", "Task identifier"),
            "confidence": prop("number", "Confidence score 0.0-1.0"),
            "summary": prop("string", "Completion summary"),
            "artifacts": {"type": "object", "description": "Artifacts produced (branches, PRs, etc.)"}
        }), vec!["agent_id", "identifier", "confidence", "summary"]),
        tool_def("fail_task", "Report task failure with reason", json!({
            "agent_id": prop("string", "Agent ID"),
            "identifier": prop("string", "Task identifier"),
            "reason": prop("string", "Failure reason")
        }), vec!["agent_id", "identifier", "reason"]),
        tool_def("unclaim_task", "Voluntarily release a claimed task", json!({
            "agent_id": prop("string", "Agent ID"),
            "identifier": prop("string", "Task identifier")
        }), vec!["agent_id", "identifier"]),
        tool_def("approve_task", "Human approves a task in validating state", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
        tool_def("reject_task", "Human rejects a task in validating state", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
        // Execution logging
        tool_def("log_task_activity", "Log execution activity for a task", json!({
            "identifier": prop("string", "Task identifier"),
            "agent_id": prop("string", "Agent ID"),
            "entry_type": prop("string", "Log type: reasoning, file_read, file_edit, command, discovery, error, checkpoint"),
            "message": prop("string", "Log message"),
            "metadata": {"type": "object", "description": "Optional metadata"}
        }), vec!["identifier", "agent_id", "entry_type", "message"]),
        // Task management
        tool_def("create_task", "Create a new task contract with structured fields", json!({
            "project_id": prop("number", "Project ID"),
            "title": prop("string", "Task title"),
            "objective": prop("string", "Task objective"),
            "status_id": prop("number", "Initial status ID"),
            "type": prop("string", "Task type: implementation, research, testing, review, decomposition"),
            "priority": prop("string", "Priority: urgent, high, medium, low"),
            "skills": {"type": "array", "items": {"type": "string"}, "description": "Required skills"},
            "complexity": prop("string", "Estimated complexity: small, medium, large"),
            "description": prop("string", "Full description (Markdown)"),
            "depends_on": {"type": "array", "items": {"type": "string"}, "description": "Identifiers this task depends on"},
            "context_files": {"type": "array", "items": {"type": "string"}, "description": "Relevant files"},
            "parent_identifier": prop("string", "Parent task identifier"),
            "timeout_minutes": prop("number", "Timeout in minutes (default: 30)")
        }), vec!["project_id", "title", "objective", "status_id"]),
        tool_def("get_task", "Get full task contract by identifier", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
        tool_def("task_replay", "Get execution replay timeline for a task", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
        tool_def("task_attempts", "Get prior attempt history for a task", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
        tool_def("agent_stats", "Get agent performance statistics", json!({
            "agent_id": prop("string", "Agent ID")
        }), vec!["agent_id"]),
        tool_def("list_agents", "List all registered agents", json!({}), vec![]),
        tool_def("system_metrics", "Get system-wide metrics for a project", json!({
            "project_id": prop("number", "Project ID")
        }), vec!["project_id"]),
        tool_def("invalidate_task", "Invalidate a completed task and cascade effects to downstream tasks", json!({
            "identifier": prop("string", "Task identifier"),
            "reason": prop("string", "Reason for invalidation")
        }), vec!["identifier", "reason"]),
        tool_def("task_graph", "Get dependency graph for a task (nodes and edges)", json!({
            "identifier": prop("string", "Task identifier")
        }), vec!["identifier"]),
    ]
}

fn resources_list() -> Vec<Value> {
    vec![
        json!({
            "uri": "kanban://projects",
            "name": "All Projects",
            "description": "List of all projects",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "kanban://project/{id}/board",
            "name": "Project Board",
            "description": "Board state for a project",
            "mimeType": "application/json",
        }),
        json!({
            "uri": "kanban://issue/{identifier}",
            "name": "Issue Details",
            "description": "Full issue details",
            "mimeType": "application/json",
        }),
    ]
}

async fn handle_tool_call(
    pool: &PgPool,
    name: &str,
    args: &Value,
) -> Result<Value, String> {
    match name {
        "list_projects" => {
            let projects =
                sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY name")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(projects))
        }
        "create_issue" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?;
            let status_id = args["status_id"].as_i64().ok_or("status_id required")?;
            let priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("none");
            let description = args.get("description").and_then(|v| v.as_str());
            let assignee_id = args.get("assignee_id").and_then(|v| v.as_i64());
            let parent_id = args.get("parent_id").and_then(|v| v.as_i64());

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

            // Atomically increment counter and get new value + prefix
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            )
            .bind(project_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
            )
            .bind(project_id)
            .bind(status_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
            )
            .bind(project_id)
            .bind(&identifier)
            .bind(title)
            .bind(description)
            .bind(status_id)
            .bind(priority)
            .bind(assignee_id)
            .bind(parent_id)
            .bind(position)
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                    .bind(issue_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(issue))
        }
        "update_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = $1",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(t) = args.get("title").and_then(|v| v.as_str()) {
                sqlx::query(
                    "UPDATE issues SET title = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(t)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            if let Some(d) = args.get("description").and_then(|v| v.as_str()) {
                sqlx::query(
                    "UPDATE issues SET description = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(d)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            if let Some(s) = args.get("status_id").and_then(|v| v.as_i64()) {
                sqlx::query(
                    "UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(s)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            if let Some(p) = args.get("priority").and_then(|v| v.as_str()) {
                sqlx::query(
                    "UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(p)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            if let Some(a) = args.get("assignee_id").and_then(|v| v.as_i64()) {
                if a == 0 {
                    sqlx::query(
                        "UPDATE issues SET assignee_id = NULL, updated_at = $1 WHERE id = $2",
                    )
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                } else {
                    sqlx::query(
                        "UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3",
                    )
                    .bind(a)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }

            let updated =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                    .bind(issue.id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(updated))
        }
        "get_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = $1",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let labels = sqlx::query_as::<_, Label>(
                "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let sub_issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE parent_id = $1",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let relations = sqlx::query_as::<_, IssueRelation>(
                "SELECT * FROM issue_relations WHERE source_issue_id = $1 OR target_issue_id = $2",
            )
            .bind(issue.id)
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!({
                "issue": issue,
                "labels": labels,
                "sub_issues": sub_issues,
                "relations": relations,
            }))
        }
        "list_issues" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let status_id = args.get("status_id").and_then(|v| v.as_i64());
            let priority = args.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string());
            let assignee_id = args.get("assignee_id").and_then(|v| v.as_i64());

            let mut param_idx = 1;
            let mut query = format!("SELECT * FROM issues WHERE project_id = ${}", param_idx);
            param_idx += 1;
            if status_id.is_some() { query.push_str(&format!(" AND status_id = ${}", param_idx)); param_idx += 1; }
            if priority.is_some() { query.push_str(&format!(" AND priority = ${}", param_idx)); param_idx += 1; }
            if assignee_id.is_some() { query.push_str(&format!(" AND assignee_id = ${}", param_idx)); }
            query.push_str(" ORDER BY position");

            let mut q = sqlx::query_as::<_, Issue>(&query).bind(project_id);
            if let Some(s) = status_id { q = q.bind(s); }
            if let Some(ref p) = priority { q = q.bind(p); }
            if let Some(a) = assignee_id { q = q.bind(a); }
            let issues = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(issues))
        }
        "search_issues" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let query = args["query"].as_str().ok_or("query required")?;
            let pattern = format!("%{}%", query);
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND (title LIKE $2 OR description LIKE $3 OR identifier LIKE $4) ORDER BY updated_at DESC",
            )
            .bind(project_id)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(issues))
        }
        "move_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = $1",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(status_id) = args.get("status_id").and_then(|v| v.as_i64()) {
                sqlx::query(
                    "UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(status_id)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            if let Some(parent_ident) =
                args.get("parent_identifier").and_then(|v| v.as_str())
            {
                let parent = sqlx::query_as::<_, Issue>(
                    "SELECT * FROM issues WHERE identifier = $1",
                )
                .bind(parent_ident)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE issues SET parent_id = $1, updated_at = $2 WHERE id = $3",
                )
                .bind(parent.id)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }

            let updated =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                    .bind(issue.id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(updated))
        }
        "bulk_update" => {
            let identifiers =
                args["identifiers"].as_array().ok_or("identifiers required")?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let mut updated_issues = vec![];
            for ident_val in identifiers {
                let ident =
                    ident_val.as_str().ok_or("identifier must be string")?;
                let issue = sqlx::query_as::<_, Issue>(
                    "SELECT * FROM issues WHERE identifier = $1",
                )
                .bind(ident)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                if let Some(s) = args.get("status_id").and_then(|v| v.as_i64()) {
                    sqlx::query(
                        "UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3",
                    )
                    .bind(s)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                if let Some(p) = args.get("priority").and_then(|v| v.as_str()) {
                    sqlx::query(
                        "UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3",
                    )
                    .bind(p)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                if let Some(a) = args.get("assignee_id").and_then(|v| v.as_i64()) {
                    sqlx::query(
                        "UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3",
                    )
                    .bind(a)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                let u = sqlx::query_as::<_, Issue>(
                    "SELECT * FROM issues WHERE id = $1",
                )
                .bind(issue.id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                updated_issues.push(u);
            }
            notify_change();
            Ok(json!(updated_issues))
        }
        "get_board" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let statuses = sqlx::query_as::<_, Status>(
                "SELECT * FROM statuses WHERE project_id = $1 ORDER BY position",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND parent_id IS NULL ORDER BY position",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let mut columns = vec![];
            for status in &statuses {
                let column_issues: Vec<&Issue> =
                    issues.iter().filter(|i| i.status_id == status.id).collect();
                columns.push(json!({
                    "status": status,
                    "issues": column_issues,
                    "count": column_issues.len(),
                }));
            }
            Ok(json!({ "columns": columns }))
        }
        "create_label" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let name = args["name"].as_str().ok_or("name required")?;
            let color = args["color"].as_str().ok_or("color required")?;
            let label_id: i64 = sqlx::query_scalar(
                "INSERT INTO labels (project_id, name, color) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(project_id)
            .bind(name)
            .bind(color)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let label =
                sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
                    .bind(label_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(label))
        }
        "add_blocker" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let blocker = args["blocker_identifier"]
                .as_str()
                .ok_or("blocker_identifier required")?;
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = $1",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let blocker_issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = $1",
            )
            .bind(blocker)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocked_by')",
            )
            .bind(issue.id)
            .bind(blocker_issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({
                "status": "ok",
                "message": format!("{} is now blocked by {}", identifier, blocker),
            }))
        }
        "list_members" => {
            let members =
                sqlx::query_as::<_, Member>("SELECT * FROM members ORDER BY name")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(members))
        }
        "add_member" => {
            let name = args["name"].as_str().ok_or("name required")?;
            let email = args.get("email").and_then(|v| v.as_str());
            let display_name = args.get("display_name").and_then(|v| v.as_str());
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let member_id: i64 = sqlx::query_scalar(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, '#6366f1', $4) RETURNING id",
            )
            .bind(name)
            .bind(display_name)
            .bind(email)
            .bind(&now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let member =
                sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1")
                    .bind(member_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(member))
        }
        "list_comments" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let comments = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE issue_id = $1 ORDER BY created_at ASC")
                .bind(issue.id).fetch_all(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(comments))
        }
        "add_comment" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let content = args["content"].as_str().ok_or("content required")?;
            let member_id = args.get("member_id").and_then(|v| v.as_i64());
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let comment_id: i64 = sqlx::query_scalar("INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id")
                .bind(issue.id).bind(member_id).bind(content).bind(&now).bind(&now)
                .fetch_one(pool).await.map_err(|e| e.to_string())?;
            let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
                .bind(comment_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(comment))
        }
        // ── Agent lifecycle ──────────────────────────────────────────
        "register_agent" => {
            let name = args["name"].as_str().ok_or("name required")?;
            let skills = args.get("skills").and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let task_types = args.get("task_types").and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let max_concurrent = args.get("max_concurrent").and_then(|v| v.as_i64()).unwrap_or(1);
            let max_complexity = args.get("max_complexity").and_then(|v| v.as_str()).unwrap_or("large");
            let agent_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT INTO agents (id, name, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat) VALUES ($1, $2, $3::jsonb, $4::jsonb, $5, $6, 'idle', $7, $8)"
            )
            .bind(&agent_id).bind(name).bind(&skills).bind(&task_types)
            .bind(max_concurrent).bind(max_complexity).bind(&now).bind(&now)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

            sqlx::query(
                "INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES ($1, 0, 0, 0.0, 0, '{}'::jsonb)"
            )
            .bind(&agent_id)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"agent_id": agent_id, "name": name}))
        }
        "agent_heartbeat" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let now = chrono::Utc::now().to_rfc3339();

            // Check if agent has active tasks
            let active: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')"
            ).bind(agent_id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let status = if active.0 > 0 { "working" } else { "idle" };
            sqlx::query("UPDATE agents SET last_heartbeat = $1, status = $2 WHERE id = $3")
                .bind(&now).bind(status).bind(agent_id)
                .execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": status, "active_tasks": active.0}))
        }
        "deregister_agent" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let now = chrono::Utc::now().to_rfc3339();

            // Reclaim any active tasks
            let reclaimed = sqlx::query(
                "UPDATE task_contracts SET claimed_by = NULL, claimed_at = NULL, task_state = 'queued' WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')"
            ).bind(agent_id).execute(pool).await.map_err(|e| e.to_string())?;

            // Sync statuses for reclaimed tasks
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = issues.project_id AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $1
                WHERE id IN (SELECT issue_id FROM task_contracts WHERE claimed_by IS NULL AND task_state = 'queued')
                  AND status_id IN (SELECT s2.id FROM statuses s2 WHERE s2.category = 'started')"
            ).bind(&now).execute(pool).await.map_err(|e| e.to_string())?;

            sqlx::query("DELETE FROM agents WHERE id = $1")
                .bind(agent_id).execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "tasks_reclaimed": reclaimed.rows_affected()}))
        }
        // ── Task work loop ───────────────────────────────────────────
        "next_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;

            // Lazy timeout recovery
            let _ = kanban_lib::orchestration::timeout::reclaim_timed_out_tasks(&pool).await;

            let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
                .bind(agent_id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let skills: Vec<String> = if let Some(overrides) = args.get("skills_override").and_then(|v| v.as_array()) {
                overrides.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            } else {
                serde_json::from_value(agent.skills.clone()).unwrap_or_default()
            };

            let contract = orchestration::routing::next_task(
                pool,
                agent_id,
                &skills,
                &agent.max_complexity,
                agent.max_concurrent,
            ).await.map_err(|e| e.to_string())?;

            match contract {
                Some(c) => Ok(json!(c)),
                None => Ok(json!(null)),
            }
        }
        "start_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent_id));
            }
            if contract.task_state != "claimed" {
                return Err(format!("Task {} is in state '{}', expected 'claimed'", identifier, contract.task_state));
            }

            sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = $1")
                .bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;

            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'state_change', 'Task started executing', $4)"
            ).bind(issue.id).bind(agent_id).bind(contract.attempt_count).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "executing"}))
        }
        "complete_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let confidence = args["confidence"].as_f64().ok_or("confidence required")?;
            let summary = args["summary"].as_str().ok_or("summary required")?;
            let artifacts = args.get("artifacts").cloned().unwrap_or(json!({}));
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent_id));
            }
            if contract.task_state != "executing" {
                return Err(format!("Task {} is in state '{}', expected 'executing'", identifier, contract.task_state));
            }

            // Get project config for thresholds
            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = $1"
            ).bind(issue.project_id).fetch_optional(pool).await.map_err(|e| e.to_string())?;

            let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.9);
            let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.7);

            let result_json = serde_json::to_string(&json!({
                "confidence": confidence,
                "summary": summary,
                "artifacts": artifacts
            })).unwrap_or_default();

            let new_state = if confidence >= auto_accept {
                "completed"
            } else if confidence >= human_review {
                "validating"
            } else {
                "validating"
            };

            sqlx::query("UPDATE task_contracts SET task_state = $1, result = $2::jsonb WHERE issue_id = $3")
                .bind(new_state).bind(&result_json).bind(issue.id)
                .execute(pool).await.map_err(|e| e.to_string())?;

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state).unwrap()
            );
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = $2 ORDER BY s.position ASC LIMIT 1
                ), updated_at = $3 WHERE id = $4"
            ).bind(issue.project_id).bind(category).bind(&now).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Log completion
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'state_change', $4, $5)"
            ).bind(issue.id).bind(agent_id).bind(contract.attempt_count)
            .bind(format!("Task completed with confidence {:.2}: {}", confidence, summary))
            .bind(&now).execute(pool).await.map_err(|e| e.to_string())?;

            // Update agent stats
            sqlx::query(
                "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + $1 WHERE agent_id = $2"
            ).bind(confidence).bind(agent_id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Auto-unblock downstream tasks when completed
            if new_state == "completed" {
                let _ = orchestration::dependency::resolve_downstream(pool, issue.id).await;
            }

            notify_change();
            Ok(json!({"status": "ok", "task_state": new_state, "confidence": confidence}))
        }
        "fail_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let reason = args["reason"].as_str().ok_or("reason required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent_id));
            }

            // Append to prior_attempts in context
            let mut context: Value = contract.context.clone();
            let attempt_record = json!({
                "attempt": contract.attempt_count,
                "agent_id": agent_id,
                "reason": reason,
                "timestamp": now
            });
            if let Some(arr) = context.get_mut("prior_attempts").and_then(|v| v.as_array_mut()) {
                arr.push(attempt_record);
            } else {
                context["prior_attempts"] = json!([attempt_record]);
            }

            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = $1"
            ).bind(issue.project_id).fetch_optional(pool).await.map_err(|e| e.to_string())?;
            let max_attempts = config.as_ref().map(|c| c.max_attempts).unwrap_or(3);

            let new_attempt = contract.attempt_count + 1;
            let new_state = if new_attempt >= max_attempts { "blocked" } else { "queued" };

            sqlx::query(
                "UPDATE task_contracts SET task_state = $1, claimed_by = NULL, claimed_at = NULL, attempt_count = $2, context = $3::jsonb WHERE issue_id = $4"
            ).bind(new_state).bind(new_attempt).bind(serde_json::to_string(&context).unwrap_or_default()).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state).unwrap()
            );
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = $2 ORDER BY s.position ASC LIMIT 1
                ), updated_at = $3 WHERE id = $4"
            ).bind(issue.project_id).bind(category).bind(&now).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Log failure
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'error', $4, $5)"
            ).bind(issue.id).bind(agent_id).bind(contract.attempt_count)
            .bind(format!("Task failed: {}", reason)).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Update agent stats
            sqlx::query(
                "UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = $1"
            ).bind(agent_id).execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "task_state": new_state, "attempt_count": new_attempt}))
        }
        "unclaim_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent_id));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1"
            ).bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;

            // Sync issue status to unstarted
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3"
            ).bind(issue.project_id).bind(&now).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Log unclaim
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'state_change', 'Task unclaimed by agent', $4)"
            ).bind(issue.id).bind(agent_id).bind(contract.attempt_count).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "queued"}))
        }
        "approve_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, contract.task_state));
            }

            sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = $1")
                .bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;

            // Sync issue status to completed
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'completed' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3"
            ).bind(issue.project_id).bind(&now).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            // Update agent stats if there was a claiming agent
            if let Some(ref agent_id) = contract.claimed_by {
                sqlx::query(
                    "UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = $1"
                ).bind(agent_id).execute(pool).await.map_err(|e| e.to_string())?;
            }

            // Auto-unblock downstream tasks
            let _ = orchestration::dependency::resolve_downstream(pool, issue.id).await;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "completed"}))
        }
        "reject_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            if contract.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, contract.task_state));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, result = NULL WHERE issue_id = $1"
            ).bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;

            // Sync issue status to unstarted
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3"
            ).bind(issue.project_id).bind(&now).bind(issue.id)
            .execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "queued"}))
        }
        // ── Execution logging ────────────────────────────────────────
        "log_task_activity" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let entry_type = args["entry_type"].as_str().ok_or("entry_type required")?;
            let message = args["message"].as_str().ok_or("message required")?;
            let metadata = args.get("metadata").map(|v| serde_json::to_string(v).unwrap_or_default());
            let now = chrono::Utc::now().to_rfc3339();

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let attempt_count: (i64,) = sqlx::query_as(
                "SELECT attempt_count FROM task_contracts WHERE issue_id = $1"
            ).bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)"
            ).bind(issue.id).bind(agent_id).bind(attempt_count.0).bind(entry_type).bind(message).bind(metadata).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok"}))
        }
        // ── Task management ──────────────────────────────────────────
        "create_task" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?;
            let objective = args["objective"].as_str().ok_or("objective required")?;
            let status_id = args["status_id"].as_i64().ok_or("status_id required")?;
            let task_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("implementation");
            let priority = args.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");
            let skills = args.get("skills").and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let complexity = args.get("complexity").and_then(|v| v.as_str());
            let description = args.get("description").and_then(|v| v.as_str());
            let depends_on = args.get("depends_on").and_then(|v| v.as_array());
            let context_files = args.get("context_files").and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let parent_identifier = args.get("parent_identifier").and_then(|v| v.as_str());
            let timeout_minutes = args.get("timeout_minutes").and_then(|v| v.as_i64()).unwrap_or(30);
            let now = chrono::Utc::now().to_rfc3339();

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

            // Create the issue first
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            ).bind(project_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
            ).bind(project_id).bind(status_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            // Resolve parent
            let parent_id = if let Some(pi) = parent_identifier {
                let parent = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(pi).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                Some(parent.id)
            } else {
                None
            };

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id"
            ).bind(project_id).bind(&identifier).bind(title).bind(description)
            .bind(status_id).bind(priority).bind(parent_id).bind(position).bind(&now).bind(&now)
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

            // Create task contract
            let context = json!({"files": serde_json::from_str::<Value>(&context_files).unwrap_or(json!([]))});
            sqlx::query(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4::jsonb, '[]'::jsonb, '[]'::jsonb, $5::jsonb, $6, $7, 0)"
            ).bind(issue_id).bind(task_type).bind(objective)
            .bind(serde_json::to_string(&context).unwrap_or_default())
            .bind(&skills).bind(complexity).bind(timeout_minutes)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;

            // Create dependency relations
            if let Some(deps) = depends_on {
                for dep_val in deps {
                    if let Some(dep_ident) = dep_val.as_str() {
                        let dep_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                            .bind(dep_ident).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                        sqlx::query(
                            "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocks')"
                        ).bind(dep_issue.id).bind(issue_id)
                        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
                    }
                }
            }

            tx.commit().await.map_err(|e| e.to_string())?;

            // Check if this task needs decomposition
            if let Ok(true) = orchestration::decomposition::check_decomposition_needed(pool, issue_id).await {
                let _ = orchestration::decomposition::create_decomposition_task(pool, issue_id).await;
            }

            // Return full contract
            let full = orchestration::routing::build_full_contract(pool, issue_id)
                .await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(full))
        }
        "get_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let full = orchestration::routing::build_full_contract(pool, issue.id)
                .await.map_err(|e| e.to_string())?;
            match full {
                Some(c) => Ok(json!(c)),
                None => Err(format!("No task contract found for {}", identifier)),
            }
        }
        "task_replay" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let logs = sqlx::query_as::<_, ExecutionLog>(
                "SELECT * FROM execution_logs WHERE issue_id = $1 ORDER BY timestamp ASC"
            ).bind(issue.id).fetch_all(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(logs))
        }
        "task_attempts" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let context: Value = contract.context.clone();
            let prior_attempts = context.get("prior_attempts").cloned().unwrap_or(json!([]));
            Ok(json!({
                "identifier": identifier,
                "attempt_count": contract.attempt_count,
                "prior_attempts": prior_attempts
            }))
        }
        "agent_stats" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let metrics = orchestration::metrics::agent_metrics(pool, agent_id).await.map_err(|e| e.to_string())?;
            Ok(json!(metrics))
        }
        "list_agents" => {
            let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at")
                .fetch_all(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(agents))
        }
        "system_metrics" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let metrics = orchestration::metrics::project_metrics(pool, project_id).await.map_err(|e| e.to_string())?;
            Ok(json!(metrics))
        }
        "invalidate_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let reason = args["reason"].as_str().ok_or("reason required")?;
            let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let result = orchestration::cascade::invalidate_task(pool, issue_id, reason).await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(result))
        }
        "task_graph" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(issue_id);

            while let Some(id) = queue.pop_front() {
                if !visited.insert(id) { continue; }
                let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1").bind(id).fetch_one(pool).await.map_err(|e| e.to_string())?;
                let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1").bind(id).fetch_optional(pool).await.map_err(|e| e.to_string())?;
                nodes.push(json!({"id": id, "identifier": &issue.identifier, "title": &issue.title, "state": contract.as_ref().map(|c| c.task_state.as_str()).unwrap_or("no-contract")}));

                let children: Vec<i64> = sqlx::query_scalar("SELECT id FROM issues WHERE parent_id = $1").bind(id).fetch_all(pool).await.map_err(|e| e.to_string())?;
                for c in children { edges.push(json!({"from": id, "to": c, "type": "parent-child"})); queue.push_back(c); }

                let rels: Vec<(i64, i64)> = sqlx::query_as("SELECT source_issue_id, target_issue_id FROM issue_relations WHERE (source_issue_id = $1 OR target_issue_id = $2) AND relation_type = 'blocks'").bind(id).bind(id).fetch_all(pool).await.map_err(|e| e.to_string())?;
                for (s, t) in rels { edges.push(json!({"from": s, "to": t, "type": "blocks"})); if s != id { queue.push_back(s); } if t != id { queue.push_back(t); } }

                if let Some(pid) = issue.parent_id { edges.push(json!({"from": pid, "to": id, "type": "parent-child"})); queue.push_back(pid); }
            }

            Ok(json!({"nodes": nodes, "edges": edges}))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

async fn handle_resource_read(pool: &PgPool, uri: &str) -> Result<Value, String> {
    if uri == "kanban://projects" {
        let projects =
            sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY name")
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;
        return Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&projects).unwrap_or_default(),
            }]
        }));
    }

    if let Some(id_str) = uri
        .strip_prefix("kanban://project/")
        .and_then(|s| s.strip_suffix("/board"))
    {
        let project_id: i64 = id_str.parse().map_err(|_| "Invalid project ID")?;
        let statuses = sqlx::query_as::<_, Status>(
            "SELECT * FROM statuses WHERE project_id = $1 ORDER BY position",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        let issues = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE project_id = $1 ORDER BY position",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        let board = json!({ "statuses": statuses, "issues": issues });
        return Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&board).unwrap_or_default(),
            }]
        }));
    }

    if let Some(identifier) = uri.strip_prefix("kanban://issue/") {
        let issue = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE identifier = $1",
        )
        .bind(identifier)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        let labels = sqlx::query_as::<_, Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1",
        )
        .bind(issue.id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        let detail = json!({ "issue": issue, "labels": labels });
        return Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&detail).unwrap_or_default(),
            }]
        }));
    }

    Err(format!("Unknown resource: {}", uri))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kanban:kanban@localhost:5433/kanban".to_string());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations").run(&pool).await.expect("Failed to run migrations");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp =
                    error(Value::Null, -32700, &format!("Parse error: {}", e));
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = req.id.unwrap_or(Value::Null);
        let params = req.params.unwrap_or(json!({}));

        let resp = match req.method.as_str() {
            "initialize" => success(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {},
                    },
                    "serverInfo": {
                        "name": "kanban-mcp",
                        "version": "0.1.0",
                    }
                }),
            ),
            "notifications/initialized" => continue,
            "tools/list" => success(id, json!({ "tools": tools_list() })),
            "tools/call" => {
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments =
                    params.get("arguments").cloned().unwrap_or(json!({}));
                match handle_tool_call(&pool, tool_name, &arguments).await {
                    Ok(result) => success(
                        id,
                        json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_default(),
                            }]
                        }),
                    ),
                    Err(e) => success(
                        id,
                        json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e),
                            }],
                            "isError": true,
                        }),
                    ),
                }
            }
            "resources/list" => {
                success(id, json!({ "resources": resources_list() }))
            }
            "resources/read" => {
                let uri = params
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match handle_resource_read(&pool, uri).await {
                    Ok(result) => success(id, result),
                    Err(e) => error(id, -32602, &e),
                }
            }
            "ping" => success(id, json!({})),
            _ => error(
                id,
                -32601,
                &format!("Method not found: {}", req.method),
            ),
        };

        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }

    Ok(())
}
