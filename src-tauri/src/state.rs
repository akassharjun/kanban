use sqlx::AnyPool;
use crate::db::DbBackend;

pub struct AppState {
    pub pool: AnyPool,
    pub backend: DbBackend,
    pub rt: tokio::runtime::Runtime,
}
