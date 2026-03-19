use sqlx::AnyPool;

async fn test_db() -> AnyPool {
    let (pool, _backend) = kanban_lib::db::init_db(Some("sqlite://:memory:"))
        .await
        .expect("Failed to initialize in-memory database");
    pool
}

async fn create_test_project(pool: &AnyPool) -> (i64, i64) {
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    let project_id: i64 = sqlx::query_scalar(
        "INSERT INTO projects (name, prefix, path, status, issue_counter, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind("Test Project")
    .bind("TST")
    .bind("/tmp/test")
    .bind("active")
    .bind(0i64)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await
    .expect("Failed to insert project");

    let statuses = [
        ("Backlog", "unstarted", 0i64),
        ("Todo", "unstarted", 1),
        ("In Progress", "started", 2),
        ("In Review", "started", 3),
        ("Blocked", "blocked", 4),
        ("Done", "completed", 5),
        ("Discarded", "discarded", 6),
    ];

    let mut backlog_status_id: i64 = 0;

    for (name, category, position) in &statuses {
        let status_id: i64 = sqlx::query_scalar(
            "INSERT INTO statuses (project_id, name, category, position) \
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(project_id)
        .bind(*name)
        .bind(*category)
        .bind(*position)
        .fetch_one(pool)
        .await
        .expect("Failed to insert status");

        if *name == "Backlog" {
            backlog_status_id = status_id;
        }
    }

    (project_id, backlog_status_id)
}

#[tokio::test]
async fn create_project_and_default_statuses() {
    let pool = test_db().await;
    let (project_id, _backlog_status_id) = create_test_project(&pool).await;

    // Verify 7 statuses were created
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM statuses WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count statuses");

    assert_eq!(count, 7, "Expected 7 default statuses, got {}", count);

    // Verify the Project model can SELECT * the row (catches model/schema mismatches)
    let project: kanban_lib::models::Project =
        sqlx::query_as("SELECT * FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT * project — model/schema mismatch");

    assert_eq!(project.name, "Test Project");
    assert_eq!(project.prefix, "TST");
    assert_eq!(project.path, Some("/tmp/test".to_string()));
}

#[tokio::test]
async fn create_issue_with_all_fields() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    // This is the exact INSERT query from the Tauri command handler
    let issue_id: i64 = sqlx::query_scalar(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, \
         assignee_id, parent_id, position, estimate, due_date, epic_id, milestone_id, \
         created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING id",
    )
    .bind(project_id)
    .bind("TST-1")
    .bind("Test Issue")
    .bind(None::<String>)      // description
    .bind(backlog_status_id)
    .bind("medium")            // priority
    .bind(None::<i64>)         // assignee_id
    .bind(None::<i64>)         // parent_id
    .bind(1.0f64)              // position
    .bind(None::<f64>)         // estimate
    .bind(None::<String>)      // due_date
    .bind(None::<i64>)         // epic_id
    .bind(None::<i64>)         // milestone_id
    .bind(&now)                // created_at
    .bind(&now)                // updated_at
    .fetch_one(&pool)
    .await
    .expect("Failed to INSERT issue — query or schema mismatch");

    // Verify the Issue model can SELECT * the row (catches model/schema mismatches)
    let issue: kanban_lib::models::Issue =
        sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT * issue — model/schema mismatch");

    assert_eq!(issue.identifier, "TST-1");
    assert_eq!(issue.title, "Test Issue");
    assert_eq!(issue.priority, "medium");
    assert_eq!(issue.epic_id, None);
    assert_eq!(issue.milestone_id, None);
}
