use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    project_id: Uuid,
    name: &str,
    created_at: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "INSERT INTO members(id,project_id,name,created_at) VALUES (?1,?2,?3,?4)",
        params![
            id.to_string(),
            project_id.to_string(),
            name,
            created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub(crate) fn update_fields(tx: &Transaction<'_>, id: Uuid, name: Option<&str>) -> Result<()> {
    if let Some(v) = name {
        tx.execute(
            "UPDATE members SET name = ?1 WHERE id = ?2",
            params![v, id.to_string()],
        )?;
    }
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM members WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}
