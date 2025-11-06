async fn generate_opencode_commit_message(config: &OpenCodeConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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