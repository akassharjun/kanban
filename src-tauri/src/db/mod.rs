pub mod compat;
pub mod watcher;

use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    Sqlite,
    Postgres,
}

const SQLITE_SCHEMA: &str = include_str!("../../migrations_sqlite/20260317000000_initial_sqlite_schema.sql");

pub async fn init_db(database_url: Option<&str>) -> Result<(AnyPool, DbBackend), Box<dyn std::error::Error>> {
    sqlx::any::install_default_drivers();

    // Check for explicit DATABASE_URL env var (not from .env file).
    // Load .env only if database_url was provided explicitly (e.g. for Postgres mode).
    let env_url = std::env::var("DATABASE_URL").ok();
    let url = database_url.or(env_url.as_deref());

    let (effective_url, backend) = match url {
        Some(u) if u.starts_with("postgres://") || u.starts_with("postgresql://") => {
            (u.to_string(), DbBackend::Postgres)
        }
        Some(u) if u.starts_with("sqlite://") => {
            let path = u.strip_prefix("sqlite://").unwrap_or("");
            let path = path.split('?').next().unwrap_or(path);
            if !path.is_empty() && path != ":memory:" {
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            (u.to_string(), DbBackend::Sqlite)
        }
        _ => {
            let data_dir = dirs::home_dir()
                .ok_or("Cannot determine home directory")?
                .join(".kanban");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("data.db");
            let url = format!("sqlite://{}?mode=rwc", db_path.display());
            (url, DbBackend::Sqlite)
        }
    };

    let pool = AnyPoolOptions::new()
        .max_connections(if backend == DbBackend::Sqlite { 1 } else { 10 })
        .connect(&effective_url)
        .await?;

    match backend {
        DbBackend::Sqlite => {
            sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
            sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;
            sqlx::query("PRAGMA busy_timeout=5000").execute(&pool).await?;

            // For existing databases: add columns that may be missing.
            let backfill = [
                "ALTER TABLE projects ADD COLUMN deleted_at TEXT",
                "ALTER TABLE projects ADD COLUMN path TEXT",
                "ALTER TABLE agents ADD COLUMN member_id INTEGER REFERENCES members(id) ON DELETE SET NULL",
                "ALTER TABLE agents ADD COLUMN last_activity_at TEXT",
                "ALTER TABLE agents ADD COLUMN worktree_path TEXT",
                "ALTER TABLE agents ADD COLUMN agent_type TEXT",
                "ALTER TABLE activity_log ADD COLUMN actor_id INTEGER REFERENCES members(id)",
                "ALTER TABLE activity_log ADD COLUMN actor_type TEXT DEFAULT 'user'",
            ];
            for stmt in &backfill {
                let _ = sqlx::query(stmt).execute(&pool).await;
            }

            // Run schema as raw SQL statements.
            // Strip comments first, then split on semicolons.
            let clean_sql: String = SQLITE_SCHEMA
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n");
            for stmt in clean_sql.split(';') {
                let stmt = stmt.trim();
                if !stmt.is_empty() {
                    let _ = sqlx::query(stmt).execute(&pool).await;
                }
            }
        }
        DbBackend::Postgres => {
            // Use runtime migration loading to avoid compile-time type registration
            // that interferes with AnyPool's SQLite type resolution.
            let migrator = sqlx::migrate::Migrator::new(
                std::path::Path::new("./migrations")
            ).await?;
            migrator.run(&pool).await?;
        }
    }

    seed_defaults(&pool).await?;

    Ok((pool, backend))
}

/// Create default member on first run (idempotent).
async fn seed_defaults(pool: &AnyPool) -> Result<(), Box<dyn std::error::Error>> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM members")
        .fetch_one(pool)
        .await?;
    if count.0 == 0 {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query(
            "INSERT INTO members (name, display_name, avatar_color, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind("You")
        .bind("You")
        .bind("#6366f1")
        .bind(&now)
        .execute(pool)
        .await?;
    }
    Ok(())
}
