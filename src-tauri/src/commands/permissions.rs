use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentPermission {
    pub id: i64,
    pub agent_id: String,
    pub permission_type: String,
    pub scope: String,
    pub allowed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionPreset {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub permissions: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub matched_rule: Option<AgentPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetPermissionEntry {
    pub permission_type: String,
    pub scope: String,
    pub allowed: bool,
}

/// Glob-style pattern matching for file paths.
/// Supports `*` (any segment chars), `**` (any path), `!` prefix (negation handled at caller level).
fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.replace("\\", "/");
    let path = path.replace("\\", "/");

    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.splitn(2, "**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');
            if !prefix.is_empty() && !path.starts_with(prefix) {
                return false;
            }
            if suffix.is_empty() {
                return true;
            }
            // Check if any suffix of path matches the suffix pattern
            for (i, _) in path.char_indices() {
                if simple_glob_matches(suffix, &path[i..]) {
                    return true;
                }
            }
            return false;
        }
    }

    simple_glob_matches(&pattern, &path)
}

/// Simple glob matching with `*` and `?`.
fn simple_glob_matches(pattern: &str, text: &str) -> bool {
    let mut star_p = None;
    let mut star_t = None;
    let mut p_pos = 0;
    let mut t_pos = 0;

    let p_chars: Vec<char> = pattern.chars().collect();
    let t_chars: Vec<char> = text.chars().collect();

    while t_pos < t_chars.len() {
        if p_pos < p_chars.len() && (p_chars[p_pos] == '?' || p_chars[p_pos] == t_chars[t_pos]) {
            p_pos += 1;
            t_pos += 1;
        } else if p_pos < p_chars.len() && p_chars[p_pos] == '*' {
            star_p = Some(p_pos);
            star_t = Some(t_pos);
            p_pos += 1;
        } else if let Some(sp) = star_p {
            p_pos = sp + 1;
            let st = star_t.unwrap() + 1;
            star_t = Some(st);
            t_pos = st;
        } else {
            return false;
        }
    }

    while p_pos < p_chars.len() && p_chars[p_pos] == '*' {
        p_pos += 1;
    }

    p_pos == p_chars.len()
}

/// Core permission checking logic.
/// 1. If no rules for this agent+type, default ALLOW
/// 2. If explicit DENY exists, DENY (deny takes precedence)
/// 3. If explicit ALLOW exists, ALLOW
/// 4. If rules exist for this type but none match scope, DENY (whitelist mode)
fn evaluate_permissions(
    rules: &[AgentPermission],
    permission_type: &str,
    scope: &str,
) -> PermissionCheckResult {
    let type_rules: Vec<&AgentPermission> = rules
        .iter()
        .filter(|r| r.permission_type == permission_type)
        .collect();

    // No rules for this type => default allow
    if type_rules.is_empty() {
        return PermissionCheckResult {
            allowed: true,
            reason: Some("No rules configured, default allow".to_string()),
            matched_rule: None,
        };
    }

    // Check for explicit deny (exact match or glob match)
    for rule in &type_rules {
        let matches = if permission_type == "file_access" {
            let pattern = rule.scope.trim_start_matches('!');
            glob_matches(pattern, scope)
        } else {
            rule.scope == scope || rule.scope == "*"
        };

        if matches && !rule.allowed {
            return PermissionCheckResult {
                allowed: false,
                reason: Some(format!("Denied by rule: {} scope={}", permission_type, rule.scope)),
                matched_rule: Some((*rule).clone()),
            };
        }
    }

    // Check for explicit allow
    for rule in &type_rules {
        let matches = if permission_type == "file_access" {
            let pattern = rule.scope.trim_start_matches('!');
            glob_matches(pattern, scope)
        } else {
            rule.scope == scope || rule.scope == "*"
        };

        if matches && rule.allowed {
            return PermissionCheckResult {
                allowed: true,
                reason: Some(format!("Allowed by rule: {} scope={}", permission_type, rule.scope)),
                matched_rule: Some((*rule).clone()),
            };
        }
    }

    // Rules exist but none matched => whitelist mode, deny
    PermissionCheckResult {
        allowed: false,
        reason: Some(format!(
            "No matching rule found for {} scope={}; {} rules exist (whitelist mode)",
            permission_type, scope, type_rules.len()
        )),
        matched_rule: None,
    }
}

// ---- Tauri Commands ----

#[tauri::command]
pub fn list_agent_permissions(
    state: State<AppState>,
    agent_id: String,
) -> Result<Vec<AgentPermission>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 ORDER BY permission_type, scope",
            )
            .bind(&agent_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn set_agent_permission(
    app: tauri::AppHandle,
    state: State<AppState>,
    agent_id: String,
    permission_type: String,
    scope: String,
    allowed: bool,
) -> Result<AgentPermission, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%SZ")
                .to_string();

            // Upsert: if same agent+type+scope exists, update; otherwise insert
            let existing: Option<AgentPermission> = sqlx::query_as(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 AND permission_type = $2 AND scope = $3",
            )
            .bind(&agent_id)
            .bind(&permission_type)
            .bind(&scope)
            .fetch_optional(&state.pool)
            .await?;

            let id = if let Some(existing) = existing {
                sqlx::query(
                    "UPDATE agent_permissions SET allowed = $1 WHERE id = $2",
                )
                .bind(allowed)
                .bind(existing.id)
                .execute(&state.pool)
                .await?;
                existing.id
            } else {
                sqlx::query_scalar(
                    "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
                )
                .bind(&agent_id)
                .bind(&permission_type)
                .bind(&scope)
                .bind(allowed)
                .bind(&now)
                .fetch_one(&state.pool)
                .await?
            };

            let perm = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(perm)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn remove_agent_permission(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            sqlx::query("DELETE FROM agent_permissions WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            let _ = app.emit("db-changed", ());
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn clear_agent_permissions(
    app: tauri::AppHandle,
    state: State<AppState>,
    agent_id: String,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            sqlx::query("DELETE FROM agent_permissions WHERE agent_id = $1")
                .bind(&agent_id)
                .execute(&state.pool)
                .await?;
            let _ = app.emit("db-changed", ());
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

// ---- Presets ----

#[tauri::command]
pub fn list_permission_presets(state: State<AppState>) -> Result<Vec<PermissionPreset>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, PermissionPreset>(
                "SELECT * FROM agent_permission_presets ORDER BY name",
            )
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn create_permission_preset(
    app: tauri::AppHandle,
    state: State<AppState>,
    name: String,
    description: Option<String>,
    permissions: String,
) -> Result<PermissionPreset, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%SZ")
                .to_string();

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO agent_permission_presets (name, description, permissions, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(&name)
            .bind(&description)
            .bind(&permissions)
            .bind(&now)
            .fetch_one(&state.pool)
            .await?;

            let preset = sqlx::query_as::<_, PermissionPreset>(
                "SELECT * FROM agent_permission_presets WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(preset)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn apply_preset_to_agent(
    app: tauri::AppHandle,
    state: State<AppState>,
    agent_id: String,
    preset_id: i64,
) -> Result<Vec<AgentPermission>, String> {
    state
        .rt
        .block_on(async {
            let preset = sqlx::query_as::<_, PermissionPreset>(
                "SELECT * FROM agent_permission_presets WHERE id = $1",
            )
            .bind(preset_id)
            .fetch_one(&state.pool)
            .await?;

            let entries: Vec<PresetPermissionEntry> =
                serde_json::from_str(&preset.permissions).map_err(|e| {
                    sqlx::Error::Protocol(format!("Invalid preset permissions JSON: {}", e))
                })?;

            let now = chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%SZ")
                .to_string();

            // Clear existing permissions for this agent, then apply preset
            sqlx::query("DELETE FROM agent_permissions WHERE agent_id = $1")
                .bind(&agent_id)
                .execute(&state.pool)
                .await?;

            for entry in &entries {
                sqlx::query(
                    "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&agent_id)
                .bind(&entry.permission_type)
                .bind(&entry.scope)
                .bind(entry.allowed)
                .bind(&now)
                .execute(&state.pool)
                .await?;
            }

            let perms = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 ORDER BY permission_type, scope",
            )
            .bind(&agent_id)
            .fetch_all(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(perms)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_permission_preset(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            sqlx::query("DELETE FROM agent_permission_presets WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            let _ = app.emit("db-changed", ());
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

// ---- Permission Checking ----

#[tauri::command]
pub fn check_permission(
    state: State<AppState>,
    agent_id: String,
    permission_type: String,
    scope: String,
) -> Result<PermissionCheckResult, String> {
    state
        .rt
        .block_on(async {
            let rules = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1",
            )
            .bind(&agent_id)
            .fetch_all(&state.pool)
            .await?;

            Ok(evaluate_permissions(&rules, &permission_type, &scope))
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn check_file_access(
    state: State<AppState>,
    agent_id: String,
    file_path: String,
) -> Result<PermissionCheckResult, String> {
    state
        .rt
        .block_on(async {
            let rules = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1",
            )
            .bind(&agent_id)
            .fetch_all(&state.pool)
            .await?;

            Ok(evaluate_permissions(&rules, "file_access", &file_path))
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn check_task_claim(
    state: State<AppState>,
    agent_id: String,
    task_identifier: String,
) -> Result<PermissionCheckResult, String> {
    state
        .rt
        .block_on(async {
            let rules = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1",
            )
            .bind(&agent_id)
            .fetch_all(&state.pool)
            .await?;

            // Get the task's project_id and type
            let issue_row: Option<(i64,)> = sqlx::query_as(
                "SELECT project_id FROM issues WHERE identifier = $1",
            )
            .bind(&task_identifier)
            .fetch_optional(&state.pool)
            .await?;

            let project_id = match issue_row {
                Some((pid,)) => pid,
                None => {
                    return Ok(PermissionCheckResult {
                        allowed: false,
                        reason: Some(format!("Task {} not found", task_identifier)),
                        matched_rule: None,
                    });
                }
            };

            // Check project_access
            let project_check =
                evaluate_permissions(&rules, "project_access", &project_id.to_string());
            if !project_check.allowed {
                return Ok(project_check);
            }

            // Check task_type
            let task_type: Option<(String,)> = sqlx::query_as(
                "SELECT type FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.identifier = $1",
            )
            .bind(&task_identifier)
            .fetch_optional(&state.pool)
            .await?;

            if let Some((tt,)) = task_type {
                let type_check = evaluate_permissions(&rules, "task_type", &tt);
                if !type_check.allowed {
                    return Ok(type_check);
                }
            }

            Ok(PermissionCheckResult {
                allowed: true,
                reason: Some("All permission checks passed".to_string()),
                matched_rule: None,
            })
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

/// Standalone async function for use in other commands (next_task, log_task_activity).
pub async fn check_permission_async(
    pool: &sqlx::AnyPool,
    agent_id: &str,
    permission_type: &str,
    scope: &str,
) -> Result<PermissionCheckResult, sqlx::Error> {
    let rules = sqlx::query_as::<_, AgentPermission>(
        "SELECT * FROM agent_permissions WHERE agent_id = $1",
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await?;

    Ok(evaluate_permissions(&rules, permission_type, scope))
}

/// Log a permission denial to execution_logs.
pub async fn log_permission_denied(
    pool: &sqlx::AnyPool,
    issue_id: i64,
    agent_id: &str,
    message: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();
    sqlx::query(
        "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, 0, 'permission_denied', $3, $4)",
    )
    .bind(issue_id)
    .bind(agent_id)
    .bind(message)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}
