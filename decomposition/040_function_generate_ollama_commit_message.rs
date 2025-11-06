async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    // Use the smart diff processing function instead of simple truncation
    let processed_diff = process_git_diff_output(diff);

    let prompt = format!(
        "Generate ONLY the raw git commit message string (one line, max 72 chars) based on the diff. Follow Conventional Commits (type: description). Do NOT include any introductory text, explanations, or ```.
Examples:
- feat: add user login
- fix: correct payment calculation
- docs: update readme
- style: format code
- refactor: simplify query
- test: add user tests
- chore: update deps

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
        println!("Provider: Ollama");
        println!("Model: {}", config.model);
        println!("URL: {}", config.url);
        println!("Max tokens: {}", config.max_tokens);
        println!("Temperature: {}", config.temperature);
        println!("\n=== Prompt ===\n{}", prompt);
        println!("\n=== Sending request to API ===");
    }

    let request_body = serde_json::json!({
        "model": config.model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": config.temperature,
            "num_predict": config.max_tokens
        }
    });

    let response = client
        .post(format!("{}/api/generate", config.url))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API returned an error ({}): {}", status, error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

    let commit_message = json["response"]
        .as_str()
        .map(|s| s.trim()
            .trim_start_matches(['\\', '/', '-', ' '])
            .trim_end_matches(['\\', '/', '-', ' ', '.'])
            .trim()
            .to_string())
        .ok_or_else(|| "No text found in API response".to_string())?;

    if commit_message.is_empty() || commit_message.len() < 3 {
        return Err("Generated commit message is too short or empty".to_string());
    }

    // For Ollama, we estimate tokens based on characters (rough approximation)
    let input_tokens = (diff.len() / 4) as i32;
    let output_tokens = (commit_message.len() / 4) as i32;
    
    let input_cost = input_tokens as f32 * 0.0 / 1000.0;
    let output_cost = output_tokens as f32 * 0.0 / 1000.0;
    let total_cost = input_cost + output_cost;

    let usage = UsageInfo {
        input_tokens,
        output_tokens,
        total_cost,
        model_used: Some(config.model.clone()),
    };

    Ok((commit_message, usage))
}