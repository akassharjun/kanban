use crate::error::{Error, Result};
use crate::types::{Project, ProjectStatus};
use chrono::DateTime;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn by_id(conn: &Connection, id: Uuid) -> Result<Project> {
    conn.query_row(
        "SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at
         FROM projects WHERE id = ?1",
        params![id.to_string()],
        row_to_project,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Project,
            id: id.to_string(),
        },
        other => other.into(),
    })
}

pub(crate) fn list_all(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at
         FROM projects ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], row_to_project)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn row_to_project(r: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let id_s: String = r.get(0)?;
    let status_s: String = r.get(5)?;
    let created_s: String = r.get(7)?;
    let updated_s: String = r.get(8)?;
    Ok(Project {
        id: Uuid::from_str(&id_s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: r.get(1)?,
        prefix: r.get(2)?,
        description: r.get(3)?,
        icon: r.get(4)?,
        status: parse_status(&status_s)?,
        next_seq: r.get(6)?,
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

fn parse_status(s: &str) -> rusqlite::Result<ProjectStatus> {
    match s {
        "active" => Ok(ProjectStatus::Active),
        "paused" => Ok(ProjectStatus::Paused),
        "completed" => Ok(ProjectStatus::Completed),
        "archived" => Ok(ProjectStatus::Archived),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown project status '{other}'"),
            )),
        )),
    }
}
