fn update_github_version(version: &str) -> Result<(), String> {
    // Check if tag exists
    let check_tag = Command::new("git")
        .args(["tag", "-l", &format!("v{}", version)])
        .output()
        .map_err(|e| format!("Failed to check tag: {}", e))?;
    
    let tag_exists = String::from_utf8_lossy(&check_tag.stdout)
        .trim()
        .len() > 0;

    if tag_exists {
        return Ok(());
    }

    // Create new tag
    let create_tag = Command::new("git")
        .args(["tag", "-a", &format!("v{}", version), "-m", &format!("Release v{}", version)])
        .output()
        .map_err(|e| format!("Failed to create tag: {}", e))?;

    if !create_tag.status.success() {
        return Err(String::from_utf8_lossy(&create_tag.stderr).to_string());
    }

    // Push new tag
    let push_tag = Command::new("git")
        .args(["push", "origin", &format!("v{}", version)])
        .output()
        .map_err(|e| format!("Failed to push tag: {}", e))?;

    if !push_tag.status.success() {
        return Err(String::from_utf8_lossy(&push_tag.stderr).to_string());
    }

    Ok(())
}