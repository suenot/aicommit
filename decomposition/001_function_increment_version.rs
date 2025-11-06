fn increment_version(version: &str) -> Result<String, Box<dyn std::error::Error>> {
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