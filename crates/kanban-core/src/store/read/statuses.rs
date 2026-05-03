use crate::error::Result;
use crate::types::{Status, StatusCategory};
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Status>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,name,category,color,position FROM statuses
         WHERE project_id = ?1 ORDER BY position ASC",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_status)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn row_to_status(r: &rusqlite::Row<'_>) -> rusqlite::Result<Status> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    let category_s: String = r.get(3)?;
    Ok(Status {
        id: Uuid::from_str(&id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        project_id: Uuid::from_str(&pid).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: r.get(2)?,
        category: parse_category(&category_s)?,
        color: r.get(4)?,
        position: r.get(5)?,
    })
}

fn parse_category(s: &str) -> rusqlite::Result<StatusCategory> {
    match s {
        "unstarted" => Ok(StatusCategory::Unstarted),
        "started" => Ok(StatusCategory::Started),
        "blocked" => Ok(StatusCategory::Blocked),
        "completed" => Ok(StatusCategory::Completed),
        "discarded" => Ok(StatusCategory::Discarded),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown status category '{other}'"),
            )),
        )),
    }
}
