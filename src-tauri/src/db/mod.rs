use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_db() -> Result<PgPool, Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://kanban:kanban@localhost:5433/kanban".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
