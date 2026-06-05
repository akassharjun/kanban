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
