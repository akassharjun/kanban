use sqlx::AnyPool;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

async fn test_db() -> AnyPool {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp = std::env::temp_dir().join(format!("kanban-test-{}-{}.db", std::process::id(), id));
    // Clean up any leftover file from a previous run
    let _ = std::fs::remove_file(&tmp);
    let url = format!("sqlite://{}?mode=rwc", tmp.display());
    let (pool, _backend) = kanban_lib::db::init_db(Some(&url))
        .await
        .expect("Failed to initialize test database");
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

// Helper to insert a single issue and return its id
async fn insert_issue(
    pool: &AnyPool,
    project_id: i64,
    identifier: &str,
    title: &str,
    status_id: i64,
    priority: &str,
    assignee_id: Option<i64>,
) -> i64 {
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    sqlx::query_scalar(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, \
         assignee_id, parent_id, position, estimate, due_date, epic_id, milestone_id, \
         created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING id",
    )
    .bind(project_id)
    .bind(identifier)
    .bind(title)
    .bind(None::<String>)
    .bind(status_id)
    .bind(priority)
    .bind(assignee_id)
    .bind(None::<i64>)
    .bind(1.0f64)
    .bind(None::<f64>)
    .bind(None::<String>)
    .bind(None::<i64>)
    .bind(None::<i64>)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await
    .expect("Failed to INSERT issue")
}

#[tokio::test]
async fn update_issue_status_and_priority() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    let issue_id = insert_issue(&pool, project_id, "TST-1", "My Issue", backlog_status_id, "low", None).await;

    // Get the "In Progress" status id
    let in_progress_id: i64 = sqlx::query_scalar(
        "SELECT id FROM statuses WHERE project_id = $1 AND name = $2",
    )
    .bind(project_id)
    .bind("In Progress")
    .fetch_one(&pool)
    .await
    .expect("Failed to get In Progress status");

    // Update status and priority
    sqlx::query("UPDATE issues SET status_id = $1, priority = $2 WHERE id = $3")
        .bind(in_progress_id)
        .bind("urgent")
        .bind(issue_id)
        .execute(&pool)
        .await
        .expect("Failed to update issue");

    // Verify via SELECT
    let issue: kanban_lib::models::Issue =
        sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT issue");

    assert_eq!(issue.status_id, in_progress_id);
    assert_eq!(issue.priority, "urgent");
}

#[tokio::test]
async fn list_issues_by_status() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    // Get the "Todo" status id
    let todo_id: i64 = sqlx::query_scalar(
        "SELECT id FROM statuses WHERE project_id = $1 AND name = $2",
    )
    .bind(project_id)
    .bind("Todo")
    .fetch_one(&pool)
    .await
    .expect("Failed to get Todo status");

    // Create 2 Backlog issues and 1 Todo issue
    insert_issue(&pool, project_id, "TST-1", "Backlog Issue 1", backlog_status_id, "medium", None).await;
    insert_issue(&pool, project_id, "TST-2", "Backlog Issue 2", backlog_status_id, "low", None).await;
    insert_issue(&pool, project_id, "TST-3", "Todo Issue 1", todo_id, "high", None).await;

    // SELECT filtered by Backlog
    let issues: Vec<kanban_lib::models::Issue> =
        sqlx::query_as("SELECT * FROM issues WHERE project_id = $1 AND status_id = $2")
            .bind(project_id)
            .bind(backlog_status_id)
            .fetch_all(&pool)
            .await
            .expect("Failed to SELECT issues by status");

    assert_eq!(issues.len(), 2, "Expected 2 Backlog issues, got {}", issues.len());

    let titles: Vec<&str> = issues.iter().map(|i| i.title.as_str()).collect();
    assert!(titles.contains(&"Backlog Issue 1"));
    assert!(titles.contains(&"Backlog Issue 2"));
}

#[tokio::test]
async fn create_member_and_assign_to_issue() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    // Verify the default "You" member from seed_defaults exists
    let you: kanban_lib::models::Member =
        sqlx::query_as("SELECT * FROM members WHERE name = $1")
            .bind("You")
            .fetch_one(&pool)
            .await
            .expect("Default 'You' member not found — seed_defaults may have failed");

    assert_eq!(you.name, "You");

    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    // Create a new member "alice"
    let alice_id: i64 = sqlx::query_scalar(
        "INSERT INTO members (name, display_name, email, avatar_color, created_at) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind("alice")
    .bind("Alice")
    .bind("alice@example.com")
    .bind("#f59e0b")
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert member alice");

    // Create an issue assigned to alice
    let issue_id = insert_issue(&pool, project_id, "TST-1", "Alice's Issue", backlog_status_id, "medium", Some(alice_id)).await;

    // Verify assignee_id matches
    let issue: kanban_lib::models::Issue =
        sqlx::query_as("SELECT * FROM issues WHERE id = $1")
            .bind(issue_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT issue");

    assert_eq!(issue.assignee_id, Some(alice_id));
}

#[tokio::test]
async fn create_label_and_attach_to_issue() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    // Create a label
    let label_id: i64 = sqlx::query_scalar(
        "INSERT INTO labels (project_id, name, color) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(project_id)
    .bind("bug")
    .bind("#ef4444")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert label");

    // Create an issue
    let issue_id = insert_issue(&pool, project_id, "TST-1", "Bug Issue", backlog_status_id, "high", None).await;

    // Attach label to issue via junction table
    sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
        .bind(issue_id)
        .bind(label_id)
        .execute(&pool)
        .await
        .expect("Failed to insert issue_label");

    // Verify via JOIN query
    let label: kanban_lib::models::Label = sqlx::query_as(
        "SELECT l.id, l.project_id, l.name, l.color \
         FROM labels l \
         JOIN issue_labels il ON il.label_id = l.id \
         WHERE il.issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to SELECT label via JOIN");

    assert_eq!(label.name, "bug");
    assert_eq!(label.color, "#ef4444");
    assert_eq!(label.project_id, project_id);
}

#[tokio::test]
async fn create_comment_and_activity_log() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    let issue_id = insert_issue(&pool, project_id, "TST-1", "Commented Issue", backlog_status_id, "medium", None).await;

    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    // Insert a comment (member_id=1 = "You" from seed_defaults)
    let comment_id: i64 = sqlx::query_scalar(
        "INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(issue_id)
    .bind(1i64)
    .bind("This is a test comment")
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert comment");

    // Verify comment via Comment model
    let comment: kanban_lib::models::Comment =
        sqlx::query_as("SELECT * FROM comments WHERE id = $1")
            .bind(comment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT comment");

    assert_eq!(comment.issue_id, issue_id);
    assert_eq!(comment.content, "This is a test comment");

    // Insert an activity_log entry
    let log_id: i64 = sqlx::query_scalar(
        "INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, actor_id, actor_type, timestamp) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(issue_id)
    .bind("status")
    .bind("Backlog")
    .bind("In Progress")
    .bind(1i64)
    .bind("user")
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("Failed to insert activity_log entry");

    // Verify activity_log via ActivityLogEntry model
    let entry: kanban_lib::models::ActivityLogEntry =
        sqlx::query_as("SELECT * FROM activity_log WHERE id = $1")
            .bind(log_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to SELECT activity_log entry");

    assert_eq!(entry.issue_id, issue_id);
    assert_eq!(entry.field_changed, "status");
    assert_eq!(entry.old_value, Some("Backlog".to_string()));
    assert_eq!(entry.new_value, Some("In Progress".to_string()));
    assert_eq!(entry.actor_id, Some(1i64));
}

#[tokio::test]
async fn delete_issue() {
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;

    let issue_id = insert_issue(&pool, project_id, "TST-1", "To Be Deleted", backlog_status_id, "medium", None).await;

    // DELETE the issue
    sqlx::query("DELETE FROM issues WHERE id = $1")
        .bind(issue_id)
        .execute(&pool)
        .await
        .expect("Failed to DELETE issue");

    // Verify COUNT=0
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to COUNT issues");

    assert_eq!(count, 0, "Expected 0 issues after delete, got {}", count);
}

#[tokio::test]
async fn soft_delete_project() {
    let pool = test_db().await;
    let (project_id, _backlog_status_id) = create_test_project(&pool).await;

    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%SZ")
        .to_string();

    // Soft delete: set deleted_at
    sqlx::query("UPDATE projects SET deleted_at = $1 WHERE id = $2")
        .bind(&now)
        .bind(project_id)
        .execute(&pool)
        .await
        .expect("Failed to soft-delete project");

    // Verify SELECT WHERE deleted_at IS NULL returns 0
    let active_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE id = $1 AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to COUNT active projects");

    assert_eq!(active_count, 0, "Expected 0 active projects, got {}", active_count);

    // Verify SELECT all returns 1 (soft delete, not hard)
    let total_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to COUNT all projects");

    assert_eq!(total_count, 1, "Expected 1 total project (soft deleted), got {}", total_count);
}

#[tokio::test]
async fn create_saved_view() {
    let pool = test_db().await;
    let (project_id, _) = create_test_project(&pool).await;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    let view_id: i64 = sqlx::query_scalar(
        "INSERT INTO saved_views (project_id, name, filters, sort_by, sort_direction, view_mode, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
    )
    .bind(project_id)
    .bind("My View")
    .bind("{\"priority\":\"high\"}")
    .bind(Some("created_at"))
    .bind(Some("desc"))
    .bind(Some("list"))
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("INSERT saved_view failed — schema may be incomplete");

    let view = sqlx::query_as::<_, kanban_lib::models::SavedView>(
        "SELECT * FROM saved_views WHERE id = $1"
    )
    .bind(view_id)
    .fetch_one(&pool)
    .await
    .expect("SELECT saved_view failed — model/schema mismatch");

    assert_eq!(view.name, "My View");
    assert_eq!(view.project_id, project_id);
}

#[tokio::test]
async fn create_automation_rule() {
    let pool = test_db().await;
    let (project_id, _) = create_test_project(&pool).await;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    let rule_id: i64 = sqlx::query_scalar(
        "INSERT INTO automation_rules (project_id, name, enabled, trigger_type, trigger_config, conditions, actions, execution_count, last_executed_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
    )
    .bind(project_id)
    .bind("Auto-assign bug label")
    .bind(true)
    .bind("issue_created")
    .bind("{}")
    .bind("[]")
    .bind("[{\"type\":\"add_label\",\"label_id\":1}]")
    .bind(0i64)
    .bind(None::<String>)
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("INSERT automation_rule failed — schema may be incomplete");

    let rule = sqlx::query_as::<_, kanban_lib::models::AutomationRule>(
        "SELECT * FROM automation_rules WHERE id = $1"
    )
    .bind(rule_id)
    .fetch_one(&pool)
    .await
    .expect("SELECT automation_rule failed — model/schema mismatch");

    assert_eq!(rule.name, "Auto-assign bug label");
    assert_eq!(rule.trigger_type, "issue_created");
    assert_eq!(rule.project_id, project_id);
}

#[tokio::test]
async fn mcp_create_issue_insert() {
    // This test verifies the exact INSERT used by the MCP server's create_issue method
    // (mcp.rs ~line 894) to catch column mismatches early
    let pool = test_db().await;
    let (project_id, backlog_status_id) = create_test_project(&pool).await;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    // Increment counter like MCP does
    let (counter, prefix): (i64, String) = sqlx::query_as(
        "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
    ).bind(project_id).fetch_one(&pool).await.expect("Failed to increment counter");
    let identifier = format!("{}-{}", prefix, counter);

    // MCP 15-column INSERT (matches mcp.rs create_issue)
    let issue_id: i64 = sqlx::query_scalar(
        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, epic_id, milestone_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING id"
    )
    .bind(project_id)
    .bind(&identifier)
    .bind("MCP Issue")
    .bind(Some("Created via MCP"))
    .bind(backlog_status_id)
    .bind("medium")
    .bind(None::<i64>)    // assignee_id
    .bind(None::<i64>)    // parent_id
    .bind(0.0f64)         // position
    .bind(Some(3.5f64))   // estimate - test with a value
    .bind(Some("2026-04-01"))  // due_date - test with a value
    .bind(None::<i64>)    // epic_id
    .bind(None::<i64>)    // milestone_id
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .expect("MCP INSERT failed — check mcp.rs create_issue query matches schema");

    let issue = sqlx::query_as::<_, kanban_lib::models::Issue>(
        "SELECT * FROM issues WHERE id = $1"
    ).bind(issue_id).fetch_one(&pool).await.expect("SELECT failed");

    assert_eq!(issue.identifier, "TST-1");
    assert_eq!(issue.estimate, Some(3.5));
    assert_eq!(issue.due_date, Some("2026-04-01".to_string()));
}

#[tokio::test]
async fn cli_project_soft_delete() {
    // Verify soft-delete works correctly (cli.rs ProjectAction::Delete)
    let pool = test_db().await;
    let (project_id, _) = create_test_project(&pool).await;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    // Soft-delete (same as CLI does now)
    sqlx::query("UPDATE projects SET deleted_at = $1, updated_at = $2 WHERE id = $3")
        .bind(&now).bind(&now).bind(project_id)
        .execute(&pool).await.expect("Soft delete failed");

    // Verify: not in active list
    let active: Vec<kanban_lib::models::Project> = sqlx::query_as(
        "SELECT * FROM projects WHERE deleted_at IS NULL"
    ).fetch_all(&pool).await.unwrap();
    assert_eq!(active.len(), 0);

    // Verify: still exists in DB
    let project = sqlx::query_as::<_, kanban_lib::models::Project>(
        "SELECT * FROM projects WHERE id = $1"
    ).bind(project_id).fetch_one(&pool).await.expect("Project should still exist");
    assert!(project.deleted_at.is_some());
}
