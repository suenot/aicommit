async fn generate_openai_compatible_commit_message(config: &OpenAICompatibleConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    // Use the smart diff processing function instead of simple truncation
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
        println!("Provider: OpenAI Compatible");
        println!("Model: {}", config.model);
        println!("API URL: {}", config.api_url);
        println!("Max tokens: {}", config.max_tokens);
        println!("Temperature: {}", config.temperature);
        println!("\n=== Prompt ===\n{}", prompt);
        println!("\n=== Sending request to API ===");
    }

    let request_body = json!({
        "model": &config.model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
    });

    let response = client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed: {} - {}", status, error_text));
    }

    let response_data: OpenRouterResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let raw_message = response_data.choices
        .get(0)
        .ok_or("No choices in response")?
        .message
        .content
        .clone();

    // Clean and validate the message (consistent with other implementations)
    let message = raw_message
        .trim()
        .trim_start_matches(['\\', '/', '-', ' '])
        .trim_end_matches(['\\', '/', '-', ' ', '.'])
        .trim()
        .to_string();

    if message.is_empty() || message.len() < 3 {
        return Err("Generated commit message is too short or empty".to_string());
    }

    let usage = UsageInfo {
        input_tokens: response_data.usage.prompt_tokens,
        output_tokens: response_data.usage.completion_tokens,
        total_cost: 0.0, // Set to 0 for OpenAI compatible APIs as we don't know the actual cost
        model_used: Some(config.model.clone()),
    };

    Ok((message, usage))
}