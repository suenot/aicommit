fn find_best_available_model(available_models: &[String], config: &SimpleFreeOpenRouterConfig) -> Option<String> {
    // Start by using the previously successful model if it's still available
    if let Some(last_model) = &config.last_used_model {
        if available_models.contains(last_model) {
            let stats = config.model_stats.get(last_model);
            if is_model_available(&stats) {
                return Some(last_model.clone());
            }
        }
    }

    // Filter models that are not in jail or blacklisted
    let available_candidates: Vec<&String> = available_models
        .iter()
        .filter(|model| {
            let stats = config.model_stats.get(*model);
            is_model_available(&stats)
        })
        .collect();
    
    // First try our curated list of preferred models in order
    for preferred in PREFERRED_FREE_MODELS {
        let preferred_str = preferred.to_string();
        if available_candidates.contains(&&preferred_str) {
            return Some(preferred_str);
        }
    }
    
    // If none of our preferred models are available, use an intelligent fallback approach
    // by analyzing model names for parameter sizes (like 70b, 32b, etc.)
    if !available_candidates.is_empty() {
        // Sort by estimated parameter count (highest first)
        let mut sorted_candidates = available_candidates.clone();
        sorted_candidates.sort_by(|a, b| {
            let a_size = extract_model_size(a);
            let b_size = extract_model_size(b);
            b_size.cmp(&a_size) // Reverse order (largest first)
        });
        
        // Return the largest available model
        return Some(sorted_candidates[0].clone());
    }
    
    // If all models are jailed or blacklisted, try the least recently jailed one
    if !available_models.is_empty() {
        // Get all jailed but not blacklisted models
        let mut jailed_models: Vec<(String, chrono::DateTime<chrono::Utc>)> = Vec::new();
        
        for model in available_models {
            if let Some(stats) = config.model_stats.get(model) {
                if !stats.blacklisted && stats.jail_until.is_some() {
                    jailed_models.push((model.clone(), stats.jail_until.unwrap()));
                }
            }
        }
        
        // Sort by jail expiry time (soonest first)
        if !jailed_models.is_empty() {
            jailed_models.sort_by_key(|x| x.1);
            return Some(jailed_models[0].0.clone());
        }
        
        // Last resort: just use any model, even blacklisted ones
        return Some(available_models[0].clone());
    }
    
    None
}