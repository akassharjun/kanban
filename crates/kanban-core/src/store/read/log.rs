use crate::error::Result;
use crate::types::ActivityEntry;
use chrono::DateTime;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_issue(conn: &Connection, issue_id: Uuid) -> Result<Vec<ActivityEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, op_id, issue_id, field, old_value, new_value, at
         FROM activity_log WHERE issue_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![issue_id.to_string()], row_to_entry)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityEntry> {
    let issue_id: Option<String> = r.get(2)?;
    let at_s: String = r.get(6)?;
    Ok(ActivityEntry {
        id: r.get(0)?,
        op_id: r.get(1)?,
        issue_id: issue_id
            .map(|s| {
                Uuid::from_str(&s).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })
            .transpose()?,
        field: r.get(3)?,
        old_value: r.get(4)?,
        new_value: r.get(5)?,
        at: DateTime::parse_from_rfc3339(&at_s)
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
