use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub all_passed: bool,
    pub checks: Vec<ValidationCheck>,
}

/// Run validation pipeline for a task's success criteria.
/// Criteria with "command" and "expect" fields are executed as shell commands.
/// Returns ValidationResult with per-check pass/fail.
pub async fn run_validation_pipeline(
    pool: &SqlitePool,
    issue_id: i64,
) -> Result<ValidationResult, sqlx::Error> {
    let contract = sqlx::query_as::<_, crate::models::TaskContract>(
        "SELECT * FROM task_contracts WHERE issue_id = ?",
    )
    .bind(issue_id)
    .fetch_one(pool)
    .await?;

    let criteria: serde_json::Value =
        serde_json::from_str(&contract.success_criteria).unwrap_or(serde_json::json!([]));

    let criteria_arr = match criteria.as_array() {
        Some(arr) => arr.clone(),
        None => {
            return Ok(ValidationResult {
                all_passed: true,
                checks: vec![],
            })
        }
    };

    let mut checks = Vec::new();
    let mut all_passed = true;

    for criterion in &criteria_arr {
        // Only run criteria that have a "command" field
        let command = match criterion.get("command").and_then(|v| v.as_str()) {
            Some(cmd) => cmd,
            None => continue, // Skip non-runnable criteria
        };

        let check_name = criterion
            .get("check")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string();

        let expect_exit_zero = criterion
            .get("expect")
            .and_then(|v| v.as_str())
            .map(|e| e.contains("exit_code == 0"))
            .unwrap_or(true);

        // Run the command with a 5-minute timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let passed = if expect_exit_zero {
                    output.status.success()
                } else {
                    true // If no exit code expectation, always pass
                };

                if !passed {
                    all_passed = false;
                }

                checks.push(ValidationCheck {
                    name: check_name,
                    passed,
                    output: Some(if stdout.len() > 2000 {
                        stdout[..2000].to_string()
                    } else {
                        stdout
                    }),
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(if stderr.len() > 2000 {
                            stderr[..2000].to_string()
                        } else {
                            stderr
                        })
                    },
                });
            }
            Ok(Err(e)) => {
                all_passed = false;
                checks.push(ValidationCheck {
                    name: check_name,
                    passed: false,
                    output: None,
                    error: Some(format!("Failed to execute: {}", e)),
                });
            }
            Err(_) => {
                all_passed = false;
                checks.push(ValidationCheck {
                    name: check_name,
                    passed: false,
                    output: None,
                    error: Some("Command timed out (5 min limit)".to_string()),
                });
            }
        }
    }

    Ok(ValidationResult { all_passed, checks })
}

/// Check if a task has runnable validation criteria (has "command" fields).
pub fn has_runnable_criteria(success_criteria_json: &str) -> bool {
    let criteria: serde_json::Value =
        serde_json::from_str(success_criteria_json).unwrap_or(serde_json::json!([]));
    criteria
        .as_array()
        .map(|arr| arr.iter().any(|c| c.get("command").is_some()))
        .unwrap_or(false)
}
