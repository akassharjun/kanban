#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-end acceptance: a full create → move → update → undo/redo loop driven
//! over the in-process stdio transport. The executable form of the design's
//! acceptance criteria 2-3.
use kanban_core::Workspace;
use kanban_mcp::server::KanbanServer;
use rmcp::model::CallToolRequestParams;
use rmcp::{ClientHandler, ServiceExt};

#[derive(Default, Clone)]
struct TestClient;
impl ClientHandler for TestClient {}

/// Call a tool with JSON args and return the parsed JSON content block.
async fn call(
    client: &rmcp::service::RunningService<rmcp::RoleClient, TestClient>,
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let result = client
        .call_tool(CallToolRequestParams::new(name.to_string()).with_arguments(args))
        .await
        .unwrap();
    let text = result.content[0]
        .as_text()
        .expect("first content block is text");
    serde_json::from_str(&text.text).unwrap()
}

#[tokio::test]
async fn e2e_create_move_update_undo_flow() {
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

    // create a project and an issue
    call(
        &client,
        "create_project",
        rmcp::object!({"name": "Auth", "prefix": "AUTH"}),
    )
    .await;
    let issue = call(
        &client,
        "create_issue",
        rmcp::object!({"project": "AUTH", "title": "Original"}),
    )
    .await;
    assert_eq!(issue["key"], "AUTH-1", "got: {issue}");
    assert_eq!(issue["status"], "Todo", "got: {issue}");
    assert!(
        issue["assignee"].is_null(),
        "a new issue is unassigned: {issue}"
    );

    // move it to another column
    call(
        &client,
        "move_issue",
        rmcp::object!({"key": "AUTH-1", "status": "In Progress"}),
    )
    .await;
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(issue["status"], "In Progress", "got: {issue}");

    // update the title (a single operation)
    call(
        &client,
        "update_issue",
        rmcp::object!({"key": "AUTH-1", "title": "Renamed"}),
    )
    .await;
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(issue["title"], "Renamed", "got: {issue}");

    // undo reverts the most recent change (the title), not the earlier move
    call(&client, "undo", rmcp::object!({})).await;
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(
        issue["title"], "Original",
        "undo should revert the title: {issue}"
    );
    assert_eq!(
        issue["status"], "In Progress",
        "undo reverts only the last change: {issue}"
    );

    // redo re-applies the title change
    call(&client, "redo", rmcp::object!({})).await;
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(issue["title"], "Renamed", "got: {issue}");

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}
