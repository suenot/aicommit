async fn get_available_free_models(api_key: &str, simulate_offline: bool) -> Result<Vec<String>, String> {
    // If simulate_offline is true, immediately return the fallback list
    if simulate_offline {
        println!("Debug: Simulating offline mode, using fallback model list");
        return fallback_to_preferred_models();
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10)) // Add a reasonable timeout
        .build()
        .unwrap_or_default();
    
    // Try to fetch models from OpenRouter API
    let response = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://suenot.github.io/aicommit/")
            .header("X-Title", "aicommit")
            .send()
    ).await {
        Ok(result) => match result {
            Ok(response) => {
                if !response.status().is_success() {
                    println!("Warning: OpenRouter API returned status code: {}", response.status());
                    return fallback_to_preferred_models();
                }
                response
            },
            Err(e) => {
                println!("Warning: Failed to connect to OpenRouter API: {}", e);
                println!("Using predefined free models as fallback...");
                return fallback_to_preferred_models();
            }
        },
        Err(_) => {
            println!("Warning: Request to OpenRouter API timed out after 15 seconds");
            println!("Using predefined free models as fallback...");
            return fallback_to_preferred_models();
        }
    };
    
    // Try to parse the response
    let models_response: Result<serde_json::Value, _> = response.json().await;
    if let Err(e) = &models_response {
        println!("Warning: Failed to parse OpenRouter API response: {}", e);
        println!("Using predefined free models as fallback...");
        return fallback_to_preferred_models();
    }
    
    let models_response = models_response.unwrap();
    let mut free_models = Vec::new();
    
    if let Some(data) = models_response["data"].as_array() {
        // First pass: check all models that are explicitly marked as free
        for model in data {
            if let Some(id) = model["id"].as_str() {
                // Multiple ways to detect if a model is free:
                
                // 1. Check if the model ID contains ":free"
                if id.contains(":free") {
                    free_models.push(id.to_string());
                    continue;
                }
                
                // 2. Check if "free" field is true
                if let Some(true) = model["free"].as_bool() {
                    free_models.push(id.to_string());
                    continue;
                }
                
                // 3. Check if "free_tokens" is greater than 0
                if let Some(tokens) = model["free_tokens"].as_u64() {
                    if tokens > 0 {
                        free_models.push(id.to_string());
                        continue;
                    }
                }
                
                // 4. Check if pricing is 0 for both prompt and completion
                if let Some(pricing) = model["pricing"].as_object() {
                    let prompt_price = pricing.get("prompt")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    
                    let completion_price = pricing.get("completion")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    
                    if prompt_price == 0.0 && completion_price == 0.0 {
                        free_models.push(id.to_string());
                        continue;
                    }
                }
            }
        }
        
        // If no free models found, try a second pass with more relaxed criteria
        if free_models.is_empty() {
            // Look for models with very low pricing (<= 0.0001)
            for model in data {
                if let Some(id) = model["id"].as_str() {
                    if let Some(pricing) = model["pricing"].as_object() {
                        let prompt_price = pricing.get("prompt")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0);
                        
                        let completion_price = pricing.get("completion")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0);
                        
                        // Consider very low-priced models as "effectively free"
                        if prompt_price <= 0.0001 && completion_price <= 0.0001 {
                            free_models.push(id.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // If we still found no free models, fall back to predefined list
    if free_models.is_empty() {
        println!("Warning: No free models found from OpenRouter API");
        println!("Using predefined free models as fallback...");
        return fallback_to_preferred_models();
    }
    
    Ok(free_models)
}