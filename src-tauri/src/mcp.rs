use crate::models::*;
use crate::orchestration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::AnyPool;
use std::io::{self, BufRead, Write};
use uuid::Uuid;

#[cfg(feature = "redis-sync")]
fn notify_change() {
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        if let Ok(client) = redis::Client::open(redis_url) {
            if let Ok(mut conn) = client.get_connection() {
                let _: Result<(), _> = redis::cmd("PUBLISH")
                    .arg("kanban:db-changed")
                    .arg("1")
                    .query(&mut conn);
            }
        }
    }
}

#[cfg(not(feature = "redis-sync"))]
fn notify_change() {}

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

/// Helper: auto-comment on an issue as an agent
async fn mcp_auto_comment(
    pool: &AnyPool,
    issue_id: i64,
    agent_id: &str,
    content: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();
    let agent_member_id: Option<i64> =
        sqlx::query_scalar("SELECT member_id FROM agents WHERE id = $1")
            .bind(agent_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(issue_id)
    .bind(agent_member_id)
    .bind(content)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
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
        tool_def("list_members", "List workspace members", json!({}), vec![]),
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
        tool_def(
            "list_comments",
            "List comments on an issue",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
            }),
            vec!["identifier"],
        ),
        tool_def(
            "add_comment",
            "Add a comment to an issue",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
                "content": prop("string", "Comment content (Markdown)"),
                "member_id": prop("number", "Member ID of the commenter"),
            }),
            vec!["identifier", "content"],
        ),
        // Agent lifecycle
        tool_def(
            "register_agent",
            "Register a new AI agent",
            json!({
                "name": prop("string", "Agent name (unique)"),
                "skills": {"type": "array", "items": {"type": "string"}, "description": "Skills the agent has"},
                "task_types": {"type": "array", "items": {"type": "string"}, "description": "Task types agent can handle"},
                "max_concurrent": prop("number", "Max concurrent tasks (default: 1)"),
                "max_complexity": prop("string", "Max complexity: small, medium, large (default: large)")
            }),
            vec!["name", "skills"],
        ),
        tool_def(
            "agent_heartbeat",
            "Send agent heartbeat",
            json!({
                "agent_id": prop("string", "Agent ID")
            }),
            vec!["agent_id"],
        ),
        tool_def(
            "deregister_agent",
            "Deregister an agent and reclaim its tasks",
            json!({
                "agent_id": prop("string", "Agent ID")
            }),
            vec!["agent_id"],
        ),
        // Task work loop
        tool_def(
            "next_task",
            "Get next available task and atomically claim it",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "skills_override": {"type": "array", "items": {"type": "string"}, "description": "Override skills for this query"}
            }),
            vec!["agent_id"],
        ),
        tool_def(
            "start_task",
            "Mark a claimed task as executing",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "identifier": prop("string", "Task identifier (e.g. KAN-42)")
            }),
            vec!["agent_id", "identifier"],
        ),
        tool_def(
            "complete_task",
            "Complete a task with confidence score and summary",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "identifier": prop("string", "Task identifier"),
                "confidence": prop("number", "Confidence score 0.0-1.0"),
                "summary": prop("string", "Completion summary"),
                "artifacts": {"type": "object", "description": "Artifacts produced (branches, PRs, etc.)"}
            }),
            vec!["agent_id", "identifier", "confidence", "summary"],
        ),
        tool_def(
            "fail_task",
            "Report task failure with reason",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "identifier": prop("string", "Task identifier"),
                "reason": prop("string", "Failure reason")
            }),
            vec!["agent_id", "identifier", "reason"],
        ),
        tool_def(
            "unclaim_task",
            "Voluntarily release a claimed task",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "identifier": prop("string", "Task identifier")
            }),
            vec!["agent_id", "identifier"],
        ),
        tool_def(
            "approve_task",
            "Human approves a task in validating state",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        tool_def(
            "reject_task",
            "Human rejects a task in validating state",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        // Execution logging
        tool_def(
            "log_task_activity",
            "Log execution activity for a task",
            json!({
                "identifier": prop("string", "Task identifier"),
                "agent_id": prop("string", "Agent ID"),
                "entry_type": prop("string", "Log type: reasoning, file_read, file_edit, command, discovery, error, checkpoint"),
                "message": prop("string", "Log message"),
                "metadata": {"type": "object", "description": "Optional metadata"}
            }),
            vec!["identifier", "agent_id", "entry_type", "message"],
        ),
        // Task management
        tool_def(
            "create_task",
            "Create a new task contract with structured fields",
            json!({
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
                "timeout_minutes": prop("number", "Timeout in minutes (default: 30)"),
                "assignee_id": prop("number", "Assignee member ID"),
                "success_criteria": {"type": "array", "items": {"type": "object"}, "description": "Success criteria (objects with check/command/expect fields)"},
                "constraints": {"type": "array", "items": {"type": "string"}, "description": "Task constraints"}
            }),
            vec!["project_id", "title", "objective", "status_id"],
        ),
        tool_def(
            "get_task",
            "Get full task contract by identifier",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        tool_def(
            "task_replay",
            "Get execution replay timeline for a task",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        tool_def(
            "task_attempts",
            "Get prior attempt history for a task",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        tool_def(
            "agent_stats",
            "Get agent performance statistics",
            json!({
                "agent_id": prop("string", "Agent ID")
            }),
            vec!["agent_id"],
        ),
        tool_def("list_agents", "List all registered agents", json!({}), vec![]),
        tool_def(
            "marketplace_register",
            "Register an agent in the marketplace",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "name": prop("string", "Agent display name"),
                "description": prop("string", "Agent description"),
                "provider": prop("string", "Provider: claude, gpt, custom"),
                "version": prop("string", "Agent version"),
                "endpoint": prop("string", "MCP endpoint or webhook URL"),
                "capabilities": {"type": "array", "items": {"type": "string"}, "description": "List of capabilities/skills"},
                "max_concurrent": prop("number", "Max concurrent tasks"),
                "max_complexity": prop("string", "Max complexity: small, medium, large"),
                "hourly_rate": prop("number", "Optional hourly rate for cost tracking")
            }),
            vec!["agent_id", "name", "capabilities"],
        ),
        tool_def(
            "marketplace_search",
            "Search marketplace for agents with specific skills",
            json!({
                "skills": {"type": "array", "items": {"type": "string"}, "description": "Required skills"},
                "max_complexity": prop("string", "Max complexity filter: small, medium, large")
            }),
            vec!["skills"],
        ),
        tool_def(
            "find_best_agent",
            "Find the best agent for a task based on skills, proficiency, and availability",
            json!({
                "task_skills": {"type": "array", "items": {"type": "string"}, "description": "Required task skills"},
                "complexity": prop("string", "Task complexity: small, medium, large")
            }),
            vec!["task_skills", "complexity"],
        ),
        tool_def("marketplace_list", "List all agents in the marketplace", json!({}), vec![]),
        tool_def(
            "system_metrics",
            "Get system-wide metrics for a project",
            json!({
                "project_id": prop("number", "Project ID")
            }),
            vec!["project_id"],
        ),
        tool_def(
            "invalidate_task",
            "Invalidate a completed task and cascade effects to downstream tasks",
            json!({
                "identifier": prop("string", "Task identifier"),
                "reason": prop("string", "Reason for invalidation")
            }),
            vec!["identifier", "reason"],
        ),
        tool_def(
            "task_graph",
            "Get dependency graph for a task (nodes and edges)",
            json!({
                "identifier": prop("string", "Task identifier")
            }),
            vec!["identifier"],
        ),
        // AI Agent Intelligence: Triage
        tool_def(
            "triage_issue",
            "Auto-triage an issue: suggest priority, labels, assignee, and epic based on title/description keywords",
            json!({
                "project_id": prop("number", "Project ID"),
                "title": prop("string", "Issue title"),
                "description": prop("string", "Issue description (optional)")
            }),
            vec!["project_id", "title"],
        ),
        tool_def(
            "auto_triage",
            "Auto-triage an existing issue and apply suggestions (priority, labels, assignee)",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)")
            }),
            vec!["identifier"],
        ),
        // AI Agent Intelligence: Decomposition
        tool_def(
            "decompose_issue",
            "Preview decomposition of an issue into sub-tasks by parsing its description (checklists, numbered lists, headings)",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)")
            }),
            vec!["identifier"],
        ),
        tool_def(
            "apply_decomposition",
            "Decompose an issue into sub-issues by parsing its description and creating child issues",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)")
            }),
            vec!["identifier"],
        ),
        // AI Agent Intelligence: Natural Language
        tool_def(
            "create_from_text",
            "Create an issue from natural language text. Parses title, description, and auto-triages priority/labels/assignee",
            json!({
                "project_id": prop("number", "Project ID"),
                "text": prop("string", "Natural language description of the issue"),
                "status_id": prop("number", "Initial status ID")
            }),
            vec!["project_id", "text", "status_id"],
        ),
        // Context Assembly
        tool_def(
            "get_task_context",
            "Get full assembled context for a task including labels, relations, comments, prior attempts, similar issues, and project path",
            json!({
                "identifier": prop("string", "Task/issue identifier (e.g. KAN-42)")
            }),
            vec!["identifier"],
        ),
        // Code Analysis
        tool_def(
            "link_file",
            "Link a file to an issue for code tracking",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
                "file_path": prop("string", "File path relative to project root"),
                "link_type": prop("string", "Link type: related, cause, fix (default: related)")
            }),
            vec!["identifier", "file_path"],
        ),
        tool_def(
            "file_heat_map",
            "Get file heat map showing files ranked by issue count",
            json!({
                "project_id": prop("number", "Project ID"),
                "limit": prop("number", "Max results (default: 20)"),
            }),
            vec!["project_id"],
        ),
        // WSJF Scoring
        tool_def(
            "set_wsjf",
            "Set WSJF scores for an issue (business_value + time_criticality + risk_reduction) / job_size",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
                "business_value": prop("number", "Business value 1-10"),
                "time_criticality": prop("number", "Time criticality 1-10"),
                "risk_reduction": prop("number", "Risk reduction / opportunity enablement 1-10"),
                "job_size": prop("number", "Job size 1-10"),
            }),
            vec!["identifier", "business_value", "time_criticality", "risk_reduction", "job_size"],
        ),
        tool_def(
            "auto_score",
            "Auto-score an issue using rule-based heuristics (priority, due date, labels, estimate)",
            json!({
                "identifier": prop("string", "Issue identifier (e.g. KAN-42)"),
            }),
            vec!["identifier"],
        ),
        tool_def(
            "ranked_backlog",
            "Get all unstarted issues ranked by WSJF score (highest first)",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "directory_heat_map",
            "Get directory heat map showing directories ranked by issue count",
            json!({
                "project_id": prop("number", "Project ID"),
                "depth": prop("number", "Directory depth to aggregate at (default: 2)")
            }),
            vec!["project_id"],
        ),
        tool_def(
            "issues_for_file",
            "Get all issues linked to a specific file",
            json!({
                "file_path": prop("string", "File path"),
                "project_id": prop("number", "Project ID")
            }),
            vec!["file_path", "project_id"],
        ),
        // Diff Issues
        tool_def(
            "create_issue_from_diff",
            "Create an issue from a code review finding with automatic file linking",
            json!({
                "project_id": prop("number", "Project ID"),
                "title": prop("string", "Issue title"),
                "description": prop("string", "Issue description"),
                "file_path": prop("string", "File path where the issue was found"),
                "line_range": prop("string", "Line range (e.g. '42-55')"),
                "severity": prop("string", "Severity: bug, improvement, todo")
            }),
            vec!["project_id", "title", "file_path", "severity"],
        ),
        // Handoff Notes
        tool_def(
            "create_handoff",
            "Create a handoff note from one agent to another for a task",
            json!({
                "task_identifier": prop("string", "Task identifier (e.g. KAN-42)"),
                "from_agent_id": prop("string", "Agent ID leaving the note"),
                "to_agent_id": prop("string", "Target agent ID (omit for any agent)"),
                "note_type": prop("string", "Type: completion, review_request, escalation, context, warning, suggestion"),
                "summary": prop("string", "Brief summary of the handoff"),
                "details": prop("string", "Longer explanation"),
                "files_changed": prop("array", "JSON array of file paths changed"),
                "risks": prop("array", "JSON array of known risks"),
                "test_results": prop("object", "Test results: {passed, failed, skipped}"),
            }),
            vec!["task_identifier", "from_agent_id", "note_type", "summary"],
        ),
        tool_def(
            "get_handoff_notes",
            "Get handoff notes for a task, optionally filtered for a specific agent",
            json!({
                "task_identifier": prop("string", "Task identifier"),
                "agent_id": prop("string", "Filter notes relevant to this agent"),
            }),
            vec!["task_identifier"],
        ),
        tool_def(
            "record_learning",
            "Record a learning from completing or failing a task",
            json!({
                "task_identifier": prop("string", "Task identifier (e.g. KAN-42)"),
                "agent_id": prop("string", "Agent ID recording the learning"),
                "outcome": prop("string", "Outcome: success, failure, partial"),
                "approach_summary": prop("string", "What the agent did"),
                "key_insight": prop("string", "Most important learning"),
                "pitfalls": prop("array", "JSON array of things that went wrong"),
                "effective_patterns": prop("array", "JSON array of things that worked well"),
                "relevant_files": prop("array", "JSON array of files involved"),
                "tags": prop("array", "JSON array of keyword tags for matching"),
            }),
            vec!["task_identifier", "agent_id", "outcome", "approach_summary"],
        ),
        tool_def(
            "find_similar_learnings",
            "Find learnings from similar past tasks",
            json!({
                "project_id": prop("number", "Project ID"),
                "title": prop("string", "Task title to match against"),
                "description": prop("string", "Task description for matching"),
                "tags": prop("array", "Tags to match against"),
                "limit": prop("number", "Max results (default 5)"),
            }),
            vec!["project_id", "title"],
        ),
        tool_def(
            "get_task_learnings",
            "Get learnings recorded for a specific task",
        // Cost tracking
        tool_def(
            "record_cost",
            "Record a cost entry for a task",
            json!({
                "task_identifier": prop("string", "Task identifier (e.g. KAN-42)"),
                "agent_id": prop("string", "Agent ID"),
                "cost_type": prop("string", "Cost type: compute_time, api_tokens, custom"),
                "amount": prop("number", "Cost amount"),
                "unit": prop("string", "Unit: minutes, tokens, dollars, credits"),
                "description": prop("string", "Optional description"),
            }),
            vec!["task_identifier", "agent_id", "cost_type", "amount", "unit"],
        ),
        tool_def(
            "task_cost",
            "Get cost summary for a task",
            json!({
                "task_identifier": prop("string", "Task identifier"),
            }),
            vec!["task_identifier"],
        ),
            "auto_score_project",
            "Auto-score all unscored issues in a project",
        tool_def(
            "project_costs",
            "Get cost summary for a project",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        // Pipelines
        tool_def(
            "list_pipelines",
            "List multi-agent pipelines for a project",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "check_budget",
            "Check budget status and alerts for a project",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        // SLA engine
        tool_def(
            "check_sla",
            "Check SLA compliance for all in-progress issues in a project",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "create_pipeline",
            "Create a multi-agent pipeline with sequential stages",
            json!({
                "project_id": prop("number", "Project ID"),
                "name": prop("string", "Pipeline name"),
                "description": prop("string", "Pipeline description"),
                "stages": {"type": "array", "items": {"type": "object"}, "description": "Array of PipelineStage objects with name, task_type, required_skills, max_complexity, timeout_minutes, title_template, objective_template, success_criteria, auto_advance"}
            }),
            vec!["project_id", "name", "stages"],
        ),
        tool_def(
            "trigger_pipeline",
            "Trigger a pipeline run, creating the first stage task",
            json!({
                "pipeline_id": prop("number", "Pipeline ID"),
                "trigger_issue_id": prop("number", "Optional issue ID that triggered this pipeline"),
                "context": prop("string", "Optional JSON context for the pipeline run")
            }),
            vec!["pipeline_id"],
        ),
        tool_def(
            "advance_pipeline",
            "Advance a pipeline run to its next stage",
            json!({
                "run_id": prop("number", "Pipeline run ID")
            }),
            vec!["run_id"],
        ),
        tool_def(
            "pipeline_status",
            "Get status of a pipeline run including stage progress",
            json!({
                "run_id": prop("number", "Pipeline run ID")
            }),
            vec!["run_id"],
        tool_def(
            "list_permissions",
            "List all permissions for an agent",
            json!({
                "agent_id": prop("string", "Agent ID")
            }),
            vec!["agent_id"],
        ),
        tool_def(
            "set_permission",
            "Set a permission for an agent (grant or deny)",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "permission_type": prop("string", "Permission type: project_access, file_access, action, task_type, max_cost"),
                "scope": prop("string", "Scope: project_id, glob pattern, action name, task type, or cost limit"),
                "allowed": prop("boolean", "true=allow, false=deny")
            }),
            vec!["agent_id", "permission_type", "scope", "allowed"],
        ),
        tool_def(
            "check_permission",
            "Check if an agent has a specific permission",
            json!({
                "agent_id": prop("string", "Agent ID"),
                "permission_type": prop("string", "Permission type to check"),
                "scope": prop("string", "Scope to check")
            }),
            vec!["agent_id", "permission_type", "scope"],
            "enforce_sla",
            "Enforce SLA policies and execute escalation actions",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
        tool_def(
            "sla_status",
            "Get SLA dashboard with policies, compliance, and events",
            json!({
                "project_id": prop("number", "Project ID"),
            }),
            vec!["project_id"],
        ),
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
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
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
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
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

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
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
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            if let Some(t) = args.get("title").and_then(|v| v.as_str()) {
                sqlx::query("UPDATE issues SET title = $1, updated_at = $2 WHERE id = $3")
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

            let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(updated))
        }
        "get_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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
            let sub_issues =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE parent_id = $1")
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
            let priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let assignee_id = args.get("assignee_id").and_then(|v| v.as_i64());

            let mut param_idx = 1;
            let mut query =
                format!("SELECT * FROM issues WHERE project_id = ${}", param_idx);
            param_idx += 1;
            if status_id.is_some() {
                query.push_str(&format!(" AND status_id = ${}", param_idx));
                param_idx += 1;
            }
            if priority.is_some() {
                query.push_str(&format!(" AND priority = ${}", param_idx));
                param_idx += 1;
            }
            if assignee_id.is_some() {
                query.push_str(&format!(" AND assignee_id = ${}", param_idx));
            }
            query.push_str(" ORDER BY position");

            let mut q = sqlx::query_as::<_, Issue>(&query).bind(project_id);
            if let Some(s) = status_id {
                q = q.bind(s);
            }
            if let Some(ref p) = priority {
                q = q.bind(p);
            }
            if let Some(a) = assignee_id {
                q = q.bind(a);
            }
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
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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
                let parent =
                    sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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

            let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
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
                let ident = ident_val.as_str().ok_or("identifier must be string")?;
                let issue =
                    sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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
                let u = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
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
            let label = sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
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
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let blocker_issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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
            let member = sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1")
                .bind(member_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(member))
        }
        "list_comments" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let comments = sqlx::query_as::<_, Comment>(
                "SELECT * FROM comments WHERE issue_id = $1 ORDER BY created_at ASC",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(comments))
        }
        "add_comment" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let content = args["content"].as_str().ok_or("content required")?;
            let member_id = args.get("member_id").and_then(|v| v.as_i64());
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let comment_id: i64 = sqlx::query_scalar(
                "INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(issue.id)
            .bind(member_id)
            .bind(content)
            .bind(&now)
            .bind(&now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
                .bind(comment_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(comment))
        }
        // ── Agent lifecycle ──────────────────────────────────────────
        "register_agent" => {
            let name_input = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let agent_type = args.get("agent_type").and_then(|v| v.as_str());
            let skills = args
                .get("skills")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let task_types = args
                .get("task_types")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let max_concurrent =
                args.get("max_concurrent").and_then(|v| v.as_i64()).unwrap_or(1);
            let max_complexity = args
                .get("max_complexity")
                .and_then(|v| v.as_str())
                .unwrap_or("large");
            let agent_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            // Generate name if not provided
            let agent_name = if name_input.is_empty() {
                orchestration::names::generate_agent_name()
            } else {
                name_input.to_string()
            };

            // Determine avatar color based on agent type
            let agent_type_str = agent_type.unwrap_or("custom");
            let avatar_color = match agent_type_str {
                "claude" | "claude-code" => "#f97316",
                "codex" => "#22c55e",
                "gemini" => "#3b82f6",
                _ => "#8b5cf6",
            };

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

            // Create a member for this agent
            let member_id: i64 = sqlx::query_scalar(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(format!("[{}] {}", agent_type_str, &agent_name))
            .bind(&agent_name)
            .bind(Option::<String>::None)
            .bind(avatar_color)
            .bind(&now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO agents (id, name, agent_type, skills, task_types, max_concurrent, max_complexity, member_id, status, registered_at, last_heartbeat) VALUES ($1, $2, $3, $4{jb}, $5{jb}, $6, $7, $8, 'idle', $9, $10)"
            ))
            .bind(&agent_id)
            .bind(&agent_name)
            .bind(&agent_type)
            .bind(&skills)
            .bind(&task_types)
            .bind(max_concurrent)
            .bind(max_complexity)
            .bind(member_id)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(&format!(
                "INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES ($1, 0, 0, 0.0, 0, '{{}}'{})",
                jb
            ))
            .bind(&agent_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"agent_id": agent_id, "name": agent_name, "member_id": member_id}))
        }
        "agent_heartbeat" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let now = chrono::Utc::now().to_rfc3339();

            // Check if agent has active tasks
            let active: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')",
            )
            .bind(agent_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            let status = if active.0 > 0 { "working" } else { "idle" };

            sqlx::query("UPDATE agents SET last_heartbeat = $1, status = $2 WHERE id = $3")
                .bind(&now)
                .bind(status)
                .bind(agent_id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": status, "active_tasks": active.0}))
        }
        "deregister_agent" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let now = chrono::Utc::now().to_rfc3339();

            // Reclaim any active tasks
            let reclaimed = sqlx::query(
                "UPDATE task_contracts SET claimed_by = NULL, claimed_at = NULL, task_state = 'queued' WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')",
            )
            .bind(agent_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync statuses for reclaimed tasks
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = issues.project_id AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $1
                WHERE id IN (SELECT issue_id FROM task_contracts WHERE claimed_by IS NULL AND task_state = 'queued')
                  AND status_id IN (SELECT s2.id FROM statuses s2 WHERE s2.category = 'started')",
            )
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query("DELETE FROM agents WHERE id = $1")
                .bind(agent_id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "tasks_reclaimed": reclaimed.rows_affected()}))
        }
        // ── Task work loop ───────────────────────────────────────────
        "next_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;

            // Lazy timeout recovery
            let _ =
                crate::orchestration::timeout::reclaim_timed_out_tasks(pool, backend).await;

            let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
                .bind(agent_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            let skills: Vec<String> = if let Some(overrides) =
                args.get("skills_override").and_then(|v| v.as_array())
            {
                overrides
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                serde_json::from_str(&agent.skills).unwrap_or_default()
            };

            let contract = orchestration::routing::next_task(
                pool,
                agent_id,
                &skills,
                &agent.max_complexity,
                agent.max_concurrent,
            )
            .await
            .map_err(|e| e.to_string())?;

            match contract {
                Some(c) => {
                    let _ = mcp_auto_comment(
                        pool,
                        c.issue_id,
                        agent_id,
                        "\u{1F916} Task claimed. Reading contract and preparing to execute.",
                    )
                    .await;
                    Ok(json!(c))
                }
                None => Ok(json!(null)),
            }
        }
        "start_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!(
                    "Task {} is not claimed by agent {}",
                    identifier, agent_id
                ));
            }
            if contract.task_state != "claimed" {
                return Err(format!(
                    "Task {} is in state '{}', expected 'claimed'",
                    identifier, contract.task_state
                ));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = $1",
            )
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'start', 'Task started executing', $4)",
            )
            .bind(issue.id)
            .bind(agent_id)
            .bind(contract.attempt_count)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            let _ =
                mcp_auto_comment(pool, issue.id, agent_id, "\u{1F527} Execution started.")
                    .await;
            notify_change();
            Ok(json!({"status": "ok", "task_state": "executing"}))
        }
        "complete_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let confidence = args["confidence"].as_f64().ok_or("confidence required")?;
            if confidence < 0.0 || confidence > 1.0 {
                return Err("Confidence must be between 0.0 and 1.0".to_string());
            }
            let summary = args["summary"].as_str().ok_or("summary required")?;
            let artifacts = args.get("artifacts").cloned().unwrap_or(json!({}));
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!(
                    "Task {} is not claimed by agent {}",
                    identifier, agent_id
                ));
            }
            if contract.task_state != "executing" {
                return Err(format!(
                    "Task {} is in state '{}', expected 'executing'",
                    identifier, contract.task_state
                ));
            }

            // Get project config for thresholds
            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = $1",
            )
            .bind(issue.project_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

            let auto_accept =
                config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.9);
            let human_review =
                config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.7);

            let result_json = serde_json::to_string(&json!({
                "confidence": confidence,
                "summary": summary,
                "artifacts": artifacts
            }))
            .unwrap_or_default();

            let new_state = if confidence >= auto_accept {
                "completed"
            } else if confidence >= human_review {
                "validating"
            } else {
                "validating"
            };

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "UPDATE task_contracts SET task_state = $1, result = $2{jb} WHERE issue_id = $3"
            ))
            .bind(new_state)
            .bind(&result_json)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state).unwrap(),
            );
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = $2 ORDER BY s.position ASC LIMIT 1
                ), updated_at = $3 WHERE id = $4",
            )
            .bind(issue.project_id)
            .bind(category)
            .bind(&now)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Log completion
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'complete', $4, $5)",
            )
            .bind(issue.id)
            .bind(agent_id)
            .bind(contract.attempt_count)
            .bind(format!(
                "Task completed with confidence {:.2}: {}",
                confidence, summary
            ))
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Update agent stats
            sqlx::query(
                "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + $1 WHERE agent_id = $2",
            )
            .bind(confidence)
            .bind(agent_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Auto-comment based on outcome
            if new_state == "completed" {
                let _ = mcp_auto_comment(
                    pool,
                    issue.id,
                    agent_id,
                    &format!(
                        "\u{2705} Task completed (confidence: {:.2}). {}",
                        confidence, summary
                    ),
                )
                .await;
            } else if new_state == "validating" {
                let _ = mcp_auto_comment(
                    pool,
                    issue.id,
                    agent_id,
                    &format!(
                        "\u{23F3} Task completed with low confidence ({:.2}). Awaiting review. {}",
                        confidence, summary
                    ),
                )
                .await;
            }

            // Auto-unblock downstream tasks when completed
            if new_state == "completed" {
                let _ = orchestration::dependency::resolve_downstream(pool, issue.id).await;
                // Check pipeline auto-advance
                let _ = crate::commands::pipelines::check_pipeline_advancement(
                    pool, backend, identifier,
                ).await;
            }

            notify_change();
            Ok(json!({"status": "ok", "task_state": new_state, "confidence": confidence}))
        }
        "fail_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let reason = args["reason"].as_str().ok_or("reason required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!(
                    "Task {} is not claimed by agent {}",
                    identifier, agent_id
                ));
            }

            // Append to prior_attempts in context
            let mut context: Value = contract.context_json();
            let attempt_record = json!({
                "attempt": contract.attempt_count,
                "agent_id": agent_id,
                "reason": reason,
                "timestamp": now
            });
            if let Some(arr) = context
                .get_mut("prior_attempts")
                .and_then(|v| v.as_array_mut())
            {
                arr.push(attempt_record);
            } else {
                context["prior_attempts"] = json!([attempt_record]);
            }

            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = $1",
            )
            .bind(issue.project_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
            let max_attempts = config.as_ref().map(|c| c.max_attempts).unwrap_or(3);

            let new_attempt = contract.attempt_count + 1;
            let new_state = if new_attempt >= max_attempts {
                "blocked"
            } else {
                "queued"
            };

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "UPDATE task_contracts SET task_state = $1, claimed_by = NULL, claimed_at = NULL, attempt_count = $2, context = $3{jb} WHERE issue_id = $4"
            ))
            .bind(new_state)
            .bind(new_attempt)
            .bind(serde_json::to_string(&context).unwrap_or_default())
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state).unwrap(),
            );
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = $2 ORDER BY s.position ASC LIMIT 1
                ), updated_at = $3 WHERE id = $4",
            )
            .bind(issue.project_id)
            .bind(category)
            .bind(&now)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Log failure
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'error', $4, $5)",
            )
            .bind(issue.id)
            .bind(agent_id)
            .bind(contract.attempt_count)
            .bind(format!("Task failed: {}", reason))
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Update agent stats
            sqlx::query(
                "UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = $1",
            )
            .bind(agent_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            let _ = mcp_auto_comment(
                pool,
                issue.id,
                agent_id,
                &format!("\u{274C} Task failed: {}", reason),
            )
            .await;

            notify_change();
            Ok(json!({"status": "ok", "task_state": new_state, "attempt_count": new_attempt}))
        }
        "unclaim_task" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.claimed_by.as_deref() != Some(agent_id) {
                return Err(format!(
                    "Task {} is not claimed by agent {}",
                    identifier, agent_id
                ));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1",
            )
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync issue status to unstarted
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3",
            )
            .bind(issue.project_id)
            .bind(&now)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Log unclaim
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'unclaim', 'Task unclaimed by agent', $4)",
            )
            .bind(issue.id)
            .bind(agent_id)
            .bind(contract.attempt_count)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            let _ = mcp_auto_comment(
                pool,
                issue.id,
                agent_id,
                "\u{21A9}\u{FE0F} Task unclaimed and returned to queue.",
            )
            .await;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "queued"}))
        }
        "approve_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.task_state != "validating" {
                return Err(format!(
                    "Task {} is in state '{}', expected 'validating'",
                    identifier, contract.task_state
                ));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = $1",
            )
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync issue status to completed
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'completed' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3",
            )
            .bind(issue.project_id)
            .bind(&now)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Update agent stats if there was a claiming agent
            if let Some(ref agent_id) = contract.claimed_by {
                sqlx::query(
                    "UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = $1",
                )
                .bind(agent_id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }

            // Auto-unblock downstream tasks
            let _ = orchestration::dependency::resolve_downstream(pool, issue.id).await;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "completed"}))
        }
        "reject_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if contract.task_state != "validating" {
                return Err(format!(
                    "Task {} is in state '{}', expected 'validating'",
                    identifier, contract.task_state
                ));
            }

            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, result = NULL WHERE issue_id = $1",
            )
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Sync issue status to unstarted
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s WHERE s.project_id = $1 AND s.category = 'unstarted' ORDER BY s.position ASC LIMIT 1
                ), updated_at = $2 WHERE id = $3",
            )
            .bind(issue.project_id)
            .bind(&now)
            .bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok", "task_state": "queued"}))
        }
        // ── Execution logging ────────────────────────────────────────
        "log_task_activity" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let entry_type = args["entry_type"].as_str().ok_or("entry_type required")?;
            let message = args["message"].as_str().ok_or("message required")?;
            let metadata = args
                .get("metadata")
                .map(|v| serde_json::to_string(v).unwrap_or_default());
            let now = chrono::Utc::now().to_rfc3339();

            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let attempt_count: (i64,) = sqlx::query_as(
                "SELECT attempt_count FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES ($1, $2, $3, $4, $5, $6{jb}, $7)"
            ))
            .bind(issue.id)
            .bind(agent_id)
            .bind(attempt_count.0)
            .bind(entry_type)
            .bind(message)
            .bind(metadata)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!({"status": "ok"}))
        }
        // ── Task management ──────────────────────────────────────────
        "create_task" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?;
            let objective = args["objective"].as_str().ok_or("objective required")?;
            let status_id = args["status_id"].as_i64().ok_or("status_id required")?;
            let task_type = args
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("implementation");
            let priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("medium");
            let skills = args
                .get("skills")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let complexity = args.get("complexity").and_then(|v| v.as_str());
            let description = args.get("description").and_then(|v| v.as_str());
            let depends_on = args.get("depends_on").and_then(|v| v.as_array());
            let context_files = args
                .get("context_files")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let parent_identifier =
                args.get("parent_identifier").and_then(|v| v.as_str());
            let timeout_minutes =
                args.get("timeout_minutes").and_then(|v| v.as_i64()).unwrap_or(30);
            let assignee_id = args.get("assignee_id").and_then(|v| v.as_i64());
            let success_criteria = args
                .get("success_criteria")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let constraints = args
                .get("constraints")
                .and_then(|v| v.as_array())
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_else(|| "[]".to_string());
            let now = chrono::Utc::now().to_rfc3339();

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

            // Create the issue first
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
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

            // Resolve parent
            let parent_id = if let Some(pi) = parent_identifier {
                let parent =
                    sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                        .bind(pi)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                Some(parent.id)
            } else {
                None
            };

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

            // Create task contract
            let context =
                json!({"files": serde_json::from_str::<Value>(&context_files).unwrap_or(json!([]))});
            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4{jb}, $5{jb}, $6{jb}, $7{jb}, $8, $9, 0)"
            ))
            .bind(issue_id)
            .bind(task_type)
            .bind(objective)
            .bind(serde_json::to_string(&context).unwrap_or_default())
            .bind(&constraints)
            .bind(&success_criteria)
            .bind(&skills)
            .bind(complexity)
            .bind(timeout_minutes)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            // Create dependency relations
            if let Some(deps) = depends_on {
                for dep_val in deps {
                    if let Some(dep_ident) = dep_val.as_str() {
                        let dep_issue = sqlx::query_as::<_, Issue>(
                            "SELECT * FROM issues WHERE identifier = $1",
                        )
                        .bind(dep_ident)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                        sqlx::query(
                            "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocks')",
                        )
                        .bind(dep_issue.id)
                        .bind(issue_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }
            }

            tx.commit().await.map_err(|e| e.to_string())?;

            // Check if this task needs decomposition
            if let Ok(true) =
                orchestration::decomposition::check_decomposition_needed(pool, issue_id).await
            {
                let _ =
                    orchestration::decomposition::create_decomposition_task(pool, issue_id)
                        .await;
            }

            // Return full contract
            let full = orchestration::routing::build_full_contract(pool, issue_id)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(full))
        }
        "get_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let full = orchestration::routing::build_full_contract(pool, issue.id)
                .await
                .map_err(|e| e.to_string())?;
            match full {
                Some(c) => Ok(json!(c)),
                None => Err(format!("No task contract found for {}", identifier)),
            }
        }
        "task_replay" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let logs = sqlx::query_as::<_, ExecutionLog>(
                "SELECT * FROM execution_logs WHERE issue_id = $1 ORDER BY timestamp ASC",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(logs))
        }
        "task_attempts" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let contract = sqlx::query_as::<_, TaskContract>(
                "SELECT * FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            let context: Value = contract.context_json();
            let prior_attempts =
                context.get("prior_attempts").cloned().unwrap_or(json!([]));
            Ok(json!({
                "identifier": identifier,
                "attempt_count": contract.attempt_count,
                "prior_attempts": prior_attempts
            }))
        }
        "agent_stats" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let metrics = orchestration::metrics::agent_metrics(pool, backend, agent_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!(metrics))
        }
        "list_agents" => {
            let agents =
                sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(json!(agents))
        }
        "system_metrics" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let metrics = orchestration::metrics::project_metrics(pool, backend, project_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!(metrics))
        }
        "invalidate_task" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let reason = args["reason"].as_str().ok_or("reason required")?;
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
            let result =
                orchestration::cascade::invalidate_task(pool, issue_id, reason)
                    .await
                    .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(result))
        }
        "task_graph" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue_id: i64 =
                sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                    .bind(identifier)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(issue_id);

            while let Some(id) = queue.pop_front() {
                if !visited.insert(id) {
                    continue;
                }
                let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                let contract = sqlx::query_as::<_, TaskContract>(
                    "SELECT * FROM task_contracts WHERE issue_id = $1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;
                nodes.push(json!({
                    "id": id,
                    "identifier": &issue.identifier,
                    "title": &issue.title,
                    "state": contract.as_ref().map(|c| c.task_state.as_str()).unwrap_or("no-contract")
                }));

                let children: Vec<i64> =
                    sqlx::query_scalar("SELECT id FROM issues WHERE parent_id = $1")
                        .bind(id)
                        .fetch_all(pool)
                        .await
                        .map_err(|e| e.to_string())?;
                for c in children {
                    edges.push(json!({"from": id, "to": c, "type": "parent-child"}));
                    queue.push_back(c);
                }

                let rels: Vec<(i64, i64)> = sqlx::query_as(
                    "SELECT source_issue_id, target_issue_id FROM issue_relations WHERE (source_issue_id = $1 OR target_issue_id = $2) AND relation_type = 'blocks'",
                )
                .bind(id)
                .bind(id)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?;
                for (s, t) in rels {
                    edges.push(json!({"from": s, "to": t, "type": "blocks"}));
                    if s != id {
                        queue.push_back(s);
                    }
                    if t != id {
                        queue.push_back(t);
                    }
                }

                if let Some(pid) = issue.parent_id {
                    edges.push(json!({"from": pid, "to": id, "type": "parent-child"}));
                    queue.push_back(pid);
                }
            }

            Ok(json!({"nodes": nodes, "edges": edges}))
        }
        // AI Agent Intelligence: Triage
        "triage_issue" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?;
            let description = args.get("description").and_then(|v| v.as_str());
            let suggestion = crate::commands::triage::triage_logic(
                pool, project_id, title, description,
            ).await?;
            Ok(json!(suggestion))
        }
        "auto_triage" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let suggestion = crate::commands::triage::triage_logic(
                pool, issue.project_id, &issue.title, issue.description.as_deref(),
            ).await?;

            // Apply suggestions
            if suggestion.confidence > 0.0 {
                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
                if let Some(ref p) = suggestion.suggested_priority {
                    if issue.priority == "none" {
                        sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                            .bind(p).bind(&now).bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;
                    }
                }
                if let Some(aid) = suggestion.suggested_assignee_id {
                    if issue.assignee_id.is_none() {
                        sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(aid).bind(&now).bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;
                    }
                }
                if let Some(eid) = suggestion.suggested_epic_id {
                    if issue.parent_id.is_none() {
                        sqlx::query("UPDATE issues SET parent_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(eid).bind(&now).bind(issue.id).execute(pool).await.map_err(|e| e.to_string())?;
                    }
                }
                for lid in &suggestion.suggested_label_ids {
                    let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING")
                        .bind(issue.id).bind(*lid).execute(pool).await;
                }
                notify_change();
            }
            Ok(json!(suggestion))
        }
        // AI Agent Intelligence: Decomposition
        "decompose_issue" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let text = issue.description.as_deref().unwrap_or("");
            if text.is_empty() {
                return Err("Issue has no description to decompose".to_string());
            }
            let tasks = crate::commands::decomposition::decompose_text(text);
            if tasks.is_empty() {
                return Err("No decomposable structure found".to_string());
            }
            Ok(json!(tasks))
        }
        "apply_decomposition" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let text = issue.description.as_deref().unwrap_or("");
            if text.is_empty() {
                return Err("Issue has no description to decompose".to_string());
            }
            let tasks = crate::commands::decomposition::decompose_text(text);
            if tasks.is_empty() {
                return Err("No decomposable structure found".to_string());
            }

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let mut created = Vec::new();
            for (idx, task) in tasks.iter().enumerate() {
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let (counter, prefix): (i64, String) = sqlx::query_as(
                    "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
                ).bind(issue.project_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                let ident = format!("{}-{}", prefix, counter);
                let max_pos: Option<f64> = sqlx::query_scalar(
                    "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
                ).bind(issue.project_id).bind(issue.status_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                let position = max_pos.unwrap_or(-1.0) + 1.0 + idx as f64;
                let priority = task.suggested_priority.as_deref().unwrap_or(&issue.priority);
                let sub_id: i64 = sqlx::query_scalar(
                    "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
                )
                .bind(issue.project_id).bind(&ident).bind(&task.title).bind(&task.description)
                .bind(issue.status_id).bind(priority).bind(issue.assignee_id).bind(issue.id)
                .bind(position).bind(&now).bind(&now)
                .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                let sub: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
                    .bind(sub_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
                tx.commit().await.map_err(|e| e.to_string())?;
                created.push(sub);
            }
            notify_change();
            Ok(json!(created))
        }
        // AI Agent Intelligence: Natural Language
        "create_from_text" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let text = args["text"].as_str().ok_or("text required")?;
            let status_id = args["status_id"].as_i64().ok_or("status_id required")?;

            let (title, description) = crate::commands::nl_create::parse_nl_text(text);
            if title.is_empty() {
                return Err("Could not extract a title from the text".to_string());
            }

            let suggestion = crate::commands::triage::triage_logic(
                pool, project_id, &title, Some(&description),
            ).await?;

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let priority = suggestion.suggested_priority.as_deref().unwrap_or("none");

            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            ).bind(project_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);
            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
            ).bind(project_id).bind(status_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
            )
            .bind(project_id).bind(&identifier).bind(&title).bind(&description)
            .bind(status_id).bind(priority).bind(suggestion.suggested_assignee_id)
            .bind(None::<i64>).bind(position).bind(&now).bind(&now)
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

            for lid in &suggestion.suggested_label_ids {
                let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING")
                    .bind(issue_id).bind(*lid).execute(&mut *tx).await;
            }

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            tx.commit().await.map_err(|e| e.to_string())?;
            notify_change();

            Ok(json!({
                "issue": issue,
                "triage": suggestion,
            }))
        }
        // Context Assembly
        "get_task_context" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let ctx = crate::commands::context::get_task_context_async(pool, identifier).await?;
            Ok(serde_json::to_value(&ctx).map_err(|e| e.to_string())?)
        }
        // Code Analysis
        "link_file" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let file_path = args["file_path"].as_str().ok_or("file_path required")?;
            let link_type = args.get("link_type").and_then(|v| v.as_str()).unwrap_or("related");

            let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = $1")
                .bind(identifier)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO issue_file_links (issue_id, file_path, link_type) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(issue_id)
            .bind(file_path)
            .bind(link_type)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"id": id, "issue_id": issue_id, "file_path": file_path, "link_type": link_type}))
        }
        "set_wsjf" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let bv = args["business_value"].as_i64().ok_or("business_value required")? as i32;
            let tc = args["time_criticality"].as_i64().ok_or("time_criticality required")? as i32;
            let rr = args["risk_reduction"].as_i64().ok_or("risk_reduction required")? as i32;
            let size = args["job_size"].as_i64().ok_or("job_size required")? as i32;

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            let bv = bv.max(1).min(10);
            let tc = tc.max(1).min(10);
            let rr = rr.max(1).min(10);
            let size = size.max(1).min(10);
            let score = (bv as f64 + tc as f64 + rr as f64) / size as f64;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            sqlx::query(
                "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
            )
            .bind(bv).bind(tc).bind(rr).bind(size).bind(score).bind(&now).bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"identifier": identifier, "business_value": bv, "time_criticality": tc, "risk_reduction": rr, "job_size": size, "wsjf_score": score}))
        }
        // Pipeline tools
        "list_pipelines" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let pipelines = sqlx::query_as::<_, crate::commands::pipelines::Pipeline>(
                "SELECT * FROM pipelines WHERE project_id = $1 ORDER BY created_at DESC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(pipelines))
        }
        "create_pipeline" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let name = args["name"].as_str().ok_or("name required")?;
            let description = args.get("description").and_then(|v| v.as_str());
            let stages = args.get("stages").ok_or("stages required")?;
            let now = chrono::Utc::now().to_rfc3339();
            let stages_str = serde_json::to_string(stages).unwrap_or_else(|_| "[]".to_string());

            let jb = crate::db::compat::jsonb_cast(backend);
            let id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipelines (project_id, name, description, stages, created_at, updated_at) VALUES ($1, $2, $3, $4{jb}, $5, $6) RETURNING id"
            ))
            .bind(project_id)
            .bind(name)
            .bind(description)
            .bind(&stages_str)
            .bind(&now)
            .bind(&now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"id": id, "name": name, "project_id": project_id}))
        }
        // Cost tracking
        "record_cost" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let cost_type = args["cost_type"].as_str().ok_or("cost_type required")?;
            let amount = args["amount"].as_f64().ok_or("amount required")?;
            let unit = args["unit"].as_str().ok_or("unit required")?;
            let description = args.get("description").and_then(|v| v.as_str());
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO task_costs (task_identifier, agent_id, cost_type, amount, unit, description, recorded_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
            )
            .bind(task_identifier)
            .bind(agent_id)
            .bind(cost_type)
            .bind(amount)
            .bind(unit)
            .bind(description)
            .bind(&now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!({"id": id, "task_identifier": task_identifier, "cost_type": cost_type, "amount": amount}))
        }
        "list_permissions" => {
            use crate::commands::permissions::AgentPermission;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let perms = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 ORDER BY permission_type, scope",
            )
            .bind(agent_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(perms))
        }
        "set_permission" => {
            use crate::commands::permissions::AgentPermission;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let permission_type = args["permission_type"].as_str().ok_or("permission_type required")?;
            let scope = args["scope"].as_str().ok_or("scope required")?;
            let allowed = args["allowed"].as_bool().ok_or("allowed required")?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            // Upsert
            let existing: Option<AgentPermission> = sqlx::query_as(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 AND permission_type = $2 AND scope = $3",
            )
            .bind(agent_id)
            .bind(permission_type)
            .bind(scope)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

            let id = if let Some(existing) = existing {
                sqlx::query("UPDATE agent_permissions SET allowed = $1 WHERE id = $2")
                    .bind(allowed)
                    .bind(existing.id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                existing.id
            } else {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
                )
                .bind(agent_id)
                .bind(permission_type)
                .bind(scope)
                .bind(allowed)
                .bind(&now)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?
            };

            let perm = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE id = $1",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!(link))
        }
        "file_heat_map" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i32;
            let entries = crate::commands::code_analysis::get_file_heat_map_async(pool, project_id, limit).await?;
            Ok(json!(entries))
        }
        "directory_heat_map" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let depth = args.get("depth").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
            let entries = crate::commands::code_analysis::get_directory_heat_map_async(pool, project_id, depth).await?;
            Ok(json!(entries))
        }
        "issues_for_file" => {
            let file_path = args["file_path"].as_str().ok_or("file_path required")?;
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let issues = crate::commands::code_analysis::get_issues_for_file_async(pool, file_path, project_id).await?;
            Ok(json!(issues))
        }
        // Diff Issues
        "create_issue_from_diff" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?.to_string();
            let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let file_path = args["file_path"].as_str().ok_or("file_path required")?.to_string();
            let line_range = args.get("line_range").and_then(|v| v.as_str()).map(|s| s.to_string());
            let severity = args["severity"].as_str().ok_or("severity required")?.to_string();

            let issue = crate::commands::diff_issues::create_issue_from_diff_async(
                pool,
                crate::commands::diff_issues::DiffIssueInput {
                    project_id,
                    title,
                    description,
                    file_path,
                    line_range,
                    severity,
                    status_id: None,
                    assignee_id: None,
                },
            ).await?;
            notify_change();
            Ok(json!(issue))
        }
        "marketplace_register" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let name = args["name"].as_str().ok_or("name required")?;
            let description = args.get("description").and_then(|v| v.as_str());
            let provider = args.get("provider").and_then(|v| v.as_str());
            let version = args.get("version").and_then(|v| v.as_str());
            let endpoint = args.get("endpoint").and_then(|v| v.as_str());
            let capabilities: Vec<String> = args.get("capabilities")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let caps_json = serde_json::to_string(&capabilities).unwrap_or_else(|_| "[]".to_string());
            let max_concurrent = args.get("max_concurrent").and_then(|v| v.as_i64()).unwrap_or(1);
            let max_complexity = args.get("max_complexity").and_then(|v| v.as_str()).unwrap_or("medium");
            let hourly_rate = args.get("hourly_rate").and_then(|v| v.as_f64());
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO agent_registry (agent_id, name, description, provider, version, endpoint, capabilities, max_concurrent, max_complexity, hourly_rate, registered_at, last_seen_at) VALUES ($1, $2, $3, $4, $5, $6, $7{jb}, $8, $9, $10, $11, $12) ON CONFLICT(agent_id) DO UPDATE SET name=$2, description=$3, provider=$4, version=$5, endpoint=$6, capabilities=$7{jb}, max_concurrent=$8, max_complexity=$9, hourly_rate=$10, last_seen_at=$12"
            ))
            .bind(agent_id).bind(name).bind(description).bind(provider).bind(version).bind(endpoint)
            .bind(&caps_json).bind(max_concurrent).bind(max_complexity).bind(hourly_rate).bind(&now).bind(&now)
            .execute(pool).await.map_err(|e| e.to_string())?;

            for cap in &capabilities {
                let _ = sqlx::query(
                    "INSERT INTO agent_capabilities (agent_id, capability) VALUES ($1, $2) ON CONFLICT(agent_id, capability) DO NOTHING"
                ).bind(agent_id).bind(cap).execute(pool).await;
            }

            notify_change();
            Ok(json!({"agent_id": agent_id, "name": name, "capabilities": capabilities, "status": "registered"}))
        }
        "marketplace_search" => {
            let skills: Vec<String> = args.get("skills")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let max_complexity = args.get("max_complexity").and_then(|v| v.as_str());

            let all: Vec<(String, String, String, String, Option<f64>, i64)> = sqlx::query_as(
                "SELECT agent_id, name, capabilities, max_complexity, rating, total_tasks FROM agent_registry ORDER BY rating DESC NULLS LAST"
            ).fetch_all(pool).await.map_err(|e| e.to_string())?;

            let complexity_order = |c: &str| match c { "small" => 1, "medium" => 2, "large" => 3, _ => 2 };
            let max_cx_val = complexity_order(max_complexity.unwrap_or("large"));

            let results: Vec<Value> = all.into_iter().filter(|(_, _, caps, mx, _, _)| {
                if complexity_order(mx) < max_cx_val { return false; }
                if skills.is_empty() { return true; }
                let caps_list: Vec<String> = serde_json::from_str(caps).unwrap_or_default();
                skills.iter().any(|s| caps_list.iter().any(|c| c.contains(s) || s.contains(c)))
            }).map(|(aid, name, caps, mx, rating, tasks)| {
                json!({"agent_id": aid, "name": name, "capabilities": serde_json::from_str::<Value>(&caps).unwrap_or(json!([])), "max_complexity": mx, "rating": rating, "total_tasks": tasks})
            }).collect();

            Ok(json!(results))
        }
        "find_best_agent" => {
            let task_skills: Vec<String> = args.get("task_skills")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let complexity = args["complexity"].as_str().ok_or("complexity required")?;

            let entries: Vec<(String, String, String, Option<f64>)> = sqlx::query_as(
                "SELECT agent_id, name, max_complexity, rating FROM agent_registry"
            ).fetch_all(pool).await.map_err(|e| e.to_string())?;

            let complexity_order = |c: &str| match c { "small" => 1, "medium" => 2, "large" => 3, _ => 2 };
            let target_cx = complexity_order(complexity);

            let mut matches = Vec::new();
            for (agent_id, name, max_cx, rating) in entries {
                if complexity_order(&max_cx) < target_cx { continue; }
                let caps: Vec<(String, f64)> = sqlx::query_as(
                    "SELECT capability, proficiency FROM agent_capabilities WHERE agent_id = $1"
                ).bind(&agent_id).fetch_all(pool).await.map_err(|e| e.to_string())?;

                let mut matched = Vec::new();
                let mut total_prof = 0.0;
                for skill in &task_skills {
                    if let Some((_, prof)) = caps.iter().find(|(c, _)| c.contains(skill) || skill.contains(c)) {
                        matched.push(skill.clone());
                        total_prof += prof;
                    }
                }
                if matched.is_empty() && !task_skills.is_empty() { continue; }

                let avg_prof = if matched.is_empty() { 0.5 } else { total_prof / matched.len() as f64 };
                let skill_ratio = if task_skills.is_empty() { 1.0 } else { matched.len() as f64 / task_skills.len() as f64 };
                let r = rating.unwrap_or(0.5);
                let score = skill_ratio * 0.4 + avg_prof * 0.3 + r * 0.3;

                matches.push(json!({"agent_id": agent_id, "name": name, "score": score, "matched_skills": matched, "avg_proficiency": avg_prof, "rating": rating}));
            }

            matches.sort_by(|a, b| b["score"].as_f64().unwrap_or(0.0).partial_cmp(&a["score"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));
            Ok(json!(matches))
        }
        "marketplace_list" => {
            let entries: Vec<(String, String, Option<String>, Option<String>, String, Option<f64>, i64)> = sqlx::query_as(
                "SELECT agent_id, name, description, provider, capabilities, rating, total_tasks FROM agent_registry ORDER BY rating DESC NULLS LAST"
            ).fetch_all(pool).await.map_err(|e| e.to_string())?;

            let results: Vec<Value> = entries.into_iter().map(|(aid, name, desc, provider, caps, rating, tasks)| {
                json!({"agent_id": aid, "name": name, "description": desc, "provider": provider, "capabilities": serde_json::from_str::<Value>(&caps).unwrap_or(json!([])), "rating": rating, "total_tasks": tasks})
            }).collect();

            Ok(json!(results))
        }
        // Handoff Notes
        "create_handoff" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let from_agent_id = args["from_agent_id"].as_str().ok_or("from_agent_id required")?;
            let to_agent_id = args.get("to_agent_id").and_then(|v| v.as_str());
            let note_type = args["note_type"].as_str().ok_or("note_type required")?;
            let summary = args["summary"].as_str().ok_or("summary required")?;
            let details = args.get("details").and_then(|v| v.as_str());
            let files_changed = args.get("files_changed").cloned().unwrap_or(json!([]));
            let risks = args.get("risks").cloned().unwrap_or(json!([]));
            let test_results = args.get("test_results").cloned();

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let files_str = serde_json::to_string(&files_changed).unwrap_or_else(|_| "[]".to_string());
            let risks_str = serde_json::to_string(&risks).unwrap_or_else(|_| "[]".to_string());
            let test_str = test_results.as_ref().map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO handoff_notes (task_identifier, from_agent_id, to_agent_id, note_type, summary, details, files_changed, risks, test_results, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '{}', $10) RETURNING id"
            )
            .bind(task_identifier)
            .bind(from_agent_id)
            .bind(to_agent_id)
            .bind(note_type)
            .bind(summary)
            .bind(details)
            .bind(&files_str)
            .bind(&risks_str)
            .bind(&test_str)
            .bind(&now)
            Ok(json!(pipeline))
        }
        "trigger_pipeline" => {
            let pipeline_id = args["pipeline_id"].as_i64().ok_or("pipeline_id required")?;
            let trigger_issue_id = args.get("trigger_issue_id").and_then(|v| v.as_i64());
            let context = args.get("context").and_then(|v| v.as_str()).map(String::from);

            let pipeline = sqlx::query_as::<_, crate::commands::pipelines::Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(pipeline_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            if !pipeline.enabled {
                return Err("Pipeline is disabled".to_string());
            }

            let stages: Vec<Value> = serde_json::from_str(&pipeline.stages).unwrap_or_default();
            if stages.is_empty() {
                return Err("Pipeline has no stages".to_string());
            }

            let now = chrono::Utc::now().to_rfc3339();
            let initial_context = context.unwrap_or_else(|| "{}".to_string());

            let trigger_title = if let Some(tid) = trigger_issue_id {
                sqlx::query_scalar::<_, String>("SELECT title FROM issues WHERE id = $1")
                    .bind(tid).fetch_optional(pool).await.map_err(|e| e.to_string())?
                    .unwrap_or_default()
            } else { String::new() };
            let trigger_description = if let Some(tid) = trigger_issue_id {
                sqlx::query_scalar::<_, Option<String>>("SELECT description FROM issues WHERE id = $1")
                    .bind(tid).fetch_optional(pool).await.map_err(|e| e.to_string())?
                    .flatten().unwrap_or_default()
            } else { String::new() };

            let jb = crate::db::compat::jsonb_cast(backend);
            let run_id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipeline_runs (pipeline_id, trigger_issue_id, status, current_stage, stage_tasks, context, started_at) VALUES ($1, $2, 'running', 0, '[]'{jb}, $3{jb}, $4) RETURNING id"
            ))
            .bind(pipeline_id)
            .bind(trigger_issue_id)
            .bind(&initial_context)
            .bind(&now)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            // Create first stage task (inline)
            let stage = &stages[0];
            let stage_name = stage["name"].as_str().unwrap_or("Stage");
            let task_type = stage["task_type"].as_str().unwrap_or("implementation");
            let skills: Vec<String> = stage["required_skills"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let complexity = stage["max_complexity"].as_str().unwrap_or("medium");
            let timeout = stage["timeout_minutes"].as_i64().unwrap_or(30);
            let sc: Vec<String> = stage["success_criteria"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let title_tmpl = stage["title_template"].as_str().unwrap_or("{{pipeline.name}}: {{stage.name}}");
            let obj_tmpl = stage["objective_template"].as_str().unwrap_or("Execute stage: {{stage.name}}");
            let title = title_tmpl.replace("{{pipeline.name}}", &pipeline.name).replace("{{stage.name}}", stage_name).replace("{{trigger.title}}", &trigger_title);
            let objective = obj_tmpl.replace("{{pipeline.name}}", &pipeline.name).replace("{{stage.name}}", stage_name).replace("{{trigger.title}}", &trigger_title).replace("{{trigger.description}}", &trigger_description);

            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
            ).bind(pipeline.project_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let task_identifier = format!("{}-{}", prefix, counter);

            let sid: i64 = sqlx::query_scalar(
                "SELECT id FROM statuses WHERE project_id = $1 AND category = 'unstarted' ORDER BY position ASC LIMIT 1",
            ).bind(pipeline.project_id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
            ).bind(pipeline.project_id).bind(sid).fetch_one(pool).await.map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let desc = format!("Pipeline run #{} - Stage 1 ({})\n\n{}", run_id, stage_name, objective);
            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'medium', $6, $7, $8) RETURNING id",
            ).bind(pipeline.project_id).bind(&task_identifier).bind(&title).bind(&desc).bind(sid).bind(position).bind(&now).bind(&now)
            .fetch_one(pool).await.map_err(|e| e.to_string())?;

            let ctx = json!({ "files": [], "related_tasks": [], "prior_attempts": [], "pipeline": { "run_id": run_id, "stage_index": 0, "pipeline_name": &pipeline.name, "stage_name": stage_name } });
            let skills_json = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());
            let sc_json = serde_json::to_string(&sc).unwrap_or_else(|_| "[]".to_string());

            sqlx::query(&format!(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4{jb}, '[]'{jb}, $5{jb}, $6{jb}, $7, $8, 0)"
            )).bind(issue_id).bind(task_type).bind(&objective).bind(&ctx.to_string()).bind(&sc_json).bind(&skills_json).bind(complexity).bind(timeout)
            .execute(pool).await.map_err(|e| e.to_string())?;

            let stage_tasks = json!([{ "stage_index": 0, "task_identifier": task_identifier, "status": "queued" }]);
            sqlx::query(&format!("UPDATE pipeline_runs SET stage_tasks = $1{jb} WHERE id = $2"))
                .bind(stage_tasks.to_string()).bind(run_id).execute(pool).await.map_err(|e| e.to_string())?;
            sqlx::query("UPDATE pipelines SET total_runs = total_runs + 1, updated_at = $1 WHERE id = $2")
                .bind(&now).bind(pipeline_id).execute(pool).await.map_err(|e| e.to_string())?;

            let run = sqlx::query_as::<_, crate::commands::pipelines::PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            ).bind(run_id).fetch_one(pool).await.map_err(|e| e.to_string())?;

            notify_change();
            Ok(json!(run))
        }
        "advance_pipeline" => {
            let run_id = args["run_id"].as_i64().ok_or("run_id required")?;
            let run = crate::commands::pipelines::advance_pipeline_internal(pool, backend, run_id)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(run))
        }
        "pipeline_status" => {
            let run_id = args["run_id"].as_i64().ok_or("run_id required")?;
            let run = sqlx::query_as::<_, crate::commands::pipelines::PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            let note = sqlx::query_as::<_, agent::HandoffNote>("SELECT * FROM handoff_notes WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(note))
        }
        "get_handoff_notes" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let agent_id = args.get("agent_id").and_then(|v| v.as_str());

            let notes = if let Some(aid) = agent_id {
                sqlx::query_as::<_, agent::HandoffNote>(
                    "SELECT * FROM handoff_notes WHERE task_identifier = $1 AND (to_agent_id IS NULL OR to_agent_id = $2) ORDER BY created_at ASC"
                )
                .bind(task_identifier)
                .bind(aid)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?
            } else {
                sqlx::query_as::<_, agent::HandoffNote>(
                    "SELECT * FROM handoff_notes WHERE task_identifier = $1 ORDER BY created_at ASC"
                )
                .bind(task_identifier)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?
            };
            Ok(json!(notes))
        }
        "record_learning" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let outcome = args["outcome"].as_str().ok_or("outcome required")?;
            let approach_summary = args["approach_summary"].as_str().ok_or("approach_summary required")?;
            let key_insight = args.get("key_insight").and_then(|v| v.as_str());
            let pitfalls = args.get("pitfalls").cloned().unwrap_or(json!([]));
            let effective_patterns = args.get("effective_patterns").cloned().unwrap_or(json!([]));
            let relevant_files = args.get("relevant_files").cloned().unwrap_or(json!([]));
            let tags = args.get("tags").cloned().unwrap_or(json!([]));

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let pitfalls_str = serde_json::to_string(&pitfalls).unwrap_or_else(|_| "[]".to_string());
            let patterns_str = serde_json::to_string(&effective_patterns).unwrap_or_else(|_| "[]".to_string());
            let files_str = serde_json::to_string(&relevant_files).unwrap_or_else(|_| "[]".to_string());
            let tags_str = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO task_learnings (task_identifier, agent_id, outcome, approach_summary, key_insight, pitfalls, effective_patterns, relevant_files, tags, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id"
            )
            .bind(task_identifier)
            .bind(agent_id)
            .bind(outcome)
            .bind(approach_summary)
            .bind(key_insight)
            .bind(&pitfalls_str)
            .bind(&patterns_str)
            .bind(&files_str)
            .bind(&tags_str)
            .bind(&now)
            let pipeline = sqlx::query_as::<_, crate::commands::pipelines::Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(run.pipeline_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            let learning = sqlx::query_as::<_, agent::TaskLearning>("SELECT * FROM task_learnings WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(learning))
        }
        "find_similar_learnings" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let title = args["title"].as_str().ok_or("title required")?;
            let description = args.get("description").and_then(|v| v.as_str());
            let tags: Vec<String> = args.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5);

            // Fetch all learnings for the project
            let learnings = sqlx::query_as::<_, agent::TaskLearning>(
                "SELECT tl.* FROM task_learnings tl JOIN issues i ON tl.task_identifier = i.identifier WHERE i.project_id = $1 ORDER BY tl.created_at DESC"
            let bv = bv.max(1).min(10);
            let tc = tc.max(1).min(10);
            let rr = rr.max(1).min(10);
            let size = size.max(1).min(10);
            let score = (bv as f64 + tc as f64 + rr as f64) / size as f64;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            sqlx::query(
                "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
            )
            .bind(bv).bind(tc).bind(rr).bind(size).bind(score).bind(&now).bind(issue.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            Ok(json!({
                "identifier": identifier,
                "business_value": bv,
                "time_criticality": tc,
                "risk_reduction": rr,
                "job_size": size,
                "wsjf_score": score
            }))
        }
        "auto_score" => {
            let identifier = args["identifier"].as_str().ok_or("identifier required")?;
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(identifier)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

            let result = crate::commands::scoring::auto_score_issue_standalone(pool, issue.id)
                .await
                .map_err(|e| e.to_string())?;

            Ok(json!(result))
        }
        "ranked_backlog" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.project_id = $1 AND s.category = 'unstarted' AND i.wsjf_score IS NOT NULL ORDER BY i.wsjf_score DESC"
            // Update budget spent
            if unit == "dollars" {
                let project_id: Option<i64> = sqlx::query_scalar(
                    "SELECT project_id FROM issues WHERE identifier = $1"
                )
                .bind(task_identifier)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?;
                if let Some(pid) = project_id {
                    let _ = sqlx::query("UPDATE cost_budgets SET spent = spent + $1 WHERE project_id = $2")
                        .bind(amount)
                        .bind(pid)
                        .execute(pool)
                        .await;
                }
            }

            notify_change();
            Ok(json!({"id": id, "task_identifier": task_identifier, "amount": amount, "unit": unit}))
        }
        "task_cost" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let costs = sqlx::query_as::<_, crate::commands::costs::TaskCost>(
                "SELECT * FROM task_costs WHERE task_identifier = $1 ORDER BY recorded_at ASC"
            )
            .bind(task_identifier)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let mut total_dollars = 0.0f64;
            let mut total_minutes = 0.0f64;
            let mut total_tokens = 0i64;
            for c in &costs {
                if c.unit == "dollars" { total_dollars += c.amount; }
                if c.cost_type == "compute_time" && c.unit == "minutes" { total_minutes += c.amount; }
                if c.cost_type == "api_tokens" && c.unit == "tokens" { total_tokens += c.amount as i64; }
            }
            Ok(json!({
                "task_identifier": task_identifier,
                "total_cost_dollars": total_dollars,
                "total_compute_minutes": total_minutes,
                "total_tokens": total_tokens,
                "entries": costs,
            }))
        }
        "project_costs" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let total: Option<f64> = sqlx::query_scalar(
                "SELECT SUM(tc.amount) FROM task_costs tc JOIN issues i ON tc.task_identifier = i.identifier WHERE i.project_id = $1 AND tc.unit = 'dollars'"
            )
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

            let budgets = sqlx::query_as::<_, crate::commands::costs::CostBudget>(
                "SELECT * FROM cost_budgets WHERE project_id = $1"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let search_tags: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            let title_words: Vec<String> = title.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .map(|w| w.to_string())
                .collect();
            let desc_words: Vec<String> = description.unwrap_or("")
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .map(|w| w.to_string())
                .collect();
            let all_words: Vec<String> = title_words.into_iter().chain(desc_words).collect();

            let mut results: Vec<Value> = Vec::new();
            for learning in learnings {
                let ltags: Vec<String> = serde_json::from_str(&learning.tags)
                    .unwrap_or_else(|_| Vec::<String>::new())
                    .iter().map(|t: &String| t.to_lowercase()).collect();
                let lwords: Vec<String> = learning.approach_summary.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() > 2)
                    .map(|w| w.to_string())
                    .collect();

                let tag_sim = jaccard_sim(&search_tags, &ltags);
                let word_sim = jaccard_sim(&all_words, &lwords);
                let score = tag_sim * 0.6 + word_sim * 0.4;

                if score > 0.0 {
                    let info: Option<(String, String)> = sqlx::query_as(
                        "SELECT title, identifier FROM issues WHERE identifier = $1"
                    )
                    .bind(&learning.task_identifier)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                    let (issue_title, issue_identifier) = info
                        .unwrap_or_else(|| (String::new(), learning.task_identifier.clone()));
                    results.push(json!({
                        "learning": learning,
                        "similarity_score": score,
                        "issue_title": issue_title,
                        "issue_identifier": issue_identifier,
                    }));
                }
            }
            results.sort_by(|a, b| {
                let sa = a["similarity_score"].as_f64().unwrap_or(0.0);
                let sb = b["similarity_score"].as_f64().unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(limit as usize);
            Ok(json!(results))
        }
        "get_task_learnings" => {
            let task_identifier = args["task_identifier"].as_str().ok_or("task_identifier required")?;
            let learnings = sqlx::query_as::<_, agent::TaskLearning>(
                "SELECT * FROM task_learnings WHERE task_identifier = $1 ORDER BY created_at DESC"
            )
            .bind(task_identifier)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            Ok(json!(learnings))
            let ranked: Vec<Value> = issues.iter().map(|i| json!({
                "identifier": i.identifier,
                "title": i.title,
                "wsjf_score": i.wsjf_score,
                "business_value": i.business_value,
                "time_criticality": i.time_criticality,
                "risk_reduction": i.risk_reduction,
                "job_size": i.job_size,
                "priority": i.priority,
            })).collect();

            Ok(json!(ranked))
        }
        "auto_score_project" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let unscored = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND wsjf_score IS NULL"
            Ok(json!({
                "project_id": project_id,
                "total_cost": total.unwrap_or(0.0),
                "budgets": budgets,
            }))
        }
        "check_budget" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let budgets = sqlx::query_as::<_, crate::commands::costs::CostBudget>(
                "SELECT * FROM cost_budgets WHERE project_id = $1"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let alerts: Vec<Value> = budgets.iter().filter_map(|b| {
                let pct = if b.amount > 0.0 { b.spent / b.amount } else { 0.0 };
                let threshold = b.alert_threshold.unwrap_or(0.8);
                if pct >= threshold {
                    Some(json!({"budget_id": b.id, "budget_type": b.budget_type, "amount": b.amount, "spent": b.spent, "percentage": pct, "alert": true}))
                } else {
                    None
                }
            }).collect();
            Ok(json!({"alerts": alerts, "all_budgets": budgets}))
        }
        // SLA engine
        "check_sla" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let statuses = crate::commands::sla::enforce_sla_async(pool, project_id).await;
            // Just check compliance without enforcing - recompute
            let policies = sqlx::query_as::<_, crate::commands::sla::SlaPolicy>(
                "SELECT * FROM sla_policies WHERE project_id = $1 AND enabled = 1"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let issues_list = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.project_id = $1 AND s.category = 'started'"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let now = chrono::Utc::now();
            let mut results = Vec::new();
            for issue in &issues_list {
                for policy in &policies {
                    if let Some(ref pf) = policy.priority_filter {
                        if !pf.is_empty() && pf != &issue.priority { continue; }
                    }
                    let start = chrono::NaiveDateTime::parse_from_str(&issue.created_at, "%Y-%m-%d %H:%M:%SZ")
                        .unwrap_or_else(|_| now.naive_utc());
                    let elapsed = now.naive_utc().signed_duration_since(start).num_minutes() as f64;
                    let remaining = policy.breach_minutes as f64 - elapsed;
                    let status = if elapsed >= policy.breach_minutes as f64 { "breached" }
                        else if elapsed >= (policy.breach_minutes - policy.warning_minutes) as f64 { "warning" }
                        else { "ok" };
                    results.push(json!({
                        "issue": issue.identifier, "title": issue.title,
                        "policy": policy.name, "status": status,
                        "elapsed_minutes": elapsed, "remaining_minutes": remaining.max(0.0),
                    }));
                }
            }
            let _ = statuses; // consume the result
            Ok(json!(results))
        }
        "enforce_sla" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let events = crate::commands::sla::enforce_sla_async(pool, project_id).await
                .map_err(|e| e.to_string())?;
            notify_change();
            Ok(json!(events))
        }
        "sla_status" => {
            let project_id = args["project_id"].as_i64().ok_or("project_id required")?;
            let policies = sqlx::query_as::<_, crate::commands::sla::SlaPolicy>(
                "SELECT * FROM sla_policies WHERE project_id = $1"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            let events = sqlx::query_as::<_, crate::commands::sla::SlaEvent>(
                "SELECT se.* FROM sla_events se JOIN sla_policies sp ON se.sla_policy_id = sp.id WHERE sp.project_id = $1 ORDER BY se.created_at DESC LIMIT 50"
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

            Ok(json!({"policies": policies, "recent_events": events}))
        }
        "check_permission" => {
            let agent_id = args["agent_id"].as_str().ok_or("agent_id required")?;
            let permission_type = args["permission_type"].as_str().ok_or("permission_type required")?;
            let scope = args["scope"].as_str().ok_or("scope required")?;

            let result = crate::commands::permissions::check_permission_async(
                pool, agent_id, permission_type, scope,
            )
            .await
            .map_err(|e| e.to_string())?;

            Ok(json!(result))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Jaccard similarity helper for MCP
fn jaccard_sim(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() { return 0.0; }
    let sa: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let sb: std::collections::HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let inter = sa.intersection(&sb).count() as f64;
    let union = sa.union(&sb).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

async fn handle_resource_read(pool: &AnyPool, uri: &str) -> Result<Value, String> {
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
        let issue =
            sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
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

pub async fn run(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
) -> Result<(), Box<dyn std::error::Error>> {
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
                let resp = error(Value::Null, -32700, &format!("Parse error: {}", e));
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
                let tool_name =
                    params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                match handle_tool_call(pool, backend, tool_name, &arguments).await {
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
            "resources/list" => success(id, json!({ "resources": resources_list() })),
            "resources/read" => {
                let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                match handle_resource_read(pool, uri).await {
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
