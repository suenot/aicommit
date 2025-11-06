async fn dry_run(cli: &Cli) -> Result<String, String> {
    // Check .gitignore at startup if not skipping
    if !cli.no_gitignore_check {
        Config::check_gitignore()?;
    }
    
    // Load configuration
    let config = Config::load()?;
    
    // Make sure we have a provider
    if config.providers.is_empty() {
        return Err("No providers configured. Please run with --add-provider to add a provider.".to_string());
    }
    
    // Check if we're in a git repository and get the diff
    let diff = match get_git_diff(cli) {
        Ok(diff) => diff,
        Err(e) => {
            return Err(format!("Failed to get git diff: {}", e));
        }
    };
    
    // Generate commit message
    let (message, _) = {
        let active_provider = config.providers.iter().find(|p| match p {
            ProviderConfig::OpenRouter(c) => c.id == config.active_provider,
            ProviderConfig::Ollama(c) => c.id == config.active_provider,
            ProviderConfig::OpenAICompatible(c) => c.id == config.active_provider,
            ProviderConfig::SimpleFreeOpenRouter(c) => c.id == config.active_provider,
            ProviderConfig::ClaudeCode(c) => c.id == config.active_provider,
            ProviderConfig::OpenCode(c) => c.id == config.active_provider,
        }).ok_or_else(|| "No active provider found".to_string())?;

        let mut attempt_count = 0;
        loop {
            if attempt_count > 0 {
                eprintln!("Retry attempt {} of {}", attempt_count + 1, config.retry_attempts);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }

            let result = match active_provider {
                ProviderConfig::OpenRouter(c) => generate_openrouter_commit_message(c, &diff, cli).await,
                ProviderConfig::Ollama(c) => generate_ollama_commit_message(c, &diff, cli).await,
                ProviderConfig::OpenAICompatible(c) => generate_openai_compatible_commit_message(c, &diff, cli).await,
                ProviderConfig::SimpleFreeOpenRouter(c) => {
                    let mut c_clone = c.clone();
                    let result = generate_simple_free_commit_message(&mut c_clone, &diff, cli).await;
                    // We don't need to save failed models in dry-run mode
                    result
                },
                ProviderConfig::ClaudeCode(c) => generate_claude_code_commit_message(c, &diff, cli).await,
                ProviderConfig::OpenCode(c) => generate_opencode_commit_message(c, &diff, cli).await,
            };

            match result {
                Ok(result) => break result,
                Err(e) => {
                    eprintln!("Attempt {} failed: {}", attempt_count + 1, e);
                    attempt_count += 1;
                    if attempt_count >= config.retry_attempts {
                        return Err(format!("Failed to generate commit message after {} attempts. Last error: {}", config.retry_attempts, e));
                    }
                }
            }
        }
    };

    // Final validation before returning in dry-run mode
    if message.trim().is_empty() {
        return Err("Aborting commit due to empty commit message.".to_string());
    }

    // In dry-run mode, only return the generated message
    Ok(message)
}