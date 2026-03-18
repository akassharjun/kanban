use crate::models::{Issue, RecurringIssue};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Deserialize)]
pub struct CreateRecurringInput {
    pub project_id: i64,
    pub title_template: String,
    pub description_template: Option<String>,
    pub status_id: i64,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub label_ids: Option<Vec<i64>>,
    pub recurrence_type: String,
    pub recurrence_config: Option<String>,
    pub next_run_at: String,
}

#[derive(Deserialize)]
pub struct UpdateRecurringInput {
    pub title_template: Option<String>,
    pub description_template: Option<String>,
    pub status_id: Option<i64>,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub label_ids: Option<Vec<i64>>,
    pub recurrence_type: Option<String>,
    pub recurrence_config: Option<String>,
    pub next_run_at: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct RecurringPreview {
    pub title: String,
    pub description: Option<String>,
    pub next_dates: Vec<String>,
}

fn render_template(template: &str, count: i64, date: &str, day: &str) -> String {
    template
        .replace("{{date}}", date)
        .replace("{{count}}", &count.to_string())
        .replace("{{day}}", day)
}

fn calculate_next_run(recurrence_type: &str, config: &str, from: &str) -> Result<String, String> {
    use chrono::{NaiveDateTime, NaiveDate, Datelike, Duration};

    let from_dt = NaiveDateTime::parse_from_str(from, "%Y-%m-%d %H:%M:%SZ")
        .or_else(|_| NaiveDateTime::parse_from_str(from, "%Y-%m-%dT%H:%M:%SZ"))
        .or_else(|_| {
            NaiveDate::parse_from_str(from, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .map_err(|e| format!("Invalid date format '{}': {}", from, e))?;

    let config_json: serde_json::Value = serde_json::from_str(config).unwrap_or(serde_json::json!({}));

    let next = match recurrence_type {
        "daily" => from_dt + Duration::days(1),
        "weekly" => from_dt + Duration::weeks(1),
        "biweekly" => from_dt + Duration::weeks(2),
        "monthly" => {
            let month = from_dt.month();
            let year = from_dt.year();
            let (new_year, new_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
            let day = from_dt.day().min(28); // safe day
            NaiveDate::from_ymd_opt(new_year, new_month, day)
                .unwrap_or(from_dt.date())
                .and_hms_opt(from_dt.time().hour(), from_dt.time().minute(), from_dt.time().second())
                .unwrap()
        },
        "custom" => {
            let interval = config_json.get("interval_days")
                .and_then(|v| v.as_i64())
                .unwrap_or(7);
            from_dt + Duration::days(interval)
        },
        _ => return Err(format!("Unknown recurrence type: {}", recurrence_type)),
    };

    Ok(next.format("%Y-%m-%d %H:%M:%SZ").to_string())
}

use chrono::Datelike;

#[tauri::command]
pub fn list_recurring(state: State<AppState>, project_id: i64) -> Result<Vec<RecurringIssue>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, RecurringIssue>(
            "SELECT * FROM recurring_issues WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_recurring(state: State<AppState>, input: CreateRecurringInput) -> Result<RecurringIssue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let priority = input.priority.unwrap_or_else(|| "medium".to_string());
        let label_ids_json = serde_json::to_string(&input.label_ids.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let config = input.recurrence_config.unwrap_or_else(|| "{}".to_string());

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO recurring_issues (project_id, title_template, description_template, status_id, priority, assignee_id, label_ids, recurrence_type, recurrence_config, next_run_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.title_template)
        .bind(&input.description_template)
        .bind(input.status_id)
        .bind(&priority)
        .bind(input.assignee_id)
        .bind(&label_ids_json)
        .bind(&input.recurrence_type)
        .bind(&config)
        .bind(&input.next_run_at)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query_as::<_, RecurringIssue>("SELECT * FROM recurring_issues WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn update_recurring(state: State<AppState>, id: i64, input: UpdateRecurringInput) -> Result<RecurringIssue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        if let Some(ref title_template) = input.title_template {
            sqlx::query("UPDATE recurring_issues SET title_template = $1, updated_at = $2 WHERE id = $3")
                .bind(title_template).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref desc) = input.description_template {
            sqlx::query("UPDATE recurring_issues SET description_template = $1, updated_at = $2 WHERE id = $3")
                .bind(desc).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(status_id) = input.status_id {
            sqlx::query("UPDATE recurring_issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                .bind(status_id).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref priority) = input.priority {
            sqlx::query("UPDATE recurring_issues SET priority = $1, updated_at = $2 WHERE id = $3")
                .bind(priority).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(assignee_id) = input.assignee_id {
            let val = if assignee_id <= 0 { None } else { Some(assignee_id) };
            if let Some(v) = val {
                sqlx::query("UPDATE recurring_issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(v).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            } else {
                sqlx::query("UPDATE recurring_issues SET assignee_id = NULL, updated_at = $1 WHERE id = $2")
                    .bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
            }
        }
        if let Some(ref label_ids) = input.label_ids {
            let json = serde_json::to_string(label_ids).unwrap_or_else(|_| "[]".to_string());
            sqlx::query("UPDATE recurring_issues SET label_ids = $1, updated_at = $2 WHERE id = $3")
                .bind(&json).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref recurrence_type) = input.recurrence_type {
            sqlx::query("UPDATE recurring_issues SET recurrence_type = $1, updated_at = $2 WHERE id = $3")
                .bind(recurrence_type).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref config) = input.recurrence_config {
            sqlx::query("UPDATE recurring_issues SET recurrence_config = $1, updated_at = $2 WHERE id = $3")
                .bind(config).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref next_run) = input.next_run_at {
            sqlx::query("UPDATE recurring_issues SET next_run_at = $1, updated_at = $2 WHERE id = $3")
                .bind(next_run).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(enabled) = input.enabled {
            sqlx::query("UPDATE recurring_issues SET enabled = $1, updated_at = $2 WHERE id = $3")
                .bind(enabled).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        sqlx::query_as::<_, RecurringIssue>("SELECT * FROM recurring_issues WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn delete_recurring(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM recurring_issues WHERE id = $1")
            .bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn toggle_recurring(state: State<AppState>, id: i64, enabled: bool) -> Result<RecurringIssue, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE recurring_issues SET enabled = $1, updated_at = $2 WHERE id = $3")
            .bind(enabled).bind(&now).bind(id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query_as::<_, RecurringIssue>("SELECT * FROM recurring_issues WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn check_recurring(state: State<AppState>, project_id: i64) -> Result<Vec<Issue>, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%SZ").to_string();
        let today = now.format("%Y-%m-%d").to_string();
        let day_name = now.format("%A").to_string();

        // Find all due recurring issues
        let due: Vec<RecurringIssue> = sqlx::query_as(
            "SELECT * FROM recurring_issues WHERE project_id = $1 AND enabled = 1 AND next_run_at <= $2"
        )
        .bind(project_id)
        .bind(&now_str)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut created_issues = Vec::new();

        for rec in due {
            let count = rec.total_created + 1;
            let title = render_template(&rec.title_template, count, &today, &day_name);
            let description = rec.description_template.as_deref()
                .map(|t| render_template(t, count, &today, &day_name));

            // Create the issue using the same pattern as create_issue
            let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;

            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            ).bind(rec.project_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
            ).bind(rec.project_id).bind(rec.status_id)
            .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id"
            )
            .bind(rec.project_id)
            .bind(&identifier)
            .bind(&title)
            .bind(&description)
            .bind(rec.status_id)
            .bind(&rec.priority)
            .bind(rec.assignee_id)
            .bind(position)
            .bind(&now_str)
            .bind(&now_str)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            // Add labels
            let label_ids: Vec<i64> = serde_json::from_str(&rec.label_ids).unwrap_or_default();
            for label_id in &label_ids {
                let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
                    .bind(issue_id).bind(label_id).execute(&mut *tx).await;
            }

            // Calculate next run
            let next_run = calculate_next_run(&rec.recurrence_type, &rec.recurrence_config, &rec.next_run_at)?;

            // Update recurring issue
            sqlx::query(
                "UPDATE recurring_issues SET last_run_at = $1, next_run_at = $2, total_created = $3, updated_at = $4 WHERE id = $5"
            )
            .bind(&now_str)
            .bind(&next_run)
            .bind(count)
            .bind(&now_str)
            .bind(rec.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

            tx.commit().await.map_err(|e| e.to_string())?;

            created_issues.push(issue);
        }

        Ok(created_issues)
    })
}

#[tauri::command]
pub fn preview_recurring(state: State<AppState>, id: i64) -> Result<RecurringPreview, String> {
    state.rt.block_on(async {
        let rec = sqlx::query_as::<_, RecurringIssue>("SELECT * FROM recurring_issues WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let now = chrono::Utc::now();
        let today = now.format("%Y-%m-%d").to_string();
        let day_name = now.format("%A").to_string();
        let count = rec.total_created + 1;

        let title = render_template(&rec.title_template, count, &today, &day_name);
        let description = rec.description_template.as_deref()
            .map(|t| render_template(t, count, &today, &day_name));

        // Calculate next 3 dates
        let mut next_dates = Vec::new();
        let mut current = rec.next_run_at.clone();
        for _ in 0..3 {
            next_dates.push(current.clone());
            current = calculate_next_run(&rec.recurrence_type, &rec.recurrence_config, &current)?;
        }

        Ok(RecurringPreview {
            title,
            description,
            next_dates,
        })
    })
}
