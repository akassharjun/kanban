use crate::db::compat::jsonb_cast;
use crate::models::agent::parse_json;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

// --- Models ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pipeline {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub stages: String,
    pub enabled: i64,
    pub total_runs: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineRun {
    pub id: i64,
    pub pipeline_id: i64,
    pub trigger_issue_id: Option<i64>,
    pub status: String,
    pub current_stage: i32,
    pub stage_tasks: String,
    pub context: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

// --- Inputs ---

#[derive(Deserialize)]
pub struct CreatePipelineInput {
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub stages: serde_json::Value, // JSON array of PipelineStage
}

#[derive(Deserialize)]
pub struct UpdatePipelineInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stages: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

// --- Tauri Commands ---

#[tauri::command]
pub fn list_pipelines(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<Pipeline>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE project_id = $1 ORDER BY created_at DESC",
            )
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pipeline(
    state: State<AppState>,
    id: i64,
) -> Result<Pipeline, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, Pipeline>("SELECT * FROM pipelines WHERE id = $1")
                .bind(id)
                .fetch_one(&state.pool)
                .await
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    input: CreatePipelineInput,
) -> Result<Pipeline, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let stages_str = serde_json::to_string(&input.stages)
                .unwrap_or_else(|_| "[]".to_string());

            let jb = jsonb_cast(&state.backend);
            let id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipelines (project_id, name, description, stages, created_at, updated_at) VALUES ($1, $2, $3, $4{jb}, $5, $6) RETURNING id"
            ))
            .bind(input.project_id)
            .bind(&input.name)
            .bind(&input.description)
            .bind(&stages_str)
            .bind(&now)
            .bind(&now)
            .fetch_one(&state.pool)
            .await?;

            let pipeline = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(pipeline)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
    input: UpdatePipelineInput,
) -> Result<Pipeline, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            if let Some(name) = &input.name {
                sqlx::query("UPDATE pipelines SET name = $1, updated_at = $2 WHERE id = $3")
                    .bind(name)
                    .bind(&now)
                    .bind(id)
                    .execute(&state.pool)
                    .await?;
            }
            if let Some(desc) = &input.description {
                sqlx::query("UPDATE pipelines SET description = $1, updated_at = $2 WHERE id = $3")
                    .bind(desc)
                    .bind(&now)
                    .bind(id)
                    .execute(&state.pool)
                    .await?;
            }
            if let Some(stages) = &input.stages {
                let jb = jsonb_cast(&state.backend);
                let stages_str = serde_json::to_string(stages)
                    .unwrap_or_else(|_| "[]".to_string());
                sqlx::query(&format!(
                    "UPDATE pipelines SET stages = $1{jb}, updated_at = $2 WHERE id = $3"
                ))
                .bind(&stages_str)
                .bind(&now)
                .bind(id)
                .execute(&state.pool)
                .await?;
            }
            if let Some(enabled) = input.enabled {
                sqlx::query("UPDATE pipelines SET enabled = $1, updated_at = $2 WHERE id = $3")
                    .bind(enabled)
                    .bind(&now)
                    .bind(id)
                    .execute(&state.pool)
                    .await?;
            }

            let pipeline = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(pipeline)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    state
        .rt
        .block_on(async {
            sqlx::query("DELETE FROM pipelines WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            let _ = app.emit("db-changed", ());
            Ok(())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

// --- Pipeline Execution ---

#[tauri::command]
pub fn trigger_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    pipeline_id: i64,
    trigger_issue_id: Option<i64>,
    context: Option<String>,
) -> Result<PipelineRun, String> {
    state
        .rt
        .block_on(async {
            let pipeline = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(pipeline_id)
            .fetch_one(&state.pool)
            .await?;

            if pipeline.enabled == 0 {
                return Err(sqlx::Error::Protocol("Pipeline is disabled".to_string()));
            }

            let stages: Vec<serde_json::Value> = serde_json::from_str(&pipeline.stages)
                .unwrap_or_default();
            if stages.is_empty() {
                return Err(sqlx::Error::Protocol("Pipeline has no stages".to_string()));
            }

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let initial_context = context.unwrap_or_else(|| "{}".to_string());

            // Get trigger issue info for template rendering
            let trigger_title = if let Some(tid) = trigger_issue_id {
                let t: Option<String> = sqlx::query_scalar(
                    "SELECT title FROM issues WHERE id = $1",
                )
                .bind(tid)
                .fetch_optional(&state.pool)
                .await?;
                t.unwrap_or_default()
            } else {
                String::new()
            };
            let trigger_description = if let Some(tid) = trigger_issue_id {
                let d: Option<Option<String>> = sqlx::query_scalar(
                    "SELECT description FROM issues WHERE id = $1",
                )
                .bind(tid)
                .fetch_optional(&state.pool)
                .await?;
                d.flatten().unwrap_or_default()
            } else {
                String::new()
            };

            let jb = jsonb_cast(&state.backend);

            // Create the pipeline run
            let run_id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipeline_runs (pipeline_id, trigger_issue_id, status, current_stage, stage_tasks, context, started_at) VALUES ($1, $2, 'running', 0, '[]'{jb}, $3{jb}, $4) RETURNING id"
            ))
            .bind(pipeline_id)
            .bind(trigger_issue_id)
            .bind(&initial_context)
            .bind(&now)
            .fetch_one(&state.pool)
            .await?;

            // Create the first stage task
            let stage = &stages[0];
            let task_identifier = create_stage_task(
                &state.pool,
                &state.backend,
                pipeline.project_id,
                &pipeline.name,
                stage,
                0,
                run_id,
                &trigger_title,
                &trigger_description,
                &initial_context,
                None, // no previous task
            )
            .await?;

            // Update stage_tasks
            let stage_tasks = serde_json::json!([{
                "stage_index": 0,
                "task_identifier": task_identifier,
                "status": "queued"
            }]);
            sqlx::query(&format!(
                "UPDATE pipeline_runs SET stage_tasks = $1{jb} WHERE id = $2"
            ))
            .bind(stage_tasks.to_string())
            .bind(run_id)
            .execute(&state.pool)
            .await?;

            // Increment total_runs
            sqlx::query("UPDATE pipelines SET total_runs = total_runs + 1, updated_at = $1 WHERE id = $2")
                .bind(&now)
                .bind(pipeline_id)
                .execute(&state.pool)
                .await?;

            let run = sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(run)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn advance_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    run_id: i64,
) -> Result<PipelineRun, String> {
    state
        .rt
        .block_on(async {
            let run = advance_pipeline_internal(&state.pool, &state.backend, run_id).await?;
            let _ = app.emit("db-changed", ());
            Ok(run)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn cancel_pipeline(
    app: tauri::AppHandle,
    state: State<AppState>,
    run_id: i64,
) -> Result<PipelineRun, String> {
    state
        .rt
        .block_on(async {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query(
                "UPDATE pipeline_runs SET status = 'cancelled', completed_at = $1 WHERE id = $2",
            )
            .bind(&now)
            .bind(run_id)
            .execute(&state.pool)
            .await?;

            let run = sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());
            Ok(run)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_pipeline_run(
    state: State<AppState>,
    run_id: i64,
) -> Result<PipelineRun, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(&state.pool)
            .await
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_pipeline_runs(
    state: State<AppState>,
    pipeline_id: i64,
) -> Result<Vec<PipelineRun>, String> {
    state
        .rt
        .block_on(async {
            sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE pipeline_id = $1 ORDER BY started_at DESC",
            )
            .bind(pipeline_id)
            .fetch_all(&state.pool)
            .await
        })
        .map_err(|e| e.to_string())
}

// --- Internal helpers ---

/// Create a task contract for a pipeline stage.
async fn create_stage_task(
    pool: &sqlx::AnyPool,
    backend: &crate::db::DbBackend,
    project_id: i64,
    pipeline_name: &str,
    stage: &serde_json::Value,
    stage_index: usize,
    run_id: i64,
    trigger_title: &str,
    trigger_description: &str,
    run_context: &str,
    previous_task_identifier: Option<&str>,
) -> Result<String, sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    let stage_name = stage["name"].as_str().unwrap_or("Stage");
    let task_type = stage["task_type"].as_str().unwrap_or("implementation");
    let skills: Vec<String> = stage["required_skills"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let complexity = stage["max_complexity"].as_str().unwrap_or("medium");
    let timeout = stage["timeout_minutes"].as_i64().unwrap_or(30);
    let success_criteria: Vec<String> = stage["success_criteria"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // Template rendering (simple {{var}} replacement)
    let title_template = stage["title_template"]
        .as_str()
        .unwrap_or("{{pipeline.name}}: {{stage.name}}");
    let objective_template = stage["objective_template"]
        .as_str()
        .unwrap_or("Execute stage: {{stage.name}}");

    let title = title_template
        .replace("{{pipeline.name}}", pipeline_name)
        .replace("{{stage.name}}", stage_name)
        .replace("{{trigger.title}}", trigger_title);
    let objective = objective_template
        .replace("{{pipeline.name}}", pipeline_name)
        .replace("{{stage.name}}", stage_name)
        .replace("{{trigger.title}}", trigger_title)
        .replace("{{trigger.description}}", trigger_description);

    // Create the issue
    let (counter, prefix): (i64, String) = sqlx::query_as(
        "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    let identifier = format!("{}-{}", prefix, counter);

    // Find unstarted status
    let status_id: i64 = sqlx::query_scalar(
        "SELECT id FROM statuses WHERE project_id = $1 AND category = 'unstarted' ORDER BY position ASC LIMIT 1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    let max_pos: Option<f64> = sqlx::query_scalar(
        "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
    )
    .bind(project_id)
    .bind(status_id)
    .fetch_one(pool)
    .await?;
    let position = max_pos.unwrap_or(-1.0) + 1.0;

    let description = format!(
        "Pipeline run #{} - Stage {} ({})\n\n{}",
        run_id,
        stage_index + 1,
        stage_name,
        objective
    );

    let issue_id: i64 = sqlx::query_scalar(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'medium', $6, $7, $8) RETURNING id",
    )
    .bind(project_id)
    .bind(&identifier)
    .bind(&title)
    .bind(&description)
    .bind(status_id)
    .bind(position)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    // Build context with pipeline metadata
    let context = serde_json::json!({
        "files": [],
        "related_tasks": [],
        "prior_attempts": [],
        "pipeline": {
            "run_id": run_id,
            "stage_index": stage_index,
            "pipeline_name": pipeline_name,
            "stage_name": stage_name,
            "accumulated_context": parse_json(run_context)
        }
    });

    let skills_json = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());
    let sc_json = serde_json::to_string(&success_criteria).unwrap_or_else(|_| "[]".to_string());
    let context_json = serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

    let jb = jsonb_cast(backend);
    sqlx::query(&format!(
        "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4{jb}, '[]'{jb}, $5{jb}, $6{jb}, $7, $8, 0)"
    ))
    .bind(issue_id)
    .bind(task_type)
    .bind(&objective)
    .bind(&context_json)
    .bind(&sc_json)
    .bind(&skills_json)
    .bind(complexity)
    .bind(timeout)
    .execute(pool)
    .await?;

    // If there's a previous task, create a blocks relation
    if let Some(prev_identifier) = previous_task_identifier {
        let prev_issue_id: i64 = sqlx::query_scalar(
            "SELECT id FROM issues WHERE identifier = $1",
        )
        .bind(prev_identifier)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocks')",
        )
        .bind(prev_issue_id)
        .bind(issue_id)
        .execute(pool)
        .await?;
    }

    Ok(identifier)
}

/// Internal: advance a pipeline run to the next stage.
pub async fn advance_pipeline_internal(
    pool: &sqlx::AnyPool,
    backend: &crate::db::DbBackend,
    run_id: i64,
) -> Result<PipelineRun, sqlx::Error> {
    let run = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;

    if run.status != "running" {
        return Err(sqlx::Error::Protocol(format!(
            "Pipeline run {} is not running (status: {})",
            run_id, run.status
        )));
    }

    let pipeline = sqlx::query_as::<_, Pipeline>(
        "SELECT * FROM pipelines WHERE id = $1",
    )
    .bind(run.pipeline_id)
    .fetch_one(pool)
    .await?;

    let stages: Vec<serde_json::Value> = serde_json::from_str(&pipeline.stages)
        .unwrap_or_default();

    let next_stage = run.current_stage as usize + 1;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    let jb = jsonb_cast(backend);

    // Update current stage task status
    let mut stage_tasks: Vec<serde_json::Value> =
        serde_json::from_str(&run.stage_tasks).unwrap_or_default();

    // Find the previous task identifier for linking
    let prev_task_identifier = stage_tasks
        .last()
        .and_then(|t| t["task_identifier"].as_str())
        .map(String::from);

    // Mark current stage as completed
    if let Some(current) = stage_tasks.last_mut() {
        current["status"] = serde_json::json!("completed");
    }

    // Accumulate context from completed task
    let mut accumulated_context: serde_json::Value =
        serde_json::from_str(&run.context).unwrap_or(serde_json::json!({}));
    if let Some(prev_id) = &prev_task_identifier {
        // Get the completed task's result
        let result: Option<String> = sqlx::query_scalar(
            "SELECT tc.result FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.identifier = $1",
        )
        .bind(prev_id)
        .fetch_optional(pool)
        .await?;

        if let Some(result_str) = result {
            let result_val = parse_json(&result_str);
            let stage_key = format!("stage_{}_result", run.current_stage);
            accumulated_context[stage_key] = result_val;
        }
    }

    if next_stage >= stages.len() {
        // Pipeline completed
        sqlx::query(&format!(
            "UPDATE pipeline_runs SET status = 'completed', completed_at = $1, stage_tasks = $2{jb}, context = $3{jb} WHERE id = $4"
        ))
        .bind(&now)
        .bind(serde_json::to_string(&stage_tasks).unwrap_or_default())
        .bind(accumulated_context.to_string())
        .bind(run_id)
        .execute(pool)
        .await?;
    } else {
        // Create next stage task
        let trigger_title = if let Some(tid) = run.trigger_issue_id {
            let t: Option<String> = sqlx::query_scalar(
                "SELECT title FROM issues WHERE id = $1",
            )
            .bind(tid)
            .fetch_optional(pool)
            .await?;
            t.unwrap_or_default()
        } else {
            String::new()
        };
        let trigger_description = if let Some(tid) = run.trigger_issue_id {
            let d: Option<Option<String>> = sqlx::query_scalar(
                "SELECT description FROM issues WHERE id = $1",
            )
            .bind(tid)
            .fetch_optional(pool)
            .await?;
            d.flatten().unwrap_or_default()
        } else {
            String::new()
        };

        let next_identifier = create_stage_task(
            pool,
            backend,
            pipeline.project_id,
            &pipeline.name,
            &stages[next_stage],
            next_stage,
            run_id,
            &trigger_title,
            &trigger_description,
            &accumulated_context.to_string(),
            prev_task_identifier.as_deref(),
        )
        .await?;

        stage_tasks.push(serde_json::json!({
            "stage_index": next_stage,
            "task_identifier": next_identifier,
            "status": "queued"
        }));

        sqlx::query(&format!(
            "UPDATE pipeline_runs SET current_stage = $1, stage_tasks = $2{jb}, context = $3{jb} WHERE id = $4"
        ))
        .bind(next_stage as i32)
        .bind(serde_json::to_string(&stage_tasks).unwrap_or_default())
        .bind(accumulated_context.to_string())
        .bind(run_id)
        .execute(pool)
        .await?;
    }

    let updated_run = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;

    Ok(updated_run)
}

/// Check if a completed task is part of a pipeline run and auto-advance if configured.
/// Called from complete_task after a task is marked completed.
pub async fn check_pipeline_advancement(
    pool: &sqlx::AnyPool,
    backend: &crate::db::DbBackend,
    task_identifier: &str,
) -> Result<(), String> {
    // Find any running pipeline run that has this task
    let runs: Vec<PipelineRun> = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE status = 'running'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for run in runs {
        let stage_tasks: Vec<serde_json::Value> =
            serde_json::from_str(&run.stage_tasks).unwrap_or_default();

        // Check if this task belongs to the current stage of this run
        if let Some(current_task) = stage_tasks.last() {
            if current_task["task_identifier"].as_str() == Some(task_identifier)
                && current_task["status"].as_str() != Some("completed")
            {
                // Check if auto_advance is enabled for this stage
                let pipeline = sqlx::query_as::<_, Pipeline>(
                    "SELECT * FROM pipelines WHERE id = $1",
                )
                .bind(run.pipeline_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

                let stages: Vec<serde_json::Value> =
                    serde_json::from_str(&pipeline.stages).unwrap_or_default();
                let stage_index = run.current_stage as usize;

                let auto_advance = stages
                    .get(stage_index)
                    .and_then(|s| s["auto_advance"].as_bool())
                    .unwrap_or(true);

                if auto_advance {
                    let _ = advance_pipeline_internal(pool, backend, run.id).await;
                }
            }
        }
    }

    Ok(())
}

/// Mark a pipeline run as failed when a stage task fails.
pub async fn check_pipeline_failure(
    pool: &sqlx::AnyPool,
    task_identifier: &str,
) -> Result<(), String> {
    let runs: Vec<PipelineRun> = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE status = 'running'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for run in runs {
        let stage_tasks: Vec<serde_json::Value> =
            serde_json::from_str(&run.stage_tasks).unwrap_or_default();

        if let Some(current_task) = stage_tasks.last() {
            if current_task["task_identifier"].as_str() == Some(task_identifier) {
                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
                let _ = sqlx::query(
                    "UPDATE pipeline_runs SET status = 'failed', completed_at = $1, error_message = $2 WHERE id = $3",
                )
                .bind(&now)
                .bind(format!("Stage task {} failed", task_identifier))
                .bind(run.id)
                .execute(pool)
                .await;
            }
        }
    }

    Ok(())
}
