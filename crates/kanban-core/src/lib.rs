//! kanban-core: data model, applier, queries, migrations.

pub mod apply;
pub mod error;
pub mod ids;
pub mod operation;
mod store;
pub mod time;
pub mod types;
pub mod validate;
pub mod workspace;

pub use error::{EntityKind, Error, Result, ValidationError};
pub use ids::{format_identifier, new_id};
pub use operation::{Operation, OperationOutcome};
pub use time::{Clock, FixedClock, SystemClock, system_clock};
pub use types::{Issue, Label, Priority, Project, ProjectStatus, Status, StatusCategory};
pub use workspace::{Workspace, WorkspacePath};
