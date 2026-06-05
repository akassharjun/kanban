use serde::Serialize;
use specta::Type;
use thiserror::Error;

/// Errors returned by Tauri command handlers to the frontend.
#[allow(dead_code)] // consumed by command handlers added in later tasks
#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "kind")]
pub enum ApiError {
    /// The requested resource was not found.
    #[error("{resource} not found: {key}")]
    NotFound { resource: String, key: String },
    /// A field failed validation.
    #[error("validation: {field}: {message}")]
    Validation { field: String, message: String },
    /// The operation conflicts with existing state.
    #[error("conflict: {message}")]
    Conflict { message: String },
    /// An internal error occurred.
    #[error("internal: {message}")]
    Internal { message: String },
}

impl From<kanban_core::Error> for ApiError {
    fn from(e: kanban_core::Error) -> Self {
        use kanban_core::{EntityKind, Error};
        match e {
            Error::NotFound { kind, id } => ApiError::NotFound {
                resource: match kind {
                    EntityKind::Project => "project",
                    EntityKind::Issue => "issue",
                    EntityKind::Label => "label",
                    EntityKind::Status => "status",
                }
                .to_string(),
                key: id,
            },
            Error::Validation(v) => ApiError::Validation {
                field: v.field,
                message: v.reason,
            },
            Error::Conflict(msg) => ApiError::Conflict { message: msg },
            Error::Db(e) => ApiError::Internal {
                message: format!("db: {e}"),
            },
            Error::Io(e) => ApiError::Internal {
                message: format!("io: {e}"),
            },
            Error::Serde(e) => ApiError::Internal {
                message: format!("serde: {e}"),
            },
            Error::InvalidSnapshot(s) => ApiError::Validation {
                field: "snapshot".to_string(),
                message: s,
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use kanban_core::{EntityKind, Error, ValidationError};

    #[test]
    fn maps_not_found_with_resource_string() {
        let api: ApiError = Error::NotFound {
            kind: EntityKind::Issue,
            id: "AUTH-12".into(),
        }
        .into();
        match api {
            ApiError::NotFound { resource, key } => {
                assert_eq!(resource, "issue");
                assert_eq!(key, "AUTH-12");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn maps_validation_with_field() {
        let api: ApiError = Error::Validation(ValidationError {
            field: "prefix".into(),
            reason: "must be uppercase".into(),
        })
        .into();
        match api {
            ApiError::Validation { field, message } => {
                assert_eq!(field, "prefix");
                assert_eq!(message, "must be uppercase");
            }
            _ => panic!("wrong variant"),
        }
    }
}
