use sqlx::SqlitePool;

pub struct AppState {
    pub pool: SqlitePool,
    pub rt: tokio::runtime::Runtime,
}
