use crate::error::{Error, Result};
use crate::types::Label;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Label>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,name,color FROM labels WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_label)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub(crate) fn for_project_via_tx(
    tx: &rusqlite::Transaction<'_>,
    project_id: Uuid,
) -> Result<Vec<Label>> {
    let mut stmt = tx.prepare(
        "SELECT id,project_id,name,color FROM labels WHERE project_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_label)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Label> {
    tx.query_row(
        "SELECT id,project_id,name,color FROM labels WHERE id = ?1",
        params![id.to_string()],
        row_to_label,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Label,
            id: id.to_string(),
        },
        other => other.into(),
    })
}

fn row_to_label(r: &rusqlite::Row<'_>) -> rusqlite::Result<Label> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    Ok(Label {
        id: Uuid::from_str(&id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        project_id: Uuid::from_str(&pid).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: r.get(2)?,
        color: r.get(3)?,
    })
}
