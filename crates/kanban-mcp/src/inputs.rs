//! Tool input parameter structs (`serde::Deserialize` + `schemars::JsonSchema`).
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectRef {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
}
