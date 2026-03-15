use sqlx::SqlitePool;

/// Returns true if a task needs decomposition.
///
/// A task needs decomposition when:
/// - It has a task_contract that is NOT of type 'decomposition', AND
/// - Either:
///   - estimated_complexity is 'large' and there are no child issues, OR
///   - success_criteria is empty ('[]' or '')
pub async fn check_decomposition_needed(
    pool: &SqlitePool,
    issue_id: i64,
) -> Result<bool, sqlx::Error> {
    // Fetch the task_contract for this issue (must exist and not be a decomposition task itself)
    let contract = sqlx::query_as::<_, (String, Option<String>, String)>(
        "SELECT type, estimated_complexity, success_criteria FROM task_contracts WHERE issue_id = ?",
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?;

    let (contract_type, estimated_complexity, success_criteria) = match contract {
        Some(c) => c,
        None => return Ok(false), // No task_contract means no decomposition needed
    };

    // Skip decomposition tasks themselves
    if contract_type == "decomposition" {
        return Ok(false);
    }

    // Check if success_criteria is empty
    let criteria_empty =
        success_criteria.is_empty() || success_criteria == "[]" || success_criteria == "null";

    if criteria_empty {
        return Ok(true);
    }

    // Check if estimated_complexity is 'large' and no child issues exist
    if estimated_complexity.as_deref() == Some("large") {
        let child_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE parent_id = ?")
                .bind(issue_id)
                .fetch_one(pool)
                .await?;

        if child_count == 0 {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Creates a decomposition task for the given parent issue.
///
/// This creates a new child issue with a 'decomposition' task_contract,
/// adds a blocking relation (decomposition blocks parent), and sets the
/// parent's task_state to 'blocked'.
///
/// Returns the new decomposition issue's id.
pub async fn create_decomposition_task(
    pool: &SqlitePool,
    parent_issue_id: i64,
) -> Result<i64, sqlx::Error> {
    // 1. Get the parent issue details
    let parent = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT project_id, identifier, title, priority FROM issues WHERE id = ?",
    )
    .bind(parent_issue_id)
    .fetch_one(pool)
    .await?;

    let (project_id, parent_identifier, parent_title, parent_priority) = parent;

    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    // 2. Start a transaction
    let mut tx = pool.begin().await?;

    // 3. Increment project's issue_counter and get new identifier
    let (counter, prefix): (i64, String) = sqlx::query_as(
        "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix",
    )
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;
    let identifier = format!("{}-{}", prefix, counter);

    // 4. Find an 'unstarted' category status for the project
    let status_id: i64 = sqlx::query_scalar(
        "SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position ASC LIMIT 1",
    )
    .bind(project_id)
    .fetch_one(&mut *tx)
    .await?;

    // 5. Create the decomposition issue
    let title = format!("Decompose: {}", parent_title);
    let description = format!(
        "Break down {} into atomic sub-tasks with clear contracts",
        parent_identifier
    );

    let result = sqlx::query(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, NULL, ?, 0.0, ?, ?)",
    )
    .bind(project_id)
    .bind(&identifier)
    .bind(&title)
    .bind(&description)
    .bind(status_id)
    .bind(&parent_priority)
    .bind(parent_issue_id)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    let new_issue_id = result.last_insert_rowid();

    // 6. Create the task_contract
    let objective = format!(
        "Analyze {} and break it into atomic, independently executable sub-tasks. Each sub-task must have a clear objective, success criteria, and skill requirements.",
        parent_identifier
    );
    let required_skills = r#"["architecture"]"#;
    let context = serde_json::json!({
        "files": [],
        "related_tasks": [parent_identifier],
        "prior_attempts": []
    });
    let context_json = serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

    sqlx::query(
        "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES (?, 'decomposition', 'queued', ?, ?, '[]', '[]', ?, 'medium', 30, 0)",
    )
    .bind(new_issue_id)
    .bind(&objective)
    .bind(&context_json)
    .bind(required_skills)
    .execute(&mut *tx)
    .await?;

    // 7. Create issue_relation: decomposition task blocks the parent
    sqlx::query(
        "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocks')",
    )
    .bind(new_issue_id)
    .bind(parent_issue_id)
    .execute(&mut *tx)
    .await?;

    // 8. Set the parent's task_state to 'blocked'
    sqlx::query("UPDATE task_contracts SET task_state = 'blocked' WHERE issue_id = ?")
        .bind(parent_issue_id)
        .execute(&mut *tx)
        .await?;

    // 9. Commit transaction
    tx.commit().await?;

    // 10. Return the new issue_id
    Ok(new_issue_id)
}
