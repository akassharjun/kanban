use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub rt: tokio::runtime::Runtime,
}
