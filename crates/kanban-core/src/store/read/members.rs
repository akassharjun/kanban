use crate::error::{Error, Result};
use crate::types::Member;
use chrono::DateTime;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Member>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,name,created_at FROM members WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_member)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub(crate) fn for_project_via_tx(
    tx: &rusqlite::Transaction<'_>,
    project_id: Uuid,
) -> Result<Vec<Member>> {
    let mut stmt = tx.prepare(
        "SELECT id,project_id,name,created_at FROM members WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_member)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Member> {
    tx.query_row(
        "SELECT id,project_id,name,created_at FROM members WHERE id = ?1",
        params![id.to_string()],
        row_to_member,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Member,
            id: id.to_string(),
        },
        other => other.into(),
    })
}

fn row_to_member(r: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    let created_s: String = r.get(3)?;
    Ok(Member {
        id: Uuid::from_str(&id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        project_id: Uuid::from_str(&pid).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: r.get(2)?,
        created_at: DateTime::parse_from_rfc3339(&created_s)
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
