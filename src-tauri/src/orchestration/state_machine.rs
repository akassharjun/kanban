use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Claimed,
    Executing,
    Validating,
    Completed,
    Blocked,
    Cancelled,
}

impl TaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Queued => "queued",
            TaskState::Claimed => "claimed",
            TaskState::Executing => "executing",
            TaskState::Validating => "validating",
            TaskState::Completed => "completed",
            TaskState::Blocked => "blocked",
            TaskState::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "queued" => Ok(TaskState::Queued),
            "claimed" => Ok(TaskState::Claimed),
            "executing" => Ok(TaskState::Executing),
            "validating" => Ok(TaskState::Validating),
            "completed" => Ok(TaskState::Completed),
            "blocked" => Ok(TaskState::Blocked),
            "cancelled" => Ok(TaskState::Cancelled),
            other => Err(format!("unknown task state: '{}'", other)),
        }
    }

    pub fn valid_transitions(&self) -> &'static [TaskState] {
        match self {
            TaskState::Queued => &[TaskState::Claimed, TaskState::Blocked, TaskState::Cancelled],
            TaskState::Claimed => &[
                TaskState::Executing,
                TaskState::Queued,
                TaskState::Blocked,
                TaskState::Cancelled,
            ],
            TaskState::Executing => &[
                TaskState::Validating,
                TaskState::Queued,
                TaskState::Blocked,
                TaskState::Cancelled,
            ],
            TaskState::Validating => &[
                TaskState::Completed,
                TaskState::Queued,
                TaskState::Blocked,
                TaskState::Cancelled,
            ],
            TaskState::Completed => &[TaskState::Queued, TaskState::Cancelled],
            TaskState::Blocked => &[TaskState::Queued, TaskState::Cancelled],
            TaskState::Cancelled => &[],
        }
    }

    pub fn can_transition_to(&self, target: TaskState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Maps a TaskState to the corresponding issue status category string.
pub fn task_state_to_status_category(state: TaskState) -> &'static str {
    match state {
        TaskState::Queued => "unstarted",
        TaskState::Claimed | TaskState::Executing | TaskState::Validating => "started",
        TaskState::Completed => "completed",
        TaskState::Blocked => "blocked",
        TaskState::Cancelled => "discarded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str_roundtrip() {
        let states = [
            TaskState::Queued,
            TaskState::Claimed,
            TaskState::Executing,
            TaskState::Validating,
            TaskState::Completed,
            TaskState::Blocked,
            TaskState::Cancelled,
        ];
        for state in states {
            let s = state.as_str();
            let parsed = TaskState::from_str(s).unwrap();
            assert_eq!(state, parsed);
        }
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(TaskState::from_str("invalid").is_err());
    }

    #[test]
    fn test_cancelled_is_terminal() {
        assert!(TaskState::Cancelled.valid_transitions().is_empty());
        assert!(!TaskState::Cancelled.can_transition_to(TaskState::Queued));
    }

    #[test]
    fn test_queued_transitions() {
        assert!(TaskState::Queued.can_transition_to(TaskState::Claimed));
        assert!(TaskState::Queued.can_transition_to(TaskState::Blocked));
        assert!(TaskState::Queued.can_transition_to(TaskState::Cancelled));
        assert!(!TaskState::Queued.can_transition_to(TaskState::Executing));
    }

    #[test]
    fn test_status_category_mapping() {
        assert_eq!(task_state_to_status_category(TaskState::Queued), "unstarted");
        assert_eq!(task_state_to_status_category(TaskState::Claimed), "started");
        assert_eq!(task_state_to_status_category(TaskState::Executing), "started");
        assert_eq!(task_state_to_status_category(TaskState::Validating), "started");
        assert_eq!(task_state_to_status_category(TaskState::Completed), "completed");
        assert_eq!(task_state_to_status_category(TaskState::Blocked), "blocked");
        assert_eq!(task_state_to_status_category(TaskState::Cancelled), "discarded");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TaskState::Executing), "executing");
    }
}
