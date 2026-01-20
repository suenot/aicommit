// Main module - orchestrates all functionality

use std::fs;
use clap::Parser;
use tokio;
use tracing::info;
use logging::{LoggingConfig, init_logging};

// Module declarations
mod logging;
mod types;
mod version;
mod git;
mod providers;
mod models;
mod utils;
mod ignore;
mod hooks;

// Use declarations from our modules
use types::*;
use version::*;
use git::*;
use models::*;

// Constants
const MAX_DIFF_CHARS: usize = 15000; // Limit diff size to prevent excessive API usage
const MAX_FILE_DIFF_CHARS: usize = 3000; // Maximum characters per file diff section

// Define a list of preferred free models from best to worst
const PREFERRED_FREE_MODELS: &[&str] = &[
    // Meta models - Llama 4 series
    "meta-llama/llama-4-maverick:free",
    "meta-llama/llama-4-scout:free",
    
    // Ultra large models (200B+)
    "nvidia/llama-3.1-nemotron-ultra-253b-v1:free",
    "qwen/qwen3-235b-a22b:free",
    
    // Very large models (70B-200B)
    "meta-llama/llama-3.1-405b:free",
    "nvidia/llama-3.3-nemotron-super-49b-v1:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "deepseek/deepseek-r1-distill-llama-70b:free",
    "shisa-ai/shisa-v2-llama3.3-70b:free",
    
    // Large models (32B-70B)
    "qwen/qwen-2.5-72b-instruct:free",
    "qwen/qwen2.5-vl-72b-instruct:free",
    "bytedance-research/ui-tars-72b:free",
    "featherless/qwerky-72b:free",
    "thudm/glm-4-32b:free",
    "thudm/glm-z1-32b:free",
    "qwen/qwen3-32b:free",
    "qwen/qwen3-30b-a3b:free",
    "qwen/qwq-32b:free",
    "qwen/qwq-32b-preview:free",
    "deepseek/deepseek-r1-distill-qwen-32b:free",
    "arliai/qwq-32b-arliai-rpr-v1:free",
    "qwen/qwen2.5-vl-32b-instruct:free",
    "open-r1/olympiccoder-32b:free",
    "qwen/qwen-2.5-coder-32b-instruct:free",
    
    // Medium-large models (14B-30B)
    "mistralai/mistral-small-3.1-24b-instruct:free",
    "mistralai/mistral-small-24b-instruct-2501:free",
    "cognitivecomputations/dolphin3.0-r1-mistral-24b:free",
    "cognitivecomputations/dolphin3.0-mistral-24b:free",
    "google/gemma-3-27b-it:free",
    "google/gemini-2.0-flash-exp:free",
    "rekaai/reka-flash-3:free",
    
    // Medium models (7B-14B)
    "qwen/qwen3-14b:free",
    "deepseek/deepseek-r1-distill-qwen-14b:free",
    "agentica-org/deepcoder-14b-preview:free",
    "moonshotai/moonlight-16b-a3b-instruct:free",
    "opengvlab/internvl3-14b:free",
    "google/gemma-3-12b-it:free",
    "meta-llama/llama-3.2-11b-vision-instruct:free",
    "thudm/glm-4-9b:free",
    "thudm/glm-z1-9b:free",
    "google/gemma-2-9b-it:free",
    "qwen/qwen3-8b:free",
    "meta-llama/llama-3.1-8b-instruct:free",
    "nousresearch/deephermes-3-llama-3-8b-preview:free",
    
    // Specialized models (various sizes)
    "deepseek/deepseek-r1:free",
    "microsoft/phi-4-reasoning-plus:free",
    "microsoft/phi-4-reasoning:free",
    "deepseek/deepseek-v3-base:free",
    "deepseek/deepseek-r1-zero:free",
    "deepseek/deepseek-prover-v2:free",
    "deepseek/deepseek-chat-v3-0324:free",
    "deepseek/deepseek-chat:free",
    "microsoft/mai-ds-r1:free",
    "tngtech/deepseek-r1t-chimera:free",
    "mistralai/mistral-nemo:free",
    
    // Small models (< 7B)
    "qwen/qwen3-4b:free",
    "google/gemma-3-4b-it:free",
    "qwen/qwen-2.5-7b-instruct:free",
    "mistralai/mistral-7b-instruct:free",
    "qwen/qwen-2.5-vl-7b-instruct:free",
    "opengvlab/internvl3-2b:free",
    "google/gemma-3-1b-it:free",
    "meta-llama/llama-3.2-3b-instruct:free",
    "allenai/molmo-7b-d:free",
    "qwen/qwen3-1.7b:free",
    "qwen/qwen2.5-vl-3b-instruct:free",
    "meta-llama/llama-3.2-1b-instruct:free",
    "qwen/qwen3-0.6b-04-28:free",
    
    // Special cases and multimodal models
    "google/learnlm-1.5-pro-experimental:free",
    "moonshotai/kimi-vl-a3b-thinking:free"
];

const MAX_CONSECUTIVE_FAILURES: usize = 3;
const INITIAL_JAIL_HOURS: i64 = 24;
const JAIL_TIME_MULTIPLIER: i64 = 2;
const MAX_JAIL_HOURS: i64 = 168; // 7 days
const BLACKLIST_AFTER_JAIL_COUNT: usize = 3;
const BLACKLIST_RETRY_DAYS: i64 = 7;

// From: 029_function_main.rs
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    // Initialize logging system
    let mut logging_config = LoggingConfig::new();

    // Adjust logging level based on CLI flags
    if cli.verbose {
        logging_config.with_debug();
    }

    let _logging_guard = match init_logging(&logging_config) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            // Continue execution even if logging fails
            None
        }
    };

    info!("Starting aicommit version {}", get_version());

    // Check .gitignore at startup
    if !cli.no_gitignore_check {
        Config::check_gitignore()?;
    }

    match () {
        _ if cli.help => {
            // Получаем версию для отображения в справке
            let version = get_version();
            
            println!("aicommit v{} - A CLI tool that generates concise and descriptive git commit messages using LLMs", version);
            println!("\nUsage:");
            println!("  aicommit [OPTIONS]");
            println!("\nOptions:");
            println!("  --add-provider       Add a new provider (interactive mode)");
            println!("  --add                Automatically stage all changes before commit");
            println!("  --add-openrouter     Add OpenRouter provider non-interactively");
            println!("  --add-simple-free    Add Simple Free OpenRouter provider (uses best available free models)");
            println!("  --add-ollama         Add Ollama provider non-interactively");
            println!("  --add-openai-compatible Add OpenAI compatible provider non-interactively");
            println!("  --openrouter-api-key=<KEY> OpenRouter API key (required for --add-openrouter)");
            println!("  --openrouter-model=<MODEL> OpenRouter model (default: mistralai/mistral-tiny)");
            println!("  --ollama-url=<URL>    Ollama API URL (default: http://localhost:11434)");
            println!("  --ollama-model=<MODEL> Ollama model (default: llama2)");
            println!("  --openai-compatible-api-key=<KEY> OpenAI compatible API key");
            println!("  --openai-compatible-api-url=<URL> OpenAI compatible API URL");
            println!("  --openai-compatible-model=<MODEL> OpenAI compatible model (default: gpt-3.5-turbo)");
            println!("  --max-tokens=<TOKENS> Max tokens for response (default: 200)");
            println!("  --temperature=<TEMP>  Temperature for generation (default: 0.2)");
            println!("  --list                List all providers");
            println!("  --set=<ID>            Set active provider by ID");
            println!("  --config              Edit configuration file");
            println!("  --version-file=<FILE> Path to version file");
            println!("  --version-iterate     Automatically increment version in version file");
            println!("  --version-cargo       Synchronize version with Cargo.toml");
            println!("  --version-npm         Synchronize version with package.json");
            println!("  --version-github      Update version on GitHub");
            println!("  --dry-run             Interactive commit message generation");
            println!("  --pull                Pull changes before commit");
            println!("  --push                Automatically push changes after commit");
            println!("  --help                Display this help message");
            println!("  --version             Display version information");
            println!("  --verbose             Display verbose information");
            println!("  --no-gitignore-check  Skip .gitignore check and creation");
            println!("  --no-aicommitignore   Skip .aicommitignore filtering (include all files in diff)");
            println!("  --watch               Watch for changes and auto-commit");
            println!("  --wait-for-edit=<DURATION> Wait for edit delay before committing (e.g. \"30s\")");
            println!("  --jail-status         Show status of all model jails and blacklists");
            println!("  --unjail=<MODEL>      Release specific model from jail/blacklist (model ID as parameter)");
            println!("  --unjail-all          Release all models from jail/blacklist");
            println!("\nGitHub Action Options:");
            println!("  --github-action       Run in GitHub Action mode (non-interactive)");
            println!("  --input-diff=<FILE>   Input diff from file (for --github-action mode)");
            println!("  --stdin               Read diff from stdin (for --github-action mode)");
            println!("  --output-format=<FMT> Output format: text, json, github (default: text)");
            println!("  --api-key=<KEY>       API key for GitHub Action mode");
            println!("  --provider=<TYPE>     Provider: openrouter, simple-free, ollama, openai-compatible");
            println!("  --model=<MODEL>       Model name for GitHub Action mode");
            println!("  --analyze-commits     Analyze commits from GitHub event context");
            println!("\nGit Hook Options:");
            println!("  --hook=install        Install prepare-commit-msg hook for automatic AI messages");
            println!("  --hook=uninstall      Remove the installed hook");
            println!("  --hook=status         Check if hook is installed");
            println!("\nExamples:");
            println!("  aicommit --add-provider");
            println!("  aicommit --add");
            println!("  aicommit --add-openrouter --openrouter-api-key=<KEY>");
            println!("  aicommit --add-simple-free --openrouter-api-key=<KEY>");
            println!("  aicommit --list");
            println!("  aicommit --set=<ID>");
            println!("  aicommit --version-file=version.txt --version-iterate");
            println!("  aicommit --watch");
            println!("  aicommit --github-action --stdin --api-key=$OPENROUTER_API_KEY");
            println!("  aicommit --github-action --analyze-commits --provider=simple-free");
            println!("  aicommit --hook=install");
            println!("  aicommit --hook=uninstall");
            println!("  aicommit");
            Ok(())
        }
        _ if cli.version => {
            println!("aicommit v{}", get_version());
            Ok(())
        }
        _ if cli.jail_status => {
            // Display jail status for all models
            let config = Config::load()?;
            
            // Find a Simple Free provider
            let mut found = false;
            
            for provider in config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    display_model_jail_status(&c)?;
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.unjail.is_some() => {
            // Unjail a specific model
            let model_id = cli.unjail.unwrap();
            let mut config = Config::load()?;
            let mut found = false;
            
            for provider in &mut config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    unjail_model(c, &model_id)?;
                    println!("Model '{}' successfully released from jail", model_id);
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.unjail_all => {
            // Unjail all models
            let mut config = Config::load()?;
            let mut found = false;
            
            for provider in &mut config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    unjail_all_models(c)?;
                    println!("All models successfully released from jail");
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.github_action => {
            // GitHub Action mode - non-interactive
            run_github_action_mode(&cli).await
        }
        _ if cli.hook.is_some() => {
            // Hook management
            let hook_cmd = cli.hook.as_ref().unwrap();
            match hook_cmd.as_str() {
                "install" => hooks::install_hook(),
                "uninstall" => hooks::uninstall_hook(),
                "status" => hooks::hook_status(),
                _ => Err(format!(
                    "Unknown hook command: '{}'. Valid commands are: install, uninstall, status",
                    hook_cmd
                )),
            }
        }
        _ if cli.add_provider => {
            Config::setup_interactive().await?;
            println!("Provider added successfully!");
            Ok(())
        }
        _ if cli.add_openrouter || cli.add_ollama || cli.add_openai_compatible || cli.add_simple_free => {
            Config::setup_non_interactive(&cli).await?;
            println!("Provider added successfully!");
            Ok(())
        }
        _ if cli.list => {
            // Список всех провайдеров
            let config = Config::load()?;
            for provider in config.providers {
                match provider {
                    ProviderConfig::OpenRouter(c) => println!("OpenRouter: {}", c.id),
                    ProviderConfig::Ollama(c) => println!("Ollama: {}", c.id),
                    ProviderConfig::OpenAICompatible(c) => println!("OpenAI Compatible: {}", c.id),
                    ProviderConfig::SimpleFreeOpenRouter(c) => println!("Simple Free OpenRouter: {}", c.id),
                    ProviderConfig::ClaudeCode(c) => println!("Claude Code: {}", c.id),
                    ProviderConfig::OpenCode(c) => println!("OpenCode: {}", c.id),
                }
            }
            Ok(())
        }
        _ if cli.set.is_some() => {
            // Установка активного провайдера
            let mut config = Config::load()?;
            let new_active_provider = cli.set.unwrap();
            let mut found = false;

            for provider in &config.providers {
                match provider {
                    ProviderConfig::OpenRouter(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::Ollama(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::OpenAICompatible(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::SimpleFreeOpenRouter(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::ClaudeCode(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::OpenCode(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                return Err(format!("Provider '{}' not found", new_active_provider));
            }

            let config_path = dirs::home_dir()
                .ok_or_else(|| "Could not find home directory".to_string())?
                .join(".aicommit.json");

            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;

            fs::write(&config_path, content)
                .map_err(|e| format!("Failed to write config file: {}", e))?;

            println!("Active provider set to {}", new_active_provider);
            Ok(())
        }
        _ if cli.config => {
            Config::edit()?;
            println!("Configuration updated.");
            Ok(())
        }
        _ if cli.dry_run => {
            // Special handling for --dry-run to provide better error messages
            match dry_run(&cli).await {
                Ok(message) => {
                    println!("{}", message);
                    Ok(())
                }
                Err(e) => {
                    // Provide more detailed error message
                    eprintln!("Error in dry-run mode: {}", e);
                    Err(e)
                }
            }
        }
        _ => {
            let config = Config::load().unwrap_or_else(|_| {
                println!("No configuration found. Run 'aicommit --add-provider' to set up a provider.");
                std::process::exit(1);
            });

            if config.active_provider.is_empty() || config.providers.is_empty() {
                println!("No active provider found. Please run 'aicommit --add-provider' to configure a provider.");
                std::process::exit(1);
            }

            // Check if we're in watch mode
            if cli.watch {
                watch_and_commit(&config, &cli).await?
            } else {
                run_commit(&config, &cli).await?;
            }
            Ok(())
        }
    }
}

// From: 033_function_dry_run.rs
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

// GitHub Action output structures
#[derive(Debug, serde::Serialize)]
struct GitHubActionOutput {
    pub commit_message: String,
    pub model_used: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub total_cost: Option<f32>,
    pub success: bool,
    pub error: Option<String>,
}

// From: github_action.rs
async fn run_github_action_mode(cli: &Cli) -> Result<(), String> {
    // Get the diff input
    let diff = get_github_action_diff(cli)?;

    if diff.trim().is_empty() {
        return output_github_action_error(cli, "No diff provided. Use --stdin, --input-diff, or --analyze-commits");
    }

    // Get or create provider configuration
    let (provider_config, mut simple_free_config) = create_github_action_provider(cli)?;

    // Generate the commit message
    let result = match &provider_config {
        ProviderConfig::OpenRouter(c) => generate_openrouter_commit_message(c, &diff, cli).await,
        ProviderConfig::Ollama(c) => generate_ollama_commit_message(c, &diff, cli).await,
        ProviderConfig::OpenAICompatible(c) => generate_openai_compatible_commit_message(c, &diff, cli).await,
        ProviderConfig::SimpleFreeOpenRouter(_) => {
            if let Some(ref mut c) = simple_free_config {
                generate_simple_free_commit_message(c, &diff, cli).await
            } else {
                Err("Simple free config not available".to_string())
            }
        },
        ProviderConfig::ClaudeCode(c) => generate_claude_code_commit_message(c, &diff, cli).await,
        ProviderConfig::OpenCode(c) => generate_opencode_commit_message(c, &diff, cli).await,
    };

    match result {
        Ok((message, usage_info)) => {
            output_github_action_result(cli, &message, Some(usage_info))
        }
        Err(e) => {
            output_github_action_error(cli, &e)
        }
    }
}

fn get_github_action_diff(cli: &Cli) -> Result<String, String> {
    // Priority: --input-diff > --stdin > --analyze-commits > env var > git diff

    if let Some(ref file_path) = cli.input_diff {
        return fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read diff file '{}': {}", file_path, e));
    }

    if cli.stdin {
        use std::io::{self, Read};
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read from stdin: {}", e))?;
        return Ok(buffer);
    }

    if cli.analyze_commits {
        return get_commits_from_github_context();
    }

    // Try environment variable
    if let Ok(diff) = std::env::var("AICOMMIT_DIFF") {
        return Ok(diff);
    }

    // Fall back to git diff
    get_git_diff(cli)
}

fn get_commits_from_github_context() -> Result<String, String> {
    // Read GitHub event context
    let event_path = std::env::var("GITHUB_EVENT_PATH")
        .map_err(|_| "GITHUB_EVENT_PATH not set. This mode requires GitHub Actions environment.")?;

    let event_content = fs::read_to_string(&event_path)
        .map_err(|e| format!("Failed to read GitHub event file: {}", e))?;

    let event: serde_json::Value = serde_json::from_str(&event_content)
        .map_err(|e| format!("Failed to parse GitHub event JSON: {}", e))?;

    // Get commits from push event
    let commits = event.get("commits")
        .and_then(|c| c.as_array())
        .ok_or_else(|| "No commits found in GitHub event. This action only works on push events.")?;

    let mut combined_diff = String::new();

    // Get the before and after SHAs for the diff range
    let before_sha = event.get("before")
        .and_then(|v| v.as_str())
        .unwrap_or("HEAD~1");
    let after_sha = event.get("after")
        .and_then(|v| v.as_str())
        .unwrap_or("HEAD");

    // Get the combined diff for all commits in the push
    let diff_output = std::process::Command::new("git")
        .args(["diff", before_sha, after_sha])
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if !diff_output.status.success() {
        // Fall back to individual commit diffs
        for commit in commits {
            if let Some(sha) = commit.get("id").and_then(|v| v.as_str()) {
                let output = std::process::Command::new("git")
                    .args(["show", "--format=", sha])
                    .output()
                    .map_err(|e| format!("Failed to get diff for commit {}: {}", sha, e))?;

                if output.status.success() {
                    let diff = String::from_utf8_lossy(&output.stdout);
                    combined_diff.push_str(&format!("# Commit: {}\n{}\n\n", sha, diff));
                }
            }
        }
    } else {
        combined_diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    }

    // Also include commit messages for context
    let mut context = String::from("# Original commit messages:\n");
    for commit in commits {
        if let Some(message) = commit.get("message").and_then(|v| v.as_str()) {
            context.push_str(&format!("- {}\n", message));
        }
    }
    context.push_str("\n# Diff:\n");
    context.push_str(&combined_diff);

    Ok(context)
}

fn create_github_action_provider(cli: &Cli) -> Result<(ProviderConfig, Option<SimpleFreeOpenRouterConfig>), String> {
    // Try to get API key from CLI or environment
    let api_key = cli.api_key.clone()
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .or_else(|| std::env::var("AICOMMIT_API_KEY").ok());

    // Determine provider type
    let provider_type = cli.provider.clone()
        .or_else(|| std::env::var("AICOMMIT_PROVIDER").ok())
        .unwrap_or_else(|| "simple-free".to_string());

    // Get model from CLI or environment
    let model = cli.model.clone()
        .or_else(|| std::env::var("AICOMMIT_MODEL").ok());

    match provider_type.as_str() {
        "openrouter" => {
            let api_key = api_key.ok_or_else(||
                "API key required for OpenRouter. Set --api-key or OPENROUTER_API_KEY environment variable.".to_string())?;

            Ok((ProviderConfig::OpenRouter(OpenRouterConfig {
                id: "github-action".to_string(),
                provider: "openrouter".to_string(),
                api_key,
                model: model.unwrap_or_else(|| "mistralai/mistral-tiny".to_string()),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            }), None))
        }
        "simple-free" => {
            let api_key = api_key.ok_or_else(||
                "API key required for Simple Free mode. Set --api-key or OPENROUTER_API_KEY environment variable.".to_string())?;

            let config = SimpleFreeOpenRouterConfig {
                id: "github-action".to_string(),
                provider: "simple_free_openrouter".to_string(),
                api_key,
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
                failed_models: Vec::new(),
                model_stats: std::collections::HashMap::new(),
                last_used_model: model,
                last_config_update: chrono::Utc::now(),
            };
            Ok((ProviderConfig::SimpleFreeOpenRouter(config.clone()), Some(config)))
        }
        "ollama" => {
            let url = std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| cli.ollama_url.clone());

            Ok((ProviderConfig::Ollama(OllamaConfig {
                id: "github-action".to_string(),
                provider: "ollama".to_string(),
                model: model.unwrap_or_else(|| cli.ollama_model.clone()),
                url,
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            }), None))
        }
        "openai-compatible" => {
            let api_key = api_key.ok_or_else(||
                "API key required for OpenAI Compatible. Set --api-key environment variable.".to_string())?;
            let api_url = std::env::var("OPENAI_COMPATIBLE_URL")
                .or_else(|_| cli.openai_compatible_api_url.clone().ok_or(()))
                .map_err(|_| "API URL required for OpenAI Compatible. Set OPENAI_COMPATIBLE_URL environment variable.".to_string())?;

            Ok((ProviderConfig::OpenAICompatible(OpenAICompatibleConfig {
                id: "github-action".to_string(),
                provider: "openai_compatible".to_string(),
                api_key,
                api_url,
                model: model.unwrap_or_else(|| cli.openai_compatible_model.clone()),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            }), None))
        }
        _ => Err(format!("Unknown provider type: {}. Valid options: openrouter, simple-free, ollama, openai-compatible", provider_type))
    }
}

fn output_github_action_result(cli: &Cli, message: &str, usage_info: Option<UsageInfo>) -> Result<(), String> {
    match cli.output_format.as_str() {
        "json" => {
            let output = GitHubActionOutput {
                commit_message: message.to_string(),
                model_used: usage_info.as_ref().and_then(|u| u.model_used.clone()),
                input_tokens: usage_info.as_ref().map(|u| u.input_tokens),
                output_tokens: usage_info.as_ref().map(|u| u.output_tokens),
                total_cost: usage_info.as_ref().map(|u| u.total_cost),
                success: true,
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)
                .map_err(|e| format!("Failed to serialize output: {}", e))?);
        }
        "github" => {
            // GitHub Actions output format
            // Set output variables for use in subsequent steps
            if let Ok(github_output) = std::env::var("GITHUB_OUTPUT") {
                let mut output_content = String::new();
                output_content.push_str(&format!("commit_message={}\n", message.replace('\n', "\\n")));
                if let Some(ref info) = usage_info {
                    if let Some(ref model) = info.model_used {
                        output_content.push_str(&format!("model_used={}\n", model));
                    }
                    output_content.push_str(&format!("input_tokens={}\n", info.input_tokens));
                    output_content.push_str(&format!("output_tokens={}\n", info.output_tokens));
                    output_content.push_str(&format!("total_cost={}\n", info.total_cost));
                }
                output_content.push_str("success=true\n");

                fs::write(&github_output, output_content)
                    .map_err(|e| format!("Failed to write to GITHUB_OUTPUT: {}", e))?;
            }
            // Also print to stdout for visibility
            println!("Generated commit message: {}", message);
            if let Some(ref info) = usage_info {
                if let Some(ref model) = info.model_used {
                    println!("Model used: {}", model);
                }
                println!("Tokens: {} input, {} output", info.input_tokens, info.output_tokens);
            }
        }
        _ => {
            // Default text format - just output the message
            println!("{}", message);
        }
    }
    Ok(())
}

fn output_github_action_error(cli: &Cli, error: &str) -> Result<(), String> {
    match cli.output_format.as_str() {
        "json" => {
            let output = GitHubActionOutput {
                commit_message: String::new(),
                model_used: None,
                input_tokens: None,
                output_tokens: None,
                total_cost: None,
                success: false,
                error: Some(error.to_string()),
            };
            println!("{}", serde_json::to_string_pretty(&output)
                .map_err(|e| format!("Failed to serialize error output: {}", e))?);
            Err(error.to_string())
        }
        "github" => {
            // GitHub Actions error format
            if let Ok(github_output) = std::env::var("GITHUB_OUTPUT") {
                let output_content = format!("success=false\nerror={}\n", error.replace('\n', "\\n"));
                let _ = fs::write(&github_output, output_content);
            }
            eprintln!("::error::{}", error);
            Err(error.to_string())
        }
        _ => {
            eprintln!("Error: {}", error);
            Err(error.to_string())
        }
    }
}

