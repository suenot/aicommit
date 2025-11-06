async fn generate_openrouter_commit_message(config: &OpenRouterConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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
        println!("Provider: OpenRouter");
        println!("Model: {}", config.model);
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
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", &config.api_key))
        .header("HTTP-Referer", "https://suenot.github.io/aicommit/")
        .header("X-Title", "aicommit")
        .header("X-Description", "A CLI tool that generates concise and descriptive git commit messages")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed: {}", response.status()));
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

    // Используем информацию о токенах из ответа API
    let usage = UsageInfo {
        input_tokens: response_data.usage.prompt_tokens,
        output_tokens: response_data.usage.completion_tokens,
        // Примерная стоимость: $0.14/100K токенов для mistral-tiny
        total_cost: (response_data.usage.total_tokens as f32) * 0.0000014,
        model_used: Some(config.model.clone()),
    };

    Ok((message, usage))
}