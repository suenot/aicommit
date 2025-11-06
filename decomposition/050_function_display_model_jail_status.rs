fn display_model_jail_status(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
    if config.model_stats.is_empty() {
        println!("No model statistics available yet.");
        return Ok(());
    }
    
    println!("\nModel Status Report:");
    println!("===================");
    
    // Group models by status
    let mut active_models = Vec::new();
    let mut jailed_models = Vec::new();
    let mut blacklisted_models = Vec::new();
    
    for (model, stats) in &config.model_stats {
        if stats.blacklisted {
            blacklisted_models.push(format_model_status(model, stats));
        } else if let Some(jail_until) = stats.jail_until {
            if chrono::Utc::now() < jail_until {
                jailed_models.push(format_model_status(model, stats));
            } else {
                active_models.push(format_model_status(model, stats));
            }
        } else {
            active_models.push(format_model_status(model, stats));
        }
    }
    
    // Sort and display each group
    if !active_models.is_empty() {
        println!("\nACTIVE MODELS:");
        active_models.sort();
        for model in active_models {
            println!("  {}", model);
        }
    }
    
    if !jailed_models.is_empty() {
        println!("\nJAILED MODELS:");
        jailed_models.sort();
        for model in jailed_models {
            println!("  {}", model);
        }
    }
    
    if !blacklisted_models.is_empty() {
        println!("\nBLACKLISTED MODELS:");
        blacklisted_models.sort();
        for model in blacklisted_models {
            println!("  {}", model);
        }
    }
    
    Ok(())
}