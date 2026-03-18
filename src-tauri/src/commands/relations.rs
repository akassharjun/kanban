use crate::state::AppState;
use crate::models::IssueRelation;
use tauri::State;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct DependencyNode {
    pub id: i64,
    pub identifier: String,
    pub title: String,
    pub status_category: String,
    pub priority: String,
    pub assignee_name: Option<String>,
}

#[derive(Serialize)]
pub struct DependencyEdge {
    pub source_id: i64,
    pub target_id: i64,
    pub relation_type: String,
}

#[derive(Serialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[tauri::command]
pub fn dependency_graph(state: State<AppState>, project_id: i64) -> Result<DependencyGraph, String> {
    state.rt.block_on(async {
        // Get all issues with their status categories and assignee names
        let rows: Vec<(i64, String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT i.id, i.identifier, i.title, s.category, i.priority, m.display_name \
             FROM issues i \
             JOIN statuses s ON i.status_id = s.id \
             LEFT JOIN members m ON i.assignee_id = m.id \
             WHERE i.project_id = $1"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let nodes: Vec<DependencyNode> = rows.iter().map(|r| DependencyNode {
            id: r.0,
            identifier: r.1.clone(),
            title: r.2.clone(),
            status_category: r.3.clone(),
            priority: r.4.clone(),
            assignee_name: r.5.clone(),
        }).collect();

        let node_ids: std::collections::HashSet<i64> = nodes.iter().map(|n| n.id).collect();

        // Get all relations where both issues are in this project
        let relations: Vec<IssueRelation> = sqlx::query_as::<_, IssueRelation>(
            "SELECT ir.* FROM issue_relations ir \
             JOIN issues i1 ON ir.source_issue_id = i1.id \
             JOIN issues i2 ON ir.target_issue_id = i2.id \
             WHERE i1.project_id = $1 AND i2.project_id = $2"
        )
        .bind(project_id)
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let edges: Vec<DependencyEdge> = relations.iter()
            .filter(|r| node_ids.contains(&r.source_issue_id) && node_ids.contains(&r.target_issue_id))
            .map(|r| DependencyEdge {
                source_id: r.source_issue_id,
                target_id: r.target_issue_id,
                relation_type: r.relation_type.clone(),
            })
            .collect();

        Ok(DependencyGraph { nodes, edges })
    })
}
