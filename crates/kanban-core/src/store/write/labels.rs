use crate::error::Result;
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    project_id: Uuid,
    name: &str,
    color: &str,
) -> Result<()> {
    tx.execute(
        "INSERT INTO labels(id,project_id,name,color) VALUES (?1,?2,?3,?4)",
        params![id.to_string(), project_id.to_string(), name, color],
    )?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM labels WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

pub(crate) fn update_fields(
    tx: &Transaction<'_>,
    id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> Result<()> {
    if let Some(v) = name {
        tx.execute(
            "UPDATE labels SET name = ?1 WHERE id = ?2",
            params![v, id.to_string()],
        )?;
    }
    if let Some(v) = color {
        tx.execute(
            "UPDATE labels SET color = ?1 WHERE id = ?2",
            params![v, id.to_string()],
        )?;
    }
    Ok(())
}

pub(crate) fn attach(tx: &Transaction<'_>, issue_id: Uuid, label_id: Uuid) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
        params![issue_id.to_string(), label_id.to_string()],
    )?;
    Ok(())
}

pub(crate) fn detach(tx: &Transaction<'_>, issue_id: Uuid, label_id: Uuid) -> Result<()> {
    tx.execute(
        "DELETE FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
        params![issue_id.to_string(), label_id.to_string()],
    )?;
    Ok(())
}
