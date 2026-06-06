use crate::error::{Error, Result};
use crate::operation::{ConflictPolicy, ImportSnapshot, Operation};
use rusqlite::{Transaction, params};

pub(crate) fn import(tx: &Transaction<'_>, args: &ImportSnapshot) -> Result<()> {
    if args.snapshot.schema_version != crate::snapshot::SNAPSHOT_SCHEMA_VERSION {
        return Err(Error::InvalidSnapshot(format!(
            "schema {} not supported (expected {})",
            args.snapshot.schema_version,
            crate::snapshot::SNAPSHOT_SCHEMA_VERSION
        )));
    }

    for p in &args.snapshot.projects {
        upsert_project(tx, p, args.policy)?;
    }
    // Members must exist before issues that reference them via assignee_id.
    for m in &args.snapshot.members {
        upsert_member(tx, m, args.policy)?;
    }
    for s in &args.snapshot.statuses {
        upsert_status(tx, s, args.policy)?;
    }
    for l in &args.snapshot.labels {
        upsert_label(tx, l, args.policy)?;
    }
    for i in &args.snapshot.issues {
        upsert_issue(tx, i, args.policy)?;
    }
    for link in &args.snapshot.issue_labels {
        tx.execute(
            "INSERT OR IGNORE INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
            params![link.issue_id.to_string(), link.label_id.to_string()],
        )?;
    }
    Ok(())
}

fn exists<S: AsRef<str>>(tx: &Transaction<'_>, table: &str, id: S) -> Result<bool> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE id = ?1");
    let n: i64 = tx.query_row(&sql, params![id.as_ref()], |r| r.get(0))?;
    Ok(n > 0)
}

/// Returns `Ok(true)` if the caller should skip the row, `Ok(false)` if the
/// caller should overwrite, `Err(Conflict)` for `Fail`.
fn handle_conflict(
    policy: ConflictPolicy,
    kind: crate::error::EntityKind,
    id: &str,
) -> Result<bool> {
    match policy {
        ConflictPolicy::Skip => Ok(true),
        ConflictPolicy::Overwrite => Ok(false),
        ConflictPolicy::Fail => Err(Error::Conflict(format!("{kind} {id} already exists"))),
    }
}

fn upsert_project(
    tx: &Transaction<'_>,
    p: &crate::types::Project,
    policy: ConflictPolicy,
) -> Result<()> {
    let id = p.id.to_string();
    if exists(tx, "projects", &id)? {
        if handle_conflict(policy, crate::EntityKind::Project, &id)? {
            return Ok(());
        }
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO projects(id,name,prefix,description,icon,status,next_seq,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            id,
            p.name,
            p.prefix,
            p.description,
            p.icon,
            p.status.as_str(),
            p.next_seq,
            p.created_at.to_rfc3339(),
            p.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn upsert_status(
    tx: &Transaction<'_>,
    s: &crate::types::Status,
    policy: ConflictPolicy,
) -> Result<()> {
    let id = s.id.to_string();
    let pid = s.project_id.to_string();
    // A status row collides on either its primary key OR the (project_id, name)
    // unique constraint. Both have to be considered before the policy gate.
    let id_clash = exists(tx, "statuses", &id)?;
    let name_clash: i64 = tx.query_row(
        "SELECT COUNT(*) FROM statuses WHERE project_id = ?1 AND name = ?2",
        params![pid, s.name],
        |r| r.get(0),
    )?;
    if id_clash || name_clash > 0 {
        if handle_conflict(policy, crate::EntityKind::Status, &id)? {
            return Ok(());
        }
        if id_clash {
            tx.execute("DELETE FROM statuses WHERE id = ?1", params![id])?;
        }
        if name_clash > 0 {
            tx.execute(
                "DELETE FROM statuses WHERE project_id = ?1 AND name = ?2",
                params![pid, s.name],
            )?;
        }
    }
    tx.execute(
        "INSERT INTO statuses(id,project_id,name,category,color,position) VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            id,
            pid,
            s.name,
            s.category.as_str(),
            s.color,
            s.position,
        ],
    )?;
    Ok(())
}

fn upsert_label(
    tx: &Transaction<'_>,
    l: &crate::types::Label,
    policy: ConflictPolicy,
) -> Result<()> {
    let id = l.id.to_string();
    let pid = l.project_id.to_string();
    // Labels collide either on primary key or (project_id, name).
    let id_clash = exists(tx, "labels", &id)?;
    let name_clash: i64 = tx.query_row(
        "SELECT COUNT(*) FROM labels WHERE project_id = ?1 AND name = ?2",
        params![pid, l.name],
        |r| r.get(0),
    )?;
    if id_clash || name_clash > 0 {
        if handle_conflict(policy, crate::EntityKind::Label, &id)? {
            return Ok(());
        }
        if id_clash {
            tx.execute("DELETE FROM labels WHERE id = ?1", params![id])?;
        }
        if name_clash > 0 {
            tx.execute(
                "DELETE FROM labels WHERE project_id = ?1 AND name = ?2",
                params![pid, l.name],
            )?;
        }
    }
    tx.execute(
        "INSERT INTO labels(id,project_id,name,color) VALUES (?1,?2,?3,?4)",
        params![id, pid, l.name, l.color],
    )?;
    Ok(())
}

fn upsert_member(
    tx: &Transaction<'_>,
    m: &crate::types::Member,
    policy: ConflictPolicy,
) -> Result<()> {
    let id = m.id.to_string();
    let pid = m.project_id.to_string();
    // A member row collides on either its primary key OR the (project_id, name)
    // unique constraint. Both have to be considered before the policy gate.
    let id_clash = exists(tx, "members", &id)?;
    let name_clash: i64 = tx.query_row(
        "SELECT COUNT(*) FROM members WHERE project_id = ?1 AND name = ?2",
        params![pid, m.name],
        |r| r.get(0),
    )?;
    if id_clash || name_clash > 0 {
        if handle_conflict(policy, crate::EntityKind::Member, &id)? {
            return Ok(());
        }
        if id_clash {
            tx.execute("DELETE FROM members WHERE id = ?1", params![id])?;
        }
        if name_clash > 0 {
            tx.execute(
                "DELETE FROM members WHERE project_id = ?1 AND name = ?2",
                params![pid, m.name],
            )?;
        }
    }
    tx.execute(
        "INSERT INTO members(id,project_id,name,created_at) VALUES (?1,?2,?3,?4)",
        params![id, pid, m.name, m.created_at.to_rfc3339()],
    )?;
    Ok(())
}

fn upsert_issue(
    tx: &Transaction<'_>,
    i: &crate::types::Issue,
    policy: ConflictPolicy,
) -> Result<()> {
    let id = i.id.to_string();
    if exists(tx, "issues", &id)? {
        if handle_conflict(policy, crate::EntityKind::Issue, &id)? {
            return Ok(());
        }
        tx.execute("DELETE FROM issues WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,priority,
                            due_date,sort_key,created_at,updated_at,assignee_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![
            id,
            i.project_id.to_string(),
            i.seq,
            i.identifier,
            i.title,
            i.description,
            i.status_id.to_string(),
            i.priority.as_str(),
            i.due_date.map(|d| d.to_string()),
            i.sort_key,
            i.created_at.to_rfc3339(),
            i.updated_at.to_rfc3339(),
            i.assignee_id.map(|a| a.to_string()),
        ],
    )?;
    Ok(())
}

/// Documentation hook describing how `ImportSnapshot` is inverted.
///
/// `ImportSnapshot` is special-cased in [`crate::apply::Workspace::apply`]:
/// the inverse is captured by exporting a snapshot of the *pre-import* state
/// and wrapping it as another `ImportSnapshot { policy: Overwrite }`. This
/// helper is therefore never called from `capture_inverse`; it exists to make
/// that contract explicit at compile time.
#[allow(dead_code)]
pub(crate) fn inverse_of_import(
    _tx: &Transaction<'_>,
    _args: &ImportSnapshot,
) -> Result<Operation> {
    Err(Error::InvalidSnapshot(
        "ImportSnapshot inverse computed via pre-snapshot capture, not in capture_inverse".into(),
    ))
}

// ----- Subtree exporters used by `inverse_of_delete_*` to produce an
// `ImportSnapshot` operation that round-trips the deleted entity (and all
// cascaded children) bit-exactly. Each leaves arrays for entities not in the
// subtree empty, which is a no-op when fed back through `import`. -----

/// Capture the project row + its statuses + its labels + its issues + the
/// `issue_labels` rows for issues in the project. Used to build the inverse of
/// `DeleteProject` so undo restores `status`, `next_seq`, timestamps, child
/// statuses/labels/issues, and label attachments — none of which a plain
/// `CreateProject` op would carry.
pub(crate) fn export_project_subtree_via_tx(
    tx: &Transaction<'_>,
    project_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{IssueLabelLink, SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let project = crate::store::read::projects::by_id_via_tx(tx, project_id)?;
    let members = crate::store::read::members::for_project_via_tx(tx, project_id)?;
    let statuses = crate::store::read::statuses::for_project_via_tx(tx, project_id)?;
    let labels = crate::store::read::labels::for_project_via_tx(tx, project_id)?;

    let mut issues = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id,project_id,seq,identifier,title,description,status_id,priority,
                    due_date,sort_key,created_at,updated_at,assignee_id
             FROM issues WHERE project_id = ?1",
        )?;
        let rows = stmt.query_map(
            params![project_id.to_string()],
            crate::store::read::issues::row_to_issue,
        )?;
        for r in rows {
            issues.push(r?);
        }
    }

    let mut issue_labels = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT il.issue_id, il.label_id
             FROM issue_labels il JOIN issues i ON i.id = il.issue_id
             WHERE i.project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id.to_string()], |r| {
            let issue_id_s: String = r.get(0)?;
            let label_id_s: String = r.get(1)?;
            Ok((issue_id_s, label_id_s))
        })?;
        for r in rows {
            let (iid, lid) = r?;
            issue_labels.push(IssueLabelLink {
                issue_id: uuid::Uuid::parse_str(&iid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.issue_id is not a uuid: {e}"))
                })?,
                label_id: uuid::Uuid::parse_str(&lid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.label_id is not a uuid: {e}"))
                })?,
            });
        }
    }

    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: vec![project],
        members,
        statuses,
        labels,
        issues,
        issue_labels,
    })
}

/// Capture the issue row + all `issue_labels` rows for that issue. Used to
/// build the inverse of `DeleteIssue` so undo restores `seq`/`identifier`/
/// `sort_key`/timestamps and label attachments.
pub(crate) fn export_issue_subtree_via_tx(
    tx: &Transaction<'_>,
    issue_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{IssueLabelLink, SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let issue = crate::store::read::issues::by_id_via_tx(tx, issue_id)?;

    let mut issue_labels = Vec::new();
    {
        let mut stmt =
            tx.prepare("SELECT issue_id, label_id FROM issue_labels WHERE issue_id = ?1")?;
        let rows = stmt.query_map(params![issue_id.to_string()], |r| {
            let issue_id_s: String = r.get(0)?;
            let label_id_s: String = r.get(1)?;
            Ok((issue_id_s, label_id_s))
        })?;
        for r in rows {
            let (iid, lid) = r?;
            issue_labels.push(IssueLabelLink {
                issue_id: uuid::Uuid::parse_str(&iid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.issue_id is not a uuid: {e}"))
                })?,
                label_id: uuid::Uuid::parse_str(&lid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.label_id is not a uuid: {e}"))
                })?,
            });
        }
    }

    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: Vec::new(),
        members: Vec::new(),
        statuses: Vec::new(),
        labels: Vec::new(),
        issues: vec![issue],
        issue_labels,
    })
}

/// Capture a single status row as a one-element snapshot. Used to build the
/// inverse of `DeleteStatus` so undo restores `category`/`color`/`position`
/// and the original id — none of which a plain `CreateStatus` op would carry
/// without re-deriving them. `snapshot::import` upserts `snapshot.statuses`.
pub(crate) fn export_status_row(
    tx: &Transaction<'_>,
    status_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let status = crate::store::read::statuses::by_id_via_tx(tx, status_id)?;
    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: Vec::new(),
        members: Vec::new(),
        statuses: vec![status],
        labels: Vec::new(),
        issues: Vec::new(),
        issue_labels: Vec::new(),
    })
}

/// Capture all of a project's status rows as a snapshot (other entity arrays
/// empty). Used to build the inverse of `ReorderStatus`: re-importing every
/// status row under `Overwrite` restores all prior `position` values.
pub(crate) fn export_project_statuses(
    tx: &Transaction<'_>,
    project_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let statuses = crate::store::read::statuses::for_project_via_tx(tx, project_id)?;
    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: Vec::new(),
        members: Vec::new(),
        statuses,
        labels: Vec::new(),
        issues: Vec::new(),
        issue_labels: Vec::new(),
    })
}

/// Capture the label row + all `issue_labels` rows that reference it. Used to
/// build the inverse of `DeleteLabel` so undo restores attachments that the
/// CASCADE delete tore down.
pub(crate) fn export_label_subtree_via_tx(
    tx: &Transaction<'_>,
    label_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{IssueLabelLink, SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let label = crate::store::read::labels::by_id_via_tx(tx, label_id)?;

    let mut issue_labels = Vec::new();
    {
        let mut stmt =
            tx.prepare("SELECT issue_id, label_id FROM issue_labels WHERE label_id = ?1")?;
        let rows = stmt.query_map(params![label_id.to_string()], |r| {
            let issue_id_s: String = r.get(0)?;
            let label_id_s: String = r.get(1)?;
            Ok((issue_id_s, label_id_s))
        })?;
        for r in rows {
            let (iid, lid) = r?;
            issue_labels.push(IssueLabelLink {
                issue_id: uuid::Uuid::parse_str(&iid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.issue_id is not a uuid: {e}"))
                })?,
                label_id: uuid::Uuid::parse_str(&lid).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.label_id is not a uuid: {e}"))
                })?,
            });
        }
    }

    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: Vec::new(),
        members: Vec::new(),
        statuses: Vec::new(),
        labels: vec![label],
        issues: Vec::new(),
        issue_labels,
    })
}

/// Capture the member row + every issue in the member's project whose
/// `assignee_id` references it. Used to build the inverse of `DeleteMember` so
/// undo restores the member AND re-applies the assignments that the
/// `ON DELETE SET NULL` foreign key cleared.
pub(crate) fn export_member_subtree(
    tx: &Transaction<'_>,
    member_id: uuid::Uuid,
) -> Result<crate::snapshot::WorkspaceSnapshot> {
    use crate::snapshot::{IssueLabelLink, SNAPSHOT_SCHEMA_VERSION, WorkspaceSnapshot};

    let member = crate::store::read::members::by_id_via_tx(tx, member_id)?;

    let mut issues = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id,project_id,seq,identifier,title,description,status_id,priority,
                    due_date,sort_key,created_at,updated_at,assignee_id
             FROM issues WHERE assignee_id = ?1",
        )?;
        let rows = stmt.query_map(
            params![member_id.to_string()],
            crate::store::read::issues::row_to_issue,
        )?;
        for r in rows {
            issues.push(r?);
        }
    }

    // Capture the assigned issues' label attachments too: undoing the delete
    // re-imports those issues under Overwrite, which deletes+reinserts each issue
    // row and cascades away its `issue_labels` — so they must be restored here.
    let mut issue_labels = Vec::new();
    {
        let mut stmt = tx.prepare("SELECT label_id FROM issue_labels WHERE issue_id = ?1")?;
        for issue in &issues {
            let rows = stmt.query_map(params![issue.id.to_string()], |r| r.get::<_, String>(0))?;
            for r in rows {
                let label_id = uuid::Uuid::parse_str(&r?).map_err(|e| {
                    Error::InvalidSnapshot(format!("issue_labels.label_id is not a uuid: {e}"))
                })?;
                issue_labels.push(IssueLabelLink {
                    issue_id: issue.id,
                    label_id,
                });
            }
        }
    }

    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects: Vec::new(),
        members: vec![member],
        statuses: Vec::new(),
        labels: Vec::new(),
        issues,
        issue_labels,
    })
}
