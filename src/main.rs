use std::fs;
use serde::{Serialize, Deserialize};
use dialoguer::{Input, Select};
use uuid::Uuid;
use serde_json::json;
use std::env;
use clap::Parser;
use std::process::Command;
use tokio;
use chrono;

const MAX_DIFF_CHARS: usize = 15000; // Limit diff size to prevent excessive API usage

#[derive(Parser, Debug)]
#[command(name = "aicommit")]
#[command(about = "A CLI tool that generates concise and descriptive git commit messages using LLMs", long_about = None)]
#[command(disable_help_flag = true)]
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

    /// Add OpenAI compatible provider non-interactively (e.g., LM Studio, custom endpoints)
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

    /// Synchronize version with package.json
    #[arg(long = "version-npm")]
    version_npm: bool,

    /// Update version on GitHub
    #[arg(long = "version-github")]
    version_github: bool,

    /// Interactive commit message generation
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Pull changes before commit
    #[arg(long = "pull")]
    pull: bool,

    /// Watch for changes and auto-commit
    #[arg(long = "watch")]
    watch: bool,

    /// Wait for edit delay before committing (e.g. "30s" for 30 seconds)
    #[arg(long = "wait-for-edit")]
    wait_for_edit: Option<String>,

    /// Automatically push changes after commit
    #[arg(long = "push")]
    push: bool,

    /// Display help information
    #[arg(long = "help")]
    help: bool,

    /// Display version information
    #[arg(long = "version")]
    version: bool,

    /// Display verbose information
    #[arg(long = "verbose")]
    verbose: bool,
    
    /// Skip .gitignore check and creation
    #[arg(long = "no-gitignore-check")]
    no_gitignore_check: bool,
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
    // Update version in Cargo.toml
    let cargo_toml_path = "Cargo.toml";
    let content = tokio::fs::read_to_string(cargo_toml_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let updated_content = content
        .lines()
        .map(|line| {
            if line.starts_with("version = ") {
                format!("version = \"{}\"", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    tokio::fs::write(cargo_toml_path, updated_content)
        .await
        .map_err(|e| format!("Failed to write {}: {}", cargo_toml_path, e))?;

    // Update version in Cargo.lock
    let cargo_lock_path = "Cargo.lock";
    let lock_content = tokio::fs::read_to_string(cargo_lock_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", cargo_lock_path, e))?;

    let mut in_aicommit_package = false;
    let updated_lock_content = lock_content
        .lines()
        .map(|line| {
            if line.starts_with("name = \"aicommit\"") {
                in_aicommit_package = true;
                line.to_string()
            } else if in_aicommit_package && line.starts_with("version = ") {
                in_aicommit_package = false;
                format!("version = \"{}\"", version)
            } else if line.trim().is_empty() {
                in_aicommit_package = false;
                line.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    tokio::fs::write(cargo_lock_path, updated_lock_content)
        .await
        .map_err(|e| format!("Failed to write {}: {}", cargo_lock_path, e))?;

    // Run cargo update to ensure Cargo.lock is in sync
    let update_output = std::process::Command::new("cargo")
        .arg("update")
        .arg("--package")
        .arg("aicommit")
        .output()
        .map_err(|e| format!("Failed to execute cargo update: {}", e))?;

    if !update_output.status.success() {
        return Err(format!(
            "Failed to update Cargo.lock: {}",
            String::from_utf8_lossy(&update_output.stderr)
        ));
    }

    Ok(())
}

/// Update version in package.json
async fn update_npm_version(version: &str) -> Result<(), String> {
    let package_path = "package.json";
    let content = tokio::fs::read_to_string(package_path)
        .await
        .map_err(|e| format!("Failed to read package.json: {}", e))?;

    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse package.json: {}", e))?;

    if let Some(obj) = json.as_object_mut() {
        obj["version"] = json!(version);
    }

    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize package.json: {}", e))?;

    tokio::fs::write(package_path, new_content)
        .await
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

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
    #[serde(default = "default_retry_attempts")]
    retry_attempts: u32,
}

fn default_retry_attempts() -> u32 {
    3
}

impl Config {
    fn new() -> Self {
        Config {
            providers: Vec::new(),
            active_provider: String::new(),
            retry_attempts: default_retry_attempts(),
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
        if !Cli::parse().no_gitignore_check {
            // Check if .gitignore exists in current directory
            if !std::path::Path::new(".gitignore").exists() {
                // Get default gitignore content
                let default_content = Self::get_default_gitignore()?;
                fs::write(".gitignore", default_content)
                    .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
                println!("Created default .gitignore file");
            }
        }
        Ok(())
    }

    fn get_default_gitignore() -> Result<String, String> {
        let default_gitignore_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".default_gitignore");

        // If default gitignore doesn't exist, create it with default content
        if !default_gitignore_path.exists() {
            let default_content = r#"# Универсальный .gitignore для большинства проектов

## Операционные системы
# macOS
*.DS_Store
.AppleDouble
.LSOverride
._*

# Windows
Thumbs.db
Thumbs.db:encryptable
ehthumbs.db
ehthumbs_vista.db
Desktop.ini
$RECYCLE.BIN/

# Linux
.directory
.dropbox
.dropbox.attr

## Общие файлы
*.log
*.log.*
*.sql
*.sqlite
*.jar
*.war
*.ear
*.zip
*.tar.gz
*.rar
*.exe
*.dll
*.so
*.dylib
*.bak
*.swp
*~
*.tmp

## Python
__pycache__/
*.py[cod]
*$py.class
.Python
env/
venv/
ENV/
env.bak/
venv.bak/
.pytest_cache/
.coverage
.coverage.*
coverage.xml
*.cover

## Golang
bin/
pkg/
*.test
*.prof

## Rust
target/
*.rs.bk

## C++
build/
*.o
*.obj
*.out
*.a
*.lib
*.pdb

## Java
target/
pom.xml.tag
pom.xml.releaseBackup
pom.xml.versionsBackup
dependency-reduced-pom.xml
release.properties
tomcat*/
*.class

## C#
bin/
obj/
*.user
*.suo
*.csproj.bak
*.cache
*.ilk
*.meta
*.ncx
*.nupkg

## Elixir
_build/
deps/
*.ez

## R
.Rhistory
.RData
.Rproj.user/
*.Rout

## JavaScript / TypeScript / Фронтенд
node_modules/
dist/
build/
*.min.*
npm-debug.log*
yarn-debug.log*
yarn-error.log*
*.tsbuildinfo

## iOS
DerivedData/
*.pbxuser
!default.pbxuser
*.mode1v3
!default.mode1v3
*.mode2v3
!default.mode2v3
*.perspectivev3
!default.perspectivev3
*.xccheckout
*.moved-aside
*.xcuserstate
*.xcworkspace
Pods/

## Android
.gradle/
build/
*.apk
*.ap_
*.aab
local.properties
*.idea/
*.iml

## IDE и редакторы
.idea/
*.iml
.vscode/
*.swp
*.swo
nbproject/
*.code-workspace

## Прочее
.env
.env.local
.env.*.local
*.cache
*.lock
*.pid
"#;
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
        .default("200".into())
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

async fn generate_openrouter_commit_message(config: &OpenRouterConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    // Truncate diff if it exceeds the character limit
    let truncated_diff = if diff.len() > MAX_DIFF_CHARS {
        format!("{}\n\n[... diff truncated due to length ...]", &diff[..MAX_DIFF_CHARS])
    } else {
        diff.to_string()
    };

    let prompt = format!(
        "Generate a concise and descriptive git commit message following the Conventional Commits specification. The message should start with a type (e.g., feat, fix, chore, docs, style, refactor, test) followed by a colon and a short description in lowercase.
Examples:
- feat: Add user authentication feature
- fix: Correct calculation error in payment module
- docs: Update README with installation instructions
- style: Format code according to style guide
- refactor: Simplify database query logic
- test: Add unit tests for user service
- chore: Update dependencies

Here is the git diff:
```diff
{}
```
Commit message:",
        truncated_diff
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

async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    // Truncate diff if it exceeds the character limit
    let truncated_diff = if diff.len() > MAX_DIFF_CHARS {
        format!("{}\n\n[... diff truncated due to length ...]", &diff[..MAX_DIFF_CHARS])
    } else {
        diff.to_string()
    };

    let prompt = format!(
        "Generate a concise and descriptive git commit message following the Conventional Commits specification (one line, max 72 chars). Start with a type (feat, fix, chore, docs, style, refactor, test), then a colon, then a short description in lowercase.
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
Commit message:",
        truncated_diff
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
    };

    Ok((commit_message, usage))
}

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

async fn generate_openai_compatible_commit_message(config: &OpenAICompatibleConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
    let client = reqwest::Client::new();

    // Truncate diff if it exceeds the character limit
    let truncated_diff = if diff.len() > MAX_DIFF_CHARS {
        format!("{}\n\n[... diff truncated due to length ...]", &diff[..MAX_DIFF_CHARS])
    } else {
        diff.to_string()
    };

    let prompt = format!(
        "Generate a concise and descriptive git commit message following the Conventional Commits specification. The message should start with a type (e.g., feat, fix, chore, docs, style, refactor, test) followed by a colon and a short description in lowercase.
Examples:
- feat: Add user authentication feature
- fix: Correct calculation error in payment module
- docs: Update README with installation instructions
- style: Format code according to style guide
- refactor: Simplify database query logic
- test: Add unit tests for user service
- chore: Update dependencies

Here is the git diff:
```diff
{}
```
Commit message:",
        truncated_diff
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
    let wait_for_edit = cli.wait_for_edit.as_ref()
        .map(|w| parse_duration(w))
        .transpose()?;

    println!("Watching for changes...");
    if let Some(delay) = wait_for_edit {
        println!("Waiting {:?} after edits before committing", delay);
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

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

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
            println!("  --openrouter-api-key OpenRouter API key");
            println!("  --openrouter-model   OpenRouter model name");
            println!("  --add-ollama        Add Ollama provider non-interactively");
            println!("  --ollama-url        Ollama API URL");
            println!("  --ollama-model      Ollama model name");
            println!("  --add-openai-compatible Add OpenAI compatible provider non-interactively (e.g., LM Studio, custom)");
            println!("  --openai-compatible-api-key OpenAI compatible API key");
            println!("  --openai-compatible-api-url OpenAI compatible API URL (e.g., for LM Studio)");
            println!("  --openai-compatible-model OpenAI compatible model name");
            println!("  --max-tokens         Max tokens for provider configuration");
            println!("  --temperature        Temperature for provider configuration");
            println!("  --list              List all providers");
            println!("  --set               Set active provider");
            println!("  --config            Edit configuration file");
            println!("  --version-file      Path to version file");
            println!("  --version-iterate   Automatically increment version in version file");
            println!("  --version-cargo     Synchronize version with Cargo.toml");
            println!("  --version-npm       Synchronize version with package.json");
            println!("  --version-github    Update version on GitHub");
            println!("  --dry-run           Interactive commit message generation");
            println!("  --pull              Pull changes before commit");
            println!("  --watch             Watch for changes and auto-commit");
            println!("  --wait-for-edit     Wait for edit delay before committing");
            println!("  --push              Automatically push changes after commit");
            println!("  --version           Display version information");
            println!("  --help              Display help information");
            println!("  --verbose           Display verbose information");
            println!("  --no-gitignore-check Skip .gitignore check and creation");
            Ok(())
        }
        _ if cli.version => {
            // Получаем версию с помощью вспомогательной функции
            let version = get_version();
            println!("aicommit version {}", version);
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

fn get_git_diff(cli: &Cli) -> Result<String, String> {
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

// Helper function to get version
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Function to handle --dry-run mode with detailed error handling
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

    // In dry-run mode, only return the generated message
    Ok(message)
}

// Implementation of run_commit function
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

    println!("Generated commit message: \"{}\"\n", message);
    println!("Tokens: {}↑ {}↓", usage_info.input_tokens, usage_info.output_tokens);
    println!("API Cost: ${:.4}", usage_info.total_cost);

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