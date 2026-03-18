use crate::state::AppState;
use crate::models::Milestone;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateMilestoneInput {
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMilestoneInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct MilestoneWithProgress {
    #[serde(flatten)]
    pub milestone: Milestone,
    pub total_issues: i64,
    pub completed_issues: i64,
}

#[tauri::command]
pub fn list_milestones(state: State<AppState>, project_id: i64) -> Result<Vec<MilestoneWithProgress>, String> {
    state.rt.block_on(async {
        let milestones = sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE project_id = $1 ORDER BY due_date ASC NULLS LAST, created_at ASC")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await?;

        let mut result = Vec::with_capacity(milestones.len());
        for m in milestones {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE milestone_id = $1")
                .bind(m.id).fetch_one(&state.pool).await?;
            let completed: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.milestone_id = $1 AND s.category IN ('completed', 'discarded')"
            ).bind(m.id).fetch_one(&state.pool).await?;
            result.push(MilestoneWithProgress { milestone: m, total_issues: total, completed_issues: completed });
        }
        Ok(result)
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_milestone(state: State<AppState>, id: i64) -> Result<MilestoneWithProgress, String> {
    state.rt.block_on(async {
        let m = sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await?;
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE milestone_id = $1")
            .bind(m.id).fetch_one(&state.pool).await?;
        let completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.milestone_id = $1 AND s.category IN ('completed', 'discarded')"
        ).bind(m.id).fetch_one(&state.pool).await?;
        Ok(MilestoneWithProgress { milestone: m, total_issues: total, completed_issues: completed })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn create_milestone(state: State<AppState>, input: CreateMilestoneInput) -> Result<Milestone, String> {
    if input.title.trim().is_empty() {
        return Err("title cannot be empty".to_string());
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO milestones (project_id, title, description, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.due_date)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_milestone(state: State<AppState>, id: i64, input: UpdateMilestoneInput) -> Result<Milestone, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        if let Some(ref title) = input.title {
            sqlx::query("UPDATE milestones SET title = $1, updated_at = $2 WHERE id = $3")
                .bind(title).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref description) = input.description {
            sqlx::query("UPDATE milestones SET description = $1, updated_at = $2 WHERE id = $3")
                .bind(description).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref due_date) = input.due_date {
            let val = if due_date.is_empty() { None } else { Some(due_date.clone()) };
            if let Some(ref v) = val {
                sqlx::query("UPDATE milestones SET due_date = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE milestones SET due_date = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await?;
            }
        }
        if let Some(ref status) = input.status {
            sqlx::query("UPDATE milestones SET status = $1, updated_at = $2 WHERE id = $3")
                .bind(status).bind(&now).bind(id).execute(&state.pool).await?;
        }

        sqlx::query_as::<_, Milestone>("SELECT * FROM milestones WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_milestone(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        // Unlink issues from this milestone first
        sqlx::query("UPDATE issues SET milestone_id = NULL WHERE milestone_id = $1")
            .bind(id).execute(&state.pool).await?;
        let result = sqlx::query("DELETE FROM milestones WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
