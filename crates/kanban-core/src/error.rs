use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("validation failed: {0}")]
    Validation(ValidationError),

    #[error("{kind} not found: {id}")]
    NotFound { kind: EntityKind, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid snapshot: {0}")]
    InvalidSnapshot(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Project,
    Issue,
    Label,
    Status,
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Project => f.write_str("Project"),
            EntityKind::Issue => f.write_str("Issue"),
            EntityKind::Label => f.write_str("Label"),
            EntityKind::Status => f.write_str("Status"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_displays_field_and_reason() {
        let err = Error::Validation(ValidationError {
            field: "prefix".to_string(),
            reason: "must be 3+ uppercase letters".to_string(),
        });
        let msg = err.to_string();
        assert!(msg.contains("prefix"), "got: {msg}");
        assert!(msg.contains("must be 3+ uppercase letters"), "got: {msg}");
    }

    #[test]
    fn not_found_displays_kind_and_id() {
        let err = Error::NotFound {
            kind: EntityKind::Issue,
            id: "KAN-42".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Issue"), "got: {msg}");
        assert!(msg.contains("KAN-42"), "got: {msg}");
    }
}
