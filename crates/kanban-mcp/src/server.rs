use std::sync::{Arc, Mutex};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};

use kanban_core::Workspace;

use crate::convert::{MemberOut, ProjectOut};
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

    /// Resolve a member by name within a project prefix to its UUID. An unknown
    /// prefix maps to `not_found`; an unknown name maps to `unknown_name` listing
    /// the project's members.
    async fn resolve_member(&self, prefix: &str, name: &str) -> Result<uuid::Uuid, McpError> {
        let owned_prefix = prefix.to_string();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(p) = ws.query_project_by_prefix(&owned_prefix)? else {
                    return Ok(None);
                };
                Ok(Some(ws.query_members_for_project(p.id)?))
            })
            .await?;
        let members = resolved.ok_or_else(|| crate::error::not_found("project", prefix))?;
        members
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.id)
            .ok_or_else(|| {
                crate::error::unknown_name(
                    "member",
                    name,
                    prefix,
                    &members.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
                )
            })
    }
}

/// Fractional sort-key midpoint between two optional neighbours.
/// Mirrors the GUI's 1024-gap convention.
fn midpoint(before: Option<f64>, after: Option<f64>) -> f64 {
    match (before, after) {
        (Some(b), Some(a)) => f64::midpoint(b, a),
        (Some(b), None) => b + 1024.0,
        (None, Some(a)) => a - 1024.0,
        (None, None) => 1024.0,
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

        // phase 1: resolve project id + its statuses + its members
        let prefix = args.project.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(p) = ws.query_project_by_prefix(&prefix)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(p.id)?;
                let members = ws.query_members_for_project(p.id)?;
                Ok(Some((p.id, statuses, members)))
            })
            .await?;
        let (project_id, statuses, members) =
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
        let mmap = crate::convert::member_name_map(&members);
        let out: Vec<crate::convert::IssueOut> = issues
            .into_iter()
            .map(|i| {
                let name = map.get(&i.status_id).cloned().unwrap_or_default();
                let assignee = i.assignee_id.and_then(|id| mmap.get(&id).cloned());
                crate::convert::IssueOut::from_issue(i, &name, assignee)
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
                let members = ws.query_members_for_project(issue.project_id)?;
                Ok(Some((issue, statuses, members)))
            })
            .await?;
        let (issue, statuses, members) =
            resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;
        let name = statuses
            .iter()
            .find(|s| s.id == issue.status_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let assignee = issue
            .assignee_id
            .and_then(|id| members.iter().find(|m| m.id == id).map(|m| m.name.clone()));
        json_content(&crate::convert::IssueOut::from_issue(
            issue, &name, assignee,
        ))
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
                // collect all statuses + members so we can name them across projects
                let projects = ws.query_projects()?;
                let mut statuses = Vec::new();
                let mut members = Vec::new();
                for p in &projects {
                    statuses.extend(ws.query_statuses_for_project(p.id)?);
                    members.extend(ws.query_members_for_project(p.id)?);
                }
                let filter = match pid {
                    Some(id) => IssueFilter::for_project(id),
                    None => IssueFilter::default(),
                };
                let issues = ws.search(&query, filter)?;
                Ok(Some((issues, statuses, members)))
            })
            .await?;
        let (issues, statuses, members) = resolved.ok_or_else(|| {
            crate::error::not_found("project", args.project.as_deref().unwrap_or(""))
        })?;
        let map = crate::convert::status_name_map(&statuses);
        let mmap = crate::convert::member_name_map(&members);
        let out: Vec<crate::convert::IssueOut> = issues
            .into_iter()
            .map(|i| {
                let name = map.get(&i.status_id).cloned().unwrap_or_default();
                let assignee = i.assignee_id.and_then(|id| mmap.get(&id).cloned());
                crate::convert::IssueOut::from_issue(i, &name, assignee)
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
        // a newly created issue is always unassigned
        json_content(&crate::convert::IssueOut::from_issue(issue, &name, None))
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

        // phase 1: resolve issue id + its project's statuses + members
        let key = args.key.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(issue) = ws.query_issue_by_identifier(&key)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(issue.project_id)?;
                let members = ws.query_members_for_project(issue.project_id)?;
                Ok(Some((issue.id, statuses, members)))
            })
            .await?;
        let (issue_id, statuses, members) =
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
        let mmap = crate::convert::member_name_map(&members);
        let name = map.get(&issue.status_id).cloned().unwrap_or_default();
        let assignee = issue.assignee_id.and_then(|id| mmap.get(&id).cloned());
        json_content(&crate::convert::IssueOut::from_issue(
            issue, &name, assignee,
        ))
    }

    #[tool(
        description = "Undo the most recent change. Note: a multi-field issue update or a move applies as more than one change, so it may take multiple undos to fully revert."
    )]
    async fn undo(&self) -> Result<CallToolResult, McpError> {
        self.blocking_mut(|ws| ws.undo().map(|_| ())).await?;
        json_content(&serde_json::json!({ "undone": true }))
    }

    #[tool(description = "Redo the most recently undone change.")]
    async fn redo(&self) -> Result<CallToolResult, McpError> {
        self.blocking_mut(|ws| ws.redo().map(|_| ())).await?;
        json_content(&serde_json::json!({ "redone": true }))
    }

    #[tool(
        description = "Move an issue: change its status column and/or reorder it before/after a sibling issue (by key). Returns the moved issue."
    )]
    async fn move_issue(
        &self,
        Parameters(args): Parameters<crate::inputs::MoveIssueInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{IssueFieldChange, Operation, ReorderIssue, UpdateIssueField};
        use kanban_core::query::IssueFilter;
        use kanban_core::types::Issue;

        if args.before.is_some() && args.after.is_some() {
            return Err(McpError::invalid_params(
                "before and after are mutually exclusive; provide at most one",
                None,
            ));
        }

        // phase 1: resolve the dragged issue + the project's statuses + all project issues
        let key = args.key.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(issue) = ws.query_issue_by_identifier(&key)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(issue.project_id)?;
                let issues = ws.query_issues(IssueFilter::for_project(issue.project_id))?;
                let members = ws.query_members_for_project(issue.project_id)?;
                Ok(Some((issue, statuses, issues, members)))
            })
            .await?;
        let (issue, statuses, project_issues, members) =
            resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;

        // target status: named, else keep current
        let target_status_id = match &args.status {
            Some(name) => {
                statuses
                    .iter()
                    .find(|s| &s.name == name)
                    .ok_or_else(|| {
                        crate::error::unknown_name(
                            "status",
                            name,
                            &args.key,
                            &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                        )
                    })?
                    .id
            }
            None => issue.status_id,
        };

        // target column: issues in the target status, excluding the dragged issue, by sort_key
        let mut col: Vec<&Issue> = project_issues
            .iter()
            .filter(|i| i.status_id == target_status_id && i.id != issue.id)
            .collect();
        col.sort_by(|a, b| a.sort_key.total_cmp(&b.sort_key));

        // neighbours from before/after sibling, else append to the end
        let (before_sk, after_sk) = if let Some(bkey) = &args.before {
            let idx = col
                .iter()
                .position(|i| &i.identifier == bkey)
                .ok_or_else(|| crate::error::not_found("sibling issue", bkey))?;
            let before = if idx > 0 {
                Some(col[idx - 1].sort_key)
            } else {
                None
            };
            (before, Some(col[idx].sort_key))
        } else if let Some(akey) = &args.after {
            let idx = col
                .iter()
                .position(|i| &i.identifier == akey)
                .ok_or_else(|| crate::error::not_found("sibling issue", akey))?;
            let after = col.get(idx + 1).map(|i| i.sort_key);
            (Some(col[idx].sort_key), after)
        } else {
            (col.last().map(|i| i.sort_key), None)
        };
        let new_sort_key = midpoint(before_sk, after_sk);

        // phase 3: apply status change (if any) + reorder, re-read
        let issue_id = issue.id;
        let current_status = issue.status_id;
        let moved = self
            .blocking_mut(move |ws| {
                if target_status_id != current_status {
                    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
                        id: issue_id,
                        change: IssueFieldChange::Status(target_status_id),
                    }))?;
                }
                ws.apply(Operation::ReorderIssue(ReorderIssue {
                    id: issue_id,
                    new_sort_key,
                }))?;
                ws.query_issue_by_id(issue_id)
            })
            .await?;

        let map = crate::convert::status_name_map(&statuses);
        let mmap = crate::convert::member_name_map(&members);
        let name = map.get(&moved.status_id).cloned().unwrap_or_default();
        let assignee = moved.assignee_id.and_then(|id| mmap.get(&id).cloned());
        json_content(&crate::convert::IssueOut::from_issue(
            moved, &name, assignee,
        ))
    }

    #[tool(
        description = "List the members (people) of a project. `project` is the prefix, e.g. AUTH."
    )]
    async fn list_members(
        &self,
        Parameters(args): Parameters<crate::inputs::ProjectRef>,
    ) -> Result<CallToolResult, McpError> {
        let members: Vec<kanban_core::types::Member> = self
            .with_project(&args.project, |ws, id, _statuses| {
                ws.query_members_for_project(id)
            })
            .await?;
        let out: Vec<MemberOut> = members.into_iter().map(MemberOut::from).collect();
        json_content(&out)
    }

    #[tool(
        description = "Add a member to a project. `project` is the prefix; `name` must be unique within the project."
    )]
    async fn create_member(
        &self,
        Parameters(args): Parameters<crate::inputs::CreateMemberInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{CreateMember, Operation};

        let prefix = args.project.clone();
        let project_id = self
            .blocking_read(move |ws| Ok(ws.query_project_by_prefix(&prefix)?.map(|p| p.id)))
            .await?
            .ok_or_else(|| crate::error::not_found("project", &args.project))?;

        let id = uuid::Uuid::now_v7();
        let name = args.name.clone();
        self.blocking_mut(move |ws| {
            ws.apply(Operation::CreateMember(CreateMember {
                id,
                project_id,
                name: args.name,
            }))
            .map(|_| ())
        })
        .await?;
        json_content(&serde_json::json!({ "name": name }))
    }

    #[tool(
        description = "Rename a member of a project. Identify the member by its current `name` within `project`."
    )]
    async fn update_member(
        &self,
        Parameters(args): Parameters<crate::inputs::UpdateMemberInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{MemberPatch, Operation, UpdateMember};

        let member_id = self.resolve_member(&args.project, &args.name).await?;
        let new_name = args.new_name.clone();
        self.blocking_mut(move |ws| {
            ws.apply(Operation::UpdateMember(UpdateMember {
                id: member_id,
                patch: MemberPatch {
                    name: Some(args.new_name),
                },
            }))
            .map(|_| ())
        })
        .await?;
        json_content(&serde_json::json!({ "name": new_name }))
    }

    #[tool(
        description = "Remove a member from a project (by `name`). Any issues assigned to them become unassigned. Undoable."
    )]
    async fn delete_member(
        &self,
        Parameters(args): Parameters<crate::inputs::MemberRef>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{DeleteMember, Operation};

        let member_id = self.resolve_member(&args.project, &args.name).await?;
        let name = args.name.clone();
        self.blocking_mut(move |ws| {
            ws.apply(Operation::DeleteMember(DeleteMember { id: member_id }))
                .map(|_| ())
        })
        .await?;
        json_content(&serde_json::json!({ "deleted": name }))
    }

    #[tool(
        description = "Assign an issue (by key) to a member of its project (by name), or unassign it by omitting `member`. Returns the updated issue."
    )]
    async fn assign_issue(
        &self,
        Parameters(args): Parameters<crate::inputs::AssignIssueInput>,
    ) -> Result<CallToolResult, McpError> {
        use kanban_core::operation::{IssueFieldChange, Operation, UpdateIssueField};

        // phase 1: resolve the issue + its project's statuses + members
        let key = args.key.clone();
        let resolved = self
            .blocking_read(move |ws| {
                let Some(issue) = ws.query_issue_by_identifier(&key)? else {
                    return Ok(None);
                };
                let statuses = ws.query_statuses_for_project(issue.project_id)?;
                let members = ws.query_members_for_project(issue.project_id)?;
                Ok(Some((issue.id, statuses, members)))
            })
            .await?;
        let (issue_id, statuses, members) =
            resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;

        // phase 2: resolve the member name within the issue's project (None = unassign)
        let assignee_id = match args.member.as_deref().filter(|m| !m.is_empty()) {
            Some(name) => Some(
                members
                    .iter()
                    .find(|m| m.name == name)
                    .ok_or_else(|| {
                        crate::error::unknown_name(
                            "member",
                            name,
                            &args.key,
                            &members.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
                        )
                    })?
                    .id,
            ),
            None => None,
        };

        // phase 3: apply the assignment, then re-read
        let issue = self
            .blocking_mut(move |ws| {
                ws.apply(Operation::UpdateIssueField(UpdateIssueField {
                    id: issue_id,
                    change: IssueFieldChange::Assignee(assignee_id),
                }))?;
                ws.query_issue_by_id(issue_id)
            })
            .await?;

        let map = crate::convert::status_name_map(&statuses);
        let mmap = crate::convert::member_name_map(&members);
        let name = map.get(&issue.status_id).cloned().unwrap_or_default();
        let assignee = issue.assignee_id.and_then(|id| mmap.get(&id).cloned());
        json_content(&crate::convert::IssueOut::from_issue(
            issue, &name, assignee,
        ))
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
