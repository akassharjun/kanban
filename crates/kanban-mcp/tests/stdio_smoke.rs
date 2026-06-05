#![allow(clippy::unwrap_used)]
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
    let text = format!("{:?}", result.content);
    assert!(text.contains("AUTH"), "got: {text}");

    client.cancel().await.unwrap();
    let _ = server_handle.await;
}
