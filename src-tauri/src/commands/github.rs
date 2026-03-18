use crate::models::{GithubConfig, GithubEvent, GitLink, CIStatus, CICheck, PRStatus};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ---- Input types ----

#[derive(Deserialize)]
pub struct SetGithubConfigInput {
    pub repo_owner: String,
    pub repo_name: String,
    pub access_token: Option<String>,
    pub branch_pattern: Option<String>,
    pub auto_link_prs: Option<bool>,
    pub auto_transition_on_merge: Option<bool>,
    pub merge_target_status_id: Option<i64>,
}

#[derive(Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub rate_limit_remaining: Option<i64>,
}

#[derive(Serialize)]
pub struct BranchNamePreview {
    pub branch_name: String,
    pub pattern: String,
}

// ---- Helper: get access token (config > env var) ----

async fn get_token(pool: &sqlx::AnyPool, project_id: i64) -> Result<String, String> {
    let config_token: Option<String> = sqlx::query_scalar(
        "SELECT access_token FROM github_config WHERE project_id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    if let Some(token) = config_token {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    std::env::var("GITHUB_PAT")
        .map_err(|_| "No GitHub access token configured. Set it in project settings or GITHUB_PAT env var.".to_string())
}

// ---- Helper: build HTTP client ----

fn github_client(token: &str) -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token).parse().map_err(|_| "Invalid token format")?,
    );
    headers.insert(
        reqwest::header::ACCEPT,
        "application/vnd.github+json".parse().unwrap(),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        "2022-11-28".parse().unwrap(),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        "kanban-app/0.1".parse().unwrap(),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())
}

// ---- Helper: generate branch name from pattern ----

fn generate_branch_name_from_pattern(pattern: &str, prefix: &str, number: i64, title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Truncate slug to keep branch name reasonable
    let slug = if slug.len() > 50 {
        slug[..50].trim_end_matches('-').to_string()
    } else {
        slug
    };

    pattern
        .replace("{{prefix}}", prefix)
        .replace("{{number}}", &number.to_string())
        .replace("{{slug}}", &slug)
}

// ---- Helper: extract issue identifier from branch name or PR body ----

fn extract_issue_identifier(text: &str, prefix: &str) -> Option<String> {
    // Look for patterns like KAN-5, KAN-42, etc.
    let pattern = format!(r"(?i)\b({})-(\d+)\b", regex_lite::escape(prefix));
    if let Ok(re) = regex_lite::Regex::new(&pattern) {
        if let Some(caps) = re.captures(text) {
            return Some(format!("{}-{}", prefix, &caps[2]));
        }
    }
    None
}

// ---- Helper: log activity ----

async fn log_activity(pool: &sqlx::AnyPool, issue_id: i64, field: &str, old_val: Option<String>, new_val: Option<String>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    sqlx::query("INSERT INTO activity_log (issue_id, field_changed, old_value, new_value, timestamp) VALUES ($1, $2, $3, $4, $5)")
        .bind(issue_id).bind(field).bind(old_val).bind(new_val).bind(&now)
        .execute(pool).await?;
    Ok(())
}

// ---- Commands ----

#[tauri::command]
pub fn get_github_config(state: State<AppState>, project_id: i64) -> Result<Option<GithubConfig>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_github_config(state: State<AppState>, project_id: i64, input: SetGithubConfigInput) -> Result<GithubConfig, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let branch_pattern = input.branch_pattern.unwrap_or_else(|| "{{prefix}}-{{number}}/{{slug}}".to_string());
        let auto_link = input.auto_link_prs.unwrap_or(true);
        let auto_transition = input.auto_transition_on_merge.unwrap_or(true);

        // Upsert
        let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await?;

        if let Some(_id) = existing {
            sqlx::query(
                "UPDATE github_config SET repo_owner = $1, repo_name = $2, access_token = $3, branch_pattern = $4, auto_link_prs = $5, auto_transition_on_merge = $6, merge_target_status_id = $7, updated_at = $8 WHERE project_id = $9"
            )
            .bind(&input.repo_owner)
            .bind(&input.repo_name)
            .bind(&input.access_token)
            .bind(&branch_pattern)
            .bind(auto_link)
            .bind(auto_transition)
            .bind(input.merge_target_status_id)
            .bind(&now)
            .bind(project_id)
            .execute(&state.pool)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO github_config (project_id, repo_owner, repo_name, access_token, branch_pattern, auto_link_prs, auto_transition_on_merge, merge_target_status_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(project_id)
            .bind(&input.repo_owner)
            .bind(&input.repo_name)
            .bind(&input.access_token)
            .bind(&branch_pattern)
            .bind(auto_link)
            .bind(auto_transition)
            .bind(input.merge_target_status_id)
            .bind(&now)
            .bind(&now)
            .execute(&state.pool)
            .await?;
        }

        sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn test_github_connection(state: State<AppState>, project_id: i64) -> Result<ConnectionTestResult, String> {
    state.rt.block_on(async {
        let token = get_token(&state.pool, project_id).await?;
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let config = config.ok_or_else(|| "GitHub config not found. Save config first.".to_string())?;
        let client = github_client(&token)?;

        let url = format!("https://api.github.com/repos/{}/{}", config.repo_owner, config.repo_name);
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

        let rate_limit = resp.headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok());

        if resp.status().is_success() {
            Ok(ConnectionTestResult {
                success: true,
                message: format!("Connected to {}/{}", config.repo_owner, config.repo_name),
                rate_limit_remaining: rate_limit,
            })
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Ok(ConnectionTestResult {
                success: false,
                message: format!("HTTP {}: {}", status.as_u16(), body),
                rate_limit_remaining: rate_limit,
            })
        }
    })
}

#[tauri::command]
pub fn generate_branch_name(state: State<AppState>, project_id: i64, issue_identifier: String) -> Result<BranchNamePreview, String> {
    state.rt.block_on(async {
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let pattern = config.as_ref().map(|c| c.branch_pattern.as_str()).unwrap_or("{{prefix}}-{{number}}/{{slug}}");

        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE identifier = $1")
            .bind(&issue_identifier)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let project = sqlx::query_as::<_, crate::models::Project>("SELECT * FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        // Parse number from identifier (e.g., KAN-5 -> 5)
        let number: i64 = issue.identifier
            .split('-')
            .last()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        let branch_name = generate_branch_name_from_pattern(pattern, &project.prefix, number, &issue.title);

        Ok(BranchNamePreview {
            branch_name,
            pattern: pattern.to_string(),
        })
    })
}

#[tauri::command]
pub fn create_branch_for_issue(state: State<AppState>, project_id: i64, issue_identifier: String) -> Result<GitLink, String> {
    state.rt.block_on(async {
        let token = get_token(&state.pool, project_id).await?;
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE identifier = $1")
            .bind(&issue_identifier)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let project = sqlx::query_as::<_, crate::models::Project>("SELECT * FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let number: i64 = issue.identifier
            .split('-')
            .last()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        let branch_name = generate_branch_name_from_pattern(&config.branch_pattern, &project.prefix, number, &issue.title);

        let client = github_client(&token)?;

        // Get default branch SHA
        let repo_url = format!("https://api.github.com/repos/{}/{}", config.repo_owner, config.repo_name);
        let repo_resp: serde_json::Value = client.get(&repo_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let default_branch = repo_resp["default_branch"].as_str().unwrap_or("main");

        let ref_url = format!("https://api.github.com/repos/{}/{}/git/refs/heads/{}", config.repo_owner, config.repo_name, default_branch);
        let ref_resp: serde_json::Value = client.get(&ref_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let sha = ref_resp["object"]["sha"].as_str()
            .ok_or_else(|| format!("Could not get SHA for branch {}", default_branch))?;

        // Create the branch
        let create_ref_url = format!("https://api.github.com/repos/{}/{}/git/refs", config.repo_owner, config.repo_name);
        let create_body = serde_json::json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": sha
        });

        let create_resp = client.post(&create_ref_url)
            .json(&create_body)
            .send().await.map_err(|e| e.to_string())?;

        if !create_resp.status().is_success() {
            let status = create_resp.status();
            let body = create_resp.text().await.unwrap_or_default();
            return Err(format!("Failed to create branch (HTTP {}): {}", status.as_u16(), body));
        }

        // Create git_link record
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let branch_url = format!("https://github.com/{}/{}/tree/{}", config.repo_owner, config.repo_name, branch_name);

        let link_id: i64 = sqlx::query_scalar(
            "INSERT INTO git_links (issue_id, link_type, url, ref_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(issue.id)
        .bind("branch")
        .bind(&branch_url)
        .bind(&branch_name)
        .bind(&now)
        .bind(&now)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Log activity
        let _ = log_activity(&state.pool, issue.id, "comment", None, Some(format!("Branch `{}` created", branch_name))).await;

        sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
            .bind(link_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn sync_github_prs(state: State<AppState>, project_id: i64) -> Result<Vec<GitLink>, String> {
    state.rt.block_on(async {
        let token = get_token(&state.pool, project_id).await?;
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let project = sqlx::query_as::<_, crate::models::Project>("SELECT * FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let client = github_client(&token)?;

        // Fetch all open + recently closed PRs (last 30)
        let prs_url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=all&per_page=30&sort=updated&direction=desc",
            config.repo_owner, config.repo_name
        );
        let prs: Vec<serde_json::Value> = client.get(&prs_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let mut linked: Vec<GitLink> = Vec::new();

        for pr in &prs {
            let pr_number = pr["number"].as_i64().unwrap_or(0);
            let pr_title = pr["title"].as_str().unwrap_or("");
            let pr_body = pr["body"].as_str().unwrap_or("");
            let pr_state = pr["state"].as_str().unwrap_or("open");
            let pr_merged = pr["merged_at"].is_string();
            let pr_branch = pr["head"]["ref"].as_str().unwrap_or("");
            let pr_url = pr["html_url"].as_str().unwrap_or("");
            let pr_author = pr["user"]["login"].as_str().unwrap_or("unknown");

            // Try to match by branch name pattern or body mentions
            let identifier = extract_issue_identifier(pr_branch, &project.prefix)
                .or_else(|| extract_issue_identifier(pr_title, &project.prefix))
                .or_else(|| extract_issue_identifier(pr_body, &project.prefix));

            let identifier = match identifier {
                Some(id) => id,
                None => continue,
            };

            // Find the issue
            let issue: Option<crate::models::Issue> = sqlx::query_as(
                "SELECT * FROM issues WHERE identifier = $1 AND project_id = $2"
            )
            .bind(&identifier)
            .bind(project_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

            let issue = match issue {
                Some(i) => i,
                None => continue,
            };

            // Check if git_link already exists for this PR
            let existing: Option<GitLink> = sqlx::query_as(
                "SELECT * FROM git_links WHERE issue_id = $1 AND link_type = 'pr' AND pr_number = $2"
            )
            .bind(issue.id)
            .bind(pr_number)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(existing_link) = existing {
                // Update existing link
                sqlx::query(
                    "UPDATE git_links SET pr_state = $1, pr_merged = $2, updated_at = $3 WHERE id = $4"
                )
                .bind(pr_state)
                .bind(pr_merged)
                .bind(&now)
                .bind(existing_link.id)
                .execute(&state.pool)
                .await
                .map_err(|e| e.to_string())?;

                let updated = sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
                    .bind(existing_link.id)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(|e| e.to_string())?;

                // Auto-transition if PR was just merged
                if pr_merged && !existing_link.pr_merged && config.auto_transition_on_merge {
                    if let Some(target_status) = config.merge_target_status_id {
                        let old_status_id = issue.status_id;
                        sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(target_status)
                            .bind(&now)
                            .bind(issue.id)
                            .execute(&state.pool)
                            .await
                            .map_err(|e| e.to_string())?;

                        let _ = log_activity(&state.pool, issue.id, "status_id",
                            Some(old_status_id.to_string()),
                            Some(target_status.to_string())).await;
                        let _ = log_activity(&state.pool, issue.id, "comment", None,
                            Some(format!("PR #{} merged by @{}", pr_number, pr_author))).await;

                        // Log event
                        let payload = serde_json::json!({
                            "pr_number": pr_number,
                            "pr_title": pr_title,
                            "actor": pr_author,
                            "issue_identifier": identifier,
                        });
                        sqlx::query(
                            "INSERT INTO github_events (project_id, event_type, issue_id, payload, processed, created_at) VALUES ($1, $2, $3, $4, 1, $5)"
                        )
                        .bind(project_id)
                        .bind("pr_merged")
                        .bind(issue.id)
                        .bind(payload.to_string())
                        .bind(&now)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }

                linked.push(updated);
            } else {
                // Create new link
                let link_id: i64 = sqlx::query_scalar(
                    "INSERT INTO git_links (issue_id, link_type, url, ref_name, pr_number, pr_state, pr_merged, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
                )
                .bind(issue.id)
                .bind("pr")
                .bind(pr_url)
                .bind(pr_branch)
                .bind(pr_number)
                .bind(pr_state)
                .bind(pr_merged)
                .bind(&now)
                .bind(&now)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| e.to_string())?;

                let _ = log_activity(&state.pool, issue.id, "comment", None,
                    Some(format!("PR #{} linked ({})", pr_number, pr_state))).await;

                // Log event
                let event_type = if pr_merged { "pr_merged" } else { "pr_opened" };
                let payload = serde_json::json!({
                    "pr_number": pr_number,
                    "pr_title": pr_title,
                    "actor": pr_author,
                    "issue_identifier": identifier,
                });
                sqlx::query(
                    "INSERT INTO github_events (project_id, event_type, issue_id, payload, processed, created_at) VALUES ($1, $2, $3, $4, 1, $5)"
                )
                .bind(project_id)
                .bind(event_type)
                .bind(issue.id)
                .bind(payload.to_string())
                .bind(&now)
                .execute(&state.pool)
                .await
                .map_err(|e| e.to_string())?;

                // Auto-transition on merge for newly discovered merged PRs
                if pr_merged && config.auto_transition_on_merge {
                    if let Some(target_status) = config.merge_target_status_id {
                        let old_status_id = issue.status_id;
                        sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                            .bind(target_status)
                            .bind(&now)
                            .bind(issue.id)
                            .execute(&state.pool)
                            .await
                            .map_err(|e| e.to_string())?;

                        let _ = log_activity(&state.pool, issue.id, "status_id",
                            Some(old_status_id.to_string()),
                            Some(target_status.to_string())).await;
                        let _ = log_activity(&state.pool, issue.id, "comment", None,
                            Some(format!("PR #{} merged by @{}", pr_number, pr_author))).await;
                    }
                }

                let link = sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
                    .bind(link_id)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(|e| e.to_string())?;

                linked.push(link);
            }
        }

        Ok(linked)
    })
}

#[tauri::command]
pub fn get_pr_status(state: State<AppState>, project_id: i64, git_link_id: i64) -> Result<PRStatus, String> {
    state.rt.block_on(async {
        let token = get_token(&state.pool, project_id).await?;
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let link = sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE id = $1")
            .bind(git_link_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let pr_number = link.pr_number.ok_or_else(|| "Not a PR link".to_string())?;
        let client = github_client(&token)?;

        // Fetch PR details
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            config.repo_owner, config.repo_name, pr_number
        );
        let pr: serde_json::Value = client.get(&pr_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        // Fetch reviews
        let reviews_url = format!("{}/reviews", pr_url);
        let reviews: Vec<serde_json::Value> = client.get(&reviews_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let review_status = reviews.iter().rev()
            .find_map(|r| {
                match r["state"].as_str() {
                    Some("APPROVED") => Some("approved"),
                    Some("CHANGES_REQUESTED") => Some("changes_requested"),
                    Some("PENDING") => Some("pending"),
                    _ => None,
                }
            })
            .unwrap_or("none")
            .to_string();

        // Fetch CI status via combined status
        let sha = pr["head"]["sha"].as_str().unwrap_or("");
        let ci_status = if !sha.is_empty() {
            let status_url = format!(
                "https://api.github.com/repos/{}/{}/commits/{}/status",
                config.repo_owner, config.repo_name, sha
            );
            let status: serde_json::Value = client.get(&status_url)
                .send().await.map_err(|e| e.to_string())?
                .json().await.map_err(|e| e.to_string())?;

            status["state"].as_str().unwrap_or("pending").to_string()
        } else {
            "pending".to_string()
        };

        // Update git_link with latest status
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let pr_state = pr["state"].as_str().unwrap_or("open");
        let pr_merged = pr["merged"].as_bool().unwrap_or(false);

        sqlx::query("UPDATE git_links SET pr_state = $1, pr_merged = $2, ci_status = $3, review_status = $4, updated_at = $5 WHERE id = $6")
            .bind(pr_state)
            .bind(pr_merged)
            .bind(&ci_status)
            .bind(&review_status)
            .bind(&now)
            .bind(git_link_id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(PRStatus {
            number: pr_number,
            title: pr["title"].as_str().unwrap_or("").to_string(),
            state: pr_state.to_string(),
            merged: pr_merged,
            review_status,
            ci_status,
            url: pr["html_url"].as_str().unwrap_or("").to_string(),
            author: pr["user"]["login"].as_str().unwrap_or("").to_string(),
        })
    })
}

#[tauri::command]
pub fn get_ci_status(state: State<AppState>, project_id: i64, issue_identifier: String) -> Result<CIStatus, String> {
    state.rt.block_on(async {
        let token = get_token(&state.pool, project_id).await?;
        let config = sqlx::query_as::<_, GithubConfig>("SELECT * FROM github_config WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        // Find issue and its PR link
        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE identifier = $1 AND project_id = $2")
            .bind(&issue_identifier)
            .bind(project_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        let pr_link: Option<GitLink> = sqlx::query_as(
            "SELECT * FROM git_links WHERE issue_id = $1 AND link_type = 'pr' ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(issue.id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let pr_link = pr_link.ok_or_else(|| "No PR linked to this issue".to_string())?;
        let pr_number = pr_link.pr_number.ok_or_else(|| "No PR number".to_string())?;

        let client = github_client(&token)?;

        // Get PR head SHA
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            config.repo_owner, config.repo_name, pr_number
        );
        let pr: serde_json::Value = client.get(&pr_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let sha = pr["head"]["sha"].as_str().unwrap_or("");

        if sha.is_empty() {
            return Ok(CIStatus { status: "unknown".to_string(), checks: vec![] });
        }

        // Fetch check runs
        let checks_url = format!(
            "https://api.github.com/repos/{}/{}/commits/{}/check-runs",
            config.repo_owner, config.repo_name, sha
        );
        let checks: serde_json::Value = client.get(&checks_url)
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        let check_runs = checks["check_runs"].as_array();
        let mut ci_checks: Vec<CICheck> = Vec::new();
        let mut overall = "success";

        if let Some(runs) = check_runs {
            for run in runs {
                let status = run["status"].as_str().unwrap_or("queued");
                let conclusion = run["conclusion"].as_str().map(|s| s.to_string());

                if status != "completed" {
                    overall = "pending";
                } else if let Some(ref c) = conclusion {
                    if c == "failure" || c == "timed_out" || c == "cancelled" {
                        overall = "failure";
                    }
                }

                ci_checks.push(CICheck {
                    name: run["name"].as_str().unwrap_or("").to_string(),
                    status: status.to_string(),
                    conclusion,
                    url: run["html_url"].as_str().map(|s| s.to_string()),
                });
            }
        }

        if ci_checks.is_empty() {
            overall = "pending";
        }

        // Update git_link
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE git_links SET ci_status = $1, updated_at = $2 WHERE id = $3")
            .bind(overall)
            .bind(&now)
            .bind(pr_link.id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(CIStatus {
            status: overall.to_string(),
            checks: ci_checks,
        })
    })
}

#[tauri::command]
pub fn list_git_links(state: State<AppState>, issue_id: i64) -> Result<Vec<GitLink>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, GitLink>("SELECT * FROM git_links WHERE issue_id = $1 ORDER BY created_at DESC")
            .bind(issue_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_github_events(state: State<AppState>, project_id: i64) -> Result<Vec<GithubEvent>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, GithubEvent>("SELECT * FROM github_events WHERE project_id = $1 ORDER BY created_at DESC LIMIT 50")
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}
