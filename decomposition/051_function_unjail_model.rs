fn unjail_model(config: &mut SimpleFreeOpenRouterConfig, model_id: &str) -> Result<(), String> {
    let model_found = if model_id == "*" {
        // Reset all models
        for (_, stats) in config.model_stats.iter_mut() {
            stats.jail_until = None;
            stats.blacklisted = false;
            stats.jail_count = 0;
        }
        true
    } else {
        // Reset specific model
        if let Some(stats) = config.model_stats.get_mut(model_id) {
            stats.jail_until = None;
            stats.blacklisted = false;
            stats.jail_count = 0;
            true
        } else {
            false
        }
    };
    
    if !model_found {
        return Err(format!("Model '{}' not found in statistics", model_id));
    }
    
    // Save updated config
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