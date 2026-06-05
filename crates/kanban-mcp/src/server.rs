use std::sync::{Arc, Mutex};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
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

    /// Run a read-only sync closure against the workspace on a blocking thread.
    /// The mutex guard never crosses `.await`. Core errors map to MCP errors.
    async fn blocking_read<T, F>(&self, f: F) -> Result<T, McpError>
    where
        T: Send + 'static,
        F: FnOnce(&Workspace) -> Result<T, kanban_core::Error> + Send + 'static,
    {
        let ws = Arc::clone(&self.workspace);
        tokio::task::spawn_blocking(move || {
            let guard = ws
                .lock()
                .map_err(|_| kanban_core::Error::Conflict("workspace mutex poisoned".into()))?;
            f(&guard)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join error: {e}"), None))?
        .map_err(to_mcp)
    }

    /// Run a mutating sync closure against the workspace on a blocking thread
    /// (for `apply`/`undo`/`redo`). The mutex guard never crosses `.await`.
    async fn blocking_mut<T, F>(&self, f: F) -> Result<T, McpError>
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

    /// Resolve a project's statuses + run a project-scoped read in one lock.
    /// `f(ws, project_id, &statuses)` yields the read result. An unknown prefix
    /// maps to a not-found error.
    async fn with_project<T, F>(&self, prefix: &str, f: F) -> Result<T, McpError>
    where
        T: Send + 'static,
        F: FnOnce(
                &Workspace,
                uuid::Uuid,
                &[kanban_core::types::Status],
            ) -> Result<T, kanban_core::Error>
            + Send
            + 'static,
    {
        let owned = prefix.to_string();
        let for_err = prefix.to_string();
        let out = self
            .blocking_read(move |ws| {
                let Some(p) = ws.query_project_by_prefix(&owned)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(p.id)?;
                f(ws, p.id, &statuses).map(Some)
            })
            .await?;
        out.ok_or_else(|| crate::error::not_found("project", &for_err))
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
        let projects = self.blocking_read(Workspace::query_projects).await?;
        let out: Vec<ProjectOut> = projects.into_iter().map(ProjectOut::from).collect();
        json_content(&out)
    }

    #[tool(
        description = "List the status columns of a project, in board order. `project` is the prefix, e.g. AUTH."
    )]
    async fn list_statuses(
        &self,
        Parameters(args): Parameters<crate::inputs::ProjectRef>,
    ) -> Result<CallToolResult, McpError> {
        let statuses = self
            .with_project(&args.project, |_ws, _id, statuses| Ok(statuses.to_vec()))
            .await?;
        let out: Vec<crate::convert::StatusOut> = statuses
            .into_iter()
            .map(crate::convert::StatusOut::from)
            .collect();
        json_content(&out)
    }

    #[tool(description = "List the labels of a project. `project` is the prefix, e.g. AUTH.")]
    async fn list_labels(
        &self,
        Parameters(args): Parameters<crate::inputs::ProjectRef>,
    ) -> Result<CallToolResult, McpError> {
        let labels: Vec<kanban_core::types::Label> = self
            .with_project(&args.project, |ws, id, _statuses| {
                ws.query_labels_for_project(id)
            })
            .await?;
        let out: Vec<crate::convert::LabelOut> = labels
            .into_iter()
            .map(crate::convert::LabelOut::from)
            .collect();
        json_content(&out)
    }

    #[tool(
        description = "List issues in a project. Optionally filter by status name, priority, or a search term."
    )]
    async fn list_issues(
        &self,
        Parameters(args): Parameters<crate::inputs::ListIssues>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::query::IssueFilter;
        use kanban_core::types::Priority;

        // phase 1: resolve project id + its statuses
        let prefix = args.project.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(p) = ws.query_project_by_prefix(&prefix)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(p.id)?;
                Ok(Some((p.id, statuses)))
            })
            .await?;
        let (project_id, statuses) =
            resolved.ok_or_else(|| crate::error::not_found("project", &args.project))?;

        // phase 2: build filter (name/priority resolution -> McpError here)
        let mut filter = IssueFilter::for_project(project_id);
        if let Some(name) = &args.status {
            let s = statuses.iter().find(|s| &s.name == name).ok_or_else(|| {
                crate::error::unknown_name(
                    "status",
                    name,
                    &args.project,
                    &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                )
            })?;
            filter.status_ids = vec![s.id];
        }
        if let Some(pr) = &args.priority {
            let parsed: Priority = pr.parse().map_err(|_| {
                McpError::invalid_params(
                    "priority must be one of none, low, medium, high, urgent",
                    None,
                )
            })?;
            filter.priorities = vec![parsed];
        }
        filter.search_text = args.search.clone();

        // phase 3: query
        let issues = self
            .blocking_read(move |ws| ws.query_issues(filter))
            .await?;
        let map = crate::convert::status_name_map(&statuses);
        let out: Vec<crate::convert::IssueOut> = issues
            .into_iter()
            .map(|i| {
                let name = map.get(&i.status_id).cloned().unwrap_or_default();
                crate::convert::IssueOut::from_issue(i, &name)
            })
            .collect();
        json_content(&out)
    }

    #[tool(
        description = "Get one issue by key (e.g. AUTH-12), including its markdown description."
    )]
    async fn get_issue(
        &self,
        Parameters(args): Parameters<crate::inputs::IssueKey>,
    ) -> Result<CallToolResult, McpError> {
        let key = args.key.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(issue) = ws.query_issue_by_identifier(&key)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(issue.project_id)?;
                Ok(Some((issue, statuses)))
            })
            .await?;
        let (issue, statuses) =
            resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;
        let name = statuses
            .iter()
            .find(|s| s.id == issue.status_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        json_content(&crate::convert::IssueOut::from_issue(issue, &name))
    }

    #[tool(
        description = "Full-text search issues by title/description. Optionally scope to a project prefix."
    )]
    async fn search_issues(
        &self,
        Parameters(args): Parameters<crate::inputs::SearchIssues>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::query::IssueFilter;

        let project = args.project.clone();
        let query = args.query.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let pid = match &project {
                    Some(prefix) => match ws.query_project_by_prefix(prefix)? {
                        Some(p) => Some(p.id),
                        None => return Ok(None), // unknown project
                    },
                    None => None,
                };
                // collect all statuses so we can name statuses across projects
                let projects = ws.query_projects()?;
                let mut statuses = Vec::new();
                for p in &projects {
                    statuses.extend(ws.query_statuses_for_project(p.id)?);
                }
                let filter = match pid {
                    Some(id) => IssueFilter::for_project(id),
                    None => IssueFilter::default(),
                };
                let issues = ws.search(&query, filter)?;
                Ok(Some((issues, statuses)))
            })
            .await?;
        let (issues, statuses) = resolved.ok_or_else(|| {
            crate::error::not_found("project", args.project.as_deref().unwrap_or(""))
        })?;
        let map = crate::convert::status_name_map(&statuses);
        let out: Vec<crate::convert::IssueOut> = issues
            .into_iter()
            .map(|i| {
                let name = map.get(&i.status_id).cloned().unwrap_or_default();
                crate::convert::IssueOut::from_issue(i, &name)
            })
            .collect();
        json_content(&out)
    }

    #[tool(description = "Create a new project. `prefix` must be 2-8 uppercase letters.")]
    async fn create_project(
        &self,
        Parameters(args): Parameters<crate::inputs::CreateProjectInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{CreateProject, Operation};
        let id = uuid::Uuid::now_v7();
        let prefix = args.prefix.clone();
        self.blocking_mut(move |ws| {
            ws.apply(Operation::CreateProject(CreateProject {
                id,
                name: args.name,
                prefix: args.prefix,
                description: args.description,
                icon: None,
            }))
            .map(|_| ())
        })
        .await?;
        json_content(&serde_json::json!({ "created": prefix }))
    }

    #[tool(
        description = "Create an issue in a project. status defaults to the first column, priority to medium. Returns the created issue including its assigned key."
    )]
    async fn create_issue(
        &self,
        Parameters(args): Parameters<crate::inputs::CreateIssueInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{CreateIssue, Operation};
        use kanban_core::types::Priority;

        // phase 1: resolve project id + statuses
        let prefix = args.project.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(p) = ws.query_project_by_prefix(&prefix)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(p.id)?;
                Ok(Some((p.id, statuses)))
            })
            .await?;
        let (project_id, statuses) =
            resolved.ok_or_else(|| crate::error::not_found("project", &args.project))?;

        // status: named or first column
        let status_id = match &args.status {
            Some(name) => {
                statuses
                    .iter()
                    .find(|s| &s.name == name)
                    .ok_or_else(|| {
                        crate::error::unknown_name(
                            "status",
                            name,
                            &args.project,
                            &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                        )
                    })?
                    .id
            }
            None => statuses
                .first()
                .map(|s| s.id)
                .ok_or_else(|| McpError::internal_error("project has no statuses", None))?,
        };

        let priority: Priority = match &args.priority {
            Some(p) => p.parse().map_err(|_| {
                McpError::invalid_params(
                    "priority must be one of none, low, medium, high, urgent",
                    None,
                )
            })?,
            None => Priority::Medium,
        };
        let due_date = match &args.due_date {
            Some(d) => Some(
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map_err(|_| McpError::invalid_params("due_date must be YYYY-MM-DD", None))?,
            ),
            None => None,
        };

        // phase 2: apply + read back the created issue (so we can return its key)
        let id = uuid::Uuid::now_v7();
        let title = args.title.clone();
        let description = args.description.clone();
        let issue = self
            .blocking_mut(move |ws| {
                ws.apply(Operation::CreateIssue(CreateIssue {
                    id,
                    project_id,
                    title,
                    description,
                    status_id,
                    priority,
                    due_date,
                    label_ids: vec![],
                }))?;
                ws.query_issue_by_id(id)
            })
            .await?;

        let map = crate::convert::status_name_map(&statuses);
        let name = map.get(&issue.status_id).cloned().unwrap_or_default();
        json_content(&crate::convert::IssueOut::from_issue(issue, &name))
    }

    #[tool(
        description = "Update fields of an issue (by key). Provide any of title, description, priority, status, due_date. Returns the updated issue."
    )]
    async fn update_issue(
        &self,
        Parameters(args): Parameters<crate::inputs::UpdateIssueInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{IssueFieldChange, Operation, UpdateIssueField};
        use kanban_core::types::Priority;

        if args.title.is_none()
            && args.description.is_none()
            && args.priority.is_none()
            && args.status.is_none()
            && args.due_date.is_none()
        {
            return Err(McpError::invalid_params(
                "provide at least one field to update (title, description, priority, status, due_date)",
                None,
            ));
        }

        // phase 1: resolve issue id + its project's statuses
        let key = args.key.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(issue) = ws.query_issue_by_identifier(&key)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(issue.project_id)?;
                Ok(Some((issue.id, statuses)))
            })
            .await?;
        let (issue_id, statuses) =
            resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;

        // phase 2: build the list of field changes (McpError resolution here)
        let mut changes: Vec<IssueFieldChange> = Vec::new();
        if let Some(t) = args.title {
            changes.push(IssueFieldChange::Title(t));
        }
        if let Some(d) = args.description {
            changes.push(IssueFieldChange::Description(Some(d)));
        }
        if let Some(p) = &args.priority {
            let parsed: Priority = p.parse().map_err(|_| {
                McpError::invalid_params(
                    "priority must be one of none, low, medium, high, urgent",
                    None,
                )
            })?;
            changes.push(IssueFieldChange::Priority(parsed));
        }
        if let Some(name) = &args.status {
            let s = statuses.iter().find(|s| &s.name == name).ok_or_else(|| {
                crate::error::unknown_name(
                    "status",
                    name,
                    &args.key,
                    &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                )
            })?;
            changes.push(IssueFieldChange::Status(s.id));
        }
        if let Some(d) = &args.due_date {
            let date = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| McpError::invalid_params("due_date must be YYYY-MM-DD", None))?;
            changes.push(IssueFieldChange::DueDate(Some(date)));
        }

        // phase 3: apply each change, then re-read
        let issue = self
            .blocking_mut(move |ws| {
                for change in changes {
                    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
                        id: issue_id,
                        change,
                    }))?;
                }
                ws.query_issue_by_id(issue_id)
            })
            .await?;

        let map = crate::convert::status_name_map(&statuses);
        let name = map.get(&issue.status_id).cloned().unwrap_or_default();
        json_content(&crate::convert::IssueOut::from_issue(issue, &name))
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
