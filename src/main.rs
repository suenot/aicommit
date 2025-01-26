use std::fs;
use serde::{Serialize, Deserialize};
use dialoguer::{Input, Select};
use uuid::Uuid;
use serde_json::json;
use std::env;
use clap::Parser;
use std::process::Command;
use regex;

#[derive(Parser, Debug)]
#[command(name = "aicommit")]
#[command(about = "A CLI tool that generates concise and descriptive git commit messages using LLMs", long_about = None)]
struct Cli {
    /// Add a new provider (interactive mode)
    #[arg(long = "add-provider")]
    add_provider: bool,

    /// Automatically stage all changes before commit
    #[arg(long = "add")]
    add: bool,

    /// Add OpenRouter provider non-interactively
    #[arg(long)]
    add_openrouter: bool,

    /// OpenRouter API key
    #[arg(long)]
    openrouter_api_key: Option<String>,

    /// OpenRouter model name
    #[arg(long, default_value = "mistralai/mistral-tiny")]
    openrouter_model: String,

    /// Add Ollama provider non-interactively
    #[arg(long)]
    add_ollama: bool,

    /// Ollama API URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Ollama model name
    #[arg(long, default_value = "llama2")]
    ollama_model: String,

    /// Add OpenAI compatible provider non-interactively
    #[arg(long)]
    add_openai_compatible: bool,

    /// OpenAI compatible API key
    #[arg(long)]
    openai_compatible_api_key: Option<String>,

    /// OpenAI compatible API URL
    #[arg(long)]
    openai_compatible_api_url: Option<String>,

    /// OpenAI compatible model name
    #[arg(long, default_value = "gpt-3.5-turbo")]
    openai_compatible_model: String,

    /// Max tokens for provider configuration
    #[arg(long, default_value = "50")]
    max_tokens: i32,

    /// Temperature for provider configuration
    #[arg(long, default_value = "0.3")]
    temperature: f32,

    /// List all providers
    #[arg(long)]
    list: bool,

    /// Set active provider
    #[arg(long)]
    set: Option<String>,

    /// Edit configuration file
    #[arg(long)]
    config: bool,

    /// Path to version file
    #[arg(long = "version-file")]
    version_file: Option<String>,

    /// Automatically increment version in version file
    #[arg(long = "version-iterate")]
    version_iterate: bool,

    /// Synchronize version with Cargo.toml
    #[arg(long = "version-cargo")]
    version_cargo: bool,

    /// Update version on GitHub
    #[arg(long = "version-github")]
    version_github: bool,

    /// Interactive commit message generation
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Pull changes before commit
    #[arg(long = "pull")]
    pull: bool,

    /// Watch for changes and auto-commit (e.g. "1m" for 1 minute interval)
    #[arg(long = "watch")]
    watch_interval: Option<String>,

    /// Wait for edit delay before committing (e.g. "30s" for 30 seconds)
    #[arg(long = "wait-for-edit")]
    wait_for_edit: Option<String>,

    /// Automatically push changes after commit
    #[arg(long = "push")]
    push: bool,

    /// Display help information
    #[arg(long = "help")]
    help: bool,
}

/// Increment version string (e.g., "0.0.37" -> "0.0.38")
fn increment_version(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = version.trim().split('.').collect();
    if parts.len() < 1 {
        return Err("Invalid version format".into());
    }

    let mut new_parts: Vec<String> = parts.iter().map(|&s| s.to_string()).collect();
    let last_idx = new_parts.len() - 1;
    
    if let Ok(mut num) = new_parts[last_idx].parse::<u32>() {
        num += 1;
        new_parts[last_idx] = num.to_string();
        Ok(new_parts.join("."))
    } else {
        Err("Invalid version number".into())
    }
}

/// Update version in file
async fn update_version_file(file_path: &str) -> Result<(), String> {
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read version file: {}", e))?;
    
    let new_version = increment_version(&content)
        .map_err(|e| format!("Failed to increment version: {}", e))?;
    
    tokio::fs::write(file_path, new_version)
        .await
        .map_err(|e| format!("Failed to write version file: {}", e))?;
    
    Ok(())
}

/// Update version in Cargo.toml
async fn update_cargo_version(version: &str) -> Result<(), String> {
    let cargo_path = "Cargo.toml";
    let content = tokio::fs::read_to_string(cargo_path)
        .await
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

    let re = regex::Regex::new(r#"(?m)^version = "(.*?)"$"#)
        .map_err(|e| format!("Failed to create regex: {}", e))?;

    let new_content = re.replace(&content, format!(r#"version = "{}""#, version).as_str());

    tokio::fs::write(cargo_path, new_content.as_bytes())
        .await
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    Ok(())
}

/// Update version on GitHub
fn update_github_version(version: &str) -> Result<(), String> {
    // Check if tag exists
    let check_tag = Command::new("git")
        .args(["tag", "-l", &format!("v{}", version)])
        .output()
        .map_err(|e| format!("Failed to check tag: {}", e))?;
    
    let tag_exists = String::from_utf8_lossy(&check_tag.stdout)
        .trim()
        .len() > 0;

    if tag_exists {
        return Ok(());
    }

    // Create new tag
    let create_tag = Command::new("git")
        .args(["tag", "-a", &format!("v{}", version), "-m", &format!("Release v{}", version)])
        .output()
        .map_err(|e| format!("Failed to create tag: {}", e))?;

    if !create_tag.status.success() {
        return Err(String::from_utf8_lossy(&create_tag.stderr).to_string());
    }

    // Push new tag
    let push_tag = Command::new("git")
        .args(["push", "origin", &format!("v{}", version)])
        .output()
        .map_err(|e| format!("Failed to push tag: {}", e))?;

    if !push_tag.status.success() {
        return Err(String::from_utf8_lossy(&push_tag.stderr).to_string());
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterConfig {
    id: String,
    provider: String,
    api_key: String,
    model: String,
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaConfig {
    id: String,
    provider: String,
    model: String,
    url: String,
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAICompatibleConfig {
    id: String,
    provider: String,
    api_key: String,
    api_url: String,
    model: String,
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
enum ProviderConfig {
    OpenRouter(OpenRouterConfig),
    Ollama(OllamaConfig),
    OpenAICompatible(OpenAICompatibleConfig),
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    providers: Vec<ProviderConfig>,
    active_provider: String,
}

impl Config {
    fn new() -> Self {
        Config {
            providers: Vec::new(),
            active_provider: String::new(),
        }
    }

    fn load() -> Result<Self, String> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    fn check_gitignore() -> Result<(), String> {
        // Check if .gitignore exists in current directory
        if !std::path::Path::new(".gitignore").exists() {
            // Get default gitignore content
            let default_content = Self::get_default_gitignore()?;
            fs::write(".gitignore", default_content)
                .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
            println!("Created default .gitignore file");
        }
        Ok(())
    }

    fn get_default_gitignore() -> Result<String, String> {
        let default_gitignore_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".default_gitignore");

        // If default gitignore doesn't exist, create it with default content
        if !default_gitignore_path.exists() {
            let default_content = "/target\n.DS_Store\n.env\n*.log\nnode_modules/\n";
            fs::write(&default_gitignore_path, default_content)
                .map_err(|e| format!("Failed to create default gitignore template: {}", e))?;
        }

        fs::read_to_string(&default_gitignore_path)
            .map_err(|e| format!("Failed to read default gitignore template: {}", e))
    }

    async fn setup_interactive() -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());

        println!("Let's set up a provider.");
        let provider_options = &["OpenRouter", "Ollama", "OpenAI Compatible"];
        let provider_selection = Select::new()
            .with_prompt("Select a provider")
            .items(provider_options)
            .interact()
            .map_err(|e| format!("Failed to get provider selection: {}", e))?;

        let provider_id = Uuid::new_v4().to_string();

        match provider_selection {
            0 => {
                let mut openrouter_config = setup_openrouter_provider().await?;
                openrouter_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
                config.active_provider = provider_id;
            }
            1 => {
                let url: String = Input::new()
                    .with_prompt("Enter Ollama API URL")
                    .default("http://localhost:11434".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get URL: {}", e))?;

                let model: String = Input::new()
                    .with_prompt("Enter model name")
                    .default("llama2".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get model name: {}", e))?;

                let max_tokens: String = Input::new()
                    .with_prompt("Enter max tokens")
                    .default("50".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get max tokens: {}", e))?;
                let max_tokens: i32 = max_tokens.parse()
                    .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

                let temperature: String = Input::new()
                    .with_prompt("Enter temperature")
                    .default("0.3".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get temperature: {}", e))?;
                let temperature: f32 = temperature.parse()
                    .map_err(|e| format!("Failed to parse temperature: {}", e))?;

                config.providers.push(ProviderConfig::Ollama(OllamaConfig {
                    id: provider_id.clone(),
                    provider: "ollama".to_string(),
                    model,
                    url,
                    max_tokens,
                    temperature,
                }));
                config.active_provider = provider_id;
            }
            2 => {
                let mut openai_compatible_config = setup_openai_compatible_provider().await?;
                openai_compatible_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenAICompatible(openai_compatible_config));
                config.active_provider = provider_id;
            }
            _ => return Err("Invalid provider selection".to_string()),
        }

        // Save the configuration
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(config)
    }

    fn edit() -> Result<(), String> {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        if !config_path.exists() {
            let default_config = Config::new();
            let content = serde_json::to_string_pretty(&default_config)
                .map_err(|e| format!("Failed to serialize default config: {}", e))?;
            fs::write(&config_path, content)
                .map_err(|e| format!("Failed to write default config: {}", e))?;
        }

        let status = Command::new(editor)
            .arg(&config_path)
            .status()
            .map_err(|e| format!("Failed to open editor: {}", e))?;

        if !status.success() {
            return Err("Editor exited with error".to_string());
        }

        Ok(())
    }

    async fn setup_non_interactive(cli: &Cli) -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());
        let provider_id = Uuid::new_v4().to_string();

        if cli.add_openrouter {
            let api_key = cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?;

            let openrouter_config = OpenRouterConfig {
                id: provider_id.clone(),
                provider: "openrouter".to_string(),
                api_key,
                model: cli.openrouter_model.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
            config.active_provider = provider_id;
        } else if cli.add_ollama {
            let ollama_config = OllamaConfig {
                id: provider_id.clone(),
                provider: "ollama".to_string(),
                model: cli.ollama_model.clone(),
                url: cli.ollama_url.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::Ollama(ollama_config));
            config.active_provider = provider_id;
        } else if cli.add_openai_compatible {
            let api_key = cli.openai_compatible_api_key.clone()
                .ok_or_else(|| "OpenAI compatible API key is required".to_string())?;
            let api_url = cli.openai_compatible_api_url.clone()
                .ok_or_else(|| "OpenAI compatible API URL is required".to_string())?;

            let openai_compatible_config = OpenAICompatibleConfig {
                id: provider_id.clone(),
                provider: "openai_compatible".to_string(),
                api_key,
                api_url,
                model: cli.openai_compatible_model.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::OpenAICompatible(openai_compatible_config));
            config.active_provider = provider_id;
        }

        // Save the configuration
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(config)
    }
}

async fn setup_openrouter_provider() -> Result<OpenRouterConfig, String> {
    let api_key: String = Input::new()
        .with_prompt("Enter OpenRouter API key")
        .interact_text()
        .map_err(|e| format!("Failed to get API key: {}", e))?;

    let model: String = Input::new()
        .with_prompt("Enter model name")
        .default("mistralai/mistral-tiny".into())
        .interact_text()
        .map_err(|e| format!("Failed to get model: {}", e))?;

    let max_tokens: String = Input::new()
        .with_prompt("Enter max tokens")
        .default("50".into())
        .interact_text()
        .map_err(|e| format!("Failed to get max tokens: {}", e))?;
    let max_tokens: i32 = max_tokens.parse()
        .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

    let temperature: String = Input::new()
        .with_prompt("Enter temperature")
        .default("0.3".into())
        .interact_text()
        .map_err(|e| format!("Failed to get temperature: {}", e))?;
    let temperature: f32 = temperature.parse()
        .map_err(|e| format!("Failed to parse temperature: {}", e))?;

    Ok(OpenRouterConfig {
        id: Uuid::new_v4().to_string(),
        provider: "openrouter".to_string(),
        api_key,
        model,
        max_tokens,
        temperature,
    })
}

async fn generate_openrouter_commit_message(config: &OpenRouterConfig, diff: &str) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    let prompt = format!("Here is the git diff, please generate a concise and descriptive commit message:\n\n{}", diff);

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
    
    let message = response_data.choices
        .get(0)
        .ok_or("No choices in response")?
        .message
        .content
        .clone();

    // Используем информацию о токенах из ответа API
    let usage = UsageInfo {
        input_tokens: response_data.usage.prompt_tokens,
        output_tokens: response_data.usage.completion_tokens,
        // Примерная стоимость: $0.14/100K токенов для mistral-tiny
        total_cost: (response_data.usage.total_tokens as f32) * 0.0000014,
    };

    Ok((message, usage))
}

async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": config.model,
        "prompt": format!("Write a clear and concise git commit message (one line, no technical terms) that describes these changes:\n\n{}", diff),
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
    };

    Ok((commit_message, usage))
}

async fn setup_openai_compatible_provider() -> Result<OpenAICompatibleConfig, String> {
    let api_key: String = Input::new()
        .with_prompt("Enter API key")
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
        "claude-3-opus",
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
        .default("50".into())
        .interact_text()
        .map_err(|e| format!("Failed to get max tokens: {}", e))?;
    let max_tokens: i32 = max_tokens.parse()
        .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

    let temperature: String = Input::new()
        .with_prompt("Enter temperature")
        .default("0.3".into())
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

async fn generate_openai_compatible_commit_message(config: &OpenAICompatibleConfig, diff: &str) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    let prompt = format!("Here is the git diff, please generate a concise and descriptive commit message:\n\n{}", diff);

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
    
    let message = response_data.choices
        .get(0)
        .ok_or("No choices in response")?
        .message
        .content
        .clone();

    let usage = UsageInfo {
        input_tokens: response_data.usage.prompt_tokens,
        output_tokens: response_data.usage.completion_tokens,
        total_cost: 0.0, // Set to 0 for OpenAI compatible APIs as we don't know the actual cost
    };

    Ok((message, usage))
}

#[derive(Debug)]
struct UsageInfo {
    input_tokens: i32,
    output_tokens: i32,
    total_cost: f32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

// Add helper function to parse duration string
fn parse_duration(duration_str: &str) -> Result<std::time::Duration, String> {
    let duration_str = duration_str.trim().to_lowercase();
    if duration_str.is_empty() {
        return Err("Duration string is empty".to_string());
    }

    let mut chars = duration_str.chars().peekable();
    let mut number = String::new();
    
    // Collect digits
    while let Some(c) = chars.peek() {
        if c.is_digit(10) {
            number.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    // Get the unit (rest of the string)
    let unit: String = chars.collect();

    if number.is_empty() {
        return Err("No duration value provided".to_string());
    }

    let value = number.parse::<u64>()
        .map_err(|_| format!("Invalid duration number: {}", number))?;

    match unit.as_str() {
        "s" => Ok(std::time::Duration::from_secs(value)),
        "m" => Ok(std::time::Duration::from_secs(value * 60)),
        "h" => Ok(std::time::Duration::from_secs(value * 3600)),
        _ => Err(format!("Invalid duration unit: '{}'. Use s, m, or h", unit)),
    }
}

async fn watch_and_commit(config: &Config, cli: &Cli) -> Result<(), String> {
    let watch_interval = cli.watch_interval.as_ref()
        .ok_or_else(|| "Watch interval not specified".to_string())
        .and_then(|i| parse_duration(i))?;

    let wait_for_edit = cli.wait_for_edit.as_ref()
        .map(|w| parse_duration(w))
        .transpose()?;

    println!("Watching for changes every {:?}", watch_interval);
    if let Some(delay) = wait_for_edit {
        println!("Waiting {:?} after edits before committing", delay);
    }

    let mut last_commit_time = std::time::Instant::now();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Try to get git diff
        match get_git_diff(cli) {
            Ok(diff) if !diff.is_empty() => {
                // If we have wait_for_edit, check if enough time has passed since last edit
                if let Some(delay) = wait_for_edit {
                    // Get the last modified time of any tracked file
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg("git ls-files -m -o --exclude-standard | xargs -I {} stat -f '%m {}' | sort -nr | head -1")
                        .output()
                        .map_err(|e| format!("Failed to get last modified time: {}", e))?;

                    if !output.status.success() {
                        continue;
                    }

                    let last_modified_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(timestamp_str) = last_modified_str.split_whitespace().next() {
                        if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            
                            if now - timestamp < delay.as_secs() {
                                continue; // Not enough time has passed since last edit
                            }
                        }
                    }
                }

                // Check if enough time has passed since last commit
                if last_commit_time.elapsed() >= watch_interval {
                    match run_commit(config, cli).await {
                        Ok(_) => {
                            last_commit_time = std::time::Instant::now();
                            println!("\nWaiting for new changes...");
                        }
                        Err(e) => println!("Failed to commit: {}", e),
                    }
                }
            }
            _ => {} // No changes or error, continue watching
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    // Check .gitignore at startup
    Config::check_gitignore()?;

    match () {
        _ if cli.help => {
            println!("aicommit - A CLI tool that generates concise and descriptive git commit messages using LLMs");
            println!("\nUsage:");
            println!("  aicommit [OPTIONS]");
            println!("\nOptions:");
            println!("  --add-provider       Add a new provider (interactive mode)");
            println!("  --add                Automatically stage all changes before commit");
            println!("  --add-openrouter     Add OpenRouter provider non-interactively");
            println!("  --openrouter-api-key OpenRouter API key");
            println!("  --openrouter-model   OpenRouter model name");
            println!("  --add-ollama        Add Ollama provider non-interactively");
            println!("  --ollama-url        Ollama API URL");
            println!("  --ollama-model      Ollama model name");
            println!("  --add-openai-compatible Add OpenAI compatible provider non-interactively");
            println!("  --openai-compatible-api-key OpenAI compatible API key");
            println!("  --openai-compatible-api-url OpenAI compatible API URL");
            println!("  --openai-compatible-model OpenAI compatible model name");
            println!("  --max-tokens         Max tokens for provider configuration");
            println!("  --temperature        Temperature for provider configuration");
            println!("  --list              List all providers");
            println!("  --set               Set active provider");
            println!("  --config            Edit configuration file");
            println!("  --version-file      Path to version file");
            println!("  --version-iterate   Automatically increment version in version file");
            println!("  --version-cargo     Synchronize version with Cargo.toml");
            println!("  --version-github    Update version on GitHub");
            println!("  --dry-run           Interactive commit message generation");
            println!("  --pull              Pull changes before commit");
            println!("  --watch             Watch for changes and auto-commit");
            println!("  --wait-for-edit     Wait for edit delay before committing");
            println!("  --push              Automatically push changes after commit");
            Ok(())
        }
        _ if cli.add_provider => {
            if cli.add_openrouter || cli.add_ollama || cli.add_openai_compatible {
                // Non-interactive provider configuration
                let _config = Config::setup_non_interactive(&cli).await?;
                println!("Provider successfully configured and set as default.");
            } else {
                // Interactive provider configuration
                let _config = Config::setup_interactive().await?;
                println!("Provider successfully configured and set as default.");
            }
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
            if cli.watch_interval.is_some() {
                watch_and_commit(&config, &cli).await?
            } else {
                // Сначала делаем коммит с текущей конфигурацией
                run_commit(&config, &cli).await?;

                // Теперь обновляем версии если указаны соответствующие параметры
                let mut new_version = String::new();

                // Обновляем версию в файле версии
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

                // Обновляем версию в Cargo.toml
                if cli.version_cargo {
                    if new_version.is_empty() {
                        return Err("Error: --version-file must be specified when using --version-cargo".to_string());
                    }
                    update_cargo_version(&new_version).await?;
                }

                // Обновляем версию на GitHub
                if cli.version_github {
                    if new_version.is_empty() {
                        return Err("Error: --version-file must be specified when using --version-github".to_string());
                    }
                    update_github_version(&new_version)?;
                }
            }
            Ok(())
        }
    }
}

async fn run_commit(config: &Config, cli: &Cli) -> Result<(), String> {
    // Stage changes if --add flag is set
    if cli.add {
        let add_output = Command::new("sh")
            .arg("-c")
            .arg("git add .")
            .output()
            .map_err(|e| format!("Failed to execute git add: {}", e))?;

        if !add_output.status.success() {
            return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
        }
    }

    // Get active provider
    let active_provider = config.providers.iter().find(|p| match p {
        ProviderConfig::OpenRouter(c) => c.id == config.active_provider,
        ProviderConfig::Ollama(c) => c.id == config.active_provider,
        ProviderConfig::OpenAICompatible(c) => c.id == config.active_provider,
    }).ok_or("No active provider found")?;

    // Get git diff
    let diff = get_git_diff(cli)?;
    if diff.is_empty() {
        return Err("No changes to commit".to_string());
    }

    let mut commit_applied = false;

    // Generate and apply commit
    if cli.dry_run {
        // ... dry run logic ...
    } else {
        let (message, usage) = match active_provider {
            ProviderConfig::OpenRouter(c) => generate_openrouter_commit_message(c, &diff).await?,
            ProviderConfig::Ollama(c) => generate_ollama_commit_message(c, &diff).await?,
            ProviderConfig::OpenAICompatible(c) => generate_openai_compatible_commit_message(c, &diff).await?,
        };

        println!("Generated commit message: \"{}\"\n", message);
        println!("Tokens: {}↑ {}↓", usage.input_tokens, usage.output_tokens);
        println!("API Cost: ${:.4}", usage.total_cost);

        create_git_commit(&message)?;
        println!("Commit successfully created.");
        commit_applied = true;
    }

    // Pull changes if --pull flag is set
    if cli.pull && commit_applied {
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

    // Push changes if --push flag is set and commit was applied
    if cli.push && commit_applied {
        let push_output = Command::new("sh")
            .arg("-c")
            .arg("git push")
            .output()
            .map_err(|e| format!("Failed to execute git push: {}", e))?;

        if !push_output.status.success() {
            return Err(String::from_utf8_lossy(&push_output.stderr).to_string());
        }

        println!("Changes successfully pushed.");
    }

    Ok(())
}

fn get_git_diff(cli: &Cli) -> Result<String, String> {
    // Stage all changes if --add flag is set
    if cli.add {
        let add_output = Command::new("sh")
            .arg("-c")
            .arg("git add .")
            .output()
            .map_err(|e| format!("Failed to execute git add: {}", e))?;

        if !add_output.status.success() {
            return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
        }
    }

    // Get diff of staged changes
    let diff_output = Command::new("sh")
        .arg("-c")
        .arg("git diff --cached")
        .output()
        .map_err(|e| format!("Failed to execute git diff: {}", e))?;

    if !diff_output.status.success() {
        return Err(String::from_utf8_lossy(&diff_output.stderr).to_string());
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    if diff.trim().is_empty() {
        return Err("No changes to commit".to_string());
    }

    Ok(diff)
}

fn create_git_commit(message: &str) -> Result<(), String> {
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
