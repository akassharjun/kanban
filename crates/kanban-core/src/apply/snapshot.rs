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
                            due_date,sort_key,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
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
