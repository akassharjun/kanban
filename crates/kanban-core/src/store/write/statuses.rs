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
