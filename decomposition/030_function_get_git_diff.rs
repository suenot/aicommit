fn get_git_diff(cli: &Cli) -> Result<String, String> {
    // First check if current directory is a git repository
    let is_git_repo = Command::new("sh")
        .arg("-c")
        .arg("git rev-parse --is-inside-work-tree")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_git_repo {
        return Err("Current directory is not a git repository".to_string());
    }

    // Check for unstaged changes first
    let status_output = Command::new("sh")
        .arg("-c")
        .arg("git status --porcelain")
        .output()
        .map_err(|e| format!("Failed to execute git status: {}", e))?;

    let status = String::from_utf8_lossy(&status_output.stdout).to_string();
    
    // If --add flag is set and there are unstaged changes, add them
    if cli.add && status.lines().any(|line| {
        line.starts_with(" M") || // Modified but not staged
        line.starts_with("MM") || // Modified and staged with new modifications
        line.starts_with("??")    // Untracked files
    }) {
        let add_output = Command::new("sh")
            .arg("-c")
            .arg("git add .")
            .output()
            .map_err(|e| format!("Failed to execute git add: {}", e))?;

        if !add_output.status.success() {
            return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
        }
    }

    // Try to get diff of staged changes
    let diff_cmd = if cli.dry_run {
        // For dry run, try to get changes in a more robust way
        // First try --cached, and if it fails, try without --cached
        match Command::new("sh")
            .arg("-c")
            .arg("git diff --cached")
            .output() {
                Ok(output) if output.status.success() => {
                    let diff = String::from_utf8_lossy(&output.stdout).to_string();
                    if !diff.trim().is_empty() {
                        return Ok(diff);
                    }
                    // If no staged changes, fall back to unstaged changes
                    "git diff"
                },
                _ => "git diff" // Fall back to unstaged changes
            }
    } else {
        "git diff --cached"
    };

    let diff_output = Command::new("sh")
        .arg("-c")
        .arg(diff_cmd)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", diff_cmd, e))?;

    if !diff_output.status.success() {
        return Err(format!("Git diff command failed: {}", String::from_utf8_lossy(&diff_output.stderr)));
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    if diff.trim().is_empty() {
        return Err("No changes to commit".to_string());
    }

    Ok(diff)
}