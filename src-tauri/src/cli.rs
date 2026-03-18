use clap::{Parser, Subcommand};
use crate::models::*;
use crate::orchestration;
use sqlx::{QueryBuilder, Row, Any, AnyPool};

#[cfg(feature = "redis-sync")]
fn notify_change() {
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        if let Ok(client) = redis::Client::open(redis_url) {
            if let Ok(mut conn) = client.get_connection() {
                let _: Result<(), _> = redis::cmd("PUBLISH").arg("kanban:db-changed").arg("1").query(&mut conn);
            }
        }
    }
}

#[cfg(not(feature = "redis-sync"))]
fn notify_change() {}

#[derive(Parser)]
#[command(name = "kanban", about = "Kanban - Desktop Project Management CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Code analysis and heat maps
    Code {
        #[command(subcommand)]
        action: CodeAction,
    },
    /// Agent marketplace
    Marketplace {
        #[command(subcommand)]
        action: MarketplaceAction,
    /// Manage multi-agent pipelines
    Pipeline {
        #[command(subcommand)]
        action: PipelineAction,
    },
    /// View system metrics
    Metrics {
        #[arg(long)]
        project: Option<i64>,
        #[arg(long)]
        agent: Option<String>,
    },
    /// Track costs and budgets
    Costs {
        #[command(subcommand)]
        action: CostAction,
    },
    /// Manage SLA policies
    Sla {
        #[command(subcommand)]
        action: SlaAction,
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
pub enum ProjectAction {
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
pub enum IssueAction {
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
    /// Auto-triage an issue (suggest priority, labels, assignee)
    Triage {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
        /// Apply suggestions automatically
        #[arg(long)]
        apply: bool,
    },
    /// Decompose an issue into sub-issues from its description
    Decompose {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
        /// Create sub-issues (default: preview only)
        #[arg(long)]
        apply: bool,
    },
    /// Create an issue from natural language text
    CreateFromText {
        /// Natural language description
        text: String,
        #[arg(short, long)]
        project: i64,
        #[arg(short, long)]
        status: i64,
    },
    /// Create an issue from a code diff finding
    FromDiff {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        line: Option<String>,
        #[arg(long)]
        severity: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        assignee: Option<i64>,
    /// Set WSJF scores for an issue
    Score {
        identifier: String,
        /// Business value (1-10)
        #[arg(long)]
        bv: i32,
        /// Time criticality (1-10)
        #[arg(long)]
        tc: i32,
        /// Risk reduction (1-10)
        #[arg(long)]
        rr: i32,
        /// Job size (1-10)
        #[arg(long)]
        size: i32,
    },
    /// Show ranked backlog by WSJF score
    Rank {
        #[arg(short, long)]
        project: i64,
    },
    /// Auto-score an issue using rule-based heuristics
    AutoScore {
        identifier: String,
    },
    /// Auto-score all unscored issues in a project
    AutoScoreAll {
        #[arg(short, long)]
        project: i64,
    },
}

#[derive(Subcommand)]
pub enum MemberAction {
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
pub enum LabelAction {
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
pub enum NotificationAction {
    /// List recent notifications
    List,
    /// Clear all notifications
    Clear,
}

#[derive(Subcommand)]
pub enum CommentAction {
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
pub enum AgentAction {
    /// Register a new agent
    Register {
        #[arg(long, default_value = "")]
        name: String,
        /// Agent type (claude, claude-code, codex, gemini, custom)
        #[arg(long)]
        agent_type: Option<String>,
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
    /// List permissions for an agent
    Permissions {
        id: String,
    },
    /// Grant a permission to an agent
    Grant {
        id: String,
        #[arg(long, name = "type")]
        permission_type: String,
        #[arg(long)]
        scope: String,
    },
    /// Deny a permission for an agent
    Deny {
        id: String,
        #[arg(long, name = "type")]
        permission_type: String,
        #[arg(long)]
        scope: String,
    },
    /// Remove all permissions for an agent
    ClearPermissions {
        id: String,
    },
    /// Apply a preset to an agent
    ApplyPreset {
        id: String,
        #[arg(long)]
        preset: String,
    },
}

#[derive(Subcommand)]
pub enum CodeAction {
    /// Show file heat map for a project
    HeatMap {
        #[arg(long)]
        project: i64,
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// Show directory heat map for a project
    DirHeatMap {
        #[arg(long)]
        project: i64,
        #[arg(long, default_value = "2")]
        depth: i32,
    },
    /// Link a file to an issue
    Link {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
        /// File path
        file_path: String,
        /// Link type: related, cause, fix
        #[arg(long, default_value = "related")]
        link_type: String,
    },
    /// Unlink a file from an issue
    Unlink {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
        /// File path
        file_path: String,
    },
    /// List files linked to an issue
    Files {
        /// Issue identifier (e.g. KAN-42)
        identifier: String,
    },
    /// List issues linked to a file
    Issues {
        file_path: String,
        #[arg(long)]
        project: i64,
    },
}

#[derive(Subcommand)]
pub enum MarketplaceAction {
    /// List all agents in the marketplace
    List,
    /// Register an agent in the marketplace
    Register {
        #[arg(long)]
        agent_id: String,
        #[arg(long)]
        name: String,
        /// Comma-delimited capabilities
        #[arg(long)]
        skills: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        max_complexity: Option<String>,
    },
    /// Search marketplace for agents with specific skills
    Search {
        /// Comma-delimited skills to search
        #[arg(long)]
        skills: String,
        #[arg(long)]
        max_complexity: Option<String>,
    },
    /// Find best agent for a task
    BestMatch {
        /// Comma-delimited required skills
        #[arg(long)]
        skills: String,
        #[arg(long, default_value = "medium")]
        complexity: String,
    },
    /// Deregister an agent from marketplace
    Deregister {
        #[arg(long)]
        agent_id: String,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
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
    /// Show dependency graph for a task
    Graph {
        identifier: String,
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
    /// Manually trigger decomposition for a task
    Decompose {
        identifier: String,
    },
    /// Get full assembled context for a task
    Context {
        identifier: String,
    },
    /// Create a handoff note for a task
    Handoff {
        identifier: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, name = "type", default_value = "completion")]
        note_type: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        details: Option<String>,
        /// Comma-delimited file paths
        #[arg(long)]
        files: Option<String>,
        /// Comma-delimited risks
        #[arg(long)]
        risks: Option<String>,
    },
    /// Record a learning from a task
    Learn {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        outcome: String,
        #[arg(long)]
        approach: String,
        #[arg(long)]
        insight: Option<String>,
        /// Comma-delimited pitfalls
        #[arg(long)]
        pitfalls: Option<String>,
        /// Comma-delimited effective patterns
        #[arg(long)]
        patterns: Option<String>,
        /// Comma-delimited tags
        #[arg(long)]
        tags: Option<String>,
    },
    /// List handoff notes for a task
    Handoffs {
        identifier: String,
    },
    /// List learnings for a task
    Learnings {
        identifier: String,
    },
}

#[derive(Subcommand)]
pub enum PipelineAction {
    /// List pipelines for a project
    List {
        #[arg(long)]
        project: i64,
    },
    /// Create a pipeline
    Create {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
        /// JSON string of stages array
        #[arg(long)]
        stages: String,
    },
    /// Trigger a pipeline run
    Trigger {
        /// Pipeline ID
        id: i64,
        /// Optional trigger issue ID
        #[arg(long)]
        issue: Option<i64>,
        /// Optional JSON context
        #[arg(long)]
        context: Option<String>,
    },
    /// Get pipeline run status
    Status {
        /// Pipeline run ID
        run_id: i64,
    },
    /// List runs for a pipeline
    Runs {
        /// Pipeline ID
        id: i64,
    },
    /// Advance a pipeline run to the next stage
    Advance {
        /// Pipeline run ID
        run_id: i64,
    },
    /// Cancel a pipeline run
    Cancel {
        /// Pipeline run ID
        run_id: i64,
    },
    /// Delete a pipeline
    Delete {
        /// Pipeline ID
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum CostAction {
    /// Show cost summary for a project
    Summary {
        #[arg(long)]
        project: i64,
    },
    /// Show cost breakdown for a specific task
    Task {
        /// Task identifier (e.g. KAN-42)
        identifier: String,
    },
    /// Record a cost entry
    Record {
        /// Task identifier
        #[arg(long)]
        task: String,
        /// Agent ID
        #[arg(long)]
        agent: String,
        /// Cost type: compute_time, api_tokens, custom
        #[arg(long, name = "type")]
        cost_type: String,
        /// Amount
        #[arg(long)]
        amount: f64,
        /// Unit: minutes, tokens, dollars, credits
        #[arg(long)]
        unit: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Set a budget for a project
    Budget {
        #[arg(long)]
        project: i64,
        /// Budget type: daily, weekly, monthly, per_task, total
        #[arg(long, name = "type")]
        budget_type: String,
        /// Budget amount
        #[arg(long)]
        amount: f64,
        /// Unit (default: dollars)
        #[arg(long)]
        unit: Option<String>,
    },
    /// Check budget status (alerts)
    Check {
        #[arg(long)]
        project: i64,
    },
}

#[derive(Subcommand)]
pub enum SlaAction {
    /// Check SLA compliance for a project
    Check {
        #[arg(long)]
        project: i64,
    },
    /// Enforce SLA policies (execute escalation actions)
    Enforce {
        #[arg(long)]
        project: i64,
    },
    /// Create an SLA policy
    Create {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        name: String,
        /// Target type: response_time, resolution_time, task_timeout
        #[arg(long, name = "type")]
        target_type: String,
        /// Priority filter (e.g. urgent, high)
        #[arg(long)]
        priority: Option<String>,
        /// Warning minutes before breach
        #[arg(long)]
        warning: i64,
        /// Breach threshold in minutes
        #[arg(long)]
        breach: i64,
        /// Escalation action JSON
        #[arg(long)]
        escalation: Option<String>,
    },
    /// List SLA policies
    List {
        #[arg(long)]
        project: i64,
    },
    /// Delete an SLA policy
    Delete {
        id: i64,
    },
    /// Show SLA dashboard
    Dashboard {
        #[arg(long)]
        project: i64,
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

pub async fn run(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    command: Commands,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Project { action } => handle_project(pool, action, json).await?,
        Commands::Issue { action } => handle_issue(pool, action, json).await?,
        Commands::Member { action } => handle_member(pool, action, json).await?,
        Commands::Label { action } => handle_label(pool, action, json).await?,
        Commands::Notifications { action } => handle_notifications(pool, action, json).await?,
        Commands::Comment { action } => handle_comment(pool, action, json).await?,
        Commands::Agent { action } => handle_agent(pool, backend, action, json).await?,
        Commands::Marketplace { action } => handle_marketplace(pool, backend, action, json).await?,
        Commands::Task { action } => handle_task(pool, backend, action, json).await?,
        Commands::Code { action } => handle_code(pool, action, json).await?,
        Commands::Pipeline { action } => handle_pipeline(pool, backend, action, json).await?,
        Commands::Metrics { project, agent } => handle_metrics(pool, backend, project, agent, json).await?,
        Commands::Costs { action } => handle_costs(pool, action, json).await?,
        Commands::Sla { action } => handle_sla(pool, action, json).await?,
        Commands::Export { output } => handle_export(pool, output).await?,
        Commands::Import { file } => handle_import(pool, file).await?,
    }
    Ok(())
}

async fn handle_project(
    pool: &AnyPool,
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
            let project_id: i64 = sqlx::query_scalar(
                "INSERT INTO projects (name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES ($1, $2, $3, 'active', $4, 0, $5, $6) RETURNING id",
            )
            .bind(&name)
            .bind(&description)
            .bind(&icon)
            .bind(&prefix)
            .bind(&now)
            .bind(&now)
            .fetch_one(pool)
            .await?;

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
                sqlx::query("INSERT INTO statuses (project_id, name, category, color, position) VALUES ($1, $2, $3, $4, $5)")
                    .bind(project_id)
                    .bind(sname)
                    .bind(cat)
                    .bind(color)
                    .bind(pos)
                    .execute(pool)
                    .await?;
            }

            let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
                .bind(project_id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&project)?);
            } else {
                println!("Created project: {} ({})", project.name, project.prefix);
            }
            notify_change();
        }
        ProjectAction::Update {
            id,
            name,
            description,
            status,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            if let Some(n) = &name {
                sqlx::query("UPDATE projects SET name = $1, updated_at = $2 WHERE id = $3")
                    .bind(n)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            if let Some(d) = &description {
                sqlx::query("UPDATE projects SET description = $1, updated_at = $2 WHERE id = $3")
                    .bind(d)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            if let Some(s) = &status {
                sqlx::query("UPDATE projects SET status = $1, updated_at = $2 WHERE id = $3")
                    .bind(s)
                    .bind(&now)
                    .bind(id)
                    .execute(pool)
                    .await?;
            }
            let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&project)?);
            } else {
                println!("Updated project: {}", project.name);
            }
            notify_change();
        }
        ProjectAction::Delete { id } => {
            sqlx::query("DELETE FROM projects WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted project {}", id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_issue(
    pool: &AnyPool,
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
            let mut param_idx = 1;
            let mut query = format!("SELECT * FROM issues WHERE project_id = ${}", param_idx);
            param_idx += 1;
            if status.is_some() {
                query.push_str(&format!(" AND status_id = ${}", param_idx));
                param_idx += 1;
            }
            if priority.is_some() {
                query.push_str(&format!(" AND priority = ${}", param_idx));
                param_idx += 1;
            }
            if assignee.is_some() {
                query.push_str(&format!(" AND assignee_id = ${}", param_idx));
                // param_idx += 1;
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
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            )
            .bind(project)
            .fetch_one(&mut *tx)
            .await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
            )
            .bind(project)
            .bind(status)
            .fetch_one(&mut *tx)
            .await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
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
            .fetch_one(&mut *tx)
            .await?;

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id)
                .fetch_one(&mut *tx)
                .await?;

            tx.commit().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&issue)?);
            } else {
                println!("Created: {} - {}", issue.identifier, issue.title);
            }
            notify_change();
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
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            if let Some(t) = &title {
                sqlx::query("UPDATE issues SET title = $1, updated_at = $2 WHERE id = $3")
                    .bind(t)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(s) = status {
                sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(s)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(p) = &priority {
                sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                    .bind(p)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(a) = assignee {
                sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                    .bind(a)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(d) = &description {
                sqlx::query("UPDATE issues SET description = $1, updated_at = $2 WHERE id = $3")
                    .bind(d)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            let updated = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&updated)?);
            } else {
                println!("Updated: {} - {}", updated.identifier, updated.title);
            }
            notify_change();
        }
        IssueAction::Search { project, query } => {
            let pattern = format!("%{}%", query);
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND (title LIKE $2 OR description LIKE $3 OR identifier LIKE $4) ORDER BY updated_at DESC",
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
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let parent_issue =
                sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(&parent)
                    .fetch_one(pool)
                    .await?;
            sqlx::query("UPDATE issues SET parent_id = $1 WHERE id = $2")
                .bind(parent_issue.id)
                .bind(issue.id)
                .execute(pool)
                .await?;
            println!("{} is now a child of {}", identifier, parent);
            notify_change();
        }
        IssueAction::Block { identifier, by } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let blocker = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&by)
                .fetch_one(pool)
                .await?;
            sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocked_by')")
                .bind(issue.id)
                .bind(blocker.id)
                .execute(pool)
                .await?;
            println!("{} is blocked by {}", identifier, by);
            notify_change();
        }
        IssueAction::Relate { identifier, to } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let target = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&to)
                .fetch_one(pool)
                .await?;
            sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'related')")
                .bind(issue.id)
                .bind(target.id)
                .execute(pool)
                .await?;
            println!("{} is related to {}", identifier, to);
            notify_change();
        }
        IssueAction::Delete { identifier } => {
            sqlx::query("DELETE FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .execute(pool)
                .await?;
            println!("Deleted {}", identifier);
            notify_change();
        }
        IssueAction::Triage { identifier, apply } => {
        IssueAction::Score { identifier, bv, tc, rr, size } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;

            let suggestion = crate::commands::triage::triage_logic(
                pool,
                issue.project_id,
                &issue.title,
                issue.description.as_deref(),
            )
            .await
            .map_err(|e| Box::<dyn std::error::Error>::from(e))?;

            if apply && suggestion.confidence > 0.0 {
                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
                if let Some(ref p) = suggestion.suggested_priority {
                    if issue.priority == "none" {
                        sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                            .bind(p).bind(&now).bind(issue.id).execute(pool).await?;
                    }
                }
                if let Some(aid) = suggestion.suggested_assignee_id {
                    if issue.assignee_id.is_none() {
                        sqlx::query("UPDATE issues SET assignee_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(aid).bind(&now).bind(issue.id).execute(pool).await?;
                    }
                }
                if let Some(eid) = suggestion.suggested_epic_id {
                    if issue.parent_id.is_none() {
                        sqlx::query("UPDATE issues SET parent_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(eid).bind(&now).bind(issue.id).execute(pool).await?;
                    }
                }
                for lid in &suggestion.suggested_label_ids {
                    let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING")
                        .bind(issue.id).bind(*lid).execute(pool).await;
                }
                notify_change();
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&suggestion)?);
            } else {
                println!("Triage for {} (confidence: {:.0}%)", identifier, suggestion.confidence * 100.0);
                println!("  Priority: {}", suggestion.suggested_priority.as_deref().unwrap_or("(none)"));
                println!("  Labels: {:?}", suggestion.suggested_label_ids);
                println!("  Assignee: {}", suggestion.suggested_assignee_id.map(|a| a.to_string()).unwrap_or("(none)".to_string()));
                println!("  Epic: {}", suggestion.suggested_epic_id.map(|e| e.to_string()).unwrap_or("(none)".to_string()));
                println!("  Reasoning: {}", suggestion.reasoning);
                if apply {
                    println!("  [Applied]");
                }
            }
        }
        IssueAction::Decompose { identifier, apply } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;

            let text = issue.description.as_deref().unwrap_or("");
            if text.is_empty() {
                eprintln!("Issue has no description to decompose");
                return Ok(());
            }

            let tasks = crate::commands::decomposition::decompose_text(text);
            if tasks.is_empty() {
                eprintln!("No decomposable structure found in description");
                return Ok(());
            }

            if apply {
                let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
                let mut created = Vec::new();
                for (idx, task) in tasks.iter().enumerate() {
                    let mut tx = pool.begin().await?;
                    let (counter, prefix): (i64, String) = sqlx::query_as(
                        "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
                    ).bind(issue.project_id).fetch_one(&mut *tx).await?;
                    let ident = format!("{}-{}", prefix, counter);
                    let max_pos: Option<f64> = sqlx::query_scalar(
                        "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
                    ).bind(issue.project_id).bind(issue.status_id).fetch_one(&mut *tx).await?;
                    let position = max_pos.unwrap_or(-1.0) + 1.0 + idx as f64;
                    let priority = task.suggested_priority.as_deref().unwrap_or(&issue.priority);
                    let sub_id: i64 = sqlx::query_scalar(
                        "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
                    )
                    .bind(issue.project_id).bind(&ident).bind(&task.title).bind(&task.description)
                    .bind(issue.status_id).bind(priority).bind(issue.assignee_id).bind(issue.id)
                    .bind(position).bind(&now).bind(&now)
                    .fetch_one(&mut *tx).await?;
                    let sub: Issue = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
                        .bind(sub_id).fetch_one(&mut *tx).await?;
                    tx.commit().await?;
                    created.push(sub);
                }
                if json {
                    println!("{}", serde_json::to_string_pretty(&created)?);
                } else {
                    println!("Created {} sub-issues for {}:", created.len(), identifier);
                    for s in &created {
                        println!("  {} - {}", s.identifier, s.title);
                    }
                }
                notify_change();
            } else {
                if json {
                    println!("{}", serde_json::to_string_pretty(&tasks)?);
                } else {
                    println!("Decomposition preview for {} ({} tasks):", identifier, tasks.len());
                    for (i, t) in tasks.iter().enumerate() {
                        println!("  {}. {}", i + 1, t.title);
                        if let Some(ref d) = t.description {
                            let preview: String = d.chars().take(80).collect();
                            println!("     {}", preview);
                        }
                    }
                    println!("\nRun with --apply to create sub-issues.");
                }
            }
        }
        IssueAction::CreateFromText { text, project, status } => {
            let (title, description) = crate::commands::nl_create::parse_nl_text(&text);
            if title.is_empty() {
                eprintln!("Could not extract a title from the text");
                return Ok(());
            }

            let suggestion = crate::commands::triage::triage_logic(
                pool, project, &title, Some(&description),
            ).await.map_err(|e| Box::<dyn std::error::Error>::from(e))?;

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let priority = suggestion.suggested_priority.as_deref().unwrap_or("none");

            let mut tx = pool.begin().await?;
            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
            ).bind(project).fetch_one(&mut *tx).await?;
            let identifier = format!("{}-{}", prefix, counter);
            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2"
            ).bind(project).bind(status).fetch_one(&mut *tx).await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
            )
            .bind(project).bind(&identifier).bind(&title).bind(&description)
            .bind(status).bind(priority).bind(suggestion.suggested_assignee_id)
            .bind(None::<i64>).bind(position).bind(&now).bind(&now)
            .fetch_one(&mut *tx).await?;

            for lid in &suggestion.suggested_label_ids {
                let _ = sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING")
                    .bind(issue_id).bind(*lid).execute(&mut *tx).await;
            }

            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                .bind(issue_id).fetch_one(&mut *tx).await?;
            tx.commit().await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&issue)?);
            } else {
                println!("Created: {} - {}", issue.identifier, issue.title);
                println!("  Priority: {}", issue.priority);
                if !suggestion.suggested_label_ids.is_empty() {
                    println!("  Labels: {:?}", suggestion.suggested_label_ids);
                }
                if let Some(aid) = suggestion.suggested_assignee_id {
                    println!("  Assignee: {}", aid);
            let bv = bv.max(1).min(10);
            let tc = tc.max(1).min(10);
            let rr = rr.max(1).min(10);
            let size = size.max(1).min(10);
            let score = (bv as f64 + tc as f64 + rr as f64) / size as f64;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

            sqlx::query(
                "UPDATE issues SET business_value = $1, time_criticality = $2, risk_reduction = $3, job_size = $4, wsjf_score = $5, updated_at = $6 WHERE id = $7"
            )
            .bind(bv).bind(tc).bind(rr).bind(size).bind(score).bind(&now).bind(issue.id)
            .execute(pool)
            .await?;

            if json {
                println!("{}", serde_json::json!({
                    "identifier": identifier,
                    "business_value": bv,
                    "time_criticality": tc,
                    "risk_reduction": rr,
                    "job_size": size,
                    "wsjf_score": score
                }));
            } else {
                println!("{} | WSJF={:.2} (bv={}, tc={}, rr={}, size={})", identifier, score, bv, tc, rr, size);
            }
            notify_change();
        }
        IssueAction::Rank { project } => {
            let issues = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.project_id = $1 AND s.category = 'unstarted' AND i.wsjf_score IS NOT NULL ORDER BY i.wsjf_score DESC"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            if json {
                let ranked: Vec<serde_json::Value> = issues.iter().map(|i| serde_json::json!({
                    "identifier": i.identifier,
                    "title": i.title,
                    "wsjf_score": i.wsjf_score,
                    "business_value": i.business_value,
                    "time_criticality": i.time_criticality,
                    "risk_reduction": i.risk_reduction,
                    "job_size": i.job_size,
                    "priority": i.priority,
                })).collect();
                println!("{}", serde_json::to_string_pretty(&ranked)?);
            } else {
                println!("{:<10} {:<8} {:<6} {:<6} {:<6} {:<6} {}", "ID", "WSJF", "BV", "TC", "RR", "Size", "Title");
                println!("{}", "-".repeat(70));
                for i in &issues {
                    println!(
                        "{:<10} {:<8.2} {:<6} {:<6} {:<6} {:<6} {}",
                        i.identifier,
                        i.wsjf_score.unwrap_or(0.0),
                        i.business_value.unwrap_or(0),
                        i.time_criticality.unwrap_or(0),
                        i.risk_reduction.unwrap_or(0),
                        i.job_size.unwrap_or(0),
                        i.title
                    );
                }
            }
        }
        IssueAction::AutoScore { identifier } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier)
                .fetch_one(pool)
                .await?;
            let result = crate::commands::scoring::auto_score_issue_standalone(pool, issue.id).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{} | WSJF={:.2} | {}", identifier, result.wsjf_score, result.reasoning);
            }
            notify_change();
        }
        IssueAction::AutoScoreAll { project } => {
            let unscored = sqlx::query_as::<_, Issue>(
                "SELECT * FROM issues WHERE project_id = $1 AND wsjf_score IS NULL"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            let mut results = Vec::new();
            for issue in &unscored {
                let result = crate::commands::scoring::auto_score_issue_standalone(pool, issue.id).await?;
                results.push(result);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                println!("Auto-scored {} issues:", results.len());
                for r in &results {
                    println!("  Issue {} | WSJF={:.2} | {}", r.issue_id, r.wsjf_score, r.reasoning);
                }
            }
            notify_change();
        }
        IssueAction::FromDiff { project, title, file, line, severity, description, assignee } => {
            let issue = crate::commands::diff_issues::create_issue_from_diff_async(
                pool,
                crate::commands::diff_issues::DiffIssueInput {
                    project_id: project,
                    title,
                    description,
                    file_path: file,
                    line_range: line,
                    severity,
                    status_id: None,
                    assignee_id: assignee,
                },
            ).await.map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&issue)?);
            } else {
                println!("{} | {} | {}", issue.identifier, issue.title, issue.priority);
            }
            notify_change();
        }
    }
    Ok(())
}

async fn handle_member(
    pool: &AnyPool,
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
            let member_id: i64 = sqlx::query_scalar(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(&name)
            .bind(&display_name)
            .bind(&email)
            .bind(color)
            .bind(&now)
            .fetch_one(pool)
            .await?;
            let member = sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1")
                .bind(member_id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&member)?);
            } else {
                println!("Added member: {}", member.name);
            }
            notify_change();
        }
        MemberAction::Delete { id } => {
            sqlx::query("DELETE FROM members WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted member {}", id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_label(
    pool: &AnyPool,
    action: LabelAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        LabelAction::List { project } => {
            let labels = sqlx::query_as::<_, Label>(
                "SELECT * FROM labels WHERE project_id = $1 ORDER BY name",
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
            let label_id: i64 = sqlx::query_scalar(
                "INSERT INTO labels (project_id, name, color) VALUES ($1, $2, $3) RETURNING id",
            )
                    .bind(project)
                    .bind(&name)
                    .bind(&color)
                    .fetch_one(pool)
                    .await?;
            let label = sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = $1")
                .bind(label_id)
                .fetch_one(pool)
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&label)?);
            } else {
                println!("Created label: {} ({})", label.name, label.color);
            }
            notify_change();
        }
        LabelAction::Delete { id } => {
            sqlx::query("DELETE FROM labels WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted label {}", id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_notifications(
    pool: &AnyPool,
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
            notify_change();
        }
    }
    Ok(())
}

async fn handle_comment(
    pool: &AnyPool,
    action: CommentAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CommentAction::List { identifier } => {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier).fetch_one(pool).await?;
            let comments = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE issue_id = $1 ORDER BY created_at ASC")
                .bind(issue.id).fetch_all(pool).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&comments)?);
            } else {
                if comments.is_empty() {
                    println!("No comments on {}", identifier);
                } else {
                    for c in &comments {
                        let member_name = if let Some(mid) = c.member_id {
                            sqlx::query_scalar::<_, String>("SELECT COALESCE(display_name, name) FROM members WHERE id = $1")
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
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                .bind(&identifier).fetch_one(pool).await?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let comment_id: i64 = sqlx::query_scalar("INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5) RETURNING id")
                .bind(issue.id).bind(member).bind(&content).bind(&now).bind(&now)
                .fetch_one(pool).await?;
            let comment = sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
                .bind(comment_id).fetch_one(pool).await?;
            if json { println!("{}", serde_json::to_string_pretty(&comment)?); }
            else { println!("Comment added to {} (id: {})", identifier, comment.id); }
            notify_change();
        }
        CommentAction::Delete { id } => {
            sqlx::query("DELETE FROM comments WHERE id = $1").bind(id).execute(pool).await?;
            println!("Deleted comment {}", id);
            notify_change();
        }
    }
    Ok(())
}

/// Helper: resolve an issue identifier (e.g. "KAN-42") to the issue row.
async fn resolve_issue(pool: &AnyPool, identifier: &str) -> Result<Issue, Box<dyn std::error::Error>> {
    let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
        .bind(identifier)
        .fetch_one(pool)
        .await?;
    Ok(issue)
}

/// Helper: sync an issue's status_id to a status matching the given category.
async fn sync_issue_status_to_category(pool: &AnyPool, issue_id: i64, category: &str) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE issues SET status_id = (
            SELECT s.id FROM statuses s
            WHERE s.project_id = (SELECT project_id FROM issues WHERE id = $1)
              AND s.category = $2
            ORDER BY s.position ASC LIMIT 1
         ), updated_at = $3
         WHERE id = $4",
    )
    .bind(issue_id)
    .bind(category)
    .bind(&now)
    .bind(issue_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Helper: auto-comment on an issue as an agent
async fn cli_auto_comment(pool: &AnyPool, issue_id: i64, agent_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    let agent_member_id: Option<i64> = sqlx::query_scalar(
        "SELECT member_id FROM agents WHERE id = $1"
    ).bind(agent_id).fetch_optional(pool).await?;
    sqlx::query(
        "INSERT INTO comments (issue_id, member_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
    ).bind(issue_id).bind(agent_member_id).bind(content).bind(&now).bind(&now)
    .execute(pool).await?;
    Ok(())
}

async fn handle_agent(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    action: AgentAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        AgentAction::Register {
            name,
            agent_type,
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

            // Generate name if not provided
            let agent_name = if name.is_empty() {
                orchestration::names::generate_agent_name()
            } else {
                name
            };

            // Determine avatar color based on agent type
            let agent_type_str = agent_type.as_deref().unwrap_or("custom");
            let avatar_color = match agent_type_str {
                "claude" | "claude-code" => "#f97316",
                "codex" => "#22c55e",
                "gemini" => "#3b82f6",
                _ => "#8b5cf6",
            };

            // Create a member for this agent
            let member_id: i64 = sqlx::query_scalar(
                "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"
            )
            .bind(format!("[{}] {}", agent_type_str, &agent_name))
            .bind(&agent_name)
            .bind(Option::<String>::None)
            .bind(avatar_color)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO agents (id, name, agent_type, skills, task_types, max_concurrent, max_complexity, member_id, status, registered_at, last_heartbeat) VALUES ($1, $2, $3, $4{jb}, $5{jb}, $6, $7, $8, 'online', $9, $10)",
            ))
            .bind(&id)
            .bind(&agent_name)
            .bind(&agent_type)
            .bind(&skills_json)
            .bind(&task_types_json)
            .bind(max_concurrent)
            .bind(&max_complexity)
            .bind(member_id)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            sqlx::query(&format!(
                "INSERT INTO agent_stats (agent_id, tasks_completed, tasks_failed, total_confidence, total_completion_time_seconds, skills_breakdown) VALUES ($1, 0, 0, 0.0, 0, '{{}}'{jb})",
            ))
            .bind(&id)
            .execute(pool)
            .await?;

            if json {
                let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
                    .bind(&id)
                    .fetch_one(pool)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&agent)?);
            } else {
                println!("Registered agent: {} (id: {})", agent_name, id);
            }
            notify_change();
        }
        AgentAction::Heartbeat { id } => {
            let now = chrono::Utc::now().to_rfc3339();
            let active_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')",
            )
            .bind(&id)
            .fetch_one(pool)
            .await?;
            let new_status = if active_count.0 > 0 { "busy" } else { "online" };
            sqlx::query("UPDATE agents SET last_heartbeat = $1, status = $2 WHERE id = $3")
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
            notify_change();
        }
        AgentAction::Deregister { id } => {
            let now = chrono::Utc::now().to_rfc3339();
            // Reclaim active tasks
            let active_tasks: Vec<(i64,)> = sqlx::query_as(
                "SELECT issue_id FROM task_contracts WHERE claimed_by = $1 AND task_state IN ('claimed', 'executing')",
            )
            .bind(&id)
            .fetch_all(pool)
            .await?;

            for (issue_id,) in &active_tasks {
                sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1")
                    .bind(issue_id)
                    .execute(pool)
                    .await?;
                sync_issue_status_to_category(pool, *issue_id, "unstarted").await?;
                sqlx::query(
                    "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, 0, 'reclaim', 'Agent deregistered, task requeued', $3)",
                )
                .bind(issue_id)
                .bind(&id)
                .bind(&now)
                .execute(pool)
                .await?;
            }

            sqlx::query("DELETE FROM agents WHERE id = $1")
                .bind(&id)
                .execute(pool)
                .await?;
            println!("Deregistered agent {} ({} tasks requeued)", id, active_tasks.len());
            notify_change();
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
            let stats = sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = $1")
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
                    "skills_breakdown": stats.skills_breakdown_json(),
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
        AgentAction::Permissions { id } => {
            use crate::commands::permissions::AgentPermission;
            let perms = sqlx::query_as::<_, AgentPermission>(
                "SELECT * FROM agent_permissions WHERE agent_id = $1 ORDER BY permission_type, scope",
            )
            .bind(&id)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&perms)?);
            } else {
                if perms.is_empty() {
                    println!("No permissions configured for agent {} (full access by default)", id);
                } else {
                    println!("Permissions for agent {}:", id);
                    for p in &perms {
                        let allow_str = if p.allowed { "ALLOW" } else { "DENY" };
                        println!("  [{}] {} scope={}", allow_str, p.permission_type, p.scope);
                    }
                }
            }
        }
        AgentAction::Grant { id, permission_type, scope } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query(
                "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, 1, $4)",
            )
            .bind(&id)
            .bind(&permission_type)
            .bind(&scope)
            .bind(&now)
            .execute(pool)
            .await?;
            println!("Granted {} permission for scope '{}' to agent {}", permission_type, scope, id);
            notify_change();
        }
        AgentAction::Deny { id, permission_type, scope } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query(
                "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, 0, $4)",
            )
            .bind(&id)
            .bind(&permission_type)
            .bind(&scope)
            .bind(&now)
            .execute(pool)
            .await?;
            println!("Denied {} permission for scope '{}' for agent {}", permission_type, scope, id);
            notify_change();
        }
        AgentAction::ClearPermissions { id } => {
            sqlx::query("DELETE FROM agent_permissions WHERE agent_id = $1")
                .bind(&id)
                .execute(pool)
                .await?;
            println!("Cleared all permissions for agent {}", id);
            notify_change();
        }
        AgentAction::ApplyPreset { id, preset } => {
            use crate::commands::permissions::{AgentPermission, PermissionPreset, PresetPermissionEntry};
            let preset_row = sqlx::query_as::<_, PermissionPreset>(
                "SELECT * FROM agent_permission_presets WHERE name = $1",
            )
            .bind(&preset)
            .fetch_one(pool)
            .await?;
            let entries: Vec<PresetPermissionEntry> = serde_json::from_str(&preset_row.permissions)?;
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query("DELETE FROM agent_permissions WHERE agent_id = $1")
                .bind(&id)
                .execute(pool)
                .await?;
            for entry in &entries {
                sqlx::query(
                    "INSERT INTO agent_permissions (agent_id, permission_type, scope, allowed, created_at) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&id)
                .bind(&entry.permission_type)
                .bind(&entry.scope)
                .bind(entry.allowed)
                .bind(&now)
                .execute(pool)
                .await?;
            }
            println!("Applied preset '{}' ({} rules) to agent {}", preset, entries.len(), id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_marketplace(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    action: MarketplaceAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MarketplaceAction::List => {
            let entries: Vec<(String, String, Option<String>, Option<String>, String, Option<f64>, i64)> = sqlx::query_as(
                "SELECT agent_id, name, description, provider, capabilities, rating, total_tasks FROM agent_registry ORDER BY rating DESC NULLS LAST"
            ).fetch_all(pool).await?;
            if json {
                let results: Vec<serde_json::Value> = entries.iter().map(|(aid, name, desc, provider, caps, rating, tasks)| {
                    serde_json::json!({"agent_id": aid, "name": name, "description": desc, "provider": provider, "capabilities": serde_json::from_str::<serde_json::Value>(caps).unwrap_or(serde_json::json!([])), "rating": rating, "total_tasks": tasks})
                }).collect();
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for (aid, name, _desc, provider, _caps, rating, tasks) in &entries {
                    println!("{} | {} | {} | rating: {} | tasks: {}", aid, name, provider.as_deref().unwrap_or("-"), rating.map(|r| format!("{:.2}", r)).unwrap_or_else(|| "-".to_string()), tasks);
                }
                if entries.is_empty() { println!("No agents registered in marketplace."); }
            }
        }
        MarketplaceAction::Register { agent_id, name, skills, provider, description, max_complexity } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let caps: Vec<String> = skills.split(',').map(|s| s.trim().to_string()).collect();
            let caps_json = serde_json::to_string(&caps)?;
            let jb = crate::db::compat::jsonb_cast(backend);
            let mx = max_complexity.as_deref().unwrap_or("medium");

            sqlx::query(&format!(
                "INSERT INTO agent_registry (agent_id, name, description, provider, capabilities, max_concurrent, max_complexity, registered_at, last_seen_at) VALUES ($1, $2, $3, $4, $5{jb}, 1, $6, $7, $8) ON CONFLICT(agent_id) DO UPDATE SET name=$2, description=$3, provider=$4, capabilities=$5{jb}, max_complexity=$6, last_seen_at=$8"
            ))
            .bind(&agent_id).bind(&name).bind(&description).bind(&provider)
            .bind(&caps_json).bind(mx).bind(&now).bind(&now)
            .execute(pool).await?;

            for cap in &caps {
                let _ = sqlx::query(
                    "INSERT INTO agent_capabilities (agent_id, capability) VALUES ($1, $2) ON CONFLICT(agent_id, capability) DO NOTHING"
                ).bind(&agent_id).bind(cap).execute(pool).await;
            }

            if json {
                println!("{}", serde_json::json!({"agent_id": agent_id, "name": name, "capabilities": caps, "status": "registered"}));
            } else {
                println!("Registered {} in marketplace (skills: {})", name, skills);
            }
            notify_change();
        }
        MarketplaceAction::Search { skills, max_complexity } => {
            let skill_list: Vec<String> = skills.split(',').map(|s| s.trim().to_string()).collect();
            let all: Vec<(String, String, String, String, Option<f64>, i64)> = sqlx::query_as(
                "SELECT agent_id, name, capabilities, max_complexity, rating, total_tasks FROM agent_registry ORDER BY rating DESC NULLS LAST"
            ).fetch_all(pool).await?;

            let complexity_order = |c: &str| match c { "small" => 1, "medium" => 2, "large" => 3, _ => 2 };
            let max_cx_val = complexity_order(max_complexity.as_deref().unwrap_or("large"));

            let results: Vec<&(String, String, String, String, Option<f64>, i64)> = all.iter().filter(|(_, _, caps, mx, _, _)| {
                if complexity_order(mx) < max_cx_val { return false; }
                let caps_list: Vec<String> = serde_json::from_str(caps).unwrap_or_default();
                skill_list.iter().any(|s| caps_list.iter().any(|c| c.contains(s) || s.contains(c)))
            }).collect();

            if json {
                let r: Vec<serde_json::Value> = results.iter().map(|(aid, name, caps, _, rating, tasks)| {
                    serde_json::json!({"agent_id": aid, "name": name, "capabilities": serde_json::from_str::<serde_json::Value>(caps).unwrap_or(serde_json::json!([])), "rating": rating, "total_tasks": tasks})
                }).collect();
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                for (aid, name, _, _, rating, tasks) in &results {
                    println!("{} | {} | rating: {} | tasks: {}", aid, name, rating.map(|r| format!("{:.2}", r)).unwrap_or_else(|| "-".to_string()), tasks);
                }
                if results.is_empty() { println!("No matching agents found."); }
            }
        }
        MarketplaceAction::BestMatch { skills, complexity } => {
            let skill_list: Vec<String> = skills.split(',').map(|s| s.trim().to_string()).collect();
            let entries: Vec<(String, String, String, Option<f64>)> = sqlx::query_as(
                "SELECT agent_id, name, max_complexity, rating FROM agent_registry"
            ).fetch_all(pool).await?;

            let complexity_order = |c: &str| match c { "small" => 1, "medium" => 2, "large" => 3, _ => 2 };
            let target_cx = complexity_order(&complexity);

            let mut matches: Vec<(String, String, f64, Vec<String>)> = Vec::new();
            for (agent_id, name, max_cx, rating) in entries {
                if complexity_order(&max_cx) < target_cx { continue; }
                let caps: Vec<(String, f64)> = sqlx::query_as(
                    "SELECT capability, proficiency FROM agent_capabilities WHERE agent_id = $1"
                ).bind(&agent_id).fetch_all(pool).await?;

                let mut matched = Vec::new();
                let mut total_prof = 0.0;
                for skill in &skill_list {
                    if let Some((_, prof)) = caps.iter().find(|(c, _)| c.contains(skill) || skill.contains(c)) {
                        matched.push(skill.clone());
                        total_prof += prof;
                    }
                }
                if matched.is_empty() && !skill_list.is_empty() { continue; }
                let avg_prof = if matched.is_empty() { 0.5 } else { total_prof / matched.len() as f64 };
                let skill_ratio = if skill_list.is_empty() { 1.0 } else { matched.len() as f64 / skill_list.len() as f64 };
                let r = rating.unwrap_or(0.5);
                let score = skill_ratio * 0.4 + avg_prof * 0.3 + r * 0.3;
                matches.push((agent_id, name, score, matched));
            }
            matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            if json {
                let r: Vec<serde_json::Value> = matches.iter().map(|(aid, name, score, matched)| {
                    serde_json::json!({"agent_id": aid, "name": name, "score": score, "matched_skills": matched})
                }).collect();
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                for (aid, name, score, matched) in &matches {
                    println!("{} | {} | score: {:.2} | matched: {}", aid, name, score, matched.join(", "));
                }
                if matches.is_empty() { println!("No matching agents found."); }
            }
        }
        MarketplaceAction::Deregister { agent_id } => {
            sqlx::query("DELETE FROM agent_capabilities WHERE agent_id = $1")
                .bind(&agent_id).execute(pool).await?;
            sqlx::query("DELETE FROM agent_registry WHERE agent_id = $1")
                .bind(&agent_id).execute(pool).await?;
            println!("Deregistered {} from marketplace", agent_id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_task(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    action: TaskAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        TaskAction::Next { agent, skills } => {
            // Lazy timeout recovery - reclaim stale tasks before routing
            let _ = crate::orchestration::timeout::reclaim_timed_out_tasks(pool, backend).await;

            let agent_row = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
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
                    let _ = cli_auto_comment(pool, contract.issue_id, &agent, "\u{1F916} Task claimed. Reading contract and preparing to execute.").await;
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
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = $1")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'start', 'Task execution started', $4)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            let _ = cli_auto_comment(pool, issue.id, &agent, "\u{1F527} Execution started.").await;
            if json {
                println!("{}", serde_json::json!({"status": "executing", "identifier": identifier}));
            } else {
                println!("Started: {}", identifier);
            }
            notify_change();
        }
        TaskAction::Complete {
            identifier,
            agent,
            confidence,
            summary,
            artifacts,
        } => {
            if confidence < 0.0 || confidence > 1.0 {
                return Err("Confidence must be between 0.0 and 1.0".into());
            }
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();

            // Get project config thresholds
            let config = sqlx::query_as::<_, ProjectAgentConfig>(
                "SELECT * FROM project_agent_config WHERE project_id = $1",
            )
            .bind(issue.project_id)
            .fetch_optional(pool)
            .await?;

            let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.85);
            let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.50);

            let new_state = if confidence >= auto_accept {
                "completed"
            } else if confidence >= human_review {
                "validating"
            } else {
                "queued"
            };

            let result_json = serde_json::json!({
                "confidence": confidence,
                "summary": summary,
                "artifacts": artifacts.as_ref().and_then(|a| serde_json::from_str::<serde_json::Value>(a).ok()),
            });

            let jb = crate::db::compat::jsonb_cast(backend);
            if new_state == "queued" {
                // Auto-reject: requeue with cleared claim
                sqlx::query(&format!(
                    "UPDATE task_contracts SET task_state = 'queued', result = $1{jb}, claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = $2"
                ))
                    .bind(result_json.to_string())
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            } else {
                sqlx::query(&format!(
                    "UPDATE task_contracts SET task_state = $1, result = $2{jb} WHERE issue_id = $3"
                ))
                    .bind(new_state)
                    .bind(result_json.to_string())
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }

            // Sync issue status
            let category = orchestration::state_machine::task_state_to_status_category(
                orchestration::state_machine::TaskState::from_str(new_state)
                    .map_err(|e| Box::<dyn std::error::Error>::from(e))?,
            );
            sync_issue_status_to_category(pool, issue.id, category).await?;

            // Log
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'complete', $4, $5)",
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
                "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + $1 WHERE agent_id = $2",
            )
            .bind(confidence)
            .bind(&agent)
            .execute(pool)
            .await?;

            // Auto-comment based on outcome
            if new_state == "completed" {
                let _ = cli_auto_comment(pool, issue.id, &agent, &format!("\u{2705} Task completed (confidence: {:.2}). {}", confidence, summary)).await;
            } else if new_state == "validating" {
                let _ = cli_auto_comment(pool, issue.id, &agent, &format!("\u{23F3} Task completed with low confidence ({:.2}). Awaiting review. {}", confidence, summary)).await;
            }

            // Auto-unblock downstream tasks when completed
            if new_state == "completed" {
                let _ = crate::orchestration::dependency::resolve_downstream(pool, issue.id).await;
            }

            if json {
                println!("{}", serde_json::json!({"status": new_state, "identifier": identifier, "confidence": confidence}));
            } else {
                println!("Completed: {} (state: {}, confidence: {:.2})", identifier, new_state, confidence);
            }
            notify_change();
        }
        TaskAction::Fail {
            identifier,
            agent,
            reason,
        } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            let now = chrono::Utc::now().to_rfc3339();

            // Append to prior_attempts in context
            let mut context: serde_json::Value = tc.context_json();
            if let Some(obj) = context.as_object_mut() {
                let arr = obj.entry("prior_attempts").or_insert(serde_json::json!([]));
                if let Some(a) = arr.as_array_mut() {
                    a.push(serde_json::json!({
                "attempt": tc.attempt_count,
                "agent": agent,
                "reason": reason,
                "timestamp": now,
            }));
                }
            }

            let jb = crate::db::compat::jsonb_cast(backend);
            // Requeue
            sqlx::query(&format!(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1, context = $1{jb} WHERE issue_id = $2",
            ))
            .bind(context.to_string())
            .bind(issue.id)
            .execute(pool)
            .await?;

            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;

            // Log
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'fail', $4, $5)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&reason)
            .bind(&now)
            .execute(pool)
            .await?;

            // Update agent stats
            sqlx::query("UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = $1")
                .bind(&agent)
                .execute(pool)
                .await?;

            let _ = cli_auto_comment(pool, issue.id, &agent, &format!("\u{274C} Task failed: {}", reason)).await;

            if json {
                println!("{}", serde_json::json!({"status": "requeued", "identifier": identifier}));
            } else {
                println!("Failed and requeued: {}", identifier);
            }
            notify_change();
        }
        TaskAction::Unclaim { identifier, agent } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.claimed_by.as_deref() != Some(&agent) {
                return Err(format!("Task {} is not claimed by agent {}", identifier, agent).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'unclaim', 'Task unclaimed by agent', $4)",
            )
            .bind(issue.id)
            .bind(&agent)
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
            let _ = cli_auto_comment(pool, issue.id, &agent, "\u{21A9}\u{FE0F} Task unclaimed and returned to queue.").await;
            if json {
                println!("{}", serde_json::json!({"status": "queued", "identifier": identifier}));
            } else {
                println!("Unclaimed: {}", identifier);
            }
            notify_change();
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
                "SELECT attempt_count FROM task_contracts WHERE issue_id = $1",
            )
            .bind(issue.id)
            .fetch_one(pool)
            .await?;
            let now = chrono::Utc::now().to_rfc3339();
            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES ($1, $2, $3, $4, $5, $6{jb}, $7)",
            ))
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
            notify_change();
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
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
            )
            .bind(project)
            .fetch_one(&mut *tx)
            .await?;
            let identifier = format!("{}-{}", prefix, counter);

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
            )
            .bind(project)
            .bind(status)
            .fetch_one(&mut *tx)
            .await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            // Resolve parent
            let parent_id: Option<i64> = if let Some(ref p) = parent {
                let pi = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                    .bind(p)
                    .fetch_one(&mut *tx)
                    .await?;
                Some(pi.id)
            } else {
                None
            };

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
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
            .fetch_one(&mut *tx)
            .await?;

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

            let jb = crate::db::compat::jsonb_cast(backend);
            sqlx::query(&format!(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4{jb}, $5{jb}, $6{jb}, $7{jb}, $8, $9, 0)",
            ))
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
                    let dep_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE identifier = $1")
                        .bind(dep_ident)
                        .fetch_one(&mut *tx)
                        .await?;
                    sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, 'blocks')")
                        .bind(dep_issue.id)
                        .bind(issue_id)
                        .execute(&mut *tx)
                        .await?;
                }
            }

            tx.commit().await?;

            // Check if this task needs decomposition
            if let Ok(true) = crate::orchestration::decomposition::check_decomposition_needed(pool, issue_id).await {
                let _ = crate::orchestration::decomposition::create_decomposition_task(pool, issue_id).await;
            }

            let contract = orchestration::routing::build_full_contract(pool, issue_id).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&contract)?);
            } else {
                println!("Created: {} - {}", identifier, title);
            }
            notify_change();
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
                "SELECT * FROM execution_logs WHERE issue_id = $1 ORDER BY timestamp ASC",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&logs)?);
            } else if logs.is_empty() {
                println!("No execution logs for {}", identifier);
            } else {
                let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                    .bind(issue.id).fetch_optional(pool).await?;

                println!("{}: {}", identifier, issue.title);
                if let Some(ref c) = contract {
                    let confidence = c.result.as_ref()
                        .map(|s| crate::models::agent::parse_json(s))
                        .and_then(|v| v.get("confidence").and_then(|c| c.as_f64()));
                    println!("State: {} | Attempts: {}{}", c.task_state, c.attempt_count,
                        confidence.map(|c| format!(" | Confidence: {:.2}", c)).unwrap_or_default());
                }
                println!("---");
                for log in &logs {
                    let type_label = match log.entry_type.as_str() {
                        "claim" => "CLAIM   ",
                        "start" => "START   ",
                        "reasoning" => "THINK   ",
                        "file_read" => "READ    ",
                        "file_edit" => "EDIT    ",
                        "command" => "RUN     ",
                        "discovery" => "DISCOVER",
                        "error" => "ERROR   ",
                        "result" | "complete" => "RESULT  ",
                        "checkpoint" => "CHECK   ",
                        "timeout" => "TIMEOUT ",
                        "unblocked" => "UNBLOCK ",
                        _ => &log.entry_type,
                    };
                    let time = if log.timestamp.len() >= 19 { &log.timestamp[11..19] } else { &log.timestamp };
                    println!("[{}] {} {}", time, type_label, log.message);
                }
            }
        }
        TaskAction::Attempts { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            let context: serde_json::Value = tc.context_json();
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
            let mut qb: QueryBuilder<Any> = QueryBuilder::new(
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
                "SELECT * FROM issues WHERE parent_id = $1 ORDER BY position",
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
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, tc.task_state).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = $1")
                .bind(issue.id)
                .execute(pool)
                .await?;
            sync_issue_status_to_category(pool, issue.id, "completed").await?;
            // Update agent stats if there's a claimed_by
            if let Some(ref agent_id) = tc.claimed_by {
                sqlx::query("UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = $1")
                    .bind(agent_id)
                    .execute(pool)
                    .await?;
            }
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'approve', 'Task approved', $4)",
            )
            .bind(issue.id)
            .bind(tc.claimed_by.as_deref().unwrap_or("system"))
            .bind(tc.attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;

            // Auto-unblock downstream tasks
            let _ = crate::orchestration::dependency::resolve_downstream(pool, issue.id).await;

            if json {
                println!("{}", serde_json::json!({"status": "completed", "identifier": identifier}));
            } else {
                println!("Approved: {}", identifier);
            }
            notify_change();
        }
        TaskAction::Reject { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1")
                .bind(issue.id)
                .fetch_one(pool)
                .await?;
            if tc.task_state != "validating" {
                return Err(format!("Task {} is in state '{}', expected 'validating'", identifier, tc.task_state).into());
            }
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = $1",
            )
            .bind(issue.id)
            .execute(pool)
            .await?;
            sync_issue_status_to_category(pool, issue.id, "unstarted").await?;
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'reject', 'Task rejected, requeued', $4)",
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
            notify_change();
        }
        TaskAction::Invalidate { identifier, reason } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let result = crate::orchestration::cascade::invalidate_task(pool, issue.id, &reason).await?;
            if json {
                println!("{}", serde_json::json!({"success": true, "data": result}));
            } else {
                println!("Task {} invalidated", identifier);
                println!("  {} tasks blocked, {} warned, {} review tasks created",
                    result.tasks_blocked.len(), result.tasks_warned.len(), result.review_tasks_created.len());
            }
            notify_change();
        }
        TaskAction::Graph { identifier } => {
            let issue_id = resolve_issue(pool, &identifier).await?.id;

            // Build graph by walking relations
            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(issue_id);

            while let Some(id) = queue.pop_front() {
                if !visited.insert(id) { continue; }
                let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1").bind(id).fetch_one(pool).await?;
                let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = $1").bind(id).fetch_optional(pool).await?;
                let state = contract.as_ref().map(|c| c.task_state.as_str()).unwrap_or("no-contract");

                nodes.push(serde_json::json!({"id": id, "identifier": &issue.identifier, "title": &issue.title, "state": state}));

                // Children
                let children: Vec<i64> = sqlx::query_scalar("SELECT id FROM issues WHERE parent_id = $1").bind(id).fetch_all(pool).await?;
                for c in children { edges.push(serde_json::json!({"from": id, "to": c, "type": "parent-child"})); queue.push_back(c); }

                // Relations
                let rels: Vec<(i64, i64)> = sqlx::query_as("SELECT source_issue_id, target_issue_id FROM issue_relations WHERE (source_issue_id = $1 OR target_issue_id = $2) AND relation_type = 'blocks'").bind(id).bind(id).fetch_all(pool).await?;
                for (s, t) in rels { edges.push(serde_json::json!({"from": s, "to": t, "type": "blocks"})); if s != id { queue.push_back(s); } if t != id { queue.push_back(t); } }

                if let Some(pid) = issue.parent_id { edges.push(serde_json::json!({"from": pid, "to": id, "type": "parent-child"})); queue.push_back(pid); }
            }

            if json {
                println!("{}", serde_json::json!({"success": true, "data": {"nodes": nodes, "edges": edges}}));
            } else {
                println!("Graph for {}:", identifier);
                for node in &nodes {
                    let state_str = node["state"].as_str().unwrap_or("?");
                    let symbol = match state_str { "completed" => "\u{2713}", "executing" => "\u{25b6}", "blocked" => "\u{2717}", "queued" => "\u{25cb}", _ => "?" };
                    println!("  {} {} {} - {}", symbol, node["identifier"].as_str().unwrap_or(""), state_str, node["title"].as_str().unwrap_or(""));
                }
                if !edges.is_empty() {
                    println!("Dependencies:");
                    for edge in &edges {
                        println!("  {} -> {} ({})", edge["from"], edge["to"], edge["type"].as_str().unwrap_or(""));
                    }
                }
            }
        }
        TaskAction::Search { project, query } => {
            let pattern = format!("%{}%", query);
            let rows = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN task_contracts tc ON tc.issue_id = i.id WHERE i.project_id = $1 AND (i.title LIKE $2 OR tc.objective LIKE $3 OR i.identifier LIKE $4) ORDER BY i.updated_at DESC",
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
                sqlx::query("UPDATE issues SET title = $1, updated_at = $2 WHERE id = $3")
                    .bind(t)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref p) = priority {
                sqlx::query("UPDATE issues SET priority = $1, updated_at = $2 WHERE id = $3")
                    .bind(p)
                    .bind(&now)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref c) = complexity {
                sqlx::query("UPDATE task_contracts SET estimated_complexity = $1 WHERE issue_id = $2")
                    .bind(c)
                    .bind(issue.id)
                    .execute(pool)
                    .await?;
            }
            if let Some(ref s) = skills {
                let skills_json = serde_json::to_string(
                    &s.split(',').map(|s| s.trim()).collect::<Vec<_>>(),
                )?;
                let jb = crate::db::compat::jsonb_cast(backend);
                sqlx::query(&format!(
                    "UPDATE task_contracts SET required_skills = $1{jb} WHERE issue_id = $2"
                ))
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
            notify_change();
        }
        TaskAction::Decompose { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            match crate::orchestration::decomposition::create_decomposition_task(pool, issue.id).await {
                Ok(new_id) => {
                    let new_issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
                        .bind(new_id).fetch_one(pool).await?;
                    if json {
                        println!("{}", serde_json::json!({"success": true, "data": {"decomposition_task": new_issue.identifier}}));
                    } else {
                        println!("Created decomposition task: {}", new_issue.identifier);
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"success": false, "error": e.to_string()}));
                    } else {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            notify_change();
        }
        TaskAction::Context { identifier } => {
            let ctx = crate::commands::context::get_task_context_async(pool, &identifier)
                .await
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&ctx)?);
            } else {
                println!("=== Task Context: {} ===", ctx.issue.identifier);
                println!("Title: {}", ctx.issue.title);
                println!("Priority: {} | Status ID: {}", ctx.issue.priority, ctx.issue.status_id);
                if let Some(ref desc) = ctx.issue.description {
                    println!("Description: {}", desc);
                }
                if let Some(ref path) = ctx.project_path {
                    println!("Project path: {}", path);
                }
                if !ctx.context_files.is_empty() {
                    println!("\nContext files:");
                    for f in &ctx.context_files {
                        println!("  - {}", f);
                    }
                }
                if !ctx.labels.is_empty() {
                    println!("\nLabels: {}", ctx.labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>().join(", "));
                }
                if let Some(ref parent) = ctx.parent_issue {
                    println!("\nParent: {} - {}", parent.identifier, parent.title);
                }
                if !ctx.sub_issues.is_empty() {
                    println!("\nSub-issues:");
                    for s in &ctx.sub_issues {
                        println!("  {} - {}", s.identifier, s.title);
                    }
                }
                if !ctx.blocking_issues.is_empty() {
                    println!("\nBlocked by:");
                    for b in &ctx.blocking_issues {
                        println!("  {} - {}", b.identifier, b.title);
                    }
                }
                if !ctx.blocked_issues.is_empty() {
                    println!("\nBlocks:");
                    for b in &ctx.blocked_issues {
                        println!("  {} - {}", b.identifier, b.title);
                    }
                }
                if !ctx.prior_attempts.is_empty() {
                    println!("\nPrior attempts:");
                    for a in &ctx.prior_attempts {
                        println!("  #{} by {} -> {} {}", a.attempt_number, a.agent_name, a.result, a.reason.as_deref().unwrap_or(""));
                    }
                }
                if !ctx.similar_completed_issues.is_empty() {
                    println!("\nSimilar completed issues:");
                    for s in &ctx.similar_completed_issues {
                        println!("  {} - {}", s.identifier, s.title);
                    }
                }
                if !ctx.comments.is_empty() {
                    println!("\nComments ({}):", ctx.comments.len());
                    for c in &ctx.comments {
                        println!("  [{}] {}", c.created_at, c.content);
                    }
                }
            }
        }
        TaskAction::Handoff {
            identifier,
            from,
            to,
            note_type,
            summary,
            details,
            files,
            risks,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let files_json = files
                .map(|f| serde_json::to_string(&f.split(',').map(|s| s.trim()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "[]".to_string());
            let risks_json = risks
                .map(|r| serde_json::to_string(&r.split(',').map(|s| s.trim()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "[]".to_string());

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO handoff_notes (task_identifier, from_agent_id, to_agent_id, note_type, summary, details, files_changed, risks, metadata, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}', $9) RETURNING id"
            )
            .bind(&identifier)
            .bind(&from)
            .bind(&to)
            .bind(&note_type)
            .bind(&summary)
            .bind(&details)
            .bind(&files_json)
            .bind(&risks_json)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            let note = sqlx::query_as::<_, agent::HandoffNote>("SELECT * FROM handoff_notes WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&note).unwrap_or_default());
            } else {
                println!("Created handoff note #{} for {} (type: {})", note.id, identifier, note.note_type);
            }
            notify_change();
        }
        TaskAction::Learn {
            identifier,
            agent,
            outcome,
            approach,
            insight,
            pitfalls,
            patterns,
            tags,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let pitfalls_json = pitfalls
                .map(|p| serde_json::to_string(&p.split(',').map(|s| s.trim()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "[]".to_string());
            let patterns_json = patterns
                .map(|p| serde_json::to_string(&p.split(',').map(|s| s.trim()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "[]".to_string());
            let tags_json = tags
                .map(|t| serde_json::to_string(&t.split(',').map(|s| s.trim()).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "[]".to_string());

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO task_learnings (task_identifier, agent_id, outcome, approach_summary, key_insight, pitfalls, effective_patterns, relevant_files, tags, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, '[]', $8, $9) RETURNING id"
            )
            .bind(&identifier)
            .bind(&agent)
            .bind(&outcome)
            .bind(&approach)
            .bind(&insight)
            .bind(&pitfalls_json)
            .bind(&patterns_json)
            .bind(&tags_json)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            let learning = sqlx::query_as::<_, agent::TaskLearning>("SELECT * FROM task_learnings WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&learning).unwrap_or_default());
            } else {
                println!("Recorded learning #{} for {} (outcome: {})", learning.id, identifier, learning.outcome);
            }
            notify_change();
        }
        TaskAction::Handoffs { identifier } => {
            let notes = sqlx::query_as::<_, agent::HandoffNote>(
                "SELECT * FROM handoff_notes WHERE task_identifier = $1 ORDER BY created_at ASC"
            )
            .bind(&identifier)
            .fetch_all(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&notes).unwrap_or_default());
            } else if notes.is_empty() {
                println!("No handoff notes for {}", identifier);
            } else {
                for note in &notes {
                    println!("[{}] {} -> {}: {} - {}",
                        note.note_type,
                        note.from_agent_id,
                        note.to_agent_id.as_deref().unwrap_or("any"),
                        note.summary,
                        note.created_at,
                    );
                }
            }
        }
        TaskAction::Learnings { identifier } => {
            let learnings = sqlx::query_as::<_, agent::TaskLearning>(
                "SELECT * FROM task_learnings WHERE task_identifier = $1 ORDER BY created_at DESC"
            )
            .bind(&identifier)
            .fetch_all(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&learnings).unwrap_or_default());
            } else if learnings.is_empty() {
                println!("No learnings for {}", identifier);
            } else {
                for l in &learnings {
                    println!("[{}] {} by {}: {}",
                        l.outcome, l.task_identifier, l.agent_id, l.approach_summary,
                    );
                    if let Some(ref insight) = l.key_insight {
                        println!("  Insight: {}", insight);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_pipeline(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    action: PipelineAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::commands::pipelines::{Pipeline, PipelineRun};

    match action {
        PipelineAction::List { project } => {
            let pipelines = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE project_id = $1 ORDER BY created_at DESC",
            )
            .bind(project)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&pipelines)?);
            } else {
                if pipelines.is_empty() {
                    println!("No pipelines found for project {}", project);
                } else {
                    for p in &pipelines {
                        let stages: Vec<serde_json::Value> =
                            serde_json::from_str(&p.stages).unwrap_or_default();
                        println!(
                            "{} | {} | {} stages | {} runs | {}",
                            p.id,
                            p.name,
                            stages.len(),
                            p.total_runs,
                            if p.enabled { "enabled" } else { "disabled" }
                        );
                    }
                }
            }
        }
        PipelineAction::Create {
            project,
            name,
            description,
            stages,
        } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            // Validate stages JSON
            let _: Vec<serde_json::Value> = serde_json::from_str(&stages)
                .map_err(|e| format!("Invalid stages JSON: {}", e))?;

            let jb = crate::db::compat::jsonb_cast(backend);
            let id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipelines (project_id, name, description, stages, created_at, updated_at) VALUES ($1, $2, $3, $4{jb}, $5, $6) RETURNING id"
            ))
            .bind(project)
            .bind(&name)
            .bind(&description)
            .bind(&stages)
            .bind(&now)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            let pipeline = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(id)
            .fetch_one(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&pipeline)?);
            } else {
                println!("Created pipeline: {} (id: {})", pipeline.name, pipeline.id);
            }
            notify_change();
        }
        PipelineAction::Trigger { id, issue, context } => {
            let pipeline = sqlx::query_as::<_, Pipeline>(
                "SELECT * FROM pipelines WHERE id = $1",
            )
            .bind(id)
            .fetch_one(pool)
            .await?;

            if !pipeline.enabled {
                eprintln!("Pipeline is disabled");
                std::process::exit(1);
            }

            let stages: Vec<serde_json::Value> =
                serde_json::from_str(&pipeline.stages).unwrap_or_default();
            if stages.is_empty() {
                eprintln!("Pipeline has no stages");
                std::process::exit(1);
            }

            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let initial_context = context.unwrap_or_else(|| "{}".to_string());

            let trigger_title = if let Some(tid) = issue {
                let t: Option<String> = sqlx::query_scalar("SELECT title FROM issues WHERE id = $1")
                    .bind(tid).fetch_optional(pool).await?;
                t.unwrap_or_default()
            } else { String::new() };
            let trigger_description = if let Some(tid) = issue {
                let d: Option<Option<String>> = sqlx::query_scalar("SELECT description FROM issues WHERE id = $1")
                    .bind(tid).fetch_optional(pool).await?;
                d.flatten().unwrap_or_default()
            } else { String::new() };

            let jb = crate::db::compat::jsonb_cast(backend);
            let run_id: i64 = sqlx::query_scalar(&format!(
                "INSERT INTO pipeline_runs (pipeline_id, trigger_issue_id, status, current_stage, stage_tasks, context, started_at) VALUES ($1, $2, 'running', 0, '[]'{jb}, $3{jb}, $4) RETURNING id"
            ))
            .bind(id)
            .bind(issue)
            .bind(&initial_context)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            // Create first stage task using create_stage_task logic inline
            let stage = &stages[0];
            let stage_name = stage["name"].as_str().unwrap_or("Stage");
            let task_type = stage["task_type"].as_str().unwrap_or("implementation");
            let skills: Vec<String> = stage["required_skills"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let complexity = stage["max_complexity"].as_str().unwrap_or("medium");
            let timeout = stage["timeout_minutes"].as_i64().unwrap_or(30);
            let sc: Vec<String> = stage["success_criteria"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let title_template = stage["title_template"].as_str()
                .unwrap_or("{{pipeline.name}}: {{stage.name}}");
            let obj_template = stage["objective_template"].as_str()
                .unwrap_or("Execute stage: {{stage.name}}");

            let title = title_template
                .replace("{{pipeline.name}}", &pipeline.name)
                .replace("{{stage.name}}", stage_name)
                .replace("{{trigger.title}}", &trigger_title);
            let objective = obj_template
                .replace("{{pipeline.name}}", &pipeline.name)
                .replace("{{stage.name}}", stage_name)
                .replace("{{trigger.title}}", &trigger_title)
                .replace("{{trigger.description}}", &trigger_description);

            let (counter, prefix): (i64, String) = sqlx::query_as(
                "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix",
            ).bind(pipeline.project_id).fetch_one(pool).await?;
            let task_identifier = format!("{}-{}", prefix, counter);

            let sid: i64 = sqlx::query_scalar(
                "SELECT id FROM statuses WHERE project_id = $1 AND category = 'unstarted' ORDER BY position ASC LIMIT 1",
            ).bind(pipeline.project_id).fetch_one(pool).await?;

            let max_pos: Option<f64> = sqlx::query_scalar(
                "SELECT MAX(position) FROM issues WHERE project_id = $1 AND status_id = $2",
            ).bind(pipeline.project_id).bind(sid).fetch_one(pool).await?;
            let position = max_pos.unwrap_or(-1.0) + 1.0;

            let desc = format!("Pipeline run #{} - Stage 1 ({})\n\n{}", run_id, stage_name, objective);

            let issue_id: i64 = sqlx::query_scalar(
                "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'medium', $6, $7, $8) RETURNING id",
            ).bind(pipeline.project_id).bind(&task_identifier).bind(&title).bind(&desc).bind(sid).bind(position).bind(&now).bind(&now)
            .fetch_one(pool).await?;

            let ctx = serde_json::json!({
                "files": [], "related_tasks": [], "prior_attempts": [],
                "pipeline": { "run_id": run_id, "stage_index": 0, "pipeline_name": &pipeline.name, "stage_name": stage_name, "accumulated_context": serde_json::from_str::<serde_json::Value>(&initial_context).unwrap_or_default() }
            });
            let skills_json = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());
            let sc_json = serde_json::to_string(&sc).unwrap_or_else(|_| "[]".to_string());
            let ctx_json = ctx.to_string();

            sqlx::query(&format!(
                "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes, attempt_count) VALUES ($1, $2, 'queued', $3, $4{jb}, '[]'{jb}, $5{jb}, $6{jb}, $7, $8, 0)"
            )).bind(issue_id).bind(task_type).bind(&objective).bind(&ctx_json).bind(&sc_json).bind(&skills_json).bind(complexity).bind(timeout)
            .execute(pool).await?;

            let stage_tasks = serde_json::json!([{ "stage_index": 0, "task_identifier": task_identifier, "status": "queued" }]);
            sqlx::query(&format!(
                "UPDATE pipeline_runs SET stage_tasks = $1{jb} WHERE id = $2"
            )).bind(stage_tasks.to_string()).bind(run_id).execute(pool).await?;

            sqlx::query("UPDATE pipelines SET total_runs = total_runs + 1, updated_at = $1 WHERE id = $2")
                .bind(&now).bind(id).execute(pool).await?;

            if json {
                let run = sqlx::query_as::<_, PipelineRun>("SELECT * FROM pipeline_runs WHERE id = $1")
                    .bind(run_id).fetch_one(pool).await?;
                println!("{}", serde_json::to_string_pretty(&run)?);
            } else {
                println!("Triggered pipeline '{}' (run #{})", pipeline.name, run_id);
                println!("First stage task: {}", task_identifier);
            }
            notify_change();
        }
        PipelineAction::Status { run_id } => {
            let run = sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE id = $1",
            )
            .bind(run_id)
            .fetch_one(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&run)?);
            } else {
                let pipeline = sqlx::query_as::<_, Pipeline>(
                    "SELECT * FROM pipelines WHERE id = $1",
                )
                .bind(run.pipeline_id)
                .fetch_one(pool)
                .await?;
                let stages: Vec<serde_json::Value> =
                    serde_json::from_str(&pipeline.stages).unwrap_or_default();
                let stage_tasks: Vec<serde_json::Value> =
                    serde_json::from_str(&run.stage_tasks).unwrap_or_default();

                println!("Pipeline: {} (run #{})", pipeline.name, run.id);
                println!("Status: {} | Stage: {}/{}", run.status, run.current_stage + 1, stages.len());
                for st in &stage_tasks {
                    println!(
                        "  Stage {}: {} [{}]",
                        st["stage_index"].as_i64().unwrap_or(0) + 1,
                        st["task_identifier"].as_str().unwrap_or("?"),
                        st["status"].as_str().unwrap_or("?")
                    );
                }
                if let Some(err) = &run.error_message {
                    println!("Error: {}", err);
                }
            }
        }
        PipelineAction::Runs { id } => {
            let runs = sqlx::query_as::<_, PipelineRun>(
                "SELECT * FROM pipeline_runs WHERE pipeline_id = $1 ORDER BY started_at DESC",
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&runs)?);
            } else {
                for r in &runs {
                    println!(
                        "Run #{} | {} | Stage {} | {}",
                        r.id, r.status, r.current_stage + 1, r.started_at
                    );
                }
            }
        }
        PipelineAction::Advance { run_id } => {
            let run = crate::commands::pipelines::advance_pipeline_internal(pool, backend, run_id)
                .await
                .map_err(|e| format!("{}", e))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&run)?);
            } else {
                println!("Pipeline run #{} advanced to stage {}", run.id, run.current_stage + 1);
                println!("Status: {}", run.status);
            }
            notify_change();
        }
        PipelineAction::Cancel { run_id } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            sqlx::query("UPDATE pipeline_runs SET status = 'cancelled', completed_at = $1 WHERE id = $2")
                .bind(&now).bind(run_id).execute(pool).await?;
            println!("Pipeline run #{} cancelled", run_id);
            notify_change();
        }
        PipelineAction::Delete { id } => {
            sqlx::query("DELETE FROM pipelines WHERE id = $1")
                .bind(id).execute(pool).await?;
            println!("Pipeline {} deleted", id);
            notify_change();
        }
    }
    Ok(())
}

async fn handle_metrics(
    pool: &AnyPool,
    backend: &crate::db::DbBackend,
    project: Option<i64>,
    agent: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref agent_id) = agent {
        let metrics = crate::orchestration::metrics::agent_metrics(pool, backend, agent_id).await?;
        if json {
            println!("{}", serde_json::json!({"success": true, "data": metrics}));
        } else {
            println!("Agent: {} ({})", metrics.name, metrics.agent_id);
            println!("Status: {} | Active tasks: {}", metrics.status, metrics.current_tasks.len());
            println!("Completed: {} | Failed: {} | Success rate: {:.0}%", metrics.tasks_completed, metrics.tasks_failed, metrics.success_rate * 100.0);
            println!("Avg confidence: {:.2} | Avg time: {:.1}m", metrics.avg_confidence, metrics.avg_completion_time_minutes);
            if !metrics.current_tasks.is_empty() {
                println!("Active: {}", metrics.current_tasks.join(", "));
            }
        }
    } else if let Some(pid) = project {
        let metrics = crate::orchestration::metrics::project_metrics(pool, backend, pid).await?;
        if json {
            println!("{}", serde_json::json!({"success": true, "data": metrics}));
        } else {
            println!("Project {} Metrics:", pid);
            println!("  Total: {} | Completed: {} | Queued: {} | In Progress: {} | Blocked: {} | Validating: {}",
                metrics.total_tasks, metrics.completed, metrics.queued, metrics.in_progress, metrics.blocked, metrics.validating);
            println!("  Failed attempts: {} | Agents online: {}", metrics.failed_attempts, metrics.agents_online);
            if let Some(conf) = metrics.avg_confidence { println!("  Avg confidence: {:.2}", conf); }
            println!("  Completed (24h): {}", metrics.tasks_completed_24h);
            println!("  Types: {}", metrics.task_type_breakdown);
        }
    } else {
        eprintln!("Specify --project or --agent");
        std::process::exit(1);
    }
    Ok(())
}

async fn handle_export(
    pool: &AnyPool,
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
    pool: &AnyPool,
    file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&file)?;
    let data: ExportData = serde_json::from_str(&content)?;

    // Import in order to satisfy foreign keys
    for m in &data.members {
        sqlx::query("INSERT INTO members (id, name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, display_name = EXCLUDED.display_name, email = EXCLUDED.email, avatar_color = EXCLUDED.avatar_color, created_at = EXCLUDED.created_at")
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
        sqlx::query("INSERT INTO projects (id, name, description, icon, status, prefix, issue_counter, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description, icon = EXCLUDED.icon, status = EXCLUDED.status, prefix = EXCLUDED.prefix, issue_counter = EXCLUDED.issue_counter, created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at")
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
        sqlx::query("INSERT INTO statuses (id, project_id, name, category, color, icon, position) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET project_id = EXCLUDED.project_id, name = EXCLUDED.name, category = EXCLUDED.category, color = EXCLUDED.color, icon = EXCLUDED.icon, position = EXCLUDED.position")
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
        sqlx::query("INSERT INTO labels (id, project_id, name, color) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET project_id = EXCLUDED.project_id, name = EXCLUDED.name, color = EXCLUDED.color")
            .bind(l.id)
            .bind(l.project_id)
            .bind(&l.name)
            .bind(&l.color)
            .execute(pool)
            .await?;
    }
    for i in &data.issues {
        sqlx::query("INSERT INTO issues (id, project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, estimate, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) ON CONFLICT (id) DO UPDATE SET project_id = EXCLUDED.project_id, identifier = EXCLUDED.identifier, title = EXCLUDED.title, description = EXCLUDED.description, status_id = EXCLUDED.status_id, priority = EXCLUDED.priority, assignee_id = EXCLUDED.assignee_id, parent_id = EXCLUDED.parent_id, position = EXCLUDED.position, estimate = EXCLUDED.estimate, due_date = EXCLUDED.due_date, created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at")
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
        sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT (issue_id, label_id) DO NOTHING")
            .bind(il.issue_id)
            .bind(il.label_id)
            .execute(pool)
            .await?;
    }
    for r in &data.issue_relations {
        sqlx::query("INSERT INTO issue_relations (id, source_issue_id, target_issue_id, relation_type) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET source_issue_id = EXCLUDED.source_issue_id, target_issue_id = EXCLUDED.target_issue_id, relation_type = EXCLUDED.relation_type")
            .bind(r.id)
            .bind(r.source_issue_id)
            .bind(r.target_issue_id)
            .bind(&r.relation_type)
            .execute(pool)
            .await?;
    }
    for t in &data.issue_templates {
        sqlx::query("INSERT INTO issue_templates (id, project_id, name, description_template, default_status_id, default_priority, default_label_ids, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO UPDATE SET project_id = EXCLUDED.project_id, name = EXCLUDED.name, description_template = EXCLUDED.description_template, default_status_id = EXCLUDED.default_status_id, default_priority = EXCLUDED.default_priority, default_label_ids = EXCLUDED.default_label_ids, created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at")
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
    notify_change();
    Ok(())
}

async fn handle_code(
    pool: &AnyPool,
    action: CodeAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CodeAction::HeatMap { project, limit } => {
            let entries = crate::commands::code_analysis::get_file_heat_map_async(pool, project, limit)
                .await
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                println!("{:<60} {:>6} {:>6} {}", "FILE", "ISSUES", "BUGS", "LAST");
                println!("{}", "-".repeat(90));
                for e in &entries {
                    println!("{:<60} {:>6} {:>6} {}", e.file_path, e.issue_count, e.bug_count, e.last_issue_at);
                }
            }
        }
        CodeAction::DirHeatMap { project, depth } => {
            let entries = crate::commands::code_analysis::get_directory_heat_map_async(pool, project, depth)
                .await
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                println!("{:<50} {:>6} {:>6}", "DIRECTORY", "ISSUES", "FILES");
                println!("{}", "-".repeat(66));
                for e in &entries {
                    println!("{:<50} {:>6} {:>6}", e.directory, e.issue_count, e.file_count);
                }
            }
        }
        CodeAction::Link { identifier, file_path, link_type } => {
            let issue = resolve_issue(pool, &identifier).await?;
            sqlx::query(
                "INSERT INTO issue_file_links (issue_id, file_path, link_type) VALUES ($1, $2, $3)",
            )
            .bind(issue.id)
            .bind(&file_path)
            .bind(&link_type)
            .execute(pool)
            .await?;
            println!("Linked {} to {} ({})", file_path, identifier, link_type);
            notify_change();
        }
        CodeAction::Unlink { identifier, file_path } => {
            let issue = resolve_issue(pool, &identifier).await?;
            sqlx::query("DELETE FROM issue_file_links WHERE issue_id = $1 AND file_path = $2")
                .bind(issue.id)
                .bind(&file_path)
                .execute(pool)
                .await?;
            println!("Unlinked {} from {}", file_path, identifier);
            notify_change();
        }
        CodeAction::Files { identifier } => {
            let issue = resolve_issue(pool, &identifier).await?;
            let links = sqlx::query_as::<_, crate::models::IssueFileLink>(
                "SELECT * FROM issue_file_links WHERE issue_id = $1 ORDER BY created_at ASC",
            )
            .bind(issue.id)
            .fetch_all(pool)
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&links)?);
            } else {
                for link in &links {
                    println!("{} [{}] {}", link.file_path, link.link_type, link.created_at);
                }
            }
        }
        CodeAction::Issues { file_path, project } => {
            let issues = crate::commands::code_analysis::get_issues_for_file_async(pool, &file_path, project)
                .await
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&issues)?);
            } else {
                for i in &issues {
                    println!("{} | {} | {}", i.identifier, i.title, i.priority);
// ---- Cost Tracking ----

use crate::commands::costs::{TaskCost, CostBudget};
use crate::commands::sla::{SlaPolicy, SlaEvent};

async fn handle_costs(
    pool: &AnyPool,
    action: CostAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CostAction::Summary { project } => {
            let total_cost: Option<f64> = sqlx::query_scalar(
                "SELECT SUM(tc.amount) FROM task_costs tc JOIN issues i ON tc.task_identifier = i.identifier WHERE i.project_id = $1 AND tc.unit = 'dollars'"
            )
            .bind(project)
            .fetch_one(pool)
            .await?;

            let budgets = sqlx::query_as::<_, CostBudget>(
                "SELECT * FROM cost_budgets WHERE project_id = $1"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            if json {
                let data = serde_json::json!({
                    "project_id": project,
                    "total_cost": total_cost.unwrap_or(0.0),
                    "budgets": budgets,
                });
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                println!("Project {} Cost Summary", project);
                println!("  Total spend: ${:.2}", total_cost.unwrap_or(0.0));
                for b in &budgets {
                    let pct = if b.amount > 0.0 { (b.spent / b.amount) * 100.0 } else { 0.0 };
                    println!("  Budget ({} {}): ${:.2} / ${:.2} ({:.0}%)", b.budget_type, b.unit, b.spent, b.amount, pct);
                }
            }
        }
        CostAction::Task { identifier } => {
            let costs = sqlx::query_as::<_, TaskCost>(
                "SELECT * FROM task_costs WHERE task_identifier = $1 ORDER BY recorded_at ASC"
            )
            .bind(&identifier)
            .fetch_all(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&costs)?);
            } else {
                let mut total = 0.0f64;
                for c in &costs {
                    println!("  {} | {} {} | {} | {}", c.cost_type, c.amount, c.unit, c.agent_id, c.recorded_at);
                    if c.unit == "dollars" { total += c.amount; }
                }
                println!("Total: ${:.2}", total);
            }
        }
        CostAction::Record { task, agent, cost_type, amount, unit, description } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO task_costs (task_identifier, agent_id, cost_type, amount, unit, description, recorded_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
            )
            .bind(&task)
            .bind(&agent)
            .bind(&cost_type)
            .bind(amount)
            .bind(&unit)
            .bind(&description)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            if json {
                let cost = sqlx::query_as::<_, TaskCost>("SELECT * FROM task_costs WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&cost)?);
            } else {
                println!("Recorded cost #{}: {} {} ({}) for task {} by agent {}", id, amount, unit, cost_type, task, agent);
            }
            notify_change();
        }
        CostAction::Budget { project, budget_type, amount, unit } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let u = unit.unwrap_or_else(|| "dollars".to_string());
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO cost_budgets (project_id, budget_type, amount, unit, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"
            )
            .bind(project)
            .bind(&budget_type)
            .bind(amount)
            .bind(&u)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            if json {
                let budget = sqlx::query_as::<_, CostBudget>("SELECT * FROM cost_budgets WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&budget)?);
            } else {
                println!("Set {} budget: {} {} for project {}", budget_type, amount, u, project);
            }
            notify_change();
        }
        CostAction::Check { project } => {
            let budgets = sqlx::query_as::<_, CostBudget>(
                "SELECT * FROM cost_budgets WHERE project_id = $1"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            let alerts: Vec<_> = budgets.iter().filter_map(|b| {
                let pct = if b.amount > 0.0 { b.spent / b.amount } else { 0.0 };
                let threshold = b.alert_threshold.unwrap_or(0.8);
                if pct >= threshold {
                    Some(serde_json::json!({
                        "budget_id": b.id,
                        "budget_type": b.budget_type,
                        "amount": b.amount,
                        "spent": b.spent,
                        "percentage": pct,
                        "alert": true,
                    }))
                } else {
                    None
                }
            }).collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&alerts)?);
            } else {
                if alerts.is_empty() {
                    println!("All budgets within limits.");
                } else {
                    for a in &alerts {
                        println!("ALERT: {} budget at {:.0}% (${:.2} / ${:.2})",
                            a["budget_type"].as_str().unwrap_or(""),
                            a["percentage"].as_f64().unwrap_or(0.0) * 100.0,
                            a["spent"].as_f64().unwrap_or(0.0),
                            a["amount"].as_f64().unwrap_or(0.0));
                    }
                }
            }
        }
    }
    Ok(())
}

// ---- SLA Engine ----

async fn handle_sla(
    pool: &AnyPool,
    action: SlaAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        SlaAction::Check { project } => {
            // Reuse the same logic from commands/sla.rs but directly
            let policies = sqlx::query_as::<_, SlaPolicy>(
                "SELECT * FROM sla_policies WHERE project_id = $1 AND enabled = 1"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            let issues_list = sqlx::query_as::<_, Issue>(
                "SELECT i.* FROM issues i JOIN statuses s ON i.status_id = s.id WHERE i.project_id = $1 AND s.category = 'started'"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            let now = chrono::Utc::now();
            let mut results = Vec::new();

            for issue in &issues_list {
                for policy in &policies {
                    if let Some(ref pf) = policy.priority_filter {
                        if !pf.is_empty() && pf != &issue.priority {
                            continue;
                        }
                    }
                    let elapsed_minutes = {
                        let start = chrono::NaiveDateTime::parse_from_str(&issue.created_at, "%Y-%m-%d %H:%M:%SZ")
                            .unwrap_or_else(|_| now.naive_utc());
                        now.naive_utc().signed_duration_since(start).num_minutes() as f64
                    };
                    let remaining = policy.breach_minutes as f64 - elapsed_minutes;
                    let status = if elapsed_minutes >= policy.breach_minutes as f64 {
                        "breached"
                    } else if elapsed_minutes >= (policy.breach_minutes - policy.warning_minutes) as f64 {
                        "warning"
                    } else {
                        "ok"
                    };
                    results.push(serde_json::json!({
                        "issue": issue.identifier,
                        "policy": policy.name,
                        "status": status,
                        "elapsed_minutes": elapsed_minutes,
                        "remaining_minutes": remaining.max(0.0),
                    }));
                }
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for r in &results {
                    println!("{} | {} | {} | {:.0}m elapsed | {:.0}m remaining",
                        r["issue"].as_str().unwrap_or(""),
                        r["policy"].as_str().unwrap_or(""),
                        r["status"].as_str().unwrap_or(""),
                        r["elapsed_minutes"].as_f64().unwrap_or(0.0),
                        r["remaining_minutes"].as_f64().unwrap_or(0.0));
                }
                if results.is_empty() {
                    println!("No active SLA tracking.");
                }
            }
        }
        SlaAction::Enforce { project } => {
            let events = crate::commands::sla::enforce_sla_async(pool, project).await
                .map_err(|e| -> Box<dyn std::error::Error> { Box::from(e) })?;
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else {
                if events.is_empty() {
                    println!("No SLA violations to enforce.");
                } else {
                    for e in &events {
                        println!("[{}] {}", e.event_type, e.message);
                    }
                }
            }
            notify_change();
        }
        SlaAction::Create { project, name, target_type, priority, warning, breach, escalation } => {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
            let esc = escalation.unwrap_or_else(|| "{}".to_string());
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO sla_policies (project_id, name, target_type, priority_filter, warning_minutes, breach_minutes, escalation_action, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
            )
            .bind(project)
            .bind(&name)
            .bind(&target_type)
            .bind(&priority)
            .bind(warning)
            .bind(breach)
            .bind(&esc)
            .bind(&now)
            .fetch_one(pool)
            .await?;

            let policy = sqlx::query_as::<_, SlaPolicy>("SELECT * FROM sla_policies WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&policy)?);
            } else {
                println!("Created SLA policy '{}' (id: {}): {} warning at {}m, breach at {}m",
                    name, id, target_type, warning, breach);
            }
            notify_change();
        }
        SlaAction::List { project } => {
            let policies = sqlx::query_as::<_, SlaPolicy>(
                "SELECT * FROM sla_policies WHERE project_id = $1 ORDER BY created_at DESC"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&policies)?);
            } else {
                for p in &policies {
                    let enabled = if p.enabled == 1 { "enabled" } else { "disabled" };
                    println!("{} | {} | {} | warn:{}m breach:{}m | {} | {}",
                        p.id, p.name, p.target_type, p.warning_minutes, p.breach_minutes,
                        p.priority_filter.as_deref().unwrap_or("all"), enabled);
                }
            }
        }
        SlaAction::Delete { id } => {
            sqlx::query("DELETE FROM sla_policies WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            println!("Deleted SLA policy {}", id);
            notify_change();
        }
        SlaAction::Dashboard { project } => {
            let policies = sqlx::query_as::<_, SlaPolicy>(
                "SELECT * FROM sla_policies WHERE project_id = $1"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            let events = sqlx::query_as::<_, SlaEvent>(
                "SELECT se.* FROM sla_events se JOIN sla_policies sp ON se.sla_policy_id = sp.id WHERE sp.project_id = $1 ORDER BY se.created_at DESC LIMIT 20"
            )
            .bind(project)
            .fetch_all(pool)
            .await?;

            if json {
                let data = serde_json::json!({
                    "policies": policies,
                    "recent_events": events,
                });
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                println!("SLA Dashboard for project {}", project);
                println!("  Policies: {}", policies.len());
                println!("  Recent events:");
                for e in &events {
                    println!("    [{}] {} (issue #{})", e.event_type, e.message, e.issue_id);
                }
            }
        }
    }
    Ok(())
}
