use kanban_lib::db;
use kanban_lib::models::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::io::{self, BufRead, Write};

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
    pool: &SqlitePool,
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
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix"
            )
            .bind(project_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?",
            )
            .bind(project_id)
            .bind(status_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let result = sqlx::query(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                    .bind(result.last_insert_rowid())
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;
            Ok(json!(issue))
        }
        "update_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(t) = args.get("title").and_then(|v| v.as_str()) {
                sqlx::query(
                    "UPDATE issues SET title = ?, updated_at = ? WHERE id = ?",
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
                    "UPDATE issues SET description = ?, updated_at = ? WHERE id = ?",
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
                    "UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?",
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
                    "UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?",
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
                        "UPDATE issues SET assignee_id = NULL, updated_at = ? WHERE id = ?",
                    )
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                } else {
                    sqlx::query(
                        "UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?",
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
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                    .bind(issue.id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(updated))
        }
        "get_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let labels = sqlx::query_as::<_, Label>(
                "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = ?",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let sub_issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE parent_id = ?",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let relations = sqlx::query_as::<_, IssueRelation>(
                "SELECT * FROM issue_relations WHERE source_issue_id = ? OR target_issue_id = ?",
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

            let mut query = String::from("SELECT * FROM issues WHERE project_id = ?");
            if status_id.is_some() { query.push_str(" AND status_id = ?"); }
            if priority.is_some() { query.push_str(" AND priority = ?"); }
            if assignee_id.is_some() { query.push_str(" AND assignee_id = ?"); }
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
                "SELECT * FROM issues WHERE project_id = ? AND (title LIKE ? OR description LIKE ? OR identifier LIKE ?) ORDER BY updated_at DESC",
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
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(status_id) = args.get("status_id").and_then(|v| v.as_i64()) {
                sqlx::query(
                    "UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?",
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
                    "SELECT * FROM issues WHERE identifier = ?",
                )
                .bind(parent_ident)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE issues SET parent_id = ?, updated_at = ? WHERE id = ?",
                )
                .bind(parent.id)
                .bind(&now)
                .bind(issue.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }

            let updated =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                    .bind(issue.id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
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
                    "SELECT * FROM issues WHERE identifier = ?",
                )
                .bind(ident)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                if let Some(s) = args.get("status_id").and_then(|v| v.as_i64()) {
                    sqlx::query(
                        "UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?",
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
                        "UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?",
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
                        "UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?",
                    )
                    .bind(a)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                let u = sqlx::query_as::<_, Issue>(
                    "SELECT * FROM issues WHERE id = ?",
                )
                .bind(issue.id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
                updated_issues.push(u);
            }
            Ok(json!(updated_issues))
        }
        "get_board" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let statuses = sqlx::query_as::<_, Status>(
                "SELECT * FROM statuses WHERE project_id = ? ORDER BY position",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = ? AND parent_id IS NULL ORDER BY position",
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
            let result = sqlx::query(
                "INSERT INTO labels (project_id, name, color) VALUES (?, ?, ?)",
            )
            .bind(project_id)
            .bind(name)
            .bind(color)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            let label =
                sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
                    .bind(result.last_insert_rowid())
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(label))
        }
        "add_blocker" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let blocker = args["blocker_identifier"]
                .as_str()
                .ok_or("blocker_identifier required")?;
            let issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(identifier)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let blocker_issue = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE identifier = ?",
            )
            .bind(blocker)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocked_by')",
            )
            .bind(issue.id)
            .bind(blocker_issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
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
            let result = sqlx::query(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES (?, ?, ?, '#6366f1', ?)",
            )
            .bind(name)
            .bind(display_name)
            .bind(email)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            let member =
                sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ?")
                    .bind(result.last_insert_rowid())
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(member))
        }
        "list_comments" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let comments = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE issue_id = ? ORDER BY created_at ASC")
                .bind(issue.id).fetch_all(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(comments))
        }
        "add_comment" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let content = args["content"].as_str().ok_or("content required")?;
            let member_id = args.get("member_id").and_then(|v| v.as_i64());
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let result = sqlx::query("INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                .bind(issue.id).bind(member_id).bind(content).bind(&now).bind(&now)
                .execute(pool).await.map_err(|e| e.to_string())?;
            let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
                .bind(result.last_insert_rowid()).fetch_one(pool).await.map_err(|e| e.to_string())?;
            Ok(json!(comment))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

async fn handle_resource_read(pool: &SqlitePool, uri: &str) -> Result<Value, String> {
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
            "SELECT * FROM statuses WHERE project_id = ? ORDER BY position",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        let issues = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE project_id = ? ORDER BY position",
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
            "SELECT * FROM issues WHERE identifier = ?",
        )
        .bind(identifier)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        let labels = sqlx::query_as::<_, Label>(
            "SELECT l.* FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = ?",
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
    let pool = db::init_db().await?;

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
