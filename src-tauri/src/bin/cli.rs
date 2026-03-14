use clap::{Parser, Subcommand};
use kanban_lib::db;
use kanban_lib::models::*;
use sqlx::SqlitePool;

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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let prio = priority.unwrap_or_else(|| "none".to_string());
            let proj = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
                .bind(project)
                .fetch_one(pool)
                .await?;
            let counter = proj.issue_counter + 1;
            let identifier = format!("{}-{}", proj.prefix, counter);
            sqlx::query("UPDATE projects SET issue_counter = ? WHERE id = ?")
                .bind(counter)
                .bind(project)
                .execute(pool)
                .await?;

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?",
            )
            .bind(project)
            .bind(status)
            .fetch_one(pool)
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
            .execute(pool)
            .await?;

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(pool)
                .await?;
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
