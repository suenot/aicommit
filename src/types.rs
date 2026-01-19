// Types module - structures, enums, and implementations

use serde::{Serialize, Deserialize};
use clap::Parser;
use chrono;
use tracing::info;
use std::fs;
use std::env;
use std::process::Command;
use dialoguer::{Select, Input};
use uuid::Uuid;
use crate::providers::{setup_openrouter_provider, setup_openai_compatible_provider};

// From: 000_struct_Cli.rs
#[derive(Parser, Debug)]
#[command(name = "aicommit")]
#[command(about = "A CLI tool that generates concise and descriptive git commit messages using LLMs", long_about = None)]
#[command(disable_help_flag = true)]
#[command(bin_name = "aicommit")]
pub struct Cli {
    /// Add a new provider (interactive mode)
    #[arg(long = "add-provider")]
    pub add_provider: bool,

    /// Automatically stage all changes before commit
    #[arg(long = "add")]
    pub add: bool,

    /// Add OpenRouter provider non-interactively
    #[arg(long)]
    pub add_openrouter: bool,

    /// OpenRouter API key
    #[arg(long)]
    pub openrouter_api_key: Option<String>,

    /// OpenRouter model name
    #[arg(long, default_value = "mistralai/mistral-tiny")]
    pub openrouter_model: String,

    /// Add Simple Free OpenRouter provider (uses best available free models automatically)
    #[arg(long)]
    pub add_simple_free: bool,

    /// Add Ollama provider non-interactively
    #[arg(long)]
    pub add_ollama: bool,

    /// Ollama API URL
    #[arg(long, default_value = "http://localhost:11434")]
    pub ollama_url: String,

    /// Ollama model name
    #[arg(long, default_value = "llama2")]
    pub ollama_model: String,

    /// Add OpenAI compatible provider non-interactively (e.g., LM Studio, custom endpoints)
    #[arg(long)]
    pub add_openai_compatible: bool,

    /// OpenAI compatible API key
    #[arg(long)]
    pub openai_compatible_api_key: Option<String>,

    /// OpenAI compatible API URL
    #[arg(long)]
    pub openai_compatible_api_url: Option<String>,

    /// OpenAI compatible model name
    #[arg(long, default_value = "gpt-3.5-turbo")]
    pub openai_compatible_model: String,

    /// Max tokens for provider configuration
    #[arg(long, default_value = "200")]
    pub max_tokens: i32,

    /// Temperature for provider configuration
    #[arg(long, default_value = "0.2")]
    pub temperature: f32,

    /// List all providers
    #[arg(long)]
    pub list: bool,

    /// Set active provider
    #[arg(long)]
    pub set: Option<String>,

    /// Edit configuration file
    #[arg(long)]
    pub config: bool,

    /// Path to version file
    #[arg(long = "version-file")]
    pub version_file: Option<String>,

    /// Automatically increment version in version file
    #[arg(long = "version-iterate")]
    pub version_iterate: bool,

    /// Synchronize version with Cargo.toml
    #[arg(long = "version-cargo")]
    pub version_cargo: bool,

    /// Synchronize version with package.json
    #[arg(long = "version-npm")]
    pub version_npm: bool,

    /// Update version on GitHub
    #[arg(long = "version-github")]
    pub version_github: bool,

    /// Interactive commit message generation
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Pull changes before commit
    #[arg(long = "pull")]
    pub pull: bool,

    /// Watch for changes and auto-commit
    #[arg(long = "watch")]
    pub watch: bool,

    /// Wait for edit delay before committing (e.g. "30s" for 30 seconds)
    #[arg(long = "wait-for-edit")]
    pub wait_for_edit: Option<String>,

    /// Automatically push changes after commit
    #[arg(long = "push")]
    pub push: bool,

    /// Display help information
    #[arg(long = "help")]
    pub help: bool,

    /// Display version information
    #[arg(long = "version")]
    pub version: bool,

    /// Display verbose information
    #[arg(long = "verbose")]
    pub verbose: bool,
    
    /// Skip .gitignore check and creation
    #[arg(long = "no-gitignore-check")]
    pub no_gitignore_check: bool,

    /// Set the git commit message without using AI. (For CI/CD or offline use cases)
    #[arg(long)]
    pub msg: Option<String>,

    /// Force the use of offline mode (uses fallback model list) for testing purposes
    #[arg(long, hide = true)]
    pub simulate_offline: bool,
    
    /// Show status of all model jails and blacklists
    #[arg(long = "jail-status")]
    pub jail_status: bool,
    
    /// Release specific model from jail/blacklist (model ID as parameter)
    #[arg(long = "unjail")]
    pub unjail: Option<String>,
    
    /// Release all models from jail/blacklist
    #[arg(long = "unjail-all")]
    pub unjail_all: bool,

    /// Run in GitHub Action mode (non-interactive, reads from stdin or env vars)
    #[arg(long = "github-action")]
    pub github_action: bool,

    /// Input diff from file (for --github-action mode)
    #[arg(long = "input-diff")]
    pub input_diff: Option<String>,

    /// Read diff from stdin (for --github-action mode)
    #[arg(long = "stdin")]
    pub stdin: bool,

    /// Output format for --github-action mode (text, json, github)
    #[arg(long = "output-format", default_value = "text")]
    pub output_format: String,

    /// API key for GitHub Action mode (overrides config)
    #[arg(long = "api-key")]
    pub api_key: Option<String>,

    /// Provider type for GitHub Action mode (openrouter, simple-free, ollama, openai-compatible)
    #[arg(long = "provider")]
    pub provider: Option<String>,

    /// Model name for GitHub Action mode (overrides config)
    #[arg(long = "model")]
    pub model: Option<String>,

    /// Analyze commits from GitHub event and suggest improved messages
    #[arg(long = "analyze-commits")]
    pub analyze_commits: bool,
}

// From: 006_struct_OpenRouterConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: i32,
    pub temperature: f32,
}

// From: 007_struct_ModelStats.rs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelStats {
    pub success_count: usize,
    pub failure_count: usize,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub jail_until: Option<chrono::DateTime<chrono::Utc>>,
    pub jail_count: usize,
    pub blacklisted: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub blacklisted_since: Option<chrono::DateTime<chrono::Utc>>,
}

// From: 009_struct_SimpleFreeOpenRouterConfig.rs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleFreeOpenRouterConfig {
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub max_tokens: i32,
    pub temperature: f32,
    #[serde(default)]
    pub failed_models: Vec<String>,
    #[serde(default)]
    pub model_stats: std::collections::HashMap<String, ModelStats>,
    #[serde(default)]
    pub last_used_model: Option<String>,
    #[serde(default = "chrono::Utc::now")]
    pub last_config_update: chrono::DateTime<chrono::Utc>,
}

// From: 010_struct_OllamaConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub url: String,
    pub max_tokens: i32,
    pub temperature: f32,
}

// From: 011_struct_OpenAICompatibleConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAICompatibleConfig {
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub max_tokens: i32,
    pub temperature: f32,
}

// From: 012_struct_ClaudeCodeConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub id: String,
    pub provider: String,
}

// From: 013_struct_OpenCodeConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    pub id: String,
    pub provider: String,
}

// From: 015_struct_Config.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
    pub active_provider: String,
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
}

// From: 022_struct_UsageInfo.rs
#[derive(Debug)]
pub struct UsageInfo {
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_cost: f32,
    pub model_used: Option<String>,
}

// From: 023_struct_OpenRouterResponse.rs
#[derive(Debug, Deserialize)]
pub struct OpenRouterResponse {
    pub choices: Vec<OpenRouterChoice>,
    pub usage: OpenRouterUsage,
}

// From: 024_struct_OpenRouterChoice.rs
#[derive(Debug, Deserialize)]
pub struct OpenRouterChoice {
    pub message: OpenRouterMessage,
}

// From: 025_struct_OpenRouterMessage.rs
#[derive(Debug, Deserialize)]
pub struct OpenRouterMessage {
    pub content: String,
}

// From: 026_struct_OpenRouterUsage.rs
#[derive(Debug, Deserialize)]
pub struct OpenRouterUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

// From: 014_enum_ProviderConfig.rs
#[derive(Debug, Serialize, Deserialize)]
pub enum ProviderConfig {
    OpenRouter(OpenRouterConfig),
    Ollama(OllamaConfig),
    OpenAICompatible(OpenAICompatibleConfig),
    SimpleFreeOpenRouter(SimpleFreeOpenRouterConfig),
    ClaudeCode(ClaudeCodeConfig),
    OpenCode(OpenCodeConfig),
}

// From: 008_impl_impl_Default.rs
impl Default for ModelStats {
    fn default() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            jail_until: None,
            jail_count: 0,
            blacklisted: false,
            blacklisted_since: None,
        }
    }
}

// From: 016_function_default_retry_attempts.rs
fn default_retry_attempts() -> u32 {
    3
}

// From: 017_impl_impl_Config.rs
impl Config {
    pub fn new() -> Self {
        Config {
            providers: Vec::new(),
            active_provider: String::new(),
            retry_attempts: default_retry_attempts(),
        }
    }

    pub fn load() -> Result<Self, String> {
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

    pub fn check_gitignore() -> Result<(), String> {
        if !Cli::parse().no_gitignore_check {
            // Check if .gitignore exists in current directory
            if !std::path::Path::new(".gitignore").exists() {
                // Get default gitignore content
                let default_content = Self::get_default_gitignore()?;
                fs::write(".gitignore", default_content)
                    .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
                info!("Created default .gitignore file");
            }
        }
        Ok(())
    }

    pub fn get_default_gitignore() -> Result<String, String> {
        let default_gitignore_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".default_gitignore");

        // If default gitignore doesn't exist, create it with default content
        if !default_gitignore_path.exists() {
            let default_content = r#"*.DS_Store
.AppleDouble
.LSOverride
._*

Thumbs.db
Thumbs.db:encryptable
ehthumbs.db
ehthumbs_vista.db
Desktop.ini
$RECYCLE.BIN/

.directory
.dropbox
.dropbox.attr

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

bin/
pkg/
*.test
*.prof

## Rust
target/
*.rs.bk

build/
*.o
*.obj
*.out
*.a
*.lib
*.pdb

target/
pom.xml.tag
pom.xml.releaseBackup
pom.xml.versionsBackup
dependency-reduced-pom.xml
release.properties
tomcat*/
*.class

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

_build/
deps/
*.ez

.Rhistory
.RData
.Rproj.user/
*.Rout

node_modules/
dist/
build/
*.min.*
npm-debug.log*
yarn-debug.log*
yarn-error.log*
*.tsbuildinfo

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

.gradle/
build/
*.apk
*.ap_
*.aab
local.properties
*.idea/
*.iml

.idea/
*.iml
.vscode/
*.swp
*.swo
nbproject/
*.code-workspace

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

    pub async fn setup_interactive() -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());

        info!("Setting up a provider");
        let provider_options = &["Free OpenRouter (recommended)", "OpenRouter", "Ollama", "OpenAI Compatible", "Claude Code", "OpenCode"];
        let provider_selection = Select::new()
            .with_prompt("Select a provider")
            .items(provider_options)
            .default(0)
            .interact()
            .map_err(|e| format!("Failed to get provider selection: {}", e))?;

        let provider_id = Uuid::new_v4().to_string();

        match provider_selection {
            0 => {
                let api_key: String = Input::new()
                    .with_prompt("Enter OpenRouter API key")
                    .interact_text()
                    .map_err(|e| format!("Failed to get API key: {}", e))?;

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

                let simple_free_config = SimpleFreeOpenRouterConfig {
                    id: provider_id.clone(),
                    provider: "simple_free_openrouter".to_string(),
                    api_key,
                    max_tokens,
                    temperature,
                    failed_models: Vec::new(),
                    model_stats: std::collections::HashMap::new(),
                    last_used_model: None,
                    last_config_update: chrono::Utc::now(),
                };

                config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
                config.active_provider = provider_id;
            }
            1 => {
                let mut openrouter_config = setup_openrouter_provider().await?;
                openrouter_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
                config.active_provider = provider_id;
            }
            2 => {
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
            3 => {
                let mut openai_compatible_config = setup_openai_compatible_provider().await?;
                openai_compatible_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenAICompatible(openai_compatible_config));
                config.active_provider = provider_id;
            }
            4 => {
                config.providers.push(ProviderConfig::ClaudeCode(ClaudeCodeConfig {
                    id: provider_id.clone(),
                    provider: "claude_code".to_string(),
                }));
                config.active_provider = provider_id;
            }
            5 => {
                config.providers.push(ProviderConfig::OpenCode(OpenCodeConfig {
                    id: provider_id.clone(),
                    provider: "opencode".to_string(),
                }));
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

    pub fn edit() -> Result<(), String> {
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

    pub async fn setup_non_interactive(cli: &Cli) -> Result<Self, String> {
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
        } else if cli.add_simple_free {
            let api_key = cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?;

            let simple_free_config = SimpleFreeOpenRouterConfig {
                id: provider_id.clone(),
                provider: "simple_free_openrouter".to_string(),
                api_key,
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
                failed_models: Vec::new(),
                model_stats: std::collections::HashMap::new(),
                last_used_model: None,
                last_config_update: chrono::Utc::now(),
            };
            config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
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

