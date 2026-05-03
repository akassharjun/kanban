use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    None,
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::None => "none",
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    /// Sort key (lower = higher priority). `None` sorts last.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Priority::Urgent => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
            Priority::None => 4,
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Priority::None),
            "urgent" => Ok(Priority::Urgent),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            other => Err(crate::error::Error::Validation(crate::error::ValidationError {
                field: "priority".into(),
                reason: format!("unknown value '{other}'"),
            })),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

impl ProjectStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Paused => "paused",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusCategory {
    Unstarted,
    Started,
    Blocked,
    Completed,
    Discarded,
}

impl StatusCategory {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            StatusCategory::Unstarted => "unstarted",
            StatusCategory::Started => "started",
            StatusCategory::Blocked => "blocked",
            StatusCategory::Completed => "completed",
            StatusCategory::Discarded => "discarded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: ProjectStatus,
    pub next_seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub category: StatusCategory,
    pub color: String,
    pub position: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub project_id: Uuid,
    pub seq: i64,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub status_id: Uuid,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub sort_key: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_round_trips_via_str() {
        for p in [
            Priority::None,
            Priority::Urgent,
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ] {
            let s = p.as_str();
            let parsed: Priority = s.parse().unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn priority_rank_orders_correctly() {
        assert!(Priority::Urgent.rank() < Priority::High.rank());
        assert!(Priority::High.rank() < Priority::Medium.rank());
        assert!(Priority::Medium.rank() < Priority::Low.rank());
        assert!(Priority::Low.rank() < Priority::None.rank());
    }

    #[test]
    fn priority_from_unknown_str_errors() {
        let err: Result<Priority, _> = "wat".parse();
        assert!(err.is_err());
    }

    #[test]
    fn project_status_as_str_round_trips_via_serde() {
        let s = serde_json::to_string(&ProjectStatus::Active).unwrap();
        assert_eq!(s, "\"active\"");
        let p: ProjectStatus = serde_json::from_str("\"paused\"").unwrap();
        assert_eq!(p, ProjectStatus::Paused);
    }
}
