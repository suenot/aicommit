fn fallback_to_preferred_models() -> Result<Vec<String>, String> {
    let mut models = Vec::new();
    
    // Add all predefined free models
    for model in PREFERRED_FREE_MODELS {
        models.push(model.to_string());
    }
    
    if models.is_empty() {
        return Err("No free models available, and fallback model list is empty".to_string());
    }
    
    Ok(models)
}