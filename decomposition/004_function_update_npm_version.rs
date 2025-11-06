async fn update_npm_version(version: &str) -> Result<(), String> {
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