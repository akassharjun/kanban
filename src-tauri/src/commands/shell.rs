#[tauri::command]
pub async fn execute_shell_command(command: String, cwd: Option<String>) -> Result<String, String> {
    use std::process::Command;

    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    let mut cmd = Command::new(shell);
    cmd.arg(flag).arg(&command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.is_empty() && stdout.is_empty() {
        Ok(format!("{}", stderr))
    } else if !stderr.is_empty() {
        Ok(format!("{}\n{}", stdout, stderr))
    } else {
        Ok(stdout.to_string())
    }
}

#[tauri::command]
pub async fn list_directories(path: String) -> Result<Vec<String>, String> {
    use std::fs;
    use std::path::Path;

    let dir = Path::new(&path);
    if !dir.exists() {
        // Try parent directory with prefix matching
        let parent = dir.parent().unwrap_or(Path::new("/"));
        let prefix = dir
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let entries = fs::read_dir(parent).map_err(|e| e.to_string())?;
        let mut dirs = Vec::new();
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) {
                    dirs.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
        dirs.sort();
        return Ok(dirs);
    }

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            dirs.push(entry.path().to_string_lossy().to_string());
        }
    }
    dirs.sort();
    Ok(dirs)
}
