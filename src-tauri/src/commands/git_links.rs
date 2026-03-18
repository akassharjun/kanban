use crate::state::AppState;
use crate::models::GitLink;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateGitLinkInput {
    pub issue_id: i64,
    pub link_type: String,
    pub ref_name: String,
    pub url: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateGitLinkInput {
    pub status: Option<String>,
    pub url: Option<String>,
}

#[tauri::command]
pub fn create_git_link(state: State<AppState>, input: CreateGitLinkInput) -> Result<GitLink, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let status = input.status.unwrap_or_else(|| "open".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO git_links (issue_id, link_type, ref_name, url, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(input.issue_id)
        .bind(&input.link_type)
        .bind(&input.ref_name)
        .bind(&input.url)
        .bind(&status)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_git_links(state: State<AppState>, issue_id: i64) -> Result<Vec<GitLink>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE issue_id = $1 ORDER BY created_at DESC")
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_git_link(state: State<AppState>, id: i64, input: UpdateGitLinkInput) -> Result<GitLink, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        let mut qb = sqlx::QueryBuilder::new("UPDATE git_links SET updated_at = ");
        qb.push_bind(&now);

        if let Some(status) = &input.status {
            qb.push(", status = ");
            qb.push_bind(status);
        }
        if let Some(url) = &input.url {
            qb.push(", url = ");
            qb.push_bind(url);
        }

        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.build().execute(&state.pool).await?;

        sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_git_link(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM git_links WHERE id = $1")
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
pub fn git_link_count(state: State<AppState>, issue_id: i64) -> Result<i64, String> {
    state.rt.block_on(async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM git_links WHERE issue_id = $1")
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}
