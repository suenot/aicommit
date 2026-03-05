async fn run_commit(config: &Config, cli: &Cli) -> Result<(), String> {
    // Update versions if specified
    let mut new_version = String::new();

    // Update version in version file
    if let Some(version_file) = cli.version_file.as_ref() {
        if cli.version_iterate {
            update_version_file(version_file).await?;
        }

        new_version = tokio::fs::read_to_string(version_file)
            .await
            .map_err(|e| format!("Failed to read version file: {}", e))?
            .trim()
            .to_string();
    }

    // Update version in Cargo.toml
    if cli.version_cargo {
        if new_version.is_empty() {
            return Err("Error: --version-file must be specified when using --version-cargo".to_string());
        }
        update_cargo_version(&new_version).await?;
    }

    // Update version in package.json
    if cli.version_npm {
        if new_version.is_empty() {
            return Err("Error: --version-file must be specified when using --version-npm".to_string());
        }
        update_npm_version(&new_version).await?;
    }

    // Update version on GitHub
    if cli.version_github {
        if new_version.is_empty() {
            return Err("Error: --version-file must be specified when using --version-github".to_string());
        }
        update_github_version(&new_version)?;
    }

    // Stage version changes if any version flags were used
    if cli.version_iterate || cli.version_cargo || cli.version_npm || cli.version_github {
        let add_output = Command::new("sh")
            .arg("-c")
            .arg("git add .")
            .output()
            .map_err(|e| format!("Failed to execute git add: {}", e))?;

        if !add_output.status.success() {
            return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
        }
    }

    // Get the diff (will handle git add if needed)
    let diff = get_git_diff(cli)?;

    // Show diff in verbose mode
    if cli.verbose {
        println!("\n=== Git Diff ===\n{}", diff);
    }

    // Generate commit message based on the active provider
    let (message, usage_info) = {
        let active_provider = config.providers.iter().find(|p| match p {
            ProviderConfig::OpenRouter(c) => c.id == config.active_provider,
            ProviderConfig::Ollama(c) => c.id == config.active_provider,
            ProviderConfig::OpenAICompatible(c) => c.id == config.active_provider,
            ProviderConfig::SimpleFreeOpenRouter(c) => c.id == config.active_provider,
            ProviderConfig::ClaudeCode(c) => c.id == config.active_provider,
            ProviderConfig::OpenCode(c) => c.id == config.active_provider,
        }).ok_or("No active provider found")?;

        let mut attempt_count = 0;
        loop {
            if attempt_count > 0 {
                println!("Retry attempt {} of {}", attempt_count + 1, config.retry_attempts);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }

            let result = match active_provider {
                ProviderConfig::OpenRouter(c) => generate_openrouter_commit_message(c, &diff, cli).await,
                ProviderConfig::Ollama(c) => generate_ollama_commit_message(c, &diff, cli).await,
                ProviderConfig::OpenAICompatible(c) => generate_openai_compatible_commit_message(c, &diff, cli).await,
                ProviderConfig::SimpleFreeOpenRouter(c) => {
                    // We need to mutate the config to track failed models, so we need to load it again to update later
                    let mut c_clone = c.clone();
                    let result = generate_simple_free_commit_message(&mut c_clone, &diff, cli).await;

                    // We've already saved the model information in the result
                    result
                },
                ProviderConfig::ClaudeCode(c) => generate_claude_code_commit_message(c, &diff, cli).await,
                ProviderConfig::OpenCode(c) => generate_opencode_commit_message(c, &diff, cli).await,
            };

            match result {
                Ok(result) => {
                    if attempt_count > 0 {
                        println!("Successfully generated commit message after {} attempts", attempt_count + 1);
                    }
                    break result;
                }
                Err(e) => {
                    println!("Attempt {} failed: {}", attempt_count + 1, e);
                    attempt_count += 1;
                    if attempt_count >= config.retry_attempts {
                        return Err(format!("Failed to generate commit message after {} attempts. Last error: {}", config.retry_attempts, e));
                    }
                }
            }
        }
    };

    // Final validation before committing
    if message.trim().is_empty() {
        return Err("Aborting commit due to empty commit message.".to_string());
    }

    println!("Generated commit message: \"{}\"\n", message);
    println!("Tokens: {}↑ {}↓", usage_info.input_tokens, usage_info.output_tokens);
    println!("API Cost: ${:.4}", usage_info.total_cost);

    // Display which model was used for Simple Free mode if applicable
    if let Some(model) = &usage_info.model_used {
        println!("Model used: {}", model);
    }

    create_git_commit(&message)?;
    println!("Commit successfully created.");

    // Pull changes if --pull flag is set
    if cli.pull {
        // Проверяем, имеет ли текущая ветка upstream
        let check_upstream = Command::new("sh")
            .arg("-c")
            .arg("git rev-parse --abbrev-ref --symbolic-full-name @{upstream} 2>/dev/null || echo \"\"")
            .output()
            .map_err(|e| format!("Failed to check upstream branch: {}", e))?;

        let has_upstream = !String::from_utf8_lossy(&check_upstream.stdout).trim().is_empty();

        // Получаем имя текущей ветки
        let branch_output = Command::new("sh")
            .arg("-c")
            .arg("git rev-parse --abbrev-ref HEAD")
            .output()
            .map_err(|e| format!("Failed to get current branch name: {}", e))?;

        let branch_name = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

        if !has_upstream {
            // Проверяем существование удаленной ветки
            let remote_branch_check = Command::new("sh")
                .arg("-c")
                .arg(format!("git ls-remote --heads origin {} | wc -l", branch_name))
                .output()
                .map_err(|e| format!("Failed to check remote branch: {}", e))?;

            let remote_branch_exists = String::from_utf8_lossy(&remote_branch_check.stdout)
                .trim()
                .parse::<i32>()
                .unwrap_or(0) > 0;

            if remote_branch_exists {
                // Настраиваем upstream для существующей удаленной ветки
                let remote_url_info = get_remote_https_url("origin")
                    .map(|url| format!(" ({})", url))
                    .unwrap_or_default();
                println!("Setting upstream for branch '{}' to 'origin/{}'{}", branch_name, branch_name, remote_url_info);
                let set_upstream = Command::new("sh")
                    .arg("-c")
                    .arg(format!("git branch --set-upstream-to=origin/{} {}", branch_name, branch_name))
                    .output()
                    .map_err(|e| format!("Failed to set upstream: {}", e))?;

                if !set_upstream.status.success() {
                    return Err(String::from_utf8_lossy(&set_upstream.stderr).to_string());
                }
            } else {
                // Если удаленная ветка не существует, пропускаем pull
                println!("Skipping pull: remote branch 'origin/{}' does not exist yet", branch_name);
                return Ok(());
            }
        }

        let pull_output = Command::new("sh")
            .arg("-c")
            .arg("git pull --no-rebase --no-edit")
            .output()
            .map_err(|e| format!("Failed to execute git pull: {}", e))?;

        if !pull_output.status.success() {
            let error_msg = String::from_utf8_lossy(&pull_output.stderr);
            if error_msg.contains("Automatic merge failed") {
                return Err("Automatic merge failed. Please resolve conflicts manually.".to_string());
            }

            return Err(format!("Failed to pull changes: {}", error_msg));
        }

        println!("Successfully pulled changes.");
    }

    // Push changes if --push flag is set
    if cli.push {
        // Проверяем, имеет ли текущая ветка upstream
        let check_upstream = Command::new("sh")
            .arg("-c")
            .arg("git rev-parse --abbrev-ref --symbolic-full-name @{upstream} 2>/dev/null || echo \"\"")
            .output()
            .map_err(|e| format!("Failed to check upstream branch: {}", e))?;

        let has_upstream = !String::from_utf8_lossy(&check_upstream.stdout).trim().is_empty();

        // Получаем имя текущей ветки
        let branch_output = Command::new("sh")
            .arg("-c")
            .arg("git rev-parse --abbrev-ref HEAD")
            .output()
            .map_err(|e| format!("Failed to get current branch name: {}", e))?;

        let branch_name = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

        let remote_url_info = get_remote_https_url("origin")
            .map(|url| format!(" ({})", url))
            .unwrap_or_default();

        let push_cmd = if has_upstream {
            // Если upstream настроен, выполняем обычный push
            "git push"
        } else {
            // Если upstream не настроен, настраиваем его
            println!("Setting upstream for branch '{}' to 'origin/{}'{}", branch_name, branch_name, remote_url_info);
            &format!("git push --set-upstream origin {}", branch_name)
        };

        let push_output = Command::new("sh")
            .arg("-c")
            .arg(push_cmd)
            .output()
            .map_err(|e| format!("Failed to execute git push: {}", e))?;

        if !push_output.status.success() {
            return Err(String::from_utf8_lossy(&push_output.stderr).to_string());
        }

        println!("Changes successfully pushed to origin/{}{}.", branch_name, remote_url_info);
    }

    Ok(())
}