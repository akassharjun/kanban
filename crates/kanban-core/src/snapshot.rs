use crate::types::{Issue, Label, Project, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub projects: Vec<Project>,
    pub statuses: Vec<Status>,
    pub issues: Vec<Issue>,
    pub labels: Vec<Label>,
    pub issue_labels: Vec<IssueLabelLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueLabelLink {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trips_via_json() {
        let snap = WorkspaceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            exported_at: Utc::now(),
            projects: vec![],
            statuses: vec![],
            issues: vec![],
            labels: vec![],
            issue_labels: vec![],
        };
        let s = serde_json::to_string(&snap).unwrap();
        let back: WorkspaceSnapshot = serde_json::from_str(&s).unwrap();
        assert_eq!(back.schema_version, 1);
    }
}
