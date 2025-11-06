async fn setup_openai_compatible_provider() -> Result<OpenAICompatibleConfig, String> {
    let api_key: String = Input::new()
        .with_prompt("Enter API key (can be any non-empty string for local models like LM Studio)")
        .interact_text()
        .map_err(|e| format!("Failed to get API key: {}", e))?;

    let api_url: String = Input::new()
        .with_prompt("Enter complete API URL (e.g., https://api.example.com/v1/chat/completions)")
        .interact_text()
        .map_err(|e| format!("Failed to get API URL: {}", e))?;

    let model_options = &[
        "gpt-3.5-turbo",
        "gpt-4",
        "gpt-4-turbo",
        "gpt-4o-mini",
        "claude-3-sonnet",
        "claude-2",
        "custom (enter manually)",
    ];
    
    let model_selection = Select::new()
        .with_prompt("Select a model")
        .items(model_options)
        .default(0)
        .interact()
        .map_err(|e| format!("Failed to get model selection: {}", e))?;

    let model = if model_selection == model_options.len() - 1 {
        // Custom model input
        Input::new()
            .with_prompt("Enter model name")
            .interact_text()
            .map_err(|e| format!("Failed to get model name: {}", e))?
    } else {
        model_options[model_selection].to_string()
    };

    let max_tokens: String = Input::new()
        .with_prompt("Enter max tokens")
        .default("200".into())
        .interact_text()
        .map_err(|e| format!("Failed to get max tokens: {}", e))?;
    let max_tokens: i32 = max_tokens.parse()
        .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

    let temperature: String = Input::new()
        .with_prompt("Enter temperature")
        .default("0.2".into())
        .interact_text()
        .map_err(|e| format!("Failed to get temperature: {}", e))?;
    let temperature: f32 = temperature.parse()
        .map_err(|e| format!("Failed to parse temperature: {}", e))?;

    Ok(OpenAICompatibleConfig {
        id: Uuid::new_v4().to_string(),
        provider: "openai_compatible".to_string(),
        api_key,
        api_url,
        model,
        max_tokens,
        temperature,
    })
}