async fn update_cargo_version(version: &str) -> Result<(), String> {
    // Update version in Cargo.toml
    let cargo_toml_path = "Cargo.toml";
    let content = tokio::fs::read_to_string(cargo_toml_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let updated_content = content
        .lines()
        .map(|line| {
            if line.starts_with("version = ") {
                format!("version = \"{}\"", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    tokio::fs::write(cargo_toml_path, updated_content)
        .await
        .map_err(|e| format!("Failed to write {}: {}", cargo_toml_path, e))?;

    // Update version in Cargo.lock
    let cargo_lock_path = "Cargo.lock";
    let lock_content = tokio::fs::read_to_string(cargo_lock_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", cargo_lock_path, e))?;

    let mut in_aicommit_package = false;
    let updated_lock_content = lock_content
        .lines()
        .map(|line| {
            if line.starts_with("name = \"aicommit\"") {
                in_aicommit_package = true;
                line.to_string()
            } else if in_aicommit_package && line.starts_with("version = ") {
                in_aicommit_package = false;
                format!("version = \"{}\"", version)
            } else if line.trim().is_empty() {
                in_aicommit_package = false;
                line.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    tokio::fs::write(cargo_lock_path, updated_lock_content)
        .await
        .map_err(|e| format!("Failed to write {}: {}", cargo_lock_path, e))?;

    // Run cargo update to ensure Cargo.lock is in sync
    let update_output = std::process::Command::new("cargo")
        .arg("update")
        .arg("--package")
        .arg("aicommit")
        .output()
        .map_err(|e| format!("Failed to execute cargo update: {}", e))?;

    if !update_output.status.success() {
        return Err(format!(
            "Failed to update Cargo.lock: {}",
            String::from_utf8_lossy(&update_output.stderr)
        ));
    }

    Ok(())
}