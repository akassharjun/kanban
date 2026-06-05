#![allow(clippy::unwrap_used, clippy::expect_used)]
use kanban_core::Workspace;
use kanban_core::operation::{CreateProject, Operation};
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
