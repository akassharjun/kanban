use crate::models::agent::HandoffNote;
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct CreateHandoffInput {
    pub task_identifier: String,
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub note_type: String,
    pub summary: String,
    pub details: Option<String>,
    pub files_changed: Option<Vec<String>>,
    pub risks: Option<Vec<String>>,
    pub test_results: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub fn create_handoff_note(state: State<AppState>, input: CreateHandoffInput) -> Result<HandoffNote, String> {
    if input.summary.trim().is_empty() {
        return Err("summary cannot be empty".to_string());
    }
    let valid_types = ["completion", "review_request", "escalation", "context", "warning", "suggestion"];
    if !valid_types.contains(&input.note_type.as_str()) {
        return Err(format!("note_type must be one of: {}", valid_types.join(", ")));
    }
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let files_changed = serde_json::to_string(&input.files_changed.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());
        let risks = serde_json::to_string(&input.risks.unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string());
        let test_results = input.test_results.map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()));
        let metadata = input.metadata
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO handoff_notes (task_identifier, from_agent_id, to_agent_id, note_type, summary, details, files_changed, risks, test_results, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
        )
        .bind(&input.task_identifier)
        .bind(&input.from_agent_id)
        .bind(&input.to_agent_id)
        .bind(&input.note_type)
        .bind(&input.summary)
        .bind(&input.details)
        .bind(&files_changed)
        .bind(&risks)
        .bind(&test_results)
        .bind(&metadata)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, HandoffNote>("SELECT * FROM handoff_notes WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_handoff_notes(state: State<AppState>, task_identifier: String) -> Result<Vec<HandoffNote>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, HandoffNote>(
            "SELECT * FROM handoff_notes WHERE task_identifier = $1 ORDER BY created_at ASC"
        )
        .bind(&task_identifier)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_handoff_for_agent(state: State<AppState>, agent_id: String, task_identifier: String) -> Result<Vec<HandoffNote>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, HandoffNote>(
            "SELECT * FROM handoff_notes WHERE task_identifier = $1 AND (to_agent_id IS NULL OR to_agent_id = $2) ORDER BY created_at ASC"
        )
        .bind(&task_identifier)
        .bind(&agent_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}
