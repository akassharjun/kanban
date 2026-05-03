use crate::error::Result;
use crate::types::{Project, ProjectStatus};
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    name: &str,
    prefix: &str,
    description: Option<&str>,
    icon: Option<&str>,
    now: DateTime<Utc>,
) -> Result<Project> {
    let now_s = now.to_rfc3339();
    tx.execute(
        "INSERT INTO projects(id,name,prefix,description,icon,status,next_seq,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,'active',1,?6,?6)",
        params![id.to_string(), name, prefix, description, icon, now_s],
    )?;
    Ok(Project {
        id,
        name: name.to_string(),
        prefix: prefix.to_string(),
        description: description.map(str::to_string),
        icon: icon.map(str::to_string),
        status: ProjectStatus::Active,
        next_seq: 1,
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute(
        "DELETE FROM projects WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(())
}

pub(crate) fn set_status(
    tx: &Transaction<'_>,
    id: Uuid,
    status: ProjectStatus,
    now: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "UPDATE projects SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), now.to_rfc3339(), id.to_string()],
    )?;
    Ok(())
}

// `Option<Option<&str>>` is a deliberate tri-state: outer `None` means "leave alone",
// `Some(None)` means "set NULL", `Some(Some(v))` means "set to v".
#[allow(clippy::option_option)]
pub(crate) fn update_fields(
    tx: &Transaction<'_>,
    id: Uuid,
    name: Option<&str>,
    description: Option<Option<&str>>,
    icon: Option<Option<&str>>,
    status: Option<ProjectStatus>,
    now: DateTime<Utc>,
) -> Result<()> {
    if let Some(v) = name {
        tx.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = description {
        tx.execute(
            "UPDATE projects SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = icon {
        tx.execute(
            "UPDATE projects SET icon = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = status {
        set_status(tx, id, v, now)?;
    }
    Ok(())
}
