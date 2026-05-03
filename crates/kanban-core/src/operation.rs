use crate::types::{Priority, ProjectStatus};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
pub enum Operation {
    CreateProject(CreateProject),
    UpdateProject(UpdateProject),
    ArchiveProject(ArchiveProject),
    DeleteProject(DeleteProject),

    CreateIssue(CreateIssue),
    UpdateIssueField(UpdateIssueField),
    ReorderIssue(ReorderIssue),
    DeleteIssue(DeleteIssue),

    CreateLabel(CreateLabel),
    UpdateLabel(UpdateLabel),
    DeleteLabel(DeleteLabel),
    AttachLabel(AttachLabel),
    DetachLabel(DetachLabel),

    ImportSnapshot(ImportSnapshot),
}

// ----- Project ops -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateProject {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateProject {
    pub id: Uuid,
    pub patch: ProjectPatch,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectPatch {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub status: Option<ProjectStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveProject {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteProject {
    pub id: Uuid,
}

// ----- Issue ops (placeholders to keep enum cohesive; implemented in Phase 8) -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIssue {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status_id: Uuid,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIssueField {
    pub id: Uuid,
    pub change: IssueFieldChange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "field", content = "value")]
pub enum IssueFieldChange {
    Title(String),
    Description(Option<String>),
    Status(Uuid),
    Priority(Priority),
    DueDate(Option<NaiveDate>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorderIssue {
    pub id: Uuid,
    pub new_sort_key: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIssue {
    pub id: Uuid,
}

// ----- Label ops (placeholders for Phase 9) -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLabel {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateLabel {
    pub id: Uuid,
    pub patch: LabelPatch,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LabelPatch {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLabel {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachLabel {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetachLabel {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationOutcome {
    pub op_id: i64,
}

// ----- Snapshot ops -----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    Skip,
    Overwrite,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportSnapshot {
    pub snapshot: crate::snapshot::WorkspaceSnapshot,
    pub policy: ConflictPolicy,
}
