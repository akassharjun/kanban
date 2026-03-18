use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsjfScore {
    pub issue_id: i64,
    pub identifier: String,
    pub title: String,
    pub business_value: i32,
    pub time_criticality: i32,
    pub risk_reduction: i32,
    pub job_size: i32,
    pub wsjf_score: f64,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScoreResult {
    pub issue_id: i64,
    pub business_value: i32,
    pub time_criticality: i32,
    pub risk_reduction: i32,
    pub job_size: i32,
    pub wsjf_score: f64,
    pub reasoning: String,
}

#[derive(Deserialize)]
pub struct SetWsjfInput {
    pub issue_id: i64,
    pub business_value: i32,
    pub time_criticality: i32,
    pub risk_reduction: i32,
    pub job_size: i32,
}

fn calculate_wsjf(bv: i32, tc: i32, rr: i32, size: i32) -> f64 {
    if size <= 0 {
        return 0.0;
    }
    (bv as f64 + tc as f64 + rr as f64) / size as f64
}

fn clamp_score(v: i32) -> i32 {
    v.max(1).min(10)
}

/// Derive business_value from priority string
fn bv_from_priority(priority: &str) -> i32 {
    match priority {
        "urgent" => 10,
        "high" => 8,
        "medium" => 5,
        "low" => 2,
        _ => 1,
    }
}

/// Derive time_criticality from due_date relative to today
fn tc_from_due_date(due_date: Option<&str>) -> (i32, &'static str) {
    let Some(dd) = due_date else {
        return (3, "no due date");
    };
    let today = chrono::Utc::now().date_naive();
    let Ok(due) = chrono::NaiveDate::parse_from_str(dd, "%Y-%m-%d") else {
        return (3, "unparseable due date");
    };
    let days = (due - today).num_days();
    if days < 0 {
        (10, "overdue")
    } else if days == 0 {
        (9, "due today")
    } else if days <= 7 {
        (7, "due this week")
    } else if days <= 30 {
        (5, "due this month")
    } else {
        (3, "due later")
    }
}

/// Derive risk_reduction from label names
fn rr_from_labels(labels: &[String]) -> (i32, &'static str) {
    let lower: Vec<String> = labels.iter().map(|l| l.to_lowercase()).collect();
    if lower.iter().any(|l| l.contains("security") || l.contains("bug") || l.contains("crash")) {
        (9, "security/bug/crash label")
    } else if lower.iter().any(|l| l.contains("performance")) {
        (6, "performance label")
    } else if lower.iter().any(|l| l.contains("feature")) {
        (3, "feature label")
    } else if lower.iter().any(|l| l.contains("doc")) {
        (1, "docs label")
    } else {
        (3, "no risk-relevant labels")
    }
}

/// Derive job_size from estimate or complexity
fn size_from_estimate(estimate: Option<f64>, complexity: Option<&str>) -> (i32, &'static str) {
    if let Some(est) = estimate {
        if est <= 3.0 {
            (2, "small estimate (1-3)")
        } else if est <= 6.0 {
            (5, "medium estimate (4-6)")
        } else {
            (8, "large estimate (7+)")
        }
    } else if let Some(c) = complexity {
        match c {
            "small" => (2, "small complexity"),
            "medium" => (5, "medium complexity"),
            "large" => (8, "large complexity"),
            _ => (5, "default medium size"),
        }
    } else {
        (5, "no estimate, default medium")
    }
}

/// Row for auto-scoring query
#[derive(Debug, sqlx::FromRow)]
struct AutoScoreRow {
    id: i64,
    identifier: String,
    priority: String,
    due_date: Option<String>,
    estimate: Option<f64>,
}

#[tauri::command]
pub fn set_wsjf_scores(
    app: tauri::AppHandle,
    state: State<AppState>,
    input: SetWsjfInput,
) -> Result<WsjfScore, String> {
    state
        .rt
        .block_on(async {
            let bv = clamp_score(input.business_value);
            let tc = clamp_score(input.time_criticality);
            let rr = clamp_score(input.risk_reduction);
            let size = clamp_score(input.job_size);
            let score = calculate_wsjf(bv, tc, rr, size);
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            sqlx::query(
                "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
            )
            .bind(bv)
            .bind(tc)
            .bind(rr)
            .bind(size)
            .bind(score)
            .bind(&now)
            .bind(input.issue_id)
            .execute(&state.pool)
            .await?;

            // Log activity
            sqlx::query(
                "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, 'wsjf_score', NULL, $2, $3)"
            )
            .bind(input.issue_id)
            .bind(format!("{:.2}", score))
            .bind(&now)
            .execute(&state.pool)
            .await?;

            let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(input.issue_id)
                .fetch_one(&state.pool)
                .await?;

            let _ = app.emit("db-changed", ());

            Ok(WsjfScore {
                issue_id: issue.id,
                identifier: issue.identifier,
                title: issue.title,
                business_value: bv,
                time_criticality: tc,
                risk_reduction: rr,
                job_size: size,
                wsjf_score: score,
                priority: issue.priority,
            })
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn auto_score_issue(
    app: tauri::AppHandle,
    state: State<AppState>,
    issue_id: i64,
) -> Result<AutoScoreResult, String> {
    state
        .rt
        .block_on(async {
            let row = sqlx::query_as::<_, AutoScoreRow>(
                "SELECT id, identifier, priority, due_date, estimate FROM issues WHERE id = $1"
            )
            .bind(issue_id)
            .fetch_one(&state.pool)
            .await?;

            // Get labels for risk_reduction
            let label_names: Vec<String> = sqlx::query_scalar(
                "SELECT l.name FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1"
            )
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await?;

            // Get complexity from task_contracts if exists
            let complexity: Option<String> = sqlx::query_scalar(
                "SELECT estimated_complexity FROM task_contracts WHERE issue_id = $1"
            )
            .bind(issue_id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();

            let bv = bv_from_priority(&row.priority);
            let (tc, tc_reason) = tc_from_due_date(row.due_date.as_deref());
            let (rr, rr_reason) = rr_from_labels(&label_names);
            let (size, size_reason) = size_from_estimate(row.estimate, complexity.as_deref());
            let score = calculate_wsjf(bv, tc, rr, size);

            let reasoning = format!(
                "business_value={} (priority={}), time_criticality={} ({}), risk_reduction={} ({}), job_size={} ({})",
                bv, row.priority, tc, tc_reason, rr, rr_reason, size, size_reason
            );

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query(
                "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
            )
            .bind(bv)
            .bind(tc)
            .bind(rr)
            .bind(size)
            .bind(score)
            .bind(&now)
            .bind(issue_id)
            .execute(&state.pool)
            .await?;

            // Log activity
            sqlx::query(
                "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, 'wsjf_auto_score', NULL, $2, $3)"
            )
            .bind(issue_id)
            .bind(format!("{:.2}", score))
            .bind(&now)
            .execute(&state.pool)
            .await?;

            let _ = app.emit("db-changed", ());

            Ok(AutoScoreResult {
                issue_id,
                business_value: bv,
                time_criticality: tc,
                risk_reduction: rr,
                job_size: size,
                wsjf_score: score,
                reasoning,
            })
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_ranked_backlog(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<WsjfScore>, String> {
    state
        .rt
        .block_on(async {
            let rows = sqlx::query_as::<_, crate::models::Issue>(
                r#"
                SELECT i.* FROM issues i
                JOIN statuses s ON i.status_id = s.id
                WHERE i.project_id = $1
                  AND s.category = 'unstarted'
                  AND i.wsjf_score IS NOT NULL
                ORDER BY i.wsjf_score DESC
                "#,
            )
            .bind(project_id)
            .fetch_all(&state.pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|i| WsjfScore {
                    issue_id: i.id,
                    identifier: i.identifier,
                    title: i.title,
                    business_value: i.business_value.unwrap_or(0) as i32,
                    time_criticality: i.time_criticality.unwrap_or(0) as i32,
                    risk_reduction: i.risk_reduction.unwrap_or(0) as i32,
                    job_size: i.job_size.unwrap_or(0) as i32,
                    wsjf_score: i.wsjf_score.unwrap_or(0.0),
                    priority: i.priority,
                })
                .collect())
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn auto_score_project(
    app: tauri::AppHandle,
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<AutoScoreResult>, String> {
    state
        .rt
        .block_on(async {
            // Find unscored issues in this project
            let rows = sqlx::query_as::<_, AutoScoreRow>(
                "SELECT id, identifier, priority, due_date, estimate FROM issues WHERE project_id = $1 AND wsjf_score IS NULL"
            )
            .bind(project_id)
            .fetch_all(&state.pool)
            .await?;

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let mut results = Vec::new();

            for row in rows {
                let label_names: Vec<String> = sqlx::query_scalar(
                    "SELECT l.name FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1"
                )
                .bind(row.id)
                .fetch_all(&state.pool)
                .await?;

                let complexity: Option<String> = sqlx::query_scalar(
                    "SELECT estimated_complexity FROM task_contracts WHERE issue_id = $1"
                )
                .bind(row.id)
                .fetch_optional(&state.pool)
                .await?
                .flatten();

                let bv = bv_from_priority(&row.priority);
                let (tc, tc_reason) = tc_from_due_date(row.due_date.as_deref());
                let (rr, rr_reason) = rr_from_labels(&label_names);
                let (size, size_reason) = size_from_estimate(row.estimate, complexity.as_deref());
                let score = calculate_wsjf(bv, tc, rr, size);

                let reasoning = format!(
                    "business_value={} (priority={}), time_criticality={} ({}), risk_reduction={} ({}), job_size={} ({})",
                    bv, row.priority, tc, tc_reason, rr, rr_reason, size, size_reason
                );

                sqlx::query(
                    "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
                )
                .bind(bv)
                .bind(tc)
                .bind(rr)
                .bind(size)
                .bind(score)
                .bind(&now)
                .bind(row.id)
                .execute(&state.pool)
                .await?;

                results.push(AutoScoreResult {
                    issue_id: row.id,
                    business_value: bv,
                    time_criticality: tc,
                    risk_reduction: rr,
                    job_size: size,
                    wsjf_score: score,
                    reasoning,
                });
            }

            let _ = app.emit("db-changed", ());

            Ok(results)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn recalculate_scores(
    app: tauri::AppHandle,
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<WsjfScore>, String> {
    state
        .rt
        .block_on(async {
            // Recalculate scores for all issues that have WSJF fields set
            let rows = sqlx::query_as::<_, crate::models::Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND business_value IS NOT NULL AND job_size IS NOT NULL"
            )
            .bind(project_id)
            .fetch_all(&state.pool)
            .await?;

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let mut results = Vec::new();

            for issue in rows {
                let bv = issue.business_value.unwrap_or(1) as i32;
                let tc = issue.time_criticality.unwrap_or(3) as i32;
                let rr = issue.risk_reduction.unwrap_or(3) as i32;
                let size = issue.job_size.unwrap_or(5) as i32;
                let score = calculate_wsjf(bv, tc, rr, size);

                sqlx::query(
                    "UPDATE issues SET wsjf_score = $1, updated_at = $2 WHERE id = $3"
                )
                .bind(score)
                .bind(&now)
                .bind(issue.id)
                .execute(&state.pool)
                .await?;

                results.push(WsjfScore {
                    issue_id: issue.id,
                    identifier: issue.identifier,
                    title: issue.title,
                    business_value: bv,
                    time_criticality: tc,
                    risk_reduction: rr,
                    job_size: size,
                    wsjf_score: score,
                    priority: issue.priority,
                });
            }

            let _ = app.emit("db-changed", ());

            Ok(results)
        })
        .map_err(|e: sqlx::Error| e.to_string())
}

/// Standalone auto-score function for use from CLI/MCP (no Tauri state)
pub async fn auto_score_issue_standalone(
    pool: &sqlx::AnyPool,
    issue_id: i64,
) -> Result<AutoScoreResult, sqlx::Error> {
    let row = sqlx::query_as::<_, AutoScoreRow>(
        "SELECT id, identifier, priority, due_date, estimate FROM issues WHERE id = $1"
    )
    .bind(issue_id)
    .fetch_one(pool)
    .await?;

    let label_names: Vec<String> = sqlx::query_scalar(
        "SELECT l.name FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = $1"
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    let complexity: Option<String> = sqlx::query_scalar(
        "SELECT estimated_complexity FROM task_contracts WHERE issue_id = $1"
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let bv = bv_from_priority(&row.priority);
    let (tc, tc_reason) = tc_from_due_date(row.due_date.as_deref());
    let (rr, rr_reason) = rr_from_labels(&label_names);
    let (size, size_reason) = size_from_estimate(row.estimate, complexity.as_deref());
    let score = calculate_wsjf(bv, tc, rr, size);

    let reasoning = format!(
        "business_value={} (priority={}), time_criticality={} ({}), risk_reduction={} ({}), job_size={} ({})",
        bv, row.priority, tc, tc_reason, rr, rr_reason, size, size_reason
    );

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    sqlx::query(
        "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
    )
    .bind(bv)
    .bind(tc)
    .bind(rr)
    .bind(size)
    .bind(score)
    .bind(&now)
    .bind(issue_id)
    .execute(pool)
    .await?;

    Ok(AutoScoreResult {
        issue_id,
        business_value: bv,
        time_criticality: tc,
        risk_reduction: rr,
        job_size: size,
        wsjf_score: score,
        reasoning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_wsjf() {
        assert_eq!(calculate_wsjf(8, 7, 5, 3), (8.0 + 7.0 + 5.0) / 3.0);
    }

    #[test]
    fn test_calculate_wsjf_zero_size() {
        assert_eq!(calculate_wsjf(8, 7, 5, 0), 0.0);
    }

    #[test]
    fn test_clamp_score() {
        assert_eq!(clamp_score(0), 1);
        assert_eq!(clamp_score(5), 5);
        assert_eq!(clamp_score(15), 10);
    }

    #[test]
    fn test_bv_from_priority() {
        assert_eq!(bv_from_priority("urgent"), 10);
        assert_eq!(bv_from_priority("high"), 8);
        assert_eq!(bv_from_priority("medium"), 5);
        assert_eq!(bv_from_priority("low"), 2);
        assert_eq!(bv_from_priority("none"), 1);
    }

    #[test]
    fn test_rr_from_labels() {
        assert_eq!(rr_from_labels(&["security".to_string()]).0, 9);
        assert_eq!(rr_from_labels(&["Bug".to_string()]).0, 9);
        assert_eq!(rr_from_labels(&["performance".to_string()]).0, 6);
        assert_eq!(rr_from_labels(&["feature".to_string()]).0, 3);
        assert_eq!(rr_from_labels(&["docs".to_string()]).0, 1);
        assert_eq!(rr_from_labels(&["ui".to_string()]).0, 3);
    }

    #[test]
    fn test_size_from_estimate() {
        assert_eq!(size_from_estimate(Some(2.0), None).0, 2);
        assert_eq!(size_from_estimate(Some(5.0), None).0, 5);
        assert_eq!(size_from_estimate(Some(10.0), None).0, 8);
        assert_eq!(size_from_estimate(None, Some("small")).0, 2);
        assert_eq!(size_from_estimate(None, None).0, 5);
    }
}
