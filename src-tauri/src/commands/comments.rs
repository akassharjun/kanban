use crate::state::AppState;
use crate::models::Comment;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateCommentInput {
    pub issue_id: i64,
    pub member_id: Option<i64>,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateCommentInput {
    pub content: String,
}

#[tauri::command]
pub fn list_comments(state: State<AppState>, issue_id: i64) -> Result<Vec<Comment>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE issue_id = $1 ORDER BY created_at ASC")
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_comment(state: State<AppState>, input: CreateCommentInput) -> Result<Comment, String> {
    if input.content.trim().is_empty() {
        return Err("content cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(input.issue_id)
        .bind(input.member_id)
        .bind(&input.content)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        // Also update the issue's updated_at
        sqlx::query("UPDATE issues SET updated_at = $1 WHERE id = $2")
            .bind(&now)
            .bind(input.issue_id)
            .execute(&state.pool)
            .await?;

        sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_comment(state: State<AppState>, id: i64, input: UpdateCommentInput) -> Result<Comment, String> {
    if input.content.trim().is_empty() {
        return Err("content cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let result = sqlx::query("UPDATE comments SET content = $1, updated_at = $2 WHERE id = $3")
            .bind(&input.content)
            .bind(&now)
            .bind(id)
            .execute(&state.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_comment(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM comments WHERE id = $1")
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
pub fn comment_count(state: State<AppState>, issue_id: i64) -> Result<i64, String> {
    state.rt.block_on(async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comments WHERE issue_id = $1")
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}
