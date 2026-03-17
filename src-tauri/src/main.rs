// Windows subsystem: hide console for GUI mode.
// We parse args first, so CLI/MCP subcommands work with console attached.
// On non-Windows platforms, this attribute does nothing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(name = "kanban", about = "Kanban - Desktop Project Management")]
struct KanbanApp {
    #[command(subcommand)]
    command: Option<KanbanCommand>,

    /// Database URL (defaults to SQLite at ~/.kanban/data.db)
    #[arg(long, global = true, env = "DATABASE_URL")]
    database_url: Option<String>,
}

#[derive(clap::Subcommand)]
enum KanbanCommand {
    /// Launch the GUI application (default)
    App,
    /// Run CLI commands
    Cli {
        #[command(subcommand)]
        action: kanban_lib::cli::Commands,

        /// Output as JSON
        #[arg(long, global = true)]
        json: bool,
    },
    /// Run MCP server (JSON-RPC over stdio)
    Mcp,
}

fn main() {
    let app = KanbanApp::parse();

    match app.command {
        None | Some(KanbanCommand::App) => {
            kanban_lib::run_gui(app.database_url);
        }
        Some(KanbanCommand::Cli { action, json }) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let (pool, backend) = kanban_lib::db::init_db(app.database_url.as_deref())
                    .await
                    .expect("Failed to connect to database");
                kanban_lib::cli::run(&pool, &backend, action, json)
                    .await
                    .expect("CLI command failed");
            });
        }
        Some(KanbanCommand::Mcp) => {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let (pool, backend) = kanban_lib::db::init_db(app.database_url.as_deref())
                    .await
                    .expect("Failed to connect to database");
                kanban_lib::mcp::run(&pool, &backend)
                    .await
                    .expect("MCP server failed");
            });
        }
    }
}
