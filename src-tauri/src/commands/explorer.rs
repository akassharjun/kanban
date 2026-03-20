use crate::models::Project;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Serialize)]
pub struct GitStatusEntry {
    pub path: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct GitCommitEntry {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct GitBranchEntry {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

#[derive(Serialize)]
pub struct GitWorktreeEntry {
    pub path: String,
    pub branch: String,
    pub is_main: bool,
}

async fn get_project_path(pool: &sqlx::AnyPool, project_id: i64) -> Result<String, String> {
    let project =
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1 AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Project not found: {}", e))?;
    project
        .path
        .ok_or_else(|| "Project has no path configured".to_string())
}

#[tauri::command]
pub fn list_project_files(
    state: State<AppState>,
    project_id: i64,
    path: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let target = match &path {
            Some(p) => {
                let full = std::path::Path::new(&project_path).join(p);
                // Security: ensure we don't escape the project dir
                if !full.starts_with(&project_path) {
                    return Err("Path traversal not allowed".to_string());
                }
                full
            }
            None => std::path::PathBuf::from(&project_path),
        };

        if !target.exists() {
            return Err(format!("Directory not found: {}", target.display()));
        }

        let mut entries = Vec::new();
        let read_dir = std::fs::read_dir(&target)
            .map_err(|e| format!("Cannot read directory: {}", e))?;
        for entry in read_dir {
            let entry = entry.map_err(|e| format!("Read error: {}", e))?;
            let meta = entry
                .metadata()
                .map_err(|e| format!("Metadata error: {}", e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden files/dirs
            if name.starts_with('.') {
                continue;
            }
            // Skip common non-useful dirs
            if meta.is_dir()
                && ["node_modules", "target", "dist", ".git", "__pycache__"]
                    .contains(&name.as_str())
            {
                continue;
            }

            let relative = entry
                .path()
                .strip_prefix(&project_path)
                .unwrap_or(entry.path().as_path())
                .to_string_lossy()
                .to_string();

            entries.push(FileEntry {
                name,
                path: relative,
                is_dir: meta.is_dir(),
                size: if meta.is_file() { meta.len() } else { 0 },
            });
        }
        entries.sort_by(|a, b| {
            // Dirs first, then alphabetical
            b.is_dir
                .cmp(&a.is_dir)
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(entries)
    })
}

#[tauri::command]
pub fn read_project_file(
    state: State<AppState>,
    project_id: i64,
    file_path: String,
) -> Result<String, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let full = std::path::Path::new(&project_path).join(&file_path);
        if !full.starts_with(&project_path) {
            return Err("Path traversal not allowed".to_string());
        }
        if !full.exists() || !full.is_file() {
            return Err(format!("File not found: {}", file_path));
        }
        // Limit to 1MB
        let meta = std::fs::metadata(&full).map_err(|e| e.to_string())?;
        if meta.len() > 1_048_576 {
            return Err("File too large (>1MB)".to_string());
        }
        std::fs::read_to_string(&full).map_err(|e| format!("Cannot read file: {}", e))
    })
}

fn run_git_command(dir: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn get_git_status(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<GitStatusEntry>, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let output = run_git_command(&project_path, &["status", "--porcelain"])?;
        Ok(output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| {
                let status = line[..2].trim().to_string();
                let path = line[3..].to_string();
                GitStatusEntry { path, status }
            })
            .collect())
    })
}

#[tauri::command]
pub fn list_git_commits(
    state: State<AppState>,
    project_id: i64,
    limit: Option<i64>,
) -> Result<Vec<GitCommitEntry>, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let limit_str = format!("{}", limit.unwrap_or(50));
        let output = run_git_command(
            &project_path,
            &[
                "log",
                "--oneline",
                "--format=%H|%s|%an|%ai",
                &format!("-{}", limit_str),
            ],
        )?;
        Ok(output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                GitCommitEntry {
                    hash: parts.first().unwrap_or(&"").to_string(),
                    message: parts.get(1).unwrap_or(&"").to_string(),
                    author: parts.get(2).unwrap_or(&"").to_string(),
                    date: parts.get(3).unwrap_or(&"").to_string(),
                }
            })
            .collect())
    })
}

#[tauri::command]
pub fn list_git_branches(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<GitBranchEntry>, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let output = run_git_command(
            &project_path,
            &["branch", "-a", "--format=%(refname:short)|%(HEAD)"],
        )?;
        Ok(output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                let name = parts.first().unwrap_or(&"").to_string();
                let is_current = parts.get(1).unwrap_or(&"") == &"*";
                let is_remote = name.starts_with("origin/");
                GitBranchEntry {
                    name,
                    is_current,
                    is_remote,
                }
            })
            .collect())
    })
}

#[tauri::command]
pub fn list_git_worktrees(
    state: State<AppState>,
    project_id: i64,
) -> Result<Vec<GitWorktreeEntry>, String> {
    state.rt.block_on(async {
        let project_path = get_project_path(&state.pool, project_id).await?;
        let output = run_git_command(&project_path, &["worktree", "list", "--porcelain"])?;
        let mut worktrees = Vec::new();
        let mut current_path = String::new();
        let mut current_branch = String::new();
        let mut is_first = true;
        for line in output.lines() {
            if line.starts_with("worktree ") {
                if !current_path.is_empty() {
                    worktrees.push(GitWorktreeEntry {
                        path: current_path.clone(),
                        branch: current_branch.clone(),
                        is_main: is_first,
                    });
                    is_first = false;
                }
                current_path = line[9..].to_string();
                current_branch = String::new();
            } else if line.starts_with("branch ") {
                current_branch = line[7..].to_string();
                // Strip refs/heads/ prefix
                if current_branch.starts_with("refs/heads/") {
                    current_branch = current_branch[11..].to_string();
                }
            }
        }
        if !current_path.is_empty() {
            worktrees.push(GitWorktreeEntry {
                path: current_path,
                branch: current_branch,
                is_main: is_first,
            });
        }
        Ok(worktrees)
    })
}
