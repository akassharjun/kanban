use std::sync::{Arc, Mutex};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};

use kanban_core::Workspace;

use crate::convert::ProjectOut;
use crate::error::to_mcp;

#[derive(Clone)]
pub struct KanbanServer {
    workspace: Arc<Mutex<Workspace>>,
    // Read by the `#[tool_handler]`-generated `ServerHandler` impl; dead-code
    // analysis can't see the read through the generated trait code.
    #[allow(dead_code)]
    tool_router: ToolRouter<KanbanServer>,
}

impl KanbanServer {
    /// Open the default workspace (`~/.kanban/data.db` or `$KANBAN_DB`).
    ///
    /// # Errors
    /// Returns an error if the workspace cannot be opened.
    pub fn from_default() -> anyhow::Result<Self> {
        let ws = Workspace::open_default()?;
        Ok(Self::with_workspace(ws))
    }

    /// Build a server around an already-open workspace (used by tests).
    #[must_use]
    pub fn with_workspace(ws: Workspace) -> Self {
        Self {
            workspace: Arc::new(Mutex::new(ws)),
            tool_router: Self::tool_router(),
        }
    }

    /// Run a sync closure against the workspace on a blocking thread; the mutex
    /// guard never crosses `.await`. Core errors are mapped to MCP errors.
    async fn blocking<T, F>(&self, f: F) -> Result<T, McpError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Workspace) -> Result<T, kanban_core::Error> + Send + 'static,
    {
        let ws = Arc::clone(&self.workspace);
        tokio::task::spawn_blocking(move || {
            let mut guard = ws
                .lock()
                .map_err(|_| kanban_core::Error::Conflict("workspace mutex poisoned".into()))?;
            f(&mut guard)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join error: {e}"), None))?
        .map_err(to_mcp)
    }
}

/// Serialize a value to a pretty-JSON text content block.
pub(crate) fn json_content<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[tool_router]
impl KanbanServer {
    #[tool(description = "List all projects (prefix, name, description).")]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        let projects = self.blocking(|ws| ws.query_projects()).await?;
        let out: Vec<ProjectOut> = projects.into_iter().map(ProjectOut::from).collect();
        json_content(&out)
    }
}

#[tool_handler]
impl ServerHandler for KanbanServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "Read and manage a local kanban board. Address projects by prefix (e.g. AUTH) \
                 and issues by key (e.g. AUTH-12). Statuses and labels are referenced by name."
                    .to_string(),
            )
    }
}
