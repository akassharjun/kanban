use sqlx::PgPool;

/// Full task contract joined with issue data, returned by next_task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullTaskContract {
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub parent_id: Option<i64>,
    pub issue_id: i64,
    pub r#type: String,
    pub task_state: String,
    pub objective: String,
    pub context: serde_json::Value,
    pub constraints: serde_json::Value,
    pub success_criteria: serde_json::Value,
    pub required_skills: serde_json::Value,
    pub estimated_complexity: Option<String>,
    pub timeout_minutes: i64,
    pub attempt_count: i64,
}

/// Intermediate row for candidate task queries.
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct CandidateRow {
    issue_id: i64,
    identifier: String,
    priority: String,
    required_skills: String,
    estimated_complexity: Option<String>,
}

/// Intermediate row for building the full contract.
#[derive(Debug, sqlx::FromRow)]
struct FullContractRow {
    identifier: String,
    title: String,
    description: Option<String>,
    priority: String,
    parent_id: Option<i64>,
    issue_id: i64,
    r#type: String,
    task_state: String,
    objective: String,
    context: String,
    constraints: String,
    success_criteria: String,
    required_skills: String,
    estimated_complexity: Option<String>,
    timeout_minutes: i64,
    attempt_count: i64,
}

fn complexity_rank(c: &str) -> i32 {
    match c {
        "small" => 1,
        "medium" => 2,
        "large" => 3,
        _ => 99,
    }
}

/// Find and atomically claim the next best task for the given agent.
///
/// Returns `None` if no suitable task is available (agent at capacity,
/// no matching tasks, or all candidates were claimed by other agents).
pub async fn next_task(
    pool: &PgPool,
    agent_id: &str,
    agent_skills: &[String],
    agent_max_complexity: &str,
    agent_max_concurrent: i64,
) -> Result<Option<FullTaskContract>, sqlx::Error> {
    // 1. Check agent capacity
    let active_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')",
    )
    .bind(agent_id)
    .fetch_one(pool)
    .await?;

    if active_count.0 >= agent_max_concurrent {
        return Ok(None);
    }

    // 2. Query candidate tasks ordered by priority then creation time
    let candidates: Vec<CandidateRow> = sqlx::query_as(
        r#"
        SELECT tc.issue_id, i.identifier, i.priority, tc.required_skills, tc.estimated_complexity
        FROM task_contracts tc
        JOIN issues i ON tc.issue_id = i.id
        WHERE tc.task_state = 'queued'
          AND NOT EXISTS (
            SELECT 1 FROM issue_relations ir
            LEFT JOIN task_contracts dtc ON dtc.issue_id = ir.source_issue_id
            WHERE ir.target_issue_id = tc.issue_id
              AND ir.relation_type = 'blocks'
              AND (dtc.task_state IS NULL OR dtc.task_state NOT IN ('completed'))
          )
        ORDER BY
          CASE i.priority WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 WHEN 'low' THEN 3 ELSE 4 END,
          i.created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let max_rank = complexity_rank(agent_max_complexity);

    for candidate in &candidates {
        // 3. Filter: complexity check
        if let Some(ref est) = candidate.estimated_complexity {
            if complexity_rank(est) > max_rank {
                continue;
            }
        }

        // 3. Filter: skills subset check
        let task_skills: Vec<String> = serde_json::from_str(&candidate.required_skills)
            .unwrap_or_default();
        let all_skills_met = task_skills.iter().all(|s| agent_skills.contains(s));
        if !all_skills_met {
            continue;
        }

        // 4. Atomic claim via optimistic locking
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE task_contracts SET claimed_by = $1, claimed_at = $2, task_state = 'claimed' \
             WHERE issue_id = $3 AND claimed_by IS NULL AND task_state = 'queued'",
        )
        .bind(agent_id)
        .bind(&now)
        .bind(candidate.issue_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            // 6. Another agent claimed it first, try next candidate
            continue;
        }

        // 5a. Sync issue status to a 'started' category status
        sqlx::query(
            "UPDATE issues SET status_id = (
                SELECT s.id FROM statuses s
                WHERE s.project_id = issues.project_id AND s.category = 'started'
                ORDER BY s.position ASC LIMIT 1
             ), updated_at = $1
             WHERE id = $2",
        )
        .bind(&now)
        .bind(candidate.issue_id)
        .execute(pool)
        .await?;

        // 5b. Log claim in execution_logs
        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) \
             VALUES ($1, $2, (SELECT attempt_count FROM task_contracts WHERE issue_id = $3), 'claim', 'Task claimed by agent', $4)",
        )
        .bind(candidate.issue_id)
        .bind(agent_id)
        .bind(candidate.issue_id)
        .bind(&now)
        .execute(pool)
        .await?;

        // 5c. Build and return full contract
        return Ok(build_full_contract(pool, candidate.issue_id).await?);
    }

    // 7. No suitable candidate found
    Ok(None)
}

/// Build a full task contract by joining task_contracts with issues.
pub async fn build_full_contract(
    pool: &PgPool,
    issue_id: i64,
) -> Result<Option<FullTaskContract>, sqlx::Error> {
    let row: Option<FullContractRow> = sqlx::query_as(
        r#"
        SELECT
            i.identifier,
            i.title,
            i.description,
            i.priority,
            i.parent_id,
            tc.issue_id,
            tc.type AS "type",
            tc.task_state,
            tc.objective,
            tc.context,
            tc.constraints,
            tc.success_criteria,
            tc.required_skills,
            tc.estimated_complexity,
            tc.timeout_minutes,
            tc.attempt_count
        FROM task_contracts tc
        JOIN issues i ON tc.issue_id = i.id
        WHERE tc.issue_id = $1
        "#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| FullTaskContract {
        identifier: r.identifier,
        title: r.title,
        description: r.description,
        priority: r.priority,
        parent_id: r.parent_id,
        issue_id: r.issue_id,
        r#type: r.r#type,
        task_state: r.task_state,
        objective: r.objective,
        context: serde_json::from_str(&r.context).unwrap_or(serde_json::Value::Object(Default::default())),
        constraints: serde_json::from_str(&r.constraints).unwrap_or(serde_json::Value::Array(Default::default())),
        success_criteria: serde_json::from_str(&r.success_criteria).unwrap_or(serde_json::Value::Array(Default::default())),
        required_skills: serde_json::from_str(&r.required_skills).unwrap_or(serde_json::Value::Array(Default::default())),
        estimated_complexity: r.estimated_complexity,
        timeout_minutes: r.timeout_minutes,
        attempt_count: r.attempt_count,
    }))
}
