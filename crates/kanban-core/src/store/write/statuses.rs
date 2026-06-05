use crate::error::Result;
use crate::ids::new_id;
use crate::types::StatusCategory;
use rusqlite::{Transaction, params};
use uuid::Uuid;

const DEFAULTS: &[(&str, StatusCategory, &str, i64)] = &[
    ("Todo", StatusCategory::Unstarted, "#94a3b8", 0),
    ("Backlog", StatusCategory::Unstarted, "#64748b", 1),
    ("In Progress", StatusCategory::Started, "#3b82f6", 2),
    ("In Review", StatusCategory::Started, "#a855f7", 3),
    ("Blocked", StatusCategory::Blocked, "#ef4444", 4),
    ("Discarded", StatusCategory::Discarded, "#6b7280", 5),
    ("Done", StatusCategory::Completed, "#22c55e", 6),
];

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    project_id: Uuid,
    name: &str,
    category: StatusCategory,
    color: &str,
    position: i64,
) -> Result<()> {
    tx.execute(
        "INSERT INTO statuses(id,project_id,name,category,color,position) VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            id.to_string(),
            project_id.to_string(),
            name,
            category.as_str(),
            color,
            position
        ],
    )?;
    Ok(())
}

pub(crate) fn update_fields(
    tx: &Transaction<'_>,
    id: Uuid,
    name: Option<&str>,
    category: Option<StatusCategory>,
    color: Option<&str>,
) -> Result<()> {
    if let Some(v) = name {
        tx.execute(
            "UPDATE statuses SET name = ?1 WHERE id = ?2",
            params![v, id.to_string()],
        )?;
    }
    if let Some(v) = category {
        tx.execute(
            "UPDATE statuses SET category = ?1 WHERE id = ?2",
            params![v.as_str(), id.to_string()],
        )?;
    }
    if let Some(v) = color {
        tx.execute(
            "UPDATE statuses SET color = ?1 WHERE id = ?2",
            params![v, id.to_string()],
        )?;
    }
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute(
        "DELETE FROM statuses WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(())
}

pub(crate) fn seed_defaults(tx: &Transaction<'_>, project_id: Uuid) -> Result<Vec<Uuid>> {
    let mut ids = Vec::with_capacity(DEFAULTS.len());
    for (name, category, color, position) in DEFAULTS {
        let id = new_id();
        tx.execute(
            "INSERT INTO statuses(id,project_id,name,category,color,position)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                id.to_string(),
                project_id.to_string(),
                name,
                category.as_str(),
                color,
                position
            ],
        )?;
        ids.push(id);
    }
    Ok(ids)
}
