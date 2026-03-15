use crate::state::AppState;
use crate::models::IssueTemplate;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTemplateInput {
    pub project_id: i64,
    pub name: String,
    pub description_template: Option<String>,
    pub default_status_id: Option<i64>,
    pub default_priority: Option<String>,
    pub default_label_ids: Option<Vec<i64>>,
}

#[derive(Deserialize)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub description_template: Option<String>,
    pub default_status_id: Option<i64>,
    pub default_priority: Option<String>,
    pub default_label_ids: Option<Vec<i64>>,
}

#[tauri::command]
pub fn list_templates(state: State<AppState>, project_id: i64) -> Result<Vec<IssueTemplate>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, IssueTemplate>("SELECT * FROM issue_templates WHERE project_id = ? ORDER BY name")
            .bind(project_id).fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_template(state: State<AppState>, input: CreateTemplateInput) -> Result<IssueTemplate, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let priority = input.default_priority.unwrap_or_else(|| "none".to_string());
        let label_ids = input.default_label_ids.map(|ids| serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())).unwrap_or_else(|| "[]".to_string());

        let result = sqlx::query(
            "INSERT INTO issue_templates (project_id, name, description_template, default_status_id, default_priority, default_label_ids, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(input.project_id).bind(&input.name).bind(&input.description_template)
        .bind(input.default_status_id).bind(&priority).bind(&label_ids).bind(&now).bind(&now)
        .execute(&state.pool).await?;

        sqlx::query_as::<_, IssueTemplate>("SELECT * FROM issue_templates WHERE id = ?")
            .bind(result.last_insert_rowid()).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_template(state: State<AppState>, id: i64, input: UpdateTemplateInput) -> Result<IssueTemplate, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        if let Some(ref name) = input.name {
            sqlx::query("UPDATE issue_templates SET name = ?, updated_at = ? WHERE id = ?")
                .bind(name).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref desc) = input.description_template {
            sqlx::query("UPDATE issue_templates SET description_template = ?, updated_at = ? WHERE id = ?")
                .bind(desc).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(status_id) = input.default_status_id {
            sqlx::query("UPDATE issue_templates SET default_status_id = ?, updated_at = ? WHERE id = ?")
                .bind(status_id).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref priority) = input.default_priority {
            sqlx::query("UPDATE issue_templates SET default_priority = ?, updated_at = ? WHERE id = ?")
                .bind(priority).bind(&now).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref label_ids) = input.default_label_ids {
            let json = serde_json::to_string(label_ids).unwrap_or_else(|_| "[]".to_string());
            sqlx::query("UPDATE issue_templates SET default_label_ids = ?, updated_at = ? WHERE id = ?")
                .bind(&json).bind(&now).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, IssueTemplate>("SELECT * FROM issue_templates WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_template(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM issue_templates WHERE id = ?").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
