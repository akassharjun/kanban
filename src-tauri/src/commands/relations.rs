use crate::state::AppState;
use crate::models::IssueRelation;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateRelationInput {
    pub source_issue_id: i64,
    pub target_issue_id: i64,
    pub relation_type: String,
}

#[tauri::command]
pub fn list_relations(state: State<AppState>, issue_id: i64) -> Result<Vec<IssueRelation>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, IssueRelation>(
            "SELECT * FROM issue_relations WHERE source_issue_id = $1 OR target_issue_id = $2"
        ).bind(issue_id).bind(issue_id).fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_relation(state: State<AppState>, input: CreateRelationInput) -> Result<IssueRelation, String> {
    state.rt.block_on(async {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(input.source_issue_id).bind(input.target_issue_id).bind(&input.relation_type)
        .fetch_one(&state.pool).await?;

        sqlx::query_as::<_, IssueRelation>("SELECT * FROM issue_relations WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_relation(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM issue_relations WHERE id = $1").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
