// Version management functions

use std::process::Command;
use serde_json::json;
use tracing::{info, error, debug};

// From: 001_function_increment_version.rs
pub fn increment_version(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = version.trim().split('.').collect();
    if parts.len() < 1 {
        return Err("Invalid version format".into());
    }

    let mut new_parts: Vec<String> = parts.iter().map(|&s| s.to_string()).collect();
    let last_idx = new_parts.len() - 1;
    
    if let Ok(mut num) = new_parts[last_idx].parse::<u32>() {
        num += 1;
        new_parts[last_idx] = num.to_string();
        Ok(new_parts.join("."))
    } else {
        Err("Invalid version number".into())
    }
}

// From: 002_function_update_version_file.rs
pub async fn update_version_file(file_path: &str) -> Result<(), String> {
    debug!("Reading version file: {}", file_path);
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to read version file: {}", e);
            error!("Version file read error: {}", error_msg);
            error_msg
        })?;

    debug!("Incrementing version from: {}", content.trim());
    let new_version = increment_version(&content)
        .map_err(|e| {
            let error_msg = format!("Failed to increment version: {}", e);
            error!("Version increment error: {}", error_msg);
            error_msg
        })?;

    debug!("Writing new version: {}", new_version.trim());
    tokio::fs::write(file_path, &new_version)
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to write version file: {}", e);
            error!("Version file write error: {}", error_msg);
            error_msg
        })?;

    info!("Successfully updated version file {} to {}", file_path, new_version.trim());
    Ok(())
}

// From: 003_function_update_cargo_version.rs
pub async fn update_cargo_version(version: &str) -> Result<(), String> {
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

// From: 004_function_update_npm_version.rs
pub async fn update_npm_version(version: &str) -> Result<(), String> {
    let package_path = "package.json";
    let content = tokio::fs::read_to_string(package_path)
        .await
        .map_err(|e| format!("Failed to read package.json: {}", e))?;

    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse package.json: {}", e))?;

    if let Some(obj) = json.as_object_mut() {
        obj["version"] = json!(version);
    }

    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize package.json: {}", e))?;

    tokio::fs::write(package_path, new_content)
        .await
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

    Ok(())
}

// From: 005_function_update_github_version.rs
pub fn update_github_version(version: &str) -> Result<(), String> {
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

// From: 032_function_get_version.rs
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

