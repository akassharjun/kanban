use clap::{Parser, Subcommand};
use kanban_lib::db;
use kanban_lib::models::*;
use kanban_lib::orchestration;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};

#[derive(Parser)]
#[command(name = "kanban", about = "Kanban - Desktop Project Management CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage projects
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Manage issues
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
    /// Manage team members
    Member {
        #[command(subcommand)]
        action: MemberAction,
    },
    /// Manage labels
    Label {
        #[command(subcommand)]
        action: LabelAction,
    },
    /// Manage notifications
    Notifications {
        #[command(subcommand)]
        action: NotificationAction,
    },
    /// Manage issue comments
    Comment {
        #[command(subcommand)]
        action: CommentAction,
    },
    /// Manage AI agents
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    /// Manage task contracts
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// View system metrics
    Metrics {
        #[arg(long)]
        project: Option<i64>,
        #[arg(long)]
        agent: Option<String>,
    },
    /// Export all data to JSON
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import data from JSON
    Import {
        /// Input file path
        file: String,
    },
}

#[derive(Subcommand)]
enum ProjectAction {
    /// List all projects
    List,
    /// Create a new project
    Create {
        name: String,
        #[arg(short, long)]
        prefix: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        icon: Option<String>,
    },
    /// Update a project
    Update {
        id: i64,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Delete a project
    Delete { id: i64 },
}

#[derive(Subcommand)]
enum IssueAction {
    /// List issues
    List {
        #[arg(short, long)]
        project: i64,
        #[arg(long)]
        status: Option<i64>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assignee: Option<i64>,
    },
    /// Create an issue
    Create {
        #[arg(short, long)]
        project: i64,
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        status: i64,
        #[arg(long)]
        priority: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        assignee: Option<i64>,
        #[arg(long)]
        parent: Option<i64>,
    },
    /// Update an issue by identifier (e.g. KAN-42)
    Update {
        identifier: String,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        status: Option<i64>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(short, long)]
        assignee: Option<i64>,
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Search issues
    Search {
        #[arg(short, long)]
        project: i64,
        query: String,
    },
    /// Set parent issue
    Move {
        identifier: String,
        #[arg(long)]
        parent: String,
    },
    /// Add a blocker relation
    Block {
        identifier: String,
        #[arg(long)]
        by: String,
    },
    /// Add a related relation
    Relate {
        identifier: String,
        #[arg(long)]
        to: String,
    },
    /// Delete an issue by identifier
    Delete { identifier: String },
}

#[derive(Subcommand)]
enum MemberAction {
    /// List all members
    List,
    /// Add a new member
    Add {
        name: String,
        #[arg(short, long)]
        email: Option<String>,
        #[arg(short, long)]
        display_name: Option<String>,
    },
    /// Delete a member
    Delete { id: i64 },
}

#[derive(Subcommand)]
enum LabelAction {
    /// List labels for a project
    List {
        #[arg(short, long)]
        project: i64,
    },
    /// Create a label
    Create {
        #[arg(short, long)]
        project: i64,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        color: String,
    },
    /// Delete a label
    Delete { id: i64 },
}

#[derive(Subcommand)]
enum NotificationAction {
    /// List recent notifications
    List,
    /// Clear all notifications
    Clear,
}

#[derive(Subcommand)]
enum CommentAction {
    /// List comments on an issue
    List {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
    },
    /// Add a comment to an issue
    Add {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
        /// Comment content (markdown)
        #[arg(short, long)]
        content: String,
        /// Member ID of the commenter
        #[arg(short, long)]
        member: Option<i64>,
    },
    /// Delete a comment
    Delete {
        /// Comment ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum AgentAction {
    /// Register a new agent
    Register {
        #[arg(long)]
        name: String,
        /// Comma-delimited skills
        #[arg(long)]
        skills: String,
        /// Comma-delimited task types (optional)
        #[arg(long)]
        task_types: Option<String>,
        #[arg(long, default_value = "1")]
        max_concurrent: i64,
        #[arg(long, default_value = "large")]
        max_complexity: String,
    },
    /// Send a heartbeat for an agent
    Heartbeat {
        #[arg(long)]
        id: String,
    },
    /// Deregister an agent
    Deregister {
        #[arg(long)]
        id: String,
    },
    /// List all agents
    List,
    /// Show stats for an agent
    Stats {
        id: String,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    /// Get next available task for an agent
    Next {
        #[arg(long)]
        agent: String,
        /// Comma-delimited skill override
        #[arg(long)]
        skills: Option<String>,
    },
    /// Start executing a claimed task
    Start {
        identifier: String,
        #[arg(long)]
        agent: String,
    },
    /// Complete a task
    Complete {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        confidence: f64,
        #[arg(long)]
        summary: String,
        /// Optional JSON string for artifacts
        #[arg(long)]
        artifacts: Option<String>,
    },
    /// Fail a task
    Fail {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        reason: String,
    },
    /// Unclaim a task
    Unclaim {
        identifier: String,
        #[arg(long)]
        agent: String,
    },
    /// Log an execution entry
    Log {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long, name = "type")]
        entry_type: String,
        #[arg(long)]
        message: String,
        /// Optional JSON metadata
        #[arg(long)]
        meta: Option<String>,
    },
    /// Create a task with a contract
    Create {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        status: i64,
        #[arg(long, name = "type")]
        task_type: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        /// Comma-delimited required skills
        #[arg(long)]
        skills: Option<String>,
        #[arg(long)]
        complexity: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// Parent issue identifier
        #[arg(long)]
        parent: Option<String>,
        /// Comma-delimited dependency identifiers
        #[arg(long)]
        depends_on: Option<String>,
        #[arg(long)]
        context_files: Option<String>,
        /// JSON string for constraints
        #[arg(long)]
        constraints: Option<String>,
        /// JSON string for success criteria
        #[arg(long)]
        success_criteria: Option<String>,
        #[arg(long)]
        assignee: Option<i64>,
        #[arg(long)]
        timeout: Option<i64>,
    },
    /// Get a task contract by identifier
    Get {
        identifier: String,
    },
    /// Replay execution logs for a task
    Replay {
        identifier: String,
    },
    /// Show prior attempts for a task
    Attempts {
        identifier: String,
    },
    /// List tasks
    List {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        available: bool,
    },
    /// List child issues
    Children {
        identifier: String,
    },
    /// Approve a validating task
    Approve {
        identifier: String,
    },
    /// Reject a validating task
    Reject {
        identifier: String,
    },
    /// Invalidate a task and block downstream
    Invalidate {
        identifier: String,
        #[arg(long)]
        reason: String,
    },
    /// Search tasks
    Search {
        #[arg(long)]
        project: i64,
        query: String,
    },
    /// Update task fields
    Update {
        identifier: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        complexity: Option<String>,
        #[arg(long)]
        skills: Option<String>,
    },
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
struct IssueLabelRow {
    issue_id: i64,
    label_id: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ExportData {
    projects: Vec<Project>,
    statuses: Vec<Status>,
    members: Vec<Member>,
    issues: Vec<Issue>,
    labels: Vec<Label>,
    issue_labels: Vec<IssueLabelRow>,
    issue_relations: Vec<IssueRelation>,
    issue_templates: Vec<IssueTemplate>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let pool = db::init_db().await?;

    match cli.command {
        Commands::Project { action } => handle_project(&pool, action, cli.json).await?,
        Commands::Issue { action } => handle_issue(&pool, action, cli.json).await?,
        Commands::Member { action } => handle_member(&pool, action, cli.json).await?,
        Commands::Label { action } => handle_label(&pool, action, cli.json).await?,
        Commands::Notifications { action } => handle_notifications(&pool, action, cli.json).await?,
        Commands::Comment { action } => handle_comment(&pool, action, cli.json).await?,
        Commands::Agent { action } => handle_agent(&pool, action, cli.json).await?,
        Commands::Task { action } => handle_task(&pool, action, cli.json).await?,
        Commands::Metrics { project, agent } => handle_metrics(&pool, project, agent, cli.json).await?,
        Commands::Export { output } => handle_export(&pool, output).await?,
        Commands::Import { file } => handle_import(&pool, file).await?,
    }

    Ok(())
}

async fn handle_project(
    pool: &SqlitePool,
    action: ProjectAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ProjectAction::List => {
            let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY name")
                .fetch_all(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&projects)?);
            } else {
                for p in &projects {
                    println!("{} | {} | {} | {}", p.id, p.prefix, p.name, p.status);
                }
            }
        }
        ProjectAction::Create {
            name,
            prefix,
            description,
            icon,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let result = sqlx::query(
                "INSERT INTO projects (name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES (?, ?, ?, 'active', ?, 0, ?, ?)",
            )
            .bind(&name)
            .bind(&description)
            .bind(&icon)
            .bind(&prefix)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;
            let project_id = result.last_insert_rowid();

            // Create default statuses
            let defaults = [
                ("Backlog", "unstarted", "#6b7280", 0),
                ("Todo", "unstarted", "#6b7280", 1),
                ("In Progress", "started", "#3b82f6", 2),
                ("In Review", "started", "#8b5cf6", 3),
                ("Blocked", "blocked", "#ef4444", 4),
                ("Done", "completed", "#22c55e", 5),
                ("Discarded", "discarded", "#6b7280", 6),
            ];
            for (sname, cat, color, pos) in defaults {
                sqlx::query("INSERT INTO statuses (project_id, name, category, color, position) VALUES (?, ?, ?, ?, ?)")
                    .bind(project_id)
                    .bind(sname)
                    .bind(cat)
                    .bind(color)
                    .bind(pos)
                    .execute(pool)
                    .await?;
            }

            let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
                .bind(project_id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&project)?);
            } else {
                println!("Created project: {} ({})", project.name, project.prefix);
            }
        }
        ProjectAction::Update {
            id,
            name,
            description,
            status,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            if let Some(n) = &name {
                sqlx::query("UPDATE projects SET name = ?, updated_at = ? WHERE id = ?")
                    .bind(n)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            if let Some(d) = &description {
                sqlx::query("UPDATE projects SET description = ?, updated_at = ? WHERE id = ?")
                    .bind(d)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            if let Some(s) = &status {
                sqlx::query("UPDATE projects SET status = ?, updated_at = ? WHERE id = ?")
                    .bind(s)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
                .bind(id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&project)?);
            } else {
                println!("Updated project: {}", project.name);
            }
        }
        ProjectAction::Delete { id } => {
            sqlx::query("DELETE FROM projects WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted project {}", id);
        }
    }
    Ok(())
}

async fn handle_issue(
    pool: &SqlitePool,
    action: IssueAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        IssueAction::List {
            project,
            status,
            priority,
            assignee,
        } => {
            let mut query = String::from("SELECT * FROM issues WHERE project_id = ?");
            if status.is_some() {
                query.push_str(" AND status_id = ?");
            }
            if priority.is_some() {
                query.push_str(" AND priority = ?");
            }
            if assignee.is_some() {
                query.push_str(" AND assignee_id = ?");
            }
            query.push_str(" ORDER BY position");

            let mut q = sqlx::query_as::<_, Issue>(&query).bind(project);
            if let Some(s) = status {
                q = q.bind(s);
            }
            if let Some(ref p) = priority {
                q = q.bind(p);
            }
            if let Some(a) = assignee {
                q = q.bind(a);
            }
            let issues = q.fetch_all(pool).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&issues)?);
            } else {
                for i in &issues {
                    println!(
                        "{} | {} | {} | {}",
                        i.identifier, i.priority, i.title, i.status_id
                    );
                }
            }
        }
        IssueAction::Create {
            project,
            title,
            status,
            priority,
            description,
            assignee,
            parent,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let prio = priority.unwrap_or_else(|| "none".to_string());

            let mut tx = pool.begin().await?;

            // Atomically increment counter and get new value + prefix
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix"
            )
            .bind(project)
            .fetch_one(&mut *tx)
            .await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?",
            )
            .bind(project)
            .bind(status)
            .fetch_one(&mut *tx)
            .await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let result = sqlx::query(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(project)
            .bind(&identifier)
            .bind(&title)
            .bind(&description)
            .bind(status)
            .bind(&prio)
            .bind(assignee)
            .bind(parent)
            .bind(position)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&issue)?);
            } else {
                println!("Created: {} - {}", issue.identifier, issue.title);
            }
        }
        IssueAction::Update {
            identifier,
            title,
            status,
            priority,
            assignee,
            description,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            if let Some(t) = &title {
                sqlx::query("UPDATE issues SET title = ?, updated_at = ? WHERE id = ?")
                    .bind(t)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(s) = status {
                sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                    .bind(s)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(p) = &priority {
                sqlx::query("UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?")
                    .bind(p)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(a) = assignee {
                sqlx::query("UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?")
                    .bind(a)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(d) = &description {
                sqlx::query("UPDATE issues SET description = ?, updated_at = ? WHERE id = ?")
                    .bind(d)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&updated)?);
            } else {
                println!("Updated: {} - {}", updated.identifier, updated.title);
            }
        }
        IssueAction::Search { project, query } => {
            let pattern = format!("%{}%", query);
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = ? AND (title LIKE ? OR description LIKE ? OR identifier LIKE ?) ORDER BY updated_at DESC",
            )
            .bind(project)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&issues)?);
            } else {
                for i in &issues {
                    println!("{} | {}", i.identifier, i.title);
                }
            }
        }
        IssueAction::Move { identifier, parent } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let parent_issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                    .bind(&parent)
                    .fetch_one(pool)
                    .await?;
            sqlx::query("UPDATE issues SET parent_id = ? WHERE id = ?")
                .bind(parent_issue.id)
                .bind(issue.id)
                .execute(pool)
                .await?;
            println!("{} is now a child of {}", identifier, parent);
        }
        IssueAction::Block { identifier, by } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let blocker = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&by)
                .fetch_one(pool)
                .await?;
            sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocked_by')")
                .bind(issue.id)
                .bind(blocker.id)
                .execute(pool)
                .await?;
            println!("{} is blocked by {}", identifier, by);
        }
        IssueAction::Relate { identifier, to } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let target = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&to)
                .fetch_one(pool)
                .await?;
            sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'related')")
                .bind(issue.id)
                .bind(target.id)
                .execute(pool)
                .await?;
            println!("{} is related to {}", identifier, to);
        }
        IssueAction::Delete { identifier } => {
            sqlx::query("DELETE FROM issues WHERE identifier = ?")
                .bind(&identifier)
                .execute(pool)
                .await?;
            println!("Deleted {}", identifier);
        }
    }
    Ok(())
}

async fn handle_member(
    pool: &SqlitePool,
    action: MemberAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MemberAction::List => {
            let members = sqlx::query_as::<_, Member>("SELECT * FROM members ORDER BY name")
                .fetch_all(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&members)?);
            } else {
                for m in &members {
                    println!(
                        "{} | {} | {}",
                        m.id,
                        m.name,
                        m.email.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        MemberAction::Add {
            name,
            email,
            display_name,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let color = "#6366f1";
            let result = sqlx::query(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&name)
            .bind(&display_name)
            .bind(&email)
            .bind(color)
            .bind(&now)
            .execute(pool)
            .await?;
            let member = sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&member)?);
            } else {
                println!("Added member: {}", member.name);
            }
        }
        MemberAction::Delete { id } => {
            sqlx::query("DELETE FROM members WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted member {}", id);
        }
    }
    Ok(())
}

async fn handle_label(
    pool: &SqlitePool,
    action: LabelAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        LabelAction::List { project } => {
            let labels = sqlx::query_as::<_, Label>(
                "SELECT * FROM labels WHERE project_id = ? ORDER BY name",
            )
            .bind(project)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&labels)?);
            } else {
                for l in &labels {
                    println!("{} | {} | {}", l.id, l.name, l.color);
                }
            }
        }
        LabelAction::Create {
            project,
            name,
            color,
        } => {
            let result =
                sqlx::query("INSERT INTO labels (project_id, name, color) VALUES (?, ?, ?)")
                    .bind(project)
                    .bind(&name)
                    .bind(&color)
                    .execute(pool)
                    .await?;
            let label = sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&label)?);
            } else {
                println!("Created label: {} ({})", label.name, label.color);
            }
        }
        LabelAction::Delete { id } => {
            sqlx::query("DELETE FROM labels WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted label {}", id);
        }
    }
    Ok(())
}

async fn handle_notifications(
    pool: &SqlitePool,
    action: NotificationAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        NotificationAction::List => {
            let notifs = sqlx::query_as::<_, Notification>(
                "SELECT * FROM notifications ORDER BY created_at DESC LIMIT 50",
            )
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&notifs)?);
            } else {
                for n in &notifs {
                    println!(
                        "[{}] {} | {}",
                        if n.read { " " } else { "*" },
                        n.created_at,
                        n.message
                    );
                }
            }
        }
        NotificationAction::Clear => {
            sqlx::query("DELETE FROM notifications")
                .execute(pool)
                .await?;
            println!("Cleared all notifications");
        }
    }
    Ok(())
}

async fn handle_comment(
    pool: &SqlitePool,
    action: CommentAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CommentAction::List { identifier } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier).fetch_one(pool).await?;
            let comments = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE issue_id = ? ORDER BY created_at ASC")
                .bind(issue.id).fetch_all(pool).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&comments)?);
            } else {
                if comments.is_empty() {
                    println!("No comments on {}", identifier);
                } else {
                    for c in &comments {
                        let member_name = if let Some(mid) = c.member_id {
                            sqlx::query_scalar::<_, String>("SELECT COALESCE(display_name, name) FROM members WHERE id = ?")
                                .bind(mid).fetch_optional(pool).await?.unwrap_or_else(|| "Unknown".to_string())
                        } else {
                            "System".to_string()
                        };
                        println!("--- #{} by {} at {} ---", c.id, member_name, c.created_at);
                        println!("{}\n", c.content);
                    }
                }
            }
        }
        CommentAction::Add { identifier, content, member } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                .bind(&identifier).fetch_one(pool).await?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let result = sqlx::query("INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                .bind(issue.id).bind(member).bind(&content).bind(&now).bind(&now)
                .execute(pool).await?;
            let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
                .bind(result.last_insert_rowid()).fetch_one(pool).await?;
            if json { println!("{}", serde_json::to_string_pretty(&comment)?); }
            else { println!("Comment added to {} (id: {})", identifier, comment.id); }
        }
        CommentAction::Delete { id } => {
            sqlx::query("DELETE FROM comments WHERE id = ?").bind(id).execute(pool).await?;
            println!("Deleted comment {}", id);
        }
    }
    Ok(())
}

/// Helper: resolve an issue identifier (e.g. "KAN-42") to the issue row.
async fn resolve_issue(pool: &SqlitePool, identifier: &str) -> Result<Issue, Box<dyn std::error::Error>> {
    let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
        .bind(identifier)
        .fetch_one(pool)
        .await?;
    Ok(issue)
}

/// Helper: sync an issue's status_id to a status matching the given category.
async fn sync_issue_status_to_category(pool: &SqlitePool, issue_id: i64, category: &str) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE issues SET status_id = (
            SELECT s.id FROM statuses s
            WHERE s.project_id = (SELECT project_id FROM issues WHERE id = ?)
              AND s.category = ?
            ORDER BY s.position ASC LIMIT 1
         ), updated_at = ?
         WHERE id = ?",
    )
    .bind(issue_id)
    .bind(category)
    .bind(&now)
    .bind(issue_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn handle_agent(
    pool: &SqlitePool,
    action: AgentAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        AgentAction::Register {
            name,
            skills,
            task_types,
            max_concurrent,
            max_complexity,
        } => {
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let skills_json = serde_json::to_string(
                &skills.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
            )?;
            let task_types_json = task_types
                .map(|tt| {
                    serde_json::to_string(
                        &tt.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| "[]".to_string());

            sqlx::query(
                "INSERT INTO agents (id, name, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?, 'online', ?, ?)",
            )
            .bind(&id)
            .bind(&name)
            .bind(&skills_json)
            .bind(&task_types_json)
            .bind(max_concurrent)
            .bind(&max_complexity)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            sqlx::query(
                "INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES (?, 0, 0, 0.0, 0, '{}')",
            )
            .bind(&id)
            .execute(pool)
            .await?;

            if json {
                let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
                    .bind(&id)
                    .fetch_one(pool)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&agent)?);
            } else {
                println!("Registered agent: {} (id: {})", name, id);
            }
        }
        AgentAction::Heartbeat { id } => {
            let now = chrono::Utc::now().to_rfc3339();
            let active_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')",
            )
            .bind(&id)
            .fetch_one(pool)
            .await?;
            let new_status = if active_count.0 > 0 { "busy" } else { "online" };
            sqlx::query("UPDATE agents SET last_heartbeat = ?, status = ? WHERE id = ?")
                .bind(&now)
                .bind(new_status)
                .bind(&id)
                .execute(pool)
                .await?;
            if json {
                println!("{}", serde_json::json!({"status": new_status, "active_tasks": active_count.0}));
            } else {
                println!("Heartbeat: {} (status: {}, active: {})", id, new_status, active_count.0);
            }
        }
        AgentAction::Deregister { id } => {
            let now = chrono::Utc::now().to_rfc3339();
            // Reclaim active tasks
            let active_tasks: Vec<(i64,)> = sqlx::query_as(
                "SELECT issue_id FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')",
            )
            .bind(&id)
            .fetch_all(pool)
            .await?;

            for (issue_id,) in &active_tasks {
                sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?")
                    .bind(issue_id)
                    .execute(pool)
                    .await?;
                sync_issue_status_to_category(pool, *issue_id, "unstarted").await?;
                sqlx::query(
                    "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, 0, 'reclaim', 'Agent deregistered, task requeued', ?)",
                )
                .bind(issue_id)
                .bind(&id)
                .bind(&now)
                .execute(pool)
                .await?;
            }

            sqlx::query("DELETE FROM agents WHERE id = ?")
                .bind(&id)
                .execute(pool)
                .await?;
            println!("Deregistered agent {} ({} tasks requeued)", id, active_tasks.len());
        }
        AgentAction::List => {
            let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at")
                .fetch_all(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&agents)?);
            } else {
                for a in &agents {
                    println!("{} | {} | {} | {}", a.id, a.name, a.status, a.skills);
                }
            }
        }
        AgentAction::Stats { id } => {
            let stats = sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = ?")
                .bind(&id)
                .fetch_one(pool)
                .await?;
            if json {
                let total = stats.tasks_completed + stats.tasks_failed;
                let avg_confidence = if stats.tasks_completed > 0 {
                    stats.total_confidence / stats.tasks_completed as f64
                } else {
                    0.0
                };
                println!("{}", serde_json::json!({
                    "agent_id": stats.agent_id,
                    "tasks_completed": stats.tasks_completed,
                    "tasks_failed": stats.tasks_failed,
                    "total_tasks": total,
                    "avg_confidence": avg_confidence,
                    "total_completion_time_seconds": stats.total_completion_time_seconds,
                    "skills_breakdown": stats.skills_breakdown,
                }));
            } else {
                let avg = if stats.tasks_completed > 0 {
                    stats.total_confidence / stats.tasks_completed as f64
                } else {
                    0.0
                };
                println!("Agent: {}", stats.agent_id);
                println!("  Completed: {}", stats.tasks_completed);
                println!("  Failed: {}", stats.tasks_failed);
                println!("  Avg confidence: {:.2}", avg);
                println!("  Total time: {}s", stats.total_completion_time_seconds);
            }
        }
    }
    Ok(())
}

async fn handle_task(
    pool: &SqlitePool,
    action: TaskAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        TaskAction::Next { agent, skills } => {
            let agent_row = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
                .bind(&agent)
                .fetch_one(pool)
                .await?;
            let skill_list: Vec<String> = if let Some(ref s) = skills {
                s.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                serde_json::from_str(&agent_row.skills).unwrap_or_default()
            };
            let result = orchestration::routing::next_task(
                pool,
                &agent,
                &skill_list,
                &agent_row.max_complexity,
                agent_row.max_concurrent,
            )
            .await?;
            match result {
                Some(contract) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&contract)?);
                    } else {
                        println!("Claimed: {} - {}", contract.identifier, contract.title);
                        println!("  Type: {}", contract.r#type);
                        println!("  Objective: {}", contract.objective);
                        println!("  Complexity: {}", contract.estimated_complexity.as_deref().unwrap_or("unset"));
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("No available tasks");
                    }
                }
            }
        }
        TaskAction::Start { identifier, agent } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = ?")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'start', 'Task execution started', ?)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            if json {
                println!("{}", serde_json::json!({"status": "executing", "identifier": identifier}));
            } else {
                println!("Started: {}", identifier);
            }
        }
        TaskAction::Complete {
            identifier,
            agent,
            confidence,
            summary,
            artifacts,
        } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();

            // Get project config thresholds
            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = ?",
            )
            .bind(issue.project_id)
            .fetch_optional(pool)
            .await?;

            let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.9);
            let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.7);

            let new_state = if confidence >= auto_accept {
                "completed"
            } else if confidence >= human_review {
                "validating"
            } else {
                "validating"
            };

            let result_json = serde_json::json!({
                "confidence": confidence,
                "summary": summary,
                "artifacts": artifacts.as_ref().and_then(|a| serde_json::from_str::<serde_json::Value>(a).ok()),
            });

            sqlx::query("UPDATE task_contracts SET task_state = ?, result = ? WHERE issue_id = ?")
                .bind(new_state)
                .bind(result_json.to_string())
                .bind(issue.id)
                .execute(pool)
                .await?;

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state)
                    .map_err(|e| Box::<dyn std::error::Error>::from(e))?,
            );
            sync_issue_status_to_category(pool, issue.id, category).await?;

            // Log
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'complete', ?, ?)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&summary)
            .bind(&now)
            .execute(pool)
            .await?;

            // Update agent stats
            sqlx::query(
                "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + ? WHERE agent_id = ?",
            )
            .bind(confidence)
            .bind(&agent)
            .execute(pool)
            .await?;

            if json {
                println!("{}", serde_json::json!({"status": new_state, "identifier": identifier, "confidence": confidence}));
            } else {
                println!("Completed: {} (state: {}, confidence: {:.2})", identifier, new_state, confidence);
            }
        }
        TaskAction::Fail {
            identifier,
            agent,
            reason,
        } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            let now = chrono::Utc::now().to_rfc3339();

            // Append to prior_attempts in context
            let mut context: serde_json::Value = serde_json::from_str(&tc.context).unwrap_or(serde_json::json!({}));
            let attempts = context.as_object_mut().unwrap()
                .entry("prior_attempts")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .unwrap();
            attempts.push(serde_json::json!({
                "attempt": tc.attempt_count,
                "agent": agent,
                "reason": reason,
                "timestamp": now,
            }));

            // Requeue
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1, context = ? WHERE issue_id = ?",
            )
            .bind(context.to_string())
            .bind(issue.id)
            .execute(pool)
            .await?;

            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;

            // Log
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'fail', ?, ?)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&reason)
            .bind(&now)
            .execute(pool)
            .await?;

            // Update agent stats
            sqlx::query("UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = ?")
                .bind(&agent)
                .execute(pool)
                .await?;

            if json {
                println!("{}", serde_json::json!({"status": "requeued", "identifier": identifier}));
            } else {
                println!("Failed and requeued: {}", identifier);
            }
        }
        TaskAction::Unclaim { identifier, agent } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'unclaim', 'Task unclaimed by agent', ?)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            if json {
                println!("{}", serde_json::json!({"status": "queued", "identifier": identifier}));
            } else {
                println!("Unclaimed: {}", identifier);
            }
        }
        TaskAction::Log {
            identifier,
            agent,
            entry_type,
            message,
            meta,
        } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let attempt_count: (i64,) = sqlx::query_as(
                "SELECT attempt_count FROM task_contracts WHERE issue_id = ?",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await?;
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(attempt_count.0)
            .bind(&entry_type)
            .bind(&message)
            .bind(&meta)
            .bind(&now)
            .execute(pool)
            .await?;
            if json {
                println!("{}", serde_json::json!({"logged": true}));
            } else {
                println!("Logged [{}] for {}", entry_type, identifier);
            }
        }
        TaskAction::Create {
            project,
            title,
            objective,
            status,
            task_type,
            priority,
            skills,
            complexity,
            description,
            parent,
            depends_on,
            context_files,
            constraints,
            success_criteria,
            assignee,
            timeout,
        } => {
            let now = chrono::Utc::now().to_rfc3339();
            let prio = priority.unwrap_or_else(|| "none".to_string());
            let tt = task_type.unwrap_or_else(|| "implementation".to_string());

            let mut tx = pool.begin().await?;

            // Increment counter
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix",
            )
            .bind(project)
            .fetch_one(&mut *tx)
            .await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?",
            )
            .bind(project)
            .bind(status)
            .fetch_one(&mut *tx)
            .await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            // Resolve parent
            let parent_id: Option<i64> = if let Some(ref p) = parent {
                let pi = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                    .bind(p)
                    .fetch_one(&mut *tx)
                    .await?;
                Some(pi.id)
            } else {
                None
            };

            let result = sqlx::query(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(project)
            .bind(&identifier)
            .bind(&title)
            .bind(&description)
            .bind(status)
            .bind(&prio)
            .bind(assignee)
            .bind(parent_id)
            .bind(position)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            let issue_id = result.last_insert_rowid();

            let skills_json = skills
                .map(|s| serde_json::to_string(&s.split(',').map(|s| s.trim()).collect::<Vec<_>>()))
                .transpose()?
                .unwrap_or_else(|| "[]".to_string());
            let constraints_str = constraints.unwrap_or_else(|| "[]".to_string());
            let criteria_str = success_criteria.unwrap_or_else(|| "[]".to_string());
            let mut context_obj = serde_json::json!({});
            if let Some(ref cf) = context_files {
                context_obj["files"] = serde_json::json!(cf);
            }

            sqlx::query(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES (?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?, 0)",
            )
            .bind(issue_id)
            .bind(&tt)
            .bind(&objective)
            .bind(context_obj.to_string())
            .bind(&constraints_str)
            .bind(&criteria_str)
            .bind(&skills_json)
            .bind(&complexity)
            .bind(timeout.unwrap_or(30))
            .execute(&mut *tx)
            .await?;

            // Create dependency relations
            if let Some(ref deps) = depends_on {
                for dep_ident in deps.split(',').map(|s| s.trim()) {
                    if dep_ident.is_empty() { continue; }
                    let dep_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = ?")
                        .bind(dep_ident)
                        .fetch_one(&mut *tx)
                        .await?;
                    sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocks')")
                        .bind(dep_issue.id)
                        .bind(issue_id)
                        .execute(&mut *tx)
                        .await?;
                }
            }

            tx.commit().await?;

            let contract = orchestration::routing::build_full_contract(pool, issue_id).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&contract)?);
            } else {
                println!("Created: {} - {}", identifier, title);
            }
        }
        TaskAction::Get { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let contract = orchestration::routing::build_full_contract(pool, issue.id).await?;
            match contract {
                Some(c) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&c)?);
                    } else {
                        println!("{} | {} | {} | {}", c.identifier, c.task_state, c.r#type, c.objective);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("No task contract for {}", identifier);
                    }
                }
            }
        }
        TaskAction::Replay { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let logs = sqlx::query_as::<_, ExecutionLog>(
                "SELECT * FROM execution_logs WHERE issue_id = ? ORDER BY timestamp ASC",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&logs)?);
            } else {
                if logs.is_empty() {
                    println!("No execution logs for {}", identifier);
                } else {
                    for log in &logs {
                        println!(
                            "[{}] #{} {} | {} | {}",
                            log.timestamp, log.attempt_number, log.entry_type, log.agent_id, log.message
                        );
                    }
                }
            }
        }
        TaskAction::Attempts { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            let context: serde_json::Value = serde_json::from_str(&tc.context).unwrap_or(serde_json::json!({}));
            let attempts = context.get("prior_attempts").cloned().unwrap_or(serde_json::json!([]));
            if json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "identifier": identifier,
                    "attempt_count": tc.attempt_count,
                    "prior_attempts": attempts,
                }))?);
            } else {
                println!("Attempts for {} (current: {}):", identifier, tc.attempt_count);
                if let Some(arr) = attempts.as_array() {
                    for a in arr {
                        println!(
                            "  #{}: agent={} reason={} at={}",
                            a.get("attempt").and_then(|v| v.as_i64()).unwrap_or(0),
                            a.get("agent").and_then(|v| v.as_str()).unwrap_or("?"),
                            a.get("reason").and_then(|v| v.as_str()).unwrap_or("?"),
                            a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?"),
                        );
                    }
                }
            }
        }
        TaskAction::List {
            project,
            status,
            agent,
            available,
        } => {
            let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
                "SELECT i.identifier, i.title, i.priority, tc.task_state, tc.claimed_by, tc.estimated_complexity FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ",
            );
            qb.push_bind(project);

            if let Some(ref s) = status {
                qb.push(" AND tc.task_state = ");
                qb.push_bind(s.clone());
            }
            if let Some(ref a) = agent {
                qb.push(" AND tc.claimed_by = ");
                qb.push_bind(a.clone());
            }
            if available {
                qb.push(" AND tc.task_state = 'queued'");
            }
            qb.push(" ORDER BY i.created_at ASC");

            let rows = qb.build().fetch_all(pool).await?;
            if json {
                let items: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "identifier": r.get::<String, _>("identifier"),
                            "title": r.get::<String, _>("title"),
                            "priority": r.get::<String, _>("priority"),
                            "task_state": r.get::<String, _>("task_state"),
                            "claimed_by": r.get::<Option<String>, _>("claimed_by"),
                            "estimated_complexity": r.get::<Option<String>, _>("estimated_complexity"),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                for r in &rows {
                    let ident: String = r.get("identifier");
                    let title: String = r.get("title");
                    let state: String = r.get("task_state");
                    let claimed: Option<String> = r.get("claimed_by");
                    println!(
                        "{} | {} | {} | {}",
                        ident,
                        state,
                        claimed.as_deref().unwrap_or("-"),
                        title,
                    );
                }
            }
        }
        TaskAction::Children { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let children = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE parent_id = ? ORDER BY position",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&children)?);
            } else {
                for c in &children {
                    println!("{} | {} | {}", c.identifier, c.priority, c.title);
                }
            }
        }
        TaskAction::Approve { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, tc.task_state).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = ?")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sync_issue_status_to_category(pool, issue.id, "completed").await?;
            // Update agent stats if there's a claimed_by
            if let Some(ref agent_id) = tc.claimed_by {
                sqlx::query("UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = ?")
                    .bind(agent_id)
                    .execute(pool)
                    .await?;
            }
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'approve', 'Task approved', ?)",
            )
            .bind(issue.id)
            .bind(tc.claimed_by.as_deref().unwrap_or("system"))
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            if json {
                println!("{}", serde_json::json!({"status": "completed", "identifier": identifier}));
            } else {
                println!("Approved: {}", identifier);
            }
        }
        TaskAction::Reject { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, tc.task_state).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = ?",
            )
            .bind(issue.id)
            .execute(pool)
            .await?;
            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'reject', 'Task rejected, requeued', ?)",
            )
            .bind(issue.id)
            .bind(tc.claimed_by.as_deref().unwrap_or("system"))
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            if json {
                println!("{}", serde_json::json!({"status": "requeued", "identifier": identifier}));
            } else {
                println!("Rejected and requeued: {}", identifier);
            }
        }
        TaskAction::Invalidate { identifier, reason } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let now = chrono::Utc::now().to_rfc3339();

            // Requeue the task
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?",
            )
            .bind(issue.id)
            .execute(pool)
            .await?;
            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;

            // Block downstream tasks
            let downstream: Vec<(i64,)> = sqlx::query_as(
                "SELECT target_issue_id FROM issue_relations WHERE source_issue_id = ? AND relation_type = 'blocks'",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await?;
            for (target_id,) in &downstream {
                sqlx::query("UPDATE task_contracts SET task_state = 'blocked' WHERE issue_id = ?")
                    .bind(target_id)
                    .execute(pool)
                    .await?;
                sync_issue_status_to_category(pool, *target_id, "blocked").await?;
            }

            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, 'system', 0, 'invalidate', ?, ?)",
            )
            .bind(issue.id)
            .bind(&reason)
            .bind(&now)
            .execute(pool)
            .await?;

            if json {
                println!("{}", serde_json::json!({"status": "invalidated", "identifier": identifier, "blocked_downstream": downstream.len()}));
            } else {
                println!("Invalidated: {} ({} downstream blocked)", identifier, downstream.len());
            }
        }
        TaskAction::Search { project, query } => {
            let pattern = format!("%{}%", query);
            let rows = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN task_contracts tc ON tc.issue_id = i.id WHERE i.project_id = ? AND (i.title LIKE ? OR tc.objective LIKE ? OR i.identifier LIKE ?) ORDER BY i.updated_at DESC",
            )
            .bind(project)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                for i in &rows {
                    println!("{} | {}", i.identifier, i.title);
                }
            }
        }
        TaskAction::Update {
            identifier,
            title,
            priority,
            complexity,
            skills,
        } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let now = chrono::Utc::now().to_rfc3339();
            if let Some(ref t) = title {
                sqlx::query("UPDATE issues SET title = ?, updated_at = ? WHERE id = ?")
                    .bind(t)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref p) = priority {
                sqlx::query("UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?")
                    .bind(p)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref c) = complexity {
                sqlx::query("UPDATE task_contracts SET estimated_complexity = ? WHERE issue_id = ?")
                    .bind(c)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref s) = skills {
                let skills_json = serde_json::to_string(
                    &s.split(',').map(|s| s.trim()).collect::<Vec<_>>(),
                )?;
                sqlx::query("UPDATE task_contracts SET required_skills = ? WHERE issue_id = ?")
                    .bind(&skills_json)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if json {
                let contract = orchestration::routing::build_full_contract(pool, issue.id).await?;
                println!("{}", serde_json::to_string_pretty(&contract)?);
            } else {
                println!("Updated: {}", identifier);
            }
        }
    }
    Ok(())
}

async fn handle_metrics(
    pool: &SqlitePool,
    project: Option<i64>,
    agent: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref agent_id) = agent {
        let stats = sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = ?")
            .bind(agent_id)
            .fetch_one(pool)
            .await?;
        let avg = if stats.tasks_completed > 0 {
            stats.total_confidence / stats.tasks_completed as f64
        } else {
            0.0
        };
        if json {
            println!("{}", serde_json::json!({
                "agent_id": stats.agent_id,
                "tasks_completed": stats.tasks_completed,
                "tasks_failed": stats.tasks_failed,
                "avg_confidence": avg,
                "total_completion_time_seconds": stats.total_completion_time_seconds,
            }));
        } else {
            println!("Agent: {}", stats.agent_id);
            println!("  Completed: {} | Failed: {} | Avg confidence: {:.2}", stats.tasks_completed, stats.tasks_failed, avg);
        }
    } else if let Some(pid) = project {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ?")
            .bind(pid).fetch_one(pool).await?;
        let completed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state = 'completed'")
            .bind(pid).fetch_one(pool).await?;
        let queued: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state = 'queued'")
            .bind(pid).fetch_one(pool).await?;
        let in_progress: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state IN ('claimed', 'executing')")
            .bind(pid).fetch_one(pool).await?;
        let online_agents: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents WHERE status IN ('online', 'busy')")
            .fetch_one(pool).await?;
        if json {
            println!("{}", serde_json::json!({
                "project_id": pid,
                "total_tasks": total.0,
                "completed": completed.0,
                "queued": queued.0,
                "in_progress": in_progress.0,
                "online_agents": online_agents.0,
            }));
        } else {
            println!("Project {} metrics:", pid);
            println!("  Total: {} | Completed: {} | Queued: {} | In Progress: {} | Online agents: {}", total.0, completed.0, queued.0, in_progress.0, online_agents.0);
        }
    } else {
        println!("Please specify --project or --agent");
    }
    Ok(())
}

async fn handle_export(
    pool: &SqlitePool,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = ExportData {
        projects: sqlx::query_as("SELECT * FROM projects")
            .fetch_all(pool)
            .await?,
        statuses: sqlx::query_as("SELECT * FROM statuses")
            .fetch_all(pool)
            .await?,
        members: sqlx::query_as("SELECT * FROM members")
            .fetch_all(pool)
            .await?,
        issues: sqlx::query_as("SELECT * FROM issues")
            .fetch_all(pool)
            .await?,
        labels: sqlx::query_as("SELECT * FROM labels")
            .fetch_all(pool)
            .await?,
        issue_labels: sqlx::query_as::<_, IssueLabelRow>(
            "SELECT issue_id, label_id FROM issue_labels",
        )
        .fetch_all(pool)
        .await?,
        issue_relations: sqlx::query_as("SELECT * FROM issue_relations")
            .fetch_all(pool)
            .await?,
        issue_templates: sqlx::query_as("SELECT * FROM issue_templates")
            .fetch_all(pool)
            .await?,
    };

    let json_str = serde_json::to_string_pretty(&data)?;
    if let Some(path) = output {
        std::fs::write(&path, &json_str)?;
        println!("Exported to {}", path);
    } else {
        println!("{}", json_str);
    }
    Ok(())
}

async fn handle_import(
    pool: &SqlitePool,
    file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&file)?;
    let data: ExportData = serde_json::from_str(&content)?;

    // Import in order to satisfy foreign keys
    for m in &data.members {
        sqlx::query("INSERT OR REPLACE INTO members (id, name, display_name, email, avatar_color, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(m.id)
            .bind(&m.name)
            .bind(&m.display_name)
            .bind(&m.email)
            .bind(&m.avatar_color)
            .bind(&m.created_at)
            .execute(pool)
            .await?;
    }
    for p in &data.projects {
        sqlx::query("INSERT OR REPLACE INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(p.id)
            .bind(&p.name)
            .bind(&p.description)
            .bind(&p.icon)
            .bind(&p.status)
            .bind(&p.prefix)
            .bind(p.issue_counter)
            .bind(&p.created_at)
            .bind(&p.updated_at)
            .execute(pool)
            .await?;
    }
    for s in &data.statuses {
        sqlx::query("INSERT OR REPLACE INTO statuses (id, project_id, name, category, color, icon, position) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(s.id)
            .bind(s.project_id)
            .bind(&s.name)
            .bind(&s.category)
            .bind(&s.color)
            .bind(&s.icon)
            .bind(s.position)
            .execute(pool)
            .await?;
    }
    for l in &data.labels {
        sqlx::query("INSERT OR REPLACE INTO labels (id, project_id, name, color) VALUES (?, ?, ?, ?)")
            .bind(l.id)
            .bind(l.project_id)
            .bind(&l.name)
            .bind(&l.color)
            .execute(pool)
            .await?;
    }
    for i in &data.issues {
        sqlx::query("INSERT OR REPLACE INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(i.id)
            .bind(i.project_id)
            .bind(&i.identifier)
            .bind(&i.title)
            .bind(&i.description)
            .bind(i.status_id)
            .bind(&i.priority)
            .bind(i.assignee_id)
            .bind(i.parent_id)
            .bind(i.position)
            .bind(i.estimate)
            .bind(&i.due_date)
            .bind(&i.created_at)
            .bind(&i.updated_at)
            .execute(pool)
            .await?;
    }
    for il in &data.issue_labels {
        sqlx::query("INSERT OR REPLACE INTO issue_labels (issue_id, label_id) VALUES (?, ?)")
            .bind(il.issue_id)
            .bind(il.label_id)
            .execute(pool)
            .await?;
    }
    for r in &data.issue_relations {
        sqlx::query("INSERT OR REPLACE INTO issue_relations (id, source_issue_id, target_issue_id, relation_type) VALUES (?, ?, ?, ?)")
            .bind(r.id)
            .bind(r.source_issue_id)
            .bind(r.target_issue_id)
            .bind(&r.relation_type)
            .execute(pool)
            .await?;
    }
    for t in &data.issue_templates {
        sqlx::query("INSERT OR REPLACE INTO issue_templates (id, project_id, name, description_template, default_status_id, default_priority, default_label_ids, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(t.id)
            .bind(t.project_id)
            .bind(&t.name)
            .bind(&t.description_template)
            .bind(t.default_status_id)
            .bind(&t.default_priority)
            .bind(&t.default_label_ids)
            .bind(&t.created_at)
            .bind(&t.updated_at)
            .execute(pool)
            .await?;
    }

    println!("Imported from {}", file);
    println!(
        "  {} projects, {} statuses, {} members, {} issues, {} labels, {} relations, {} templates",
        data.projects.len(),
        data.statuses.len(),
        data.members.len(),
        data.issues.len(),
        data.labels.len(),
        data.issue_relations.len(),
        data.issue_templates.len()
    );
    Ok(())
}
