use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

use kanban_mcp::server::KanbanServer;

#[tokio::main]
async fn main() -> Result<()> {
    // stdout is the JSON-RPC channel — all logging goes to stderr.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let server = KanbanServer::from_default()?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
