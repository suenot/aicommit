// Git operations

use std::process::Command;
use dialoguer::Input;
use tracing::{info, error, debug};

// From: 020_function_process_git_diff_output.rs
pub fn process_git_diff_output(diff: &str) -> String {
    // Early return if diff is small enough
    if diff.len() <= MAX_DIFF_CHARS {
        return diff.to_string();
    }
    
    // Split the diff into file sections
    let file_pattern = r"(?m)^diff --git ";
    let sections: Vec<&str> = diff.split(file_pattern).collect();
    
    // First segment is empty or small header - keep it as is
    let mut processed = if !sections.is_empty() && !sections[0].trim().is_empty() {
        sections[0].to_string()
    } else {
        String::new()
    };
    
    // Process each file section
    for (i, section) in sections.iter().enumerate().skip(1) {
        // Skip empty sections
        if section.trim().is_empty() {
            continue;
        }
        
        // Add the "diff --git " prefix back (we split on it, so it's missing)
        processed.push_str("diff --git ");
        
        // Check if this section is too large
        if section.len() > MAX_FILE_DIFF_CHARS {
            // Find the file name from the section
            let _file_name = if let Some(end) = section.find('\n') {
                section[..end].trim()
            } else {
                section.trim()
            };
            
            // Take the header part (usually 4-5 lines) - this includes the file name, index, and --- +++ lines
            let _header_end = section.lines().take(5).collect::<Vec<&str>>().join("\n").len();
            
            // Take the beginning of the diff content - safely truncate at char boundary
            let safe_len = get_safe_slice_length(section, MAX_FILE_DIFF_CHARS.min(section.len()));
            let content = &section[..safe_len];
            
            // Add truncation notice for this specific file
            processed.push_str(&format!("{}\n\n[... diff for this file truncated due to length ...]", content));
        } else {
            // Section is small enough, add it as is
            processed.push_str(section);
        }
        
        // Add separating newline if needed
        if i < sections.len() - 1 && !processed.ends_with('\n') {
            processed.push('\n');
        }
    }
    
    // Final overall truncation check (as a safety measure)
    if processed.len() > MAX_DIFF_CHARS {
        let safe_len = get_safe_slice_length(&processed, MAX_DIFF_CHARS - 100);
        processed = format!("{}...\n\n[... total diff truncated due to length (first {} chars shown) ...]", 
            &processed[..safe_len], 
            safe_len);
    }
    
    processed
}

// From: 028_function_watch_and_commit.rs
pub async fn watch_and_commit(config: &Config, cli: &Cli) -> Result<(), String> {
    let wait_for_edit = cli.wait_for_edit.as_ref()
        .map(|w| parse_duration(w))
        .transpose()?;

    info!("Watching for changes...");
    if let Some(delay) = wait_for_edit {
        info!("Waiting {:?} after edits before committing", delay);
    }

    // Initialize waiting list for files with their last modification timestamps
    let mut waiting_files: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
    
    // Отслеживание хешей содержимого файлов для определения реальных изменений
    let mut file_hashes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    loop {
        // Sleep for a short period to reduce CPU usage
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Get list of modified files
        let output = Command::new("sh")
            .arg("-c")
            .arg("git ls-files -m -o --exclude-standard")
            .output()
            .map_err(|e| format!("Failed to check modified files: {}", e))?;

        if !output.status.success() {
            continue;
        }

        let modified_files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Check if we have any new modified files
        if !modified_files.is_empty() {
            for file in &modified_files {
                // Проверяем, действительно ли содержимое файла изменилось
                // Получаем хеш содержимого файла
                let hash_output = Command::new("sh")
                    .arg("-c")
                    .arg(&format!("git hash-object \"{}\"", file.replace("\"", "\\\"")))
                    .output();
                
                match hash_output {
                    Ok(output) if output.status.success() => {
                        let new_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let old_hash = file_hashes.get(file).cloned().unwrap_or_default();
                        
                        // Проверяем, изменился ли хеш файла
                        let is_real_change = new_hash != old_hash;
                        
                        // Обновляем сохраненный хеш файла
                        file_hashes.insert(file.clone(), new_hash);
                        
                        // Если файл действительно изменился, обрабатываем изменение
                        if is_real_change {
                            // Log the change
                            println!("File changed: {}", file);
                            
                            if let Some(delay) = wait_for_edit {
                                // Check if file is already in waiting list
                                if waiting_files.contains_key(file) {
                                    // Reset timer for this file
                                    let _ready_time = std::time::Instant::now() + delay;
                                    let ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                        .map(|dt| dt.format("%H:%M:%S").to_string())
                                        .unwrap_or_else(|| "unknown time".to_string());
                                    
                                    println!("Resetting timer for file: {} (will be ready at {})", file, ready_time_str);
                                    waiting_files.insert(file.clone(), std::time::Instant::now());
                                } else {
                                    // Add file to waiting list with current timestamp
                                    let _ready_time = std::time::Instant::now() + delay;
                                    let ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                        .map(|dt| dt.format("%H:%M:%S").to_string())
                                        .unwrap_or_else(|| "unknown time".to_string());
                                    
                                    println!("Adding file to waiting list: {} (will be ready at {})", file, ready_time_str);
                                    waiting_files.insert(file.clone(), std::time::Instant::now());
                                }
                            } else {
                                // If no wait-for-edit delay specified, immediately add the file
                                let git_add = Command::new("sh")
                                    .arg("-c")
                                    .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                                    .output()
                                    .map_err(|e| format!("Failed to add file: {}", e))?;

                                if !git_add.status.success() {
                                    println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                                }
                                
                                // If there are changes to commit, do it immediately
                                match get_git_diff(cli) {
                                    Ok(diff) if !diff.is_empty() => {
                                        match run_commit(config, cli).await {
                                            Ok(_) => {
                                                println!("\nCommitted changes.");
                                                println!("Continuing to watch for changes...");
                                            }
                                            Err(e) => println!("Failed to commit: {}", e),
                                        }
                                    }
                                    _ => {} // No changes or error, continue watching
                                }
                            }
                        }
                    },
                    _ => {
                        // Если не удалось получить хеш, обрабатываем как изменение
                        // для новых файлов это нормально
                        println!("File changed: {}", file);
                        
                        if let Some(delay) = wait_for_edit {
                            if waiting_files.contains_key(file) {
                                let _ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                    .map(|dt| dt.format("%H:%M:%S").to_string())
                                    .unwrap_or_else(|| "unknown time".to_string());
                                
                                println!("Resetting timer for file: {} (will be ready at {})", file, _ready_time_str);
                                waiting_files.insert(file.clone(), std::time::Instant::now());
                            } else {
                                let _ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                    .map(|dt| dt.format("%H:%M:%S").to_string())
                                    .unwrap_or_else(|| "unknown time".to_string());
                                
                                println!("Adding file to waiting list: {} (will be ready at {})", file, _ready_time_str);
                                waiting_files.insert(file.clone(), std::time::Instant::now());
                            }
                        } else {
                            let git_add = Command::new("sh")
                                .arg("-c")
                                .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                                .output()
                                .map_err(|e| format!("Failed to add file: {}", e))?;

                            if !git_add.status.success() {
                                println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                            }
                            
                            match get_git_diff(cli) {
                                Ok(diff) if !diff.is_empty() => {
                                    match run_commit(config, cli).await {
                                        Ok(_) => {
                                            println!("\nCommitted changes.");
                                            println!("Continuing to watch for changes...");
                                        }
                                        Err(e) => println!("Failed to commit: {}", e),
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // If wait-for-edit is specified, check the waiting list
        if let Some(delay) = wait_for_edit {
            let now = std::time::Instant::now();
            let mut files_to_commit = Vec::new();
            
            // Check each file in the waiting list
            for (file, timestamp) in &waiting_files {
                if now.duration_since(*timestamp) >= delay {
                    // File has been stable for the specified time, add it to git
                    files_to_commit.push(file.clone());
                }
            }
            
            // Add stable files to git and commit
            if !files_to_commit.is_empty() {
                for file in &files_to_commit {
                    println!("File ready for commit: {} (stable for {:?})", file, delay);
                    
                    let git_add = Command::new("sh")
                        .arg("-c")
                        .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                        .output()
                        .map_err(|e| format!("Failed to add file: {}", e))?;

                    if !git_add.status.success() {
                        println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                    }
                }
                
                // Commit the changes
                match get_git_diff(cli) {
                    Ok(diff) if !diff.is_empty() => {
                        match run_commit(config, cli).await {
                            Ok(_) => {
                                println!("\nCommitted changes for stable files.");
                                println!("Continuing to watch for changes...");
                                
                                // Remove committed files from waiting list
                                for file in files_to_commit {
                                    waiting_files.remove(&file);
                                    // Также обновляем хеш после коммита
                                    if let Ok(output) = Command::new("sh")
                                        .arg("-c")
                                        .arg(&format!("git hash-object \"{}\"", file.replace("\"", "\\\"")))
                                        .output() {
                                        if output.status.success() {
                                            let new_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                            file_hashes.insert(file, new_hash);
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("Failed to commit: {}", e),
                        }
                    }
                    _ => {} // No changes or error, continue watching
                }
            }
        }
    }
}

// From: 030_function_get_git_diff.rs
pub fn get_git_diff(cli: &Cli) -> Result<String, String> {
    // First check if current directory is a git repository
    let is_git_repo = Command::new("sh")
        .arg("-c")
        .arg("git rev-parse --is-inside-work-tree")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_git_repo {
        return Err("Current directory is not a git repository".to_string());
    }

    // Check for unstaged changes first
    let status_output = Command::new("sh")
        .arg("-c")
        .arg("git status --porcelain")
        .output()
        .map_err(|e| format!("Failed to execute git status: {}", e))?;

    let status = String::from_utf8_lossy(&status_output.stdout).to_string();
    
    // If --add flag is set and there are unstaged changes, add them
    if cli.add && status.lines().any(|line| {
        line.starts_with(" M") || // Modified but not staged
        line.starts_with("MM") || // Modified and staged with new modifications
        line.starts_with("??")    // Untracked files
    }) {
        let add_output = Command::new("sh")
            .arg("-c")
            .arg("git add .")
            .output()
            .map_err(|e| format!("Failed to execute git add: {}", e))?;

        if !add_output.status.success() {
            return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
        }
    }

    // Try to get diff of staged changes
    let diff_cmd = if cli.dry_run {
        // For dry run, try to get changes in a more robust way
        // First try --cached, and if it fails, try without --cached
        match Command::new("sh")
            .arg("-c")
            .arg("git diff --cached")
            .output() {
                Ok(output) if output.status.success() => {
                    let diff = String::from_utf8_lossy(&output.stdout).to_string();
                    if !diff.trim().is_empty() {
                        return Ok(diff);
                    }
                    // If no staged changes, fall back to unstaged changes
                    "git diff"
                },
                _ => "git diff" // Fall back to unstaged changes
            }
    } else {
        "git diff --cached"
    };

    let diff_output = Command::new("sh")
        .arg("-c")
        .arg(diff_cmd)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", diff_cmd, e))?;

    if !diff_output.status.success() {
        return Err(format!("Git diff command failed: {}", String::from_utf8_lossy(&diff_output.stderr)));
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    if diff.trim().is_empty() {
        return Err("No changes to commit".to_string());
    }

    Ok(diff)
}

// From: 031_function_create_git_commit.rs
pub fn create_git_commit(message: &str) -> Result<(), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(&format!("git commit -m '{}'", message.replace("'", "'\\''")))
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// From: 034_function_run_commit.rs
pub async fn run_commit(config: &Config, cli: &Cli) -> Result<(), String> {
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
                println!("Setting upstream for branch '{}' to 'origin/{}'", branch_name, branch_name);
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

        let push_cmd = if has_upstream {
            // Если upstream настроен, выполняем обычный push
            "git push"
        } else {
            // Если upstream не настроен, настраиваем его
            println!("Setting upstream for branch '{}' to 'origin/{}'", branch_name, branch_name);
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

        println!("Changes successfully pushed.");
    }

    Ok(())
}

// From: 039_function_generate_openrouter_commit_message.rs
pub async fn generate_openrouter_commit_message(config: &OpenRouterConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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

// From: 040_function_generate_ollama_commit_message.rs
pub async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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

// From: 041_function_generate_openai_compatible_commit_message.rs
pub async fn generate_openai_compatible_commit_message(config: &OpenAICompatibleConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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

// From: 042_function_generate_simple_free_commit_message.rs
pub async fn generate_simple_free_commit_message(
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

// From: 043_function_generate_claude_code_commit_message.rs
pub async fn generate_claude_code_commit_message(config: &ClaudeCodeConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    // Use the smart diff processing function
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
        println!("Provider: Claude Code");
        println!("\n=== Prompt ===\n{}", prompt);
        println!("\n=== Executing: claude -p \"<prompt>\" ===");
    }

    // Execute claude CLI with the prompt
    let output = Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("Claude CLI not found. Please install Claude Code CLI and ensure 'claude' is in your system PATH. Installation instructions: https://docs.anthropic.com/claude/docs/claude-cli")
            } else {
                format!("Failed to execute Claude CLI: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Claude CLI failed with exit code {}: {}", output.status.code().unwrap_or(-1), stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commit_message = stdout
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_start_matches(['\\', '/', '-', ' '])
        .trim_end_matches(['\\', '/', '-', ' ', '.'])
        .trim()
        .to_string();

    if commit_message.is_empty() || commit_message.len() < 3 {
        return Err("Generated commit message is too short or empty".to_string());
    }

    // For Claude Code, we estimate tokens based on characters (rough approximation)
    let input_tokens = (diff.len() / 4) as i32;
    let output_tokens = (commit_message.len() / 4) as i32;

    let usage = UsageInfo {
        input_tokens,
        output_tokens,
        total_cost: 0.0, // Claude Code may have its own billing
        model_used: Some("claude-code".to_string()),
    };

    Ok((commit_message, usage))
}

// From: 044_function_generate_opencode_commit_message.rs
pub async fn generate_opencode_commit_message(config: &OpenCodeConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    // Use the smart diff processing function
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
        println!("Provider: OpenCode");
        println!("\n=== Prompt ===\n{}", prompt);
        println!("\n=== Executing: opencode run \"<prompt>\" ===");
    }

    // Execute opencode CLI with the prompt
    let output = Command::new("opencode")
        .arg("run")
        .arg(&prompt)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("OpenCode CLI not found. Please install OpenCode CLI and ensure 'opencode' is in your system PATH. Installation instructions: https://github.com/opencodeai/opencode")
            } else {
                format!("Failed to execute OpenCode CLI: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OpenCode CLI failed with exit code {}: {}", output.status.code().unwrap_or(-1), stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commit_message = stdout
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_start_matches(['\\', '/', '-', ' '])
        .trim_end_matches(['\\', '/', '-', ' ', '.'])
        .trim()
        .to_string();

    if commit_message.is_empty() || commit_message.len() < 3 {
        return Err("Generated commit message is too short or empty".to_string());
    }

    // For OpenCode, we estimate tokens based on characters (rough approximation)
    let input_tokens = (diff.len() / 4) as i32;
    let output_tokens = (commit_message.len() / 4) as i32;

    let usage = UsageInfo {
        input_tokens,
        output_tokens,
        total_cost: 0.0, // OpenCode may have its own billing
        model_used: Some("opencode".to_string()),
    };

    Ok((commit_message, usage))
}

