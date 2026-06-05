//! Map kanban-core errors (and resolution misses) to MCP tool errors.
// `not_found`/`unknown_name` are consumed by the resolution tools from Task 5
// onward; `to_mcp` is used by the server now.
#![allow(dead_code)]
use rmcp::ErrorData as McpError;

/// Convert a `kanban_core::Error` into an MCP tool error with an LLM-actionable message.
#[must_use]
pub fn to_mcp(err: kanban_core::Error) -> McpError {
    use kanban_core::Error;
    match err {
        Error::NotFound { kind, id } => {
            McpError::resource_not_found(format!("{kind} not found: {id}"), None)
        }
        Error::Validation(v) => {
            McpError::invalid_params(format!("{}: {}", v.field, v.reason), None)
        }
        Error::Conflict(msg) => McpError::invalid_request(msg, None),
        Error::Db(e) => McpError::internal_error(format!("db: {e}"), None),
        Error::Io(e) => McpError::internal_error(format!("io: {e}"), None),
        Error::Serde(e) => McpError::internal_error(format!("serde: {e}"), None),
        Error::InvalidSnapshot(s) => McpError::invalid_params(format!("snapshot: {s}"), None),
    }
}

/// A "not found by human identifier" error (project prefix / issue key).
#[must_use]
pub fn not_found(resource: &str, key: &str) -> McpError {
    McpError::resource_not_found(format!("{resource} not found: {key}"), None)
}

/// A "name not found within a project" error that lists the valid options.
#[must_use]
pub fn unknown_name(kind: &str, name: &str, project: &str, available: &[String]) -> McpError {
    McpError::invalid_params(
        format!(
            "{kind} '{name}' not found in project {project}; available: {}",
            available.join(", ")
        ),
        None,
    )
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn validation_maps_to_invalid_params_with_field() {
        let err = kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "prefix".into(),
            reason: "must be 2-8 uppercase letters".into(),
        });
        let mcp = to_mcp(err);
        let rendered = format!("{mcp:?}");
        assert!(rendered.contains("prefix"), "got: {rendered}");
        assert!(rendered.contains("uppercase"), "got: {rendered}");
    }

    #[test]
    fn unknown_name_lists_available() {
        let mcp = unknown_name("status", "Doing", "AUTH", &["To Do".into(), "Done".into()]);
        let rendered = format!("{mcp:?}");
        assert!(rendered.contains("Doing"), "got: {rendered}");
        assert!(rendered.contains("To Do, Done"), "got: {rendered}");
    }
}
