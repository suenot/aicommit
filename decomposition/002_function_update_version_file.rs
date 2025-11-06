async fn update_version_file(file_path: &str) -> Result<(), String> {
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