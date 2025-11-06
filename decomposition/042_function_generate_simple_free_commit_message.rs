async fn generate_simple_free_commit_message(
    config: &mut SimpleFreeOpenRouterConfig, 
    diff: &str, 
    cli: &Cli
) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    
    // Get available free models
    if cli.verbose {
        println!("\n=== Getting available free models from OpenRouter ===");
        println!("API Key: {}", config.api_key.chars().take(8).collect::<String>() + "..." + &config.api_key.chars().rev().take(4).collect::<String>());
    }
    
    let available_models = match get_available_free_models(&config.api_key, cli.simulate_offline).await {
        Ok(models) => models,
        Err(e) => {
            println!("Error fetching models from OpenRouter: {}", e);
            println!("This could be due to network connectivity issues or API changes.");
            println!("Using fallback predefined free models list...");
            
            // As a last resort, try to use our predefined list directly
            match fallback_to_preferred_models() {
                Ok(models) => models,
                Err(e) => return Err(format!("Failed to get models and fallback also failed: {}", e)),
            }
        }
    };
    
    if cli.verbose {
        println!("Found {} free models:", available_models.len());
        for (i, model) in available_models.iter().enumerate().take(10) {
            println!("  {}. {}", i+1, model);
        }
        if available_models.len() > 10 {
            println!("  ... and {} more", available_models.len() - 10);
        }
    }
    
    if available_models.is_empty() {
        return Err("No free models available on OpenRouter".to_string());
    }
    
    // Find the best available model using our advanced management system
    let model = find_best_available_model(&available_models, config)
        .ok_or_else(|| "Failed to find a suitable model, please try again later".to_string())?;
    
    // Use the smart diff processing function
    let processed_diff = process_git_diff_output(diff);

    let prompt = format!(
        "Generate ONLY the git commit message string based on the provided diff. Follow the Conventional Commits specification (type: description). Do NOT include any introductory phrases, explanations, or markdown formatting like ```.
Examples:
- feat: Add user authentication feature
- fix: Correct calculation error in payment module
- docs: Update README with installation instructions
- style: Format code according to style guide
- refactor: Simplify database query logic
- test: Add unit tests for user service
- chore: Update dependencies

Git Diff:
```diff
{}
```
Commit Message ONLY:",
        processed_diff
    );

    // Show context in verbose mode
    if cli.verbose {
        println!("\n=== Context for LLM ===");
        println!("Provider: Simple Free OpenRouter");
        println!("Model: {}", model);
        let model_status = if let Some(stats) = config.model_stats.get(&model) {
            if stats.blacklisted {
                "BLACKLISTED (being retried)"
            } else if stats.jail_until.is_some() && stats.jail_until.unwrap() > chrono::Utc::now() {
                "JAILED (being retried)"
            } else {
                "ACTIVE"
            }
        } else {
            "NEW (no history)"
        };
        println!("Model status: {}", model_status);
        println!("Max tokens: {}", config.max_tokens);
        println!("Temperature: {}", config.temperature);
        println!("\n=== Prompt ===\n{}", prompt);
        println!("\n=== Sending request to API ===");
    }

    let request_body = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
    });

    // Function to make an API request
    let make_request = async {
        client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", &config.api_key))
            .header("HTTP-Referer", "https://suenot.github.io/aicommit/")
            .header("X-Title", "aicommit")
            .header("X-Description", "A CLI tool that generates concise and descriptive git commit messages")
            .json(&request_body)
            .send()
            .await
    };

    // Make the request with a timeout
    let response = match tokio::time::timeout(std::time::Duration::from_secs(30), make_request).await {
        Ok(result) => match result {
            Ok(response) => response,
            Err(e) => {
                // Record failure with the model
                let stats = config.model_stats.entry(model.clone()).or_default();
                record_model_failure(stats);
                
                // Save the updated config
                save_simple_free_config(config)?;
                
                return Err(format!("Request error: {}", e));
            },
        },
        Err(_) => {
            // Record failure with the model - but be careful not to penalize for network timeouts
            // Only record as a model failure if this is repeated
            let stats = config.model_stats.entry(model.clone()).or_default();
            
            // Check if this is likely a network issue rather than model issue
            let is_likely_network_issue = stats.failure_count == 0 || 
                (stats.last_success.is_some() && 
                 chrono::Utc::now() - stats.last_success.unwrap() < chrono::Duration::hours(1));
            
            if !is_likely_network_issue {
                record_model_failure(stats);
                save_simple_free_config(config)?;
            }
            
            return Err("Request timed out after 30 seconds".to_string());
        },
    };

    if !response.status().is_success() {
        // Get the status code before consuming the response
        let status_code = response.status();
        
        // Try to get the error message from the response
        let error_text = match response.text().await {
            Ok(text) => format!("API error response: {}", text),
            Err(_) => format!("API returned status code: {}", status_code),
        };
        
        if cli.verbose {
            println!("Request failed for model {}: {}", model, error_text);
        }
        
        // Record failure with the model
        let stats = config.model_stats.entry(model.clone()).or_default();
        record_model_failure(stats);
        
        // Save the updated config
        save_simple_free_config(config)?;
        
        return Err(format!("API request failed with model {}: {}", model, error_text));
    }

    // Try to parse the response body
    let response_text = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let response_data: Result<OpenRouterResponse, _> = serde_json::from_str(&response_text);

    if let Err(e) = &response_data {
        if cli.verbose {
            println!("Failed to parse response: {}", e);
            println!("Response body: {}", response_text);
        }
        
        // Record failure with the model
        let stats = config.model_stats.entry(model.clone()).or_default();
        record_model_failure(stats);
        
        // Save the updated config
        save_simple_free_config(config)?;
        
        return Err(format!("Failed to parse response JSON: {} (Response: {})", e, 
                         if response_text.len() > 100 { 
                             format!("{}...", &response_text[..100]) 
                         } else { 
                             response_text.clone() 
                         }));
    }
    
    let response_data = response_data.unwrap();
    
    let raw_message = response_data.choices
        .get(0)
        .ok_or_else(|| {
            // Record failure with the model
            let stats = config.model_stats.entry(model.clone()).or_default();
            record_model_failure(stats);

            // Save the updated config
            let _ = save_simple_free_config(config);

            "No choices in response"
        })?
        .message
        .content
        .clone();

    // Clean and validate the message (similar to Ollama implementation)
    let message = raw_message
        .trim()
        .trim_start_matches(['\\', '/', '-', ' '])
        .trim_end_matches(['\\', '/', '-', ' ', '.'])
        .trim()
        .to_string();

    if message.is_empty() || message.len() < 3 {
        // Record failure with the model
        let stats = config.model_stats.entry(model.clone()).or_default();
        record_model_failure(stats);

        // Save the updated config
        let _ = save_simple_free_config(config);

        return Err("Generated commit message is too short or empty".to_string());
    }

    // Calculate usage info
    let usage = UsageInfo {
        input_tokens: response_data.usage.prompt_tokens,
        output_tokens: response_data.usage.completion_tokens,
        total_cost: 0.0, // It's free!
        model_used: Some(model.clone()),
    };

    // Record success with the model
    let stats = config.model_stats.entry(model.clone()).or_default();
    record_model_success(stats);
    
    // Update last used model
    config.last_used_model = Some(model.clone());
    
    // Save the updated config
    save_simple_free_config(config)?;

    if cli.verbose {
        println!("Successfully generated commit message using model: {}", model);
    }

    Ok((message, usage))
}