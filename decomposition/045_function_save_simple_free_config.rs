fn save_simple_free_config(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?
        .join(".aicommit.json");
        
    let mut full_config = Config::load()?;
    
    // Update the provider in the full config
    for provider in &mut full_config.providers {
        if let ProviderConfig::SimpleFreeOpenRouter(simple_config) = provider {
            if simple_config.id == config.id {
                *simple_config = config.clone();
                break;
            }
        }
    }
    
    let content = serde_json::to_string_pretty(&full_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}