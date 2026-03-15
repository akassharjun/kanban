use crate::models::{CustomField, CustomFieldValue};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateCustomFieldInput {
    pub project_id: i64,
    pub name: String,
    pub field_type: Option<String>,
    pub options: Option<String>,
    pub position: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateCustomFieldInput {
    pub name: Option<String>,
    pub field_type: Option<String>,
    pub options: Option<String>,
    pub position: Option<i64>,
}

#[tauri::command]
pub fn list_custom_fields(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<CustomField>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, CustomField>(
                "SELECT * FROM custom_fields WHERE project_id = $1 ORDER BY position ASC, id ASC",
            )
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_custom_field(
    state: State<AppState>,
    input: CreateCustomFieldInput,
) -> Result<CustomField, String> {
    if input.name.trim().is_empty() {
        return Err("name cannot be empty".to_string());
    }
    let field_type = input.field_type.unwrap_or_else(|| "text".to_string());
    match field_type.as_str() {
        "text" | "number" | "date" | "select" => {}
        _ => return Err(format!("invalid field_type: {}", field_type)),
    }
    let position = input.position.unwrap_or(0);
    state
        .rt
        .block_on(async {
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO custom_fields (project_id, name, field_type, options, position) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(input.project_id)
            .bind(input.name.trim())
            .bind(&field_type)
            .bind(&input.options)
            .bind(position)
            .fetch_one(&state.pool)
            .await?;

            sqlx::query_as::<_, CustomField>("SELECT * FROM custom_fields WHERE id = $1")
                .bind(id)
                .fetch_one(&state.pool)
                .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_custom_field(
    state: State<AppState>,
    id: i64,
    input: UpdateCustomFieldInput,
) -> Result<CustomField, String> {
    if let Some(ref ft) = input.field_type {
        match ft.as_str() {
            "text" | "number" | "date" | "select" => {}
            _ => return Err(format!("invalid field_type: {}", ft)),
        }
    }
    state
        .rt
        .block_on(async {
            let existing = sqlx::query_as::<_, CustomField>("SELECT * FROM custom_fields WHERE id = $1")
                .bind(id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| sqlx::Error::RowNotFound)?;

            let name = input.name.unwrap_or(existing.name);
            let field_type = input.field_type.unwrap_or(existing.field_type);
            let options = input.options.or(existing.options);
            let position = input.position.unwrap_or(existing.position);

            sqlx::query(
                "UPDATE custom_fields SET name = $1, field_type = $2, options = $3, position = $4 WHERE id = $5",
            )
            .bind(&name)
            .bind(&field_type)
            .bind(&options)
            .bind(position)
            .bind(id)
            .execute(&state.pool)
            .await?;

            sqlx::query_as::<_, CustomField>("SELECT * FROM custom_fields WHERE id = $1")
                .bind(id)
                .fetch_one(&state.pool)
                .await
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_custom_field(state: State<AppState>, id: i64) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            let result = sqlx::query("DELETE FROM custom_fields WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            if result.rows_affected() == 0 {
                return Err(sqlx::Error::RowNotFound);
            }
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_issue_custom_values(
    state: State<AppState>,
    issue_id: i64,
) -> Result<Vec<CustomFieldValue>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, CustomFieldValue>(
                "SELECT * FROM issue_custom_field_values WHERE issue_id = $1",
            )
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_issue_custom_value(
    state: State<AppState>,
    issue_id: i64,
    field_id: i64,
    value: Option<String>,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            sqlx::query(
                "INSERT INTO issue_custom_field_values (issue_id, field_id, value) VALUES ($1, $2, $3) ON CONFLICT(issue_id, field_id) DO UPDATE SET value = excluded.value",
            )
            .bind(issue_id)
            .bind(field_id)
            .bind(&value)
            .execute(&state.pool)
            .await?;
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}
