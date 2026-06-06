#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-end acceptance for the Spec #7 member tools: list/create/update/delete
//! members and assign/unassign issues, driven over the in-process stdio
//! transport. The executable form of the design's acceptance criteria.
use kanban_core::Workspace;
use kanban_mcp::server::KanbanServer;
use rmcp::model::CallToolRequestParams;
use rmcp::{ClientHandler, ServiceExt};

#[derive(Default, Clone)]
struct TestClient;
impl ClientHandler for TestClient {}

type Client = rmcp::service::RunningService<rmcp::RoleClient, TestClient>;

/// Call a tool with JSON args and return the parsed JSON content block.
async fn call(
    client: &Client,
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

/// Call a tool expecting it to fail; return the error rendered as a string.
async fn call_err(
    client: &Client,
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> String {
    let err = client
        .call_tool(CallToolRequestParams::new(name.to_string()).with_arguments(args))
        .await
        .expect_err("expected the tool call to error");
    format!("{err:?}")
}

async fn connect(server: KanbanServer) -> (Client, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let (server_t, client_t) = tokio::io::duplex(8192);
    let handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });
    let client = TestClient.serve(client_t).await.unwrap();
    (client, handle)
}

#[tokio::test]
async fn e2e_member_lifecycle_and_assignment() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let (client, server_handle) = connect(KanbanServer::with_workspace(ws)).await;

    // project + issue
    call(
        &client,
        "create_project",
        rmcp::object!({"name": "Auth", "prefix": "AUTH"}),
    )
    .await;
    let issue = call(
        &client,
        "create_issue",
        rmcp::object!({"project": "AUTH", "title": "Wire OAuth"}),
    )
    .await;
    assert_eq!(issue["key"], "AUTH-1", "got: {issue}");
    assert!(issue["assignee"].is_null(), "new issue unassigned: {issue}");

    // roster starts empty
    let members = call(&client, "list_members", rmcp::object!({"project": "AUTH"})).await;
    assert_eq!(members, serde_json::json!([]), "got: {members}");

    // create two members
    let alice = call(
        &client,
        "create_member",
        rmcp::object!({"project": "AUTH", "name": "Alice"}),
    )
    .await;
    assert_eq!(alice["name"], "Alice", "got: {alice}");
    call(
        &client,
        "create_member",
        rmcp::object!({"project": "AUTH", "name": "Bob"}),
    )
    .await;

    // roster lists both, ordered by name
    let members = call(&client, "list_members", rmcp::object!({"project": "AUTH"})).await;
    let names: Vec<&str> = members
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Alice", "Bob"], "got: {members}");

    // assign the issue to Alice -> reflected in the returned issue
    let assigned = call(
        &client,
        "assign_issue",
        rmcp::object!({"key": "AUTH-1", "member": "Alice"}),
    )
    .await;
    assert_eq!(assigned["assignee"], "Alice", "got: {assigned}");

    // get_issue reflects the assignee too
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(issue["assignee"], "Alice", "got: {issue}");

    // rename Alice -> Alicia; the assignment follows (same member id)
    call(
        &client,
        "update_member",
        rmcp::object!({"project": "AUTH", "name": "Alice", "new_name": "Alicia"}),
    )
    .await;
    let issue = call(&client, "get_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert_eq!(issue["assignee"], "Alicia", "rename follows: {issue}");

    // unassign (omit member) -> assignee null
    let unassigned = call(&client, "assign_issue", rmcp::object!({"key": "AUTH-1"})).await;
    assert!(unassigned["assignee"].is_null(), "got: {unassigned}");

    // delete Bob, then Alicia; roster empties
    call(
        &client,
        "delete_member",
        rmcp::object!({"project": "AUTH", "name": "Bob"}),
    )
    .await;
    call(
        &client,
        "delete_member",
        rmcp::object!({"project": "AUTH", "name": "Alicia"}),
    )
    .await;
    let members = call(&client, "list_members", rmcp::object!({"project": "AUTH"})).await;
    assert_eq!(members, serde_json::json!([]), "got: {members}");

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}

#[tokio::test]
async fn member_tool_error_paths() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    let (client, server_handle) = connect(KanbanServer::with_workspace(ws)).await;

    call(
        &client,
        "create_project",
        rmcp::object!({"name": "Auth", "prefix": "AUTH"}),
    )
    .await;
    call(
        &client,
        "create_issue",
        rmcp::object!({"project": "AUTH", "title": "Task"}),
    )
    .await;
    call(
        &client,
        "create_member",
        rmcp::object!({"project": "AUTH", "name": "Alice"}),
    )
    .await;

    // duplicate member name -> conflict
    let _dup = call_err(
        &client,
        "create_member",
        rmcp::object!({"project": "AUTH", "name": "Alice"}),
    )
    .await;

    // assign to an unknown member -> error that lists available members
    let rendered = call_err(
        &client,
        "assign_issue",
        rmcp::object!({"key": "AUTH-1", "member": "Nobody"}),
    )
    .await;
    assert!(rendered.contains("Alice"), "lists available: {rendered}");

    // delete an unknown member -> error
    let _missing = call_err(
        &client,
        "delete_member",
        rmcp::object!({"project": "AUTH", "name": "Ghost"}),
    )
    .await;

    client.cancel().await.unwrap();
    server_handle
        .await
        .expect("server task panicked")
        .expect("server task errored");
}
