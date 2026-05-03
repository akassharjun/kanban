use crate::error::{Error, Result};
use crate::types::{Issue, Priority};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    project_id: Uuid,
    title: &str,
    description: Option<&str>,
    status_id: Uuid,
    priority: Priority,
    due_date: Option<NaiveDate>,
    sort_key: f64,
    now: DateTime<Utc>,
) -> Result<Issue> {
    let now_s = now.to_rfc3339();
    // Race-safe per-project sequence allocation.
    let seq: i64 = tx
        .query_row(
            "UPDATE projects SET next_seq = next_seq + 1 WHERE id = ?1
             RETURNING next_seq - 1",
            params![project_id.to_string()],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
                kind: crate::EntityKind::Project,
                id: project_id.to_string(),
            },
            other => other.into(),
        })?;

    let prefix: String = tx.query_row(
        "SELECT prefix FROM projects WHERE id = ?1",
        params![project_id.to_string()],
        |r| r.get(0),
    )?;
    let identifier = format!("{prefix}-{seq}");

    tx.execute(
        "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,
                            priority,due_date,sort_key,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)",
        params![
            id.to_string(),
            project_id.to_string(),
            seq,
            identifier,
            title,
            description,
            status_id.to_string(),
            priority.as_str(),
            due_date.map(|d| d.to_string()),
            sort_key,
            now_s,
        ],
    )?;
    Ok(Issue {
        id,
        project_id,
        seq,
        identifier,
        title: title.to_string(),
        description: description.map(str::to_string),
        status_id,
        priority,
        due_date,
        sort_key,
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM issues WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

// Used by Task 19+ (UpdateIssueField). Allow dead_code until wired up.
// Value is taken by value because rusqlite's params! consumes ToSql impls;
// owning the Value here keeps call sites simple (`Value::Text(s.clone())`).
#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_field(
    tx: &Transaction<'_>,
    id: Uuid,
    column: &str,
    sql_value: rusqlite::types::Value,
    now: DateTime<Utc>,
) -> Result<()> {
    let sql = format!("UPDATE issues SET {column} = ?1, updated_at = ?2 WHERE id = ?3");
    tx.execute(&sql, params![sql_value, now.to_rfc3339(), id.to_string()])?;
    Ok(())
}

// Used by Task 20+ (ReorderIssue). Allow dead_code until wired up.
#[allow(dead_code)]
pub(crate) fn set_sort_key(
    tx: &Transaction<'_>,
    id: Uuid,
    key: f64,
    now: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "UPDATE issues SET sort_key = ?1, updated_at = ?2 WHERE id = ?3",
        params![key, now.to_rfc3339(), id.to_string()],
    )?;
    Ok(())
}
