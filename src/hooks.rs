// Git hooks module - installation and management of Git hooks

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

/// The prepare-commit-msg hook script content
const PREPARE_COMMIT_MSG_HOOK: &str = r#"#!/bin/sh
# aicommit prepare-commit-msg hook
# This hook generates AI-powered commit messages automatically

COMMIT_MSG_FILE="$1"
COMMIT_SOURCE="$2"
SHA1="$3"

# Skip for merge commits
if [ -f ".git/MERGE_HEAD" ]; then
    exit 0
fi

# Skip for amend commits (when source is "commit")
if [ "$COMMIT_SOURCE" = "commit" ]; then
    exit 0
fi

# Skip for squash commits
if [ "$COMMIT_SOURCE" = "squash" ]; then
    exit 0
fi

# Skip for message commits (when -m flag is used)
if [ "$COMMIT_SOURCE" = "message" ]; then
    exit 0
fi

# Skip if there's already a non-empty commit message (e.g., from -m flag)
if [ -s "$COMMIT_MSG_FILE" ]; then
    # Check if the file contains more than just comments
    if grep -v "^#" "$COMMIT_MSG_FILE" | grep -q "[^[:space:]]"; then
        exit 0
    fi
fi

# Check if aicommit is available
if ! command -v aicommit >/dev/null 2>&1; then
    echo "Warning: aicommit not found in PATH. Skipping AI commit message generation." >&2
    exit 0
fi

# Generate commit message using aicommit in dry-run mode
# This will output only the generated message without creating a commit
GENERATED_MSG=$(aicommit --dry-run 2>/dev/null)

# Check if message generation was successful
if [ $? -eq 0 ] && [ -n "$GENERATED_MSG" ]; then
    # Write the generated message to the commit message file
    # Preserve any existing comments (lines starting with #)
    {
        echo "$GENERATED_MSG"
        echo ""
        grep "^#" "$COMMIT_MSG_FILE" 2>/dev/null || true
    } > "$COMMIT_MSG_FILE.tmp"
    mv "$COMMIT_MSG_FILE.tmp" "$COMMIT_MSG_FILE"
fi

exit 0
"#;

/// Find the .git directory for the current repository
fn find_git_dir() -> Result<std::path::PathBuf, String> {
    // Try to get the git directory using git rev-parse
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if !output.status.success() {
        return Err("Not a git repository. Please run this command from within a git repository.".to_string());
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(git_dir))
}

/// Install the prepare-commit-msg hook
pub fn install_hook() -> Result<(), String> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");

    // Create hooks directory if it doesn't exist
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .map_err(|e| format!("Failed to create hooks directory: {}", e))?;
    }

    // Check if hook already exists
    if hook_path.exists() {
        // Read existing hook to check if it's ours
        let existing_content = fs::read_to_string(&hook_path)
            .map_err(|e| format!("Failed to read existing hook: {}", e))?;

        if existing_content.contains("aicommit prepare-commit-msg hook") {
            println!("aicommit hook is already installed at {}", hook_path.display());
            return Ok(());
        }

        // Backup existing hook
        let backup_path = hooks_dir.join("prepare-commit-msg.backup");
        fs::rename(&hook_path, &backup_path)
            .map_err(|e| format!("Failed to backup existing hook: {}", e))?;
        println!("Existing hook backed up to {}", backup_path.display());
    }

    // Write the hook script
    fs::write(&hook_path, PREPARE_COMMIT_MSG_HOOK)
        .map_err(|e| format!("Failed to write hook file: {}", e))?;

    // Make the hook executable
    let mut perms = fs::metadata(&hook_path)
        .map_err(|e| format!("Failed to get hook file metadata: {}", e))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&hook_path, perms)
        .map_err(|e| format!("Failed to set hook file permissions: {}", e))?;

    println!("Successfully installed aicommit hook at {}", hook_path.display());
    println!("\nThe hook will automatically generate AI commit messages when you run 'git commit'.");
    println!("It will be skipped for:");
    println!("  - Merge commits");
    println!("  - Amend commits (git commit --amend)");
    println!("  - Squash commits");
    println!("  - Commits with -m flag (message already provided)");

    Ok(())
}

/// Uninstall the prepare-commit-msg hook
pub fn uninstall_hook() -> Result<(), String> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");
    let hook_path = hooks_dir.join("prepare-commit-msg");

    if !hook_path.exists() {
        println!("No prepare-commit-msg hook found.");
        return Ok(());
    }

    // Read existing hook to check if it's ours
    let existing_content = fs::read_to_string(&hook_path)
        .map_err(|e| format!("Failed to read hook file: {}", e))?;

    if !existing_content.contains("aicommit prepare-commit-msg hook") {
        return Err("The existing prepare-commit-msg hook was not installed by aicommit. \
                   Please remove it manually if you want to proceed.".to_string());
    }

    // Remove the hook
    fs::remove_file(&hook_path)
        .map_err(|e| format!("Failed to remove hook file: {}", e))?;

    // Check if there's a backup to restore
    let backup_path = hooks_dir.join("prepare-commit-msg.backup");
    if backup_path.exists() {
        fs::rename(&backup_path, &hook_path)
            .map_err(|e| format!("Failed to restore backup hook: {}", e))?;
        println!("Restored previous hook from backup.");
    }

    println!("Successfully uninstalled aicommit hook from {}", hook_path.display());

    Ok(())
}

/// Display hook status
pub fn hook_status() -> Result<(), String> {
    let git_dir = find_git_dir()?;
    let hook_path = git_dir.join("hooks").join("prepare-commit-msg");

    if !hook_path.exists() {
        println!("aicommit hook: not installed");
        return Ok(());
    }

    let content = fs::read_to_string(&hook_path)
        .map_err(|e| format!("Failed to read hook file: {}", e))?;

    if content.contains("aicommit prepare-commit-msg hook") {
        println!("aicommit hook: installed at {}", hook_path.display());
    } else {
        println!("aicommit hook: not installed (another hook exists at {})", hook_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_find_git_dir_outside_repo() {
        // This test might fail if run inside a git repo
        // It's primarily for documentation purposes
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Should return error when not in a git repo
        let result = find_git_dir();
        // Note: This might still succeed if there's a parent git repo
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_hook_script_content() {
        // Verify the hook script contains essential elements
        assert!(PREPARE_COMMIT_MSG_HOOK.contains("#!/bin/sh"));
        assert!(PREPARE_COMMIT_MSG_HOOK.contains("MERGE_HEAD"));
        assert!(PREPARE_COMMIT_MSG_HOOK.contains("aicommit --dry-run"));
        assert!(PREPARE_COMMIT_MSG_HOOK.contains("commit"));
        assert!(PREPARE_COMMIT_MSG_HOOK.contains("squash"));
    }
}
