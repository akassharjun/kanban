//! Exit-code mapping from `kanban_core::Error` to process exit codes.
//!
//! Stable contract:
//!
//! | Code | Meaning             |
//! |------|---------------------|
//! | 0    | Success             |
//! | 1    | User error (clap)   |
//! | 2    | Not found           |
//! | 3    | Validation/conflict |
//! | 4    | Internal            |

use kanban_core::Error;

pub const EXIT_OK: i32 = 0;
#[allow(dead_code)]
pub const EXIT_USER: i32 = 1;
pub const EXIT_NOT_FOUND: i32 = 2;
pub const EXIT_VALIDATION: i32 = 3;
pub const EXIT_INTERNAL: i32 = 4;

/// Map a [`kanban_core::Error`] to its stable process exit code.
#[must_use]
pub fn code_for(err: &Error) -> i32 {
    match err {
        Error::Validation(_) | Error::Conflict(_) => EXIT_VALIDATION,
        Error::NotFound { .. } => EXIT_NOT_FOUND,
        Error::Db(_) | Error::Io(_) | Error::Serde(_) | Error::InvalidSnapshot(_) => EXIT_INTERNAL,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use kanban_core::{EntityKind, ValidationError};

    #[test]
    fn validation_maps_to_3() {
        let e = Error::Validation(ValidationError {
            field: "x".into(),
            reason: "y".into(),
        });
        assert_eq!(code_for(&e), EXIT_VALIDATION);
    }

    #[test]
    fn conflict_maps_to_3() {
        let e = Error::Conflict("dup".into());
        assert_eq!(code_for(&e), EXIT_VALIDATION);
    }

    #[test]
    fn not_found_maps_to_2() {
        let e = Error::NotFound {
            kind: EntityKind::Project,
            id: "X".into(),
        };
        assert_eq!(code_for(&e), EXIT_NOT_FOUND);
    }

    #[test]
    fn invalid_snapshot_maps_to_4() {
        let e = Error::InvalidSnapshot("bad".into());
        assert_eq!(code_for(&e), EXIT_INTERNAL);
    }
}
