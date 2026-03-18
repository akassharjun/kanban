use crate::models::{Issue, IssueFileLink};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeatEntry {
    pub file_path: String,
    pub issue_count: i64,
    pub bug_count: i64,
    pub last_issue_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryHeatEntry {
    pub directory: String,
    pub issue_count: i64,
    pub file_count: i64,
}

#[derive(Deserialize)]
pub struct LinkFileInput {
    pub issue_id: i64,
    pub file_path: String,
    pub link_type: Option<String>,
}

#[tauri::command]
pub fn link_file_to_issue(state: State<AppState>, input: LinkFileInput) -> Result<IssueFileLink, String> {
    let link_type = input.link_type.unwrap_or_else(|| "related".to_string());
    if !["related", "cause", "fix"].contains(&link_type.as_str()) {
        return Err("link_type must be one of: related, cause, fix".to_string());
    }
    state.rt.block_on(async {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO issue_file_links (issue_id, file_path, link_type) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(input.issue_id)
        .bind(&input.file_path)
        .bind(&link_type)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, IssueFileLink>("SELECT * FROM issue_file_links WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    })
    .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn unlink_file_from_issue(
    state: State<AppState>,
    issue_id: i64,
    file_path: String,
) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM issue_file_links WHERE issue_id = $1 AND file_path = $2")
            .bind(issue_id)
            .bind(&file_path)
            .execute(&state.pool)
            .await?;
        Ok(())
    })
    .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_file_links(state: State<AppState>, issue_id: i64) -> Result<Vec<IssueFileLink>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, IssueFileLink>(
            "SELECT * FROM issue_file_links WHERE issue_id = $1 ORDER BY created_at ASC",
        )
        .bind(issue_id)
        .fetch_all(&state.pool)
        .await
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_heat_map(
    state: State<AppState>,
    project_id: i64,
    limit: i32,
) -> Result<Vec<FileHeatEntry>, String> {
    state.rt.block_on(async {
        get_file_heat_map_async(&state.pool, project_id, limit).await
    })
}

pub async fn get_file_heat_map_async(
    pool: &sqlx::AnyPool,
    project_id: i64,
    limit: i32,
) -> Result<Vec<FileHeatEntry>, String> {
    let rows: Vec<(String, i64, i64, String)> = sqlx::query_as(
        "SELECT ifl.file_path,
                COUNT(DISTINCT ifl.issue_id) as issue_count,
                COUNT(DISTINCT CASE WHEN l.name = 'bug' THEN ifl.issue_id END) as bug_count,
                MAX(ifl.created_at) as last_issue_at
         FROM issue_file_links ifl
         JOIN issues i ON i.id = ifl.issue_id
         LEFT JOIN issue_labels il ON il.issue_id = i.id
         LEFT JOIN labels l ON l.id = il.label_id AND l.name = 'bug'
         WHERE i.project_id = $1
         GROUP BY ifl.file_path
         ORDER BY issue_count DESC
         LIMIT $2",
    )
    .bind(project_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(file_path, issue_count, bug_count, last_issue_at)| FileHeatEntry {
            file_path,
            issue_count,
            bug_count,
            last_issue_at,
        })
        .collect())
}

#[tauri::command]
pub fn get_directory_heat_map(
    state: State<AppState>,
    project_id: i64,
    depth: i32,
) -> Result<Vec<DirectoryHeatEntry>, String> {
    state.rt.block_on(async {
        get_directory_heat_map_async(&state.pool, project_id, depth).await
    })
}

pub async fn get_directory_heat_map_async(
    pool: &sqlx::AnyPool,
    project_id: i64,
    depth: i32,
) -> Result<Vec<DirectoryHeatEntry>, String> {
    // Get all file links for the project
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT ifl.file_path
         FROM issue_file_links ifl
         JOIN issues i ON i.id = ifl.issue_id
         WHERE i.project_id = $1",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Aggregate by directory at given depth
    let mut dir_files: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for (file_path,) in &rows {
        let parts: Vec<&str> = file_path.split('/').collect();
        let dir = if parts.len() <= depth as usize {
            parts[..parts.len().saturating_sub(1)].join("/")
        } else {
            parts[..depth as usize].join("/")
        };
        let dir = if dir.is_empty() { ".".to_string() } else { dir };
        dir_files
            .entry(dir)
            .or_default()
            .insert(file_path.clone());
    }

    // Now count issues per directory
    let mut result: Vec<DirectoryHeatEntry> = Vec::new();
    for (directory, files) in &dir_files {
        // Count unique issues linked to files in this directory
        let file_count = files.len() as i64;
        let mut issue_ids = std::collections::HashSet::new();
        for file_path in files {
            let ids: Vec<(i64,)> = sqlx::query_as(
                "SELECT DISTINCT ifl.issue_id FROM issue_file_links ifl JOIN issues i ON i.id = ifl.issue_id WHERE ifl.file_path = $1 AND i.project_id = $2",
            )
            .bind(file_path)
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
            for (id,) in ids {
                issue_ids.insert(id);
            }
        }
        result.push(DirectoryHeatEntry {
            directory: directory.clone(),
            issue_count: issue_ids.len() as i64,
            file_count,
        });
    }

    result.sort_by(|a, b| b.issue_count.cmp(&a.issue_count));
    Ok(result)
}

#[tauri::command]
pub fn get_issues_for_file(
    state: State<AppState>,
    file_path: String,
    project_id: i64,
) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        get_issues_for_file_async(&state.pool, &file_path, project_id).await
    })
}

pub async fn get_issues_for_file_async(
    pool: &sqlx::AnyPool,
    file_path: &str,
    project_id: i64,
) -> Result<Vec<Issue>, String> {
    sqlx::query_as::<_, Issue>(
        "SELECT i.* FROM issues i
         JOIN issue_file_links ifl ON ifl.issue_id = i.id
         WHERE ifl.file_path = $1 AND i.project_id = $2
         ORDER BY i.updated_at DESC",
    )
    .bind(file_path)
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}
