use crate::error::{Error, Result};
use crate::types::Issue;
use chrono::{DateTime, NaiveDate};
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn by_id(conn: &Connection, id: Uuid) -> Result<Issue> {
    conn.query_row(ISSUE_SELECT, params![id.to_string()], row_to_issue)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
                kind: crate::EntityKind::Issue,
                id: id.to_string(),
            },
            other => other.into(),
        })
}

pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Issue> {
    tx.query_row(ISSUE_SELECT, params![id.to_string()], row_to_issue)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
                kind: crate::EntityKind::Issue,
                id: id.to_string(),
            },
            other => other.into(),
        })
}

pub(crate) fn list(conn: &Connection, filter: &crate::query::IssueFilter) -> Result<Vec<Issue>> {
    let (sql, params) = filter.build_sql(ISSUE_LIST_BASE);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), row_to_issue)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

const ISSUE_SELECT: &str = "
SELECT id,project_id,seq,identifier,title,description,status_id,priority,due_date,sort_key,created_at,updated_at
FROM issues WHERE id = ?1";

pub(crate) const ISSUE_LIST_BASE: &str = "
SELECT id,project_id,seq,identifier,title,description,status_id,priority,due_date,sort_key,created_at,updated_at
FROM issues";

/// Same projection as [`ISSUE_LIST_BASE`] but with all columns qualified by
/// `issues.`. Used by FTS5-joined queries where the `issue_search` virtual
/// table also exposes `title`/`description`, making unqualified references
/// ambiguous.
pub(crate) const ISSUE_LIST_BASE_QUALIFIED: &str = "
SELECT issues.id,issues.project_id,issues.seq,issues.identifier,issues.title,issues.description,issues.status_id,issues.priority,issues.due_date,issues.sort_key,issues.created_at,issues.updated_at
FROM issues";

pub(crate) fn row_to_issue(r: &rusqlite::Row<'_>) -> rusqlite::Result<Issue> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    let sid: String = r.get(6)?;
    let priority_s: String = r.get(7)?;
    let due_s: Option<String> = r.get(8)?;
    let created_s: String = r.get(10)?;
    let updated_s: String = r.get(11)?;
    Ok(Issue {
        id: Uuid::from_str(&id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        project_id: Uuid::from_str(&pid).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        seq: r.get(2)?,
        identifier: r.get(3)?,
        title: r.get(4)?,
        description: r.get(5)?,
        status_id: Uuid::from_str(&sid).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        priority: priority_s.parse().map_err(|e: crate::error::Error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?,
        due_date: due_s
            .map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })
            .transpose()?,
        sort_key: r.get(9)?,
        created_at: DateTime::parse_from_rfc3339(&created_s)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&chrono::Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_s)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&chrono::Utc),
    })
}
