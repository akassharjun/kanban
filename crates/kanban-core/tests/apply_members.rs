#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use kanban_core::operation::{
    ConflictPolicy, CreateMember, CreateProject, DeleteMember, ImportSnapshot, MemberPatch,
    Operation, UpdateMember,
};
use kanban_core::{Workspace, new_id};

fn fresh_with_project() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "M".into(),
        prefix: "MBR".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    (ws, pid)
}

#[test]
fn create_member_inserts() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "Ada".into(),
    }))
    .unwrap();
    let members = ws.query_members_for_project(pid).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "Ada");
    assert_eq!(members[0].id, id);
    assert_eq!(members[0].project_id, pid);
}

#[test]
fn create_member_rejects_empty_name() {
    let (mut ws, pid) = fresh_with_project();
    let err = ws
        .apply(Operation::CreateMember(CreateMember {
            id: new_id(),
            project_id: pid,
            name: String::new(),
        }))
        .unwrap_err();
    assert!(err.to_string().contains("name"), "{err}");
}

#[test]
fn create_member_rejects_duplicate_name_per_project() {
    let (mut ws, pid) = fresh_with_project();
    ws.apply(Operation::CreateMember(CreateMember {
        id: new_id(),
        project_id: pid,
        name: "dup".into(),
    }))
    .unwrap();
    let err = ws
        .apply(Operation::CreateMember(CreateMember {
            id: new_id(),
            project_id: pid,
            name: "dup".into(),
        }))
        .unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn update_member_renames() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "old".into(),
    }))
    .unwrap();
    ws.apply(Operation::UpdateMember(UpdateMember {
        id,
        patch: MemberPatch {
            name: Some("new".into()),
        },
    }))
    .unwrap();
    let members = ws.query_members_for_project(pid).unwrap();
    assert_eq!(members[0].name, "new");
}

#[test]
fn create_member_undo_removes_it() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "temp".into(),
    }))
    .unwrap();
    assert_eq!(ws.query_members_for_project(pid).unwrap().len(), 1);
    ws.undo().unwrap();
    assert!(ws.query_members_for_project(pid).unwrap().is_empty());
}

#[test]
fn update_member_undo_restores_name() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "original".into(),
    }))
    .unwrap();
    ws.apply(Operation::UpdateMember(UpdateMember {
        id,
        patch: MemberPatch {
            name: Some("changed".into()),
        },
    }))
    .unwrap();
    ws.undo().unwrap();
    let members = ws.query_members_for_project(pid).unwrap();
    assert_eq!(members[0].name, "original");
}

#[test]
fn delete_member_undo_restores_member() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "kept".into(),
    }))
    .unwrap();
    ws.apply(Operation::DeleteMember(DeleteMember { id }))
        .unwrap();
    assert!(ws.query_members_for_project(pid).unwrap().is_empty());
    ws.undo().unwrap();
    let members = ws.query_members_for_project(pid).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].id, id);
    assert_eq!(members[0].name, "kept");
}

#[test]
fn snapshot_round_trips_member() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateMember(CreateMember {
        id,
        project_id: pid,
        name: "Grace".into(),
    }))
    .unwrap();

    let snap = ws.export_snapshot().unwrap();
    assert!(snap.members.iter().any(|m| m.id == id && m.name == "Grace"));

    // Import into a fresh workspace; the member must come across.
    let mut fresh = Workspace::open_in_memory().unwrap();
    fresh
        .apply(Operation::ImportSnapshot(ImportSnapshot {
            snapshot: snap,
            policy: ConflictPolicy::Overwrite,
        }))
        .unwrap();
    let members = fresh.query_members_for_project(pid).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "Grace");
}

#[test]
fn migration_0003_applied_and_issue_allows_null_assignee() {
    let ws = Workspace::open_in_memory().unwrap();
    let conn = ws._conn_for_integration_tests();
    let has_v3: bool = conn
        .query_row(
            "SELECT 1 FROM schema_migrations WHERE version = 3",
            [],
            |_| Ok(true),
        )
        .unwrap();
    assert!(has_v3);

    // Insert a project/status/issue with a NULL assignee_id directly to prove
    // the column exists and is nullable.
    conn.execute(
        "INSERT INTO projects(id,name,prefix,status,next_seq,created_at,updated_at)
         VALUES('p','P','PPP','active',1,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO statuses(id,project_id,name,category,color,position)
         VALUES('s','p','Todo','unstarted','#000000',0)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,priority,due_date,sort_key,created_at,updated_at,assignee_id)
         VALUES('i','p',1,'PPP-1','t',NULL,'s','none',NULL,1.0,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z',NULL)",
        [],
    )
    .unwrap();
    let assignee: Option<String> = conn
        .query_row("SELECT assignee_id FROM issues WHERE id = 'i'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(assignee.is_none());
}
