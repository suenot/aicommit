// Model management functions

use crate::types::*;
use std::fs;
use chrono;
use tracing::{info, error, debug};

// From: 035_function_get_available_free_models.rs
pub async fn get_available_free_models(api_key: &str, simulate_offline: bool) -> Result<Vec<String>, String> {
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

// From: 036_function_fallback_to_preferred_models.rs
pub fn fallback_to_preferred_models() -> Result<Vec<String>, String> {
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

// From: 037_function_find_best_available_model.rs
pub fn find_best_available_model(available_models: &[String], config: &SimpleFreeOpenRouterConfig) -> Option<String> {
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

// From: 038_function_extract_model_size.rs
pub fn extract_model_size(model_name: &str) -> u32 {
    let lower_name = model_name.to_lowercase();
    
    // Look for patterns like "70b", "32b", "7b", etc.
    let patterns = [
        "253b", "235b", "200b", "124b",
        "70b", "80b", "90b", "72b", "65b", 
        "40b", "32b", "30b", "24b", "20b",
        "16b", "14b", "13b", "12b", "11b", "10b",
        "9b", "8b", "7b", "6b", "5b", "4b", "3b", "2b", "1b"
    ];
    
    for pattern in patterns {
        if lower_name.contains(pattern) {
            // Extract the number from the pattern (e.g., "70b" -> 70)
            if let Ok(size) = pattern.trim_end_matches(|c| c == 'b' || c == 'B').parse::<u32>() {
                return size;
            }
        }
    }
    
    // Default size if no pattern matches
    // Check for specific keywords that might indicate a more powerful model
    if lower_name.contains("large") || lower_name.contains("ultra") {
        return 15; // Assume it's a medium-large model
    } else if lower_name.contains("medium") {
        return 10;
    } else if lower_name.contains("small") || lower_name.contains("tiny") {
        return 5;
    }
    
    // Default fallback
    0
}

// From: 046_function_is_model_available.rs
pub fn is_model_available(model_stats: &Option<&ModelStats>) -> bool {
    match model_stats {
        None => true, // No stats yet, model is available
        Some(stats) => {
            // Check if blacklisted but should be retried
            if stats.blacklisted {
                if let Some(blacklisted_since) = stats.blacklisted_since {
                    let retry_duration = chrono::Duration::days(BLACKLIST_RETRY_DAYS);
                    let now = chrono::Utc::now();
                    
                    // If blacklisted for more than retry period, give it another chance
                    if now - blacklisted_since > retry_duration {
                        return true;
                    }
                    return false;
                }
                return false;
            }
            
            // Check if currently in jail
            if let Some(jail_until) = stats.jail_until {
                if chrono::Utc::now() < jail_until {
                    return false;
                }
            }
            
            true
        }
    }
}

// From: 047_function_record_model_success.rs
pub fn record_model_success(model_stats: &mut ModelStats) {
    model_stats.success_count += 1;
    model_stats.last_success = Some(chrono::Utc::now());
    
    // Reset consecutive failures if successful
    if model_stats.last_failure.is_none() || 
       model_stats.last_success.unwrap() > model_stats.last_failure.unwrap() {
        // The model is working now, remove any jail time
        model_stats.jail_until = None;
    }
}

// From: 048_function_record_model_failure.rs
pub fn record_model_failure(model_stats: &mut ModelStats) {
    let now = chrono::Utc::now();
    model_stats.failure_count += 1;
    model_stats.last_failure = Some(now);
    
    // Check if we have consecutive failures
    let has_consecutive_failures = match model_stats.last_success {
        None => true, // Never had a success
        Some(last_success) => {
            // If last success is older than last failure, we have consecutive failures
            model_stats.last_failure.unwrap() > last_success
        }
    };
    
    if has_consecutive_failures {
        // Count consecutive failures by comparing timestamps
        let consecutive_failures = if let Some(last_success) = model_stats.last_success {
            let hours_since_success = (now - last_success).num_hours();
            // If it's been more than a day since last success, count as consecutive failures
            if hours_since_success > 24 {
                model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
            } else {
                // Count failures since last success
                1 // This is at least 1 consecutive failure
            }
        } else {
            // No success ever, count all failures
            model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
        };
        
        // Jail if we hit the threshold
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            // Calculate jail duration based on recidivism
            let jail_hours = INITIAL_JAIL_HOURS * JAIL_TIME_MULTIPLIER.pow(model_stats.jail_count as u32);
            let jail_hours = jail_hours.min(MAX_JAIL_HOURS); // Cap at maximum
            
            // Set jail expiration time
            model_stats.jail_until = Some(now + chrono::Duration::hours(jail_hours));
            model_stats.jail_count += 1;
            
            // Blacklist if consistently problematic
            if model_stats.jail_count >= BLACKLIST_AFTER_JAIL_COUNT {
                model_stats.blacklisted = true;
                model_stats.blacklisted_since = Some(now);
            }
        }
    }
}

// From: 049_function_format_model_status.rs
pub fn format_model_status(model: &str, stats: &ModelStats) -> String {
    let status = if stats.blacklisted {
        "BLACKLISTED".to_string()
    } else if let Some(jail_until) = stats.jail_until {
        if chrono::Utc::now() < jail_until {
            let remaining = jail_until - chrono::Utc::now();
            format!("JAILED ({}h remaining)", remaining.num_hours())
        } else {
            "ACTIVE".to_string()
        }
    } else {
        "ACTIVE".to_string()
    };
    
    let last_success = stats.last_success.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    let last_failure = stats.last_failure.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    format!("{}: {} (Success: {}, Failure: {}, Last success: {}, Last failure: {})",
            model, status, stats.success_count, stats.failure_count, last_success, last_failure)
}

// From: 050_function_display_model_jail_status.rs
pub fn display_model_jail_status(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
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

// From: 051_function_unjail_model.rs
pub fn unjail_model(config: &mut SimpleFreeOpenRouterConfig, model_id: &str) -> Result<(), String> {
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

// From: 052_function_unjail_all_models.rs
pub fn unjail_all_models(config: &mut SimpleFreeOpenRouterConfig) -> Result<(), String> {
    unjail_model(config, "*")
}

