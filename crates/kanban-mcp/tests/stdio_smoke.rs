#![allow(clippy::unwrap_used, clippy::expect_used)]
use kanban_core::Workspace;
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::types::Priority;
use kanban_mcp::server::KanbanServer;
use rmcp::{ClientHandler, ServiceExt, model::CallToolRequestParams};
use uuid::Uuid;

#[derive(Default, Clone)]
struct TestClient;
impl ClientHandler for TestClient {}

/// Call `list_issues` with the given arguments and return the issue keys in
/// the order the server returns them (`sort_key` order for the default filter).
async fn list_keys(
    client: &rmcp::service::RunningService<rmcp::RoleClient, TestClient>,
    args: serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let result = client
        .call_tool(CallToolRequestParams::new("list_issues").with_arguments(args))
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    parsed
        .as_array()
        .expect("issues is an array")
        .iter()
        .map(|i| i["key"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn lists_tools_and_calls_list_projects() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    ws.apply(Operation::CreateProject(CreateProject {
        id: Uuid::now_v7(),
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "list_projects"));

    let result = client
        .call_tool(CallToolRequestParams::new("list_projects"))
        .await
        .unwrap();
    // The tool returns a JSON text block; parse it and assert on the structure
    // so a field-name / nesting regression would fail the test.
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed[0]["prefix"], "AUTH", "got: {parsed}");

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn issue_read_tools() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let project_id = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id: project_id,
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    // First default status is "Todo".
    let statuses = ws.query_statuses_for_project(project_id).unwrap();
    let todo = statuses
        .iter()
        .find(|s| s.name == "Todo")
        .expect("a 'Todo' default status");

    ws.apply(Operation::CreateIssue(CreateIssue {
        id: Uuid::now_v7(),
        project_id,
        title: "Implement login flow".into(),
        description: Some("OAuth handshake details".into()),
        status_id: todo.id,
        priority: Priority::High,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "list_issues"));
    assert!(tools.iter().any(|t| t.name == "get_issue"));
    assert!(tools.iter().any(|t| t.name == "search_issues"));

    // list_issues: the issue appears, status is the NAME ("Todo").
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_issues")
                .with_arguments(rmcp::object!({"project": "AUTH"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let issues = parsed.as_array().expect("issues is an array");
    assert_eq!(issues.len(), 1, "got: {parsed}");
    assert_eq!(issues[0]["key"], "AUTH-1", "got: {parsed}");
    assert_eq!(issues[0]["title"], "Implement login flow", "got: {parsed}");
    assert_eq!(issues[0]["status"], "Todo", "got: {parsed}");
    assert_eq!(issues[0]["priority"], "high", "got: {parsed}");

    // get_issue: by key, including description + status name.
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["title"], "Implement login flow", "got: {parsed}");
    assert_eq!(parsed["status"], "Todo", "got: {parsed}");
    assert_eq!(
        parsed["description"], "OAuth handshake details",
        "got: {parsed}"
    );

    // search_issues: a word from the title matches.
    let result = client
        .call_tool(
            CallToolRequestParams::new("search_issues")
                .with_arguments(rmcp::object!({"query": "login"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let issues = parsed.as_array().expect("issues is an array");
    assert!(
        issues.iter().any(|i| i["key"] == "AUTH-1"),
        "expected AUTH-1 in search results, got: {parsed}"
    );

    // list_issues with an unknown status name surfaces a tool error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("list_issues")
                .with_arguments(rmcp::object!({"project": "AUTH", "status": "NoSuchStatus"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown status, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
async fn create_project_tool() {
    // Start from an EMPTY workspace (no seed project).
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "create_project"));

    // Valid create succeeds and echoes the prefix.
    let result = client
        .call_tool(
            CallToolRequestParams::new("create_project")
                .with_arguments(rmcp::object!({"name": "Auth Service", "prefix": "AUTH"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    assert!(text.text.contains("AUTH"), "got: {}", text.text);

    // list_projects now contains the created project.
    let result = client
        .call_tool(CallToolRequestParams::new("list_projects"))
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let projects = parsed.as_array().expect("projects is an array");
    assert!(
        projects.iter().any(|p| p["prefix"] == "AUTH"),
        "expected a project with prefix AUTH, got: {parsed}"
    );

    // Invalid prefix (lowercase) is rejected by core validation -> tool error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("create_project")
                .with_arguments(rmcp::object!({"name": "x", "prefix": "lower"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for invalid prefix, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
async fn create_issue_tool() {
    // Empty workspace, then create a project so we can add issues to it.
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "create_issue"));

    client
        .call_tool(
            CallToolRequestParams::new("create_project")
                .with_arguments(rmcp::object!({"name": "Auth Service", "prefix": "AUTH"})),
        )
        .await
        .unwrap();

    // Create an issue with defaults: returns the full IssueOut with the
    // core-assigned key ("AUTH-1"), default first status ("Todo"), default
    // priority ("medium").
    let result = client
        .call_tool(
            CallToolRequestParams::new("create_issue")
                .with_arguments(rmcp::object!({"project": "AUTH", "title": "Add login"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["key"], "AUTH-1", "got: {parsed}");
    assert_eq!(parsed["title"], "Add login", "got: {parsed}");
    assert_eq!(parsed["status"], "Todo", "got: {parsed}");
    assert_eq!(parsed["priority"], "medium", "got: {parsed}");

    // Bad priority is rejected.
    let err =
        client
            .call_tool(CallToolRequestParams::new("create_issue").with_arguments(
                rmcp::object!({"project": "AUTH", "title": "x", "priority": "bogus"}),
            ))
            .await;
    assert!(
        err.is_err(),
        "expected error for bad priority, got: {err:?}"
    );

    // Unknown status name is rejected.
    let err = client
        .call_tool(CallToolRequestParams::new("create_issue").with_arguments(
            rmcp::object!({"project": "AUTH", "title": "x", "status": "NoSuchStatus"}),
        ))
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown status, got: {err:?}"
    );

    // Unknown project is rejected.
    let err = client
        .call_tool(
            CallToolRequestParams::new("create_issue")
                .with_arguments(rmcp::object!({"project": "NOPE", "title": "x"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown project, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn update_issue_tool() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let project_id = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id: project_id,
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let statuses = ws.query_statuses_for_project(project_id).unwrap();
    let todo = statuses
        .iter()
        .find(|s| s.name == "Todo")
        .expect("a 'Todo' default status");

    ws.apply(Operation::CreateIssue(CreateIssue {
        id: Uuid::now_v7(),
        project_id,
        title: "Old".into(),
        description: None,
        status_id: todo.id,
        priority: Priority::Medium,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "update_issue"));

    // Update title + priority; status stays "Todo".
    let result =
        client
            .call_tool(CallToolRequestParams::new("update_issue").with_arguments(
                rmcp::object!({"key": "AUTH-1", "title": "New", "priority": "high"}),
            ))
            .await
            .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["title"], "New", "got: {parsed}");
    assert_eq!(parsed["priority"], "high", "got: {parsed}");
    assert_eq!(parsed["status"], "Todo", "got: {parsed}");

    // Update status only.
    let result = client
        .call_tool(
            CallToolRequestParams::new("update_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1", "status": "In Progress"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["status"], "In Progress", "got: {parsed}");

    // get_issue reflects both updates.
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["title"], "New", "got: {parsed}");
    assert_eq!(parsed["status"], "In Progress", "got: {parsed}");

    // No fields -> invalid_params error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("update_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1"})),
        )
        .await;
    assert!(err.is_err(), "expected error for no fields, got: {err:?}");

    // Unknown issue key -> not_found error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("update_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-999", "title": "x"})),
        )
        .await;
    assert!(err.is_err(), "expected error for unknown key, got: {err:?}");

    // Unknown status name -> error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("update_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1", "status": "Nope"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown status, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn move_issue_tool() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let project_id = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id: project_id,
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let statuses = ws.query_statuses_for_project(project_id).unwrap();
    let todo = statuses
        .iter()
        .find(|s| s.name == "Todo")
        .expect("a 'Todo' default status");

    // Three issues in "Todo", created in order so their sort_keys ascend.
    for title in ["First", "Second", "Third"] {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: Uuid::now_v7(),
            project_id,
            title: title.into(),
            description: None,
            status_id: todo.id,
            priority: Priority::Medium,
            due_date: None,
            label_ids: vec![],
        }))
        .unwrap();
    }

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "move_issue"));

    // Initial order is AUTH-1, AUTH-2, AUTH-3.
    assert_eq!(
        list_keys(&client, rmcp::object!({"project": "AUTH"})).await,
        vec!["AUTH-1", "AUTH-2", "AUTH-3"],
    );

    // Move AUTH-3 before AUTH-1 -> order becomes AUTH-3, AUTH-1, AUTH-2.
    client
        .call_tool(
            CallToolRequestParams::new("move_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-3", "before": "AUTH-1"})),
        )
        .await
        .unwrap();
    assert_eq!(
        list_keys(&client, rmcp::object!({"project": "AUTH"})).await,
        vec!["AUTH-3", "AUTH-1", "AUTH-2"],
    );

    // Move AUTH-1 to "In Progress" (no sibling) -> returned status is "In Progress".
    let result = client
        .call_tool(
            CallToolRequestParams::new("move_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-1", "status": "In Progress"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(parsed["key"], "AUTH-1", "got: {parsed}");
    assert_eq!(parsed["status"], "In Progress", "got: {parsed}");

    // list_issues scoped to "In Progress" contains AUTH-1.
    let in_progress = list_keys(
        &client,
        rmcp::object!({"project": "AUTH", "status": "In Progress"}),
    )
    .await;
    assert_eq!(in_progress, vec!["AUTH-1"], "got: {in_progress:?}");

    // AUTH-1 left the Todo column.
    let todo_keys = list_keys(
        &client,
        rmcp::object!({"project": "AUTH", "status": "Todo"}),
    )
    .await;
    assert_eq!(todo_keys, vec!["AUTH-3", "AUTH-2"], "got: {todo_keys:?}");

    // Unknown issue key -> error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("move_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-999", "before": "AUTH-1"})),
        )
        .await;
    assert!(err.is_err(), "expected error for unknown key, got: {err:?}");

    // Unknown sibling key -> error.
    let err = client
        .call_tool(
            CallToolRequestParams::new("move_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-2", "before": "AUTH-999"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown sibling, got: {err:?}"
    );

    // `after` reorder: move AUTH-3 to land after AUTH-2 in Todo -> [AUTH-2, AUTH-3].
    client
        .call_tool(
            CallToolRequestParams::new("move_issue")
                .with_arguments(rmcp::object!({"key": "AUTH-3", "after": "AUTH-2"})),
        )
        .await
        .unwrap();
    let todo_keys = list_keys(
        &client,
        rmcp::object!({"project": "AUTH", "status": "Todo"}),
    )
    .await;
    assert_eq!(
        todo_keys,
        vec!["AUTH-2", "AUTH-3"],
        "after-reorder, got: {todo_keys:?}"
    );

    // `before` and `after` are mutually exclusive.
    let err = client
        .call_tool(CallToolRequestParams::new("move_issue").with_arguments(
            rmcp::object!({"key": "AUTH-2", "before": "AUTH-3", "after": "AUTH-3"}),
        ))
        .await;
    assert!(
        err.is_err(),
        "expected error for before+after, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
async fn undo_redo_tools() {
    // Empty workspace — every action happens through the MCP tools.
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    // Both tools appear in the tool list.
    let tools = client.list_all_tools().await.unwrap();
    assert!(
        tools.iter().any(|t| t.name == "undo"),
        "undo missing from tools"
    );
    assert!(
        tools.iter().any(|t| t.name == "redo"),
        "redo missing from tools"
    );

    // Create a project via MCP.
    client
        .call_tool(
            CallToolRequestParams::new("create_project")
                .with_arguments(rmcp::object!({"name": "Auth Service", "prefix": "AUTH"})),
        )
        .await
        .unwrap();

    // Confirm it exists.
    let result = client
        .call_tool(CallToolRequestParams::new("list_projects"))
        .await
        .unwrap();
    let text = result.content[0].as_text().expect("text content");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let projects = parsed.as_array().expect("projects array");
    assert!(
        projects.iter().any(|p| p["prefix"] == "AUTH"),
        "expected AUTH before undo, got: {parsed}"
    );

    // Undo the create -> list_projects returns empty.
    let result = client
        .call_tool(CallToolRequestParams::new("undo"))
        .await
        .unwrap();
    let text = result.content[0].as_text().expect("text content");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(
        parsed["undone"], true,
        "expected undone:true, got: {parsed}"
    );

    let result = client
        .call_tool(CallToolRequestParams::new("list_projects"))
        .await
        .unwrap();
    let text = result.content[0].as_text().expect("text content");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let projects = parsed.as_array().expect("projects array");
    assert!(
        projects.is_empty(),
        "expected empty list after undo, got: {parsed}"
    );

    // Redo -> AUTH is back.
    let result = client
        .call_tool(CallToolRequestParams::new("redo"))
        .await
        .unwrap();
    let text = result.content[0].as_text().expect("text content");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert_eq!(
        parsed["redone"], true,
        "expected redone:true, got: {parsed}"
    );

    let result = client
        .call_tool(CallToolRequestParams::new("list_projects"))
        .await
        .unwrap();
    let text = result.content[0].as_text().expect("text content");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let projects = parsed.as_array().expect("projects array");
    assert!(
        projects.iter().any(|p| p["prefix"] == "AUTH"),
        "expected AUTH after redo, got: {parsed}"
    );

    // Undo once more (back to empty) then undo again -> error (nothing to undo).
    client
        .call_tool(CallToolRequestParams::new("undo"))
        .await
        .unwrap();

    let err = client.call_tool(CallToolRequestParams::new("undo")).await;
    assert!(
        err.is_err(),
        "expected error when nothing left to undo, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
async fn list_statuses_and_labels() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    ws.apply(Operation::CreateProject(CreateProject {
        id: Uuid::now_v7(),
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "list_statuses"));
    assert!(tools.iter().any(|t| t.name == "list_labels"));

    // A fresh project seeds 7 default statuses; assert the array is non-empty
    // and contains a known default name.
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_statuses")
                .with_arguments(rmcp::object!({"project": "AUTH"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    let statuses = parsed.as_array().expect("statuses is an array");
    assert_eq!(statuses.len(), 7, "got: {parsed}");
    assert!(
        statuses.iter().any(|s| s["name"] == "In Progress"),
        "expected an 'In Progress' status, got: {parsed}"
    );

    // Labels: a fresh project has none, so an empty array.
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_labels")
                .with_arguments(rmcp::object!({"project": "AUTH"})),
        )
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    let parsed: serde_json::Value = serde_json::from_str(&text.text).unwrap();
    assert!(parsed.as_array().expect("labels is an array").is_empty());

    // Unknown project surfaces a tool error (JSON-RPC error -> Err).
    let err = client
        .call_tool(
            CallToolRequestParams::new("list_statuses")
                .with_arguments(rmcp::object!({"project": "NOPE"})),
        )
        .await;
    assert!(
        err.is_err(),
        "expected error for unknown project, got: {err:?}"
    );

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}
