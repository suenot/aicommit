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

#[derive(Parser, Debug)]
#[command(name = "aicommit")]
#[command(about = "A CLI tool that generates concise and descriptive git commit messages using LLMs", long_about = None)]
#[command(disable_help_flag = true)]
#[command(bin_name = "aicommit")]
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

    /// Add Simple Free OpenRouter provider (uses best available free models automatically)
    #[arg(long)]
    add_simple_free: bool,

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
    #[arg(long, default_value = "200")]
    max_tokens: i32,

    /// Temperature for provider configuration
    #[arg(long, default_value = "0.2")]
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

    /// Set the git commit message without using AI. (For CI/CD or offline use cases)
    #[arg(long)]
    msg: Option<String>,

    /// Force the use of offline mode (uses fallback model list) for testing purposes
    #[arg(long, hide = true)]
    simulate_offline: bool,
    
    /// Show status of all model jails and blacklists
    #[arg(long = "jail-status")]
    jail_status: bool,
    
    /// Release specific model from jail/blacklist (model ID as parameter)
    #[arg(long = "unjail")]
    unjail: Option<String>,
    
    /// Release all models from jail/blacklist
    #[arg(long = "unjail-all")]
    unjail_all: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModelStats {
    success_count: usize,
    failure_count: usize,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_success: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_failure: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    jail_until: Option<chrono::DateTime<chrono::Utc>>,
    jail_count: usize,
    blacklisted: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    blacklisted_since: Option<chrono::DateTime<chrono::Utc>>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SimpleFreeOpenRouterConfig {
    id: String,
    provider: String,
    api_key: String,
    max_tokens: i32,
    temperature: f32,
    #[serde(default)]
    failed_models: Vec<String>,
    #[serde(default)]
    model_stats: std::collections::HashMap<String, ModelStats>,
    #[serde(default)]
    last_used_model: Option<String>,
    #[serde(default = "chrono::Utc::now")]
    last_config_update: chrono::DateTime<chrono::Utc>,
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
    SimpleFreeOpenRouter(SimpleFreeOpenRouterConfig),
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

    async fn setup_interactive() -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());

        println!("Let's set up a provider.");
        let provider_options = &["Free OpenRouter (recommended)", "OpenRouter", "Ollama", "OpenAI Compatible"];
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

// Add a new function to intelligently process git diff output
fn process_git_diff_output(diff: &str) -> String {
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

// Helper function to get a safe slice length that respects UTF-8 character boundaries
fn get_safe_slice_length(s: &str, max_len: usize) -> usize {
    if max_len >= s.len() {
        return s.len();
    }
    
    // Find the largest index <= max_len that is a char boundary
    let mut safe_len = max_len;
    while safe_len > 0 && !s.is_char_boundary(safe_len) {
        safe_len -= 1;
    }
    
    safe_len
}

#[derive(Debug)]
struct UsageInfo {
    input_tokens: i32,
    output_tokens: i32,
    total_cost: f32,
    model_used: Option<String>,
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
            println!("  --watch               Watch for changes and auto-commit");
            println!("  --wait-for-edit=<DURATION> Wait for edit delay before committing (e.g. \"30s\")");
            println!("  --jail-status         Show status of all model jails and blacklists");
            println!("  --unjail=<MODEL>      Release specific model from jail/blacklist (model ID as parameter)");
            println!("  --unjail-all          Release all models from jail/blacklist");
            println!("\nExamples:");
            println!("  aicommit --add-provider");
            println!("  aicommit --add");
            println!("  aicommit --add-openrouter --openrouter-api-key=<KEY>");
            println!("  aicommit --add-simple-free --openrouter-api-key=<KEY>");
            println!("  aicommit --list");
            println!("  aicommit --set=<ID>");
            println!("  aicommit --version-file=version.txt --version-iterate");
            println!("  aicommit --watch");
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
            ProviderConfig::SimpleFreeOpenRouter(c) => c.id == config.active_provider,
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
            ProviderConfig::SimpleFreeOpenRouter(c) => c.id == config.active_provider,
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

// Function to get available free models from OpenRouter
async fn get_available_free_models(api_key: &str, simulate_offline: bool) -> Result<Vec<String>, String> {
    // If simulate_offline is true, immediately return the fallback list
    if simulate_offline {
        println!("Debug: Simulating offline mode, using fallback model list");
        return fallback_to_preferred_models();
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10)) // Add a reasonable timeout
        .build()
        .unwrap_or_default();
    
    // Try to fetch models from OpenRouter API
    let response = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://suenot.github.io/aicommit/")
            .header("X-Title", "aicommit")
            .send()
    ).await {
        Ok(result) => match result {
            Ok(response) => {
                if !response.status().is_success() {
                    println!("Warning: OpenRouter API returned status code: {}", response.status());
                    return fallback_to_preferred_models();
                }
                response
            },
            Err(e) => {
                println!("Warning: Failed to connect to OpenRouter API: {}", e);
                println!("Using predefined free models as fallback...");
                return fallback_to_preferred_models();
            }
        },
        Err(_) => {
            println!("Warning: Request to OpenRouter API timed out after 15 seconds");
            println!("Using predefined free models as fallback...");
            return fallback_to_preferred_models();
        }
    };
    
    // Try to parse the response
    let models_response: Result<serde_json::Value, _> = response.json().await;
    if let Err(e) = &models_response {
        println!("Warning: Failed to parse OpenRouter API response: {}", e);
        println!("Using predefined free models as fallback...");
        return fallback_to_preferred_models();
    }
    
    let models_response = models_response.unwrap();
    let mut free_models = Vec::new();
    
    if let Some(data) = models_response["data"].as_array() {
        // First pass: check all models that are explicitly marked as free
        for model in data {
            if let Some(id) = model["id"].as_str() {
                // Multiple ways to detect if a model is free:
                
                // 1. Check if the model ID contains ":free"
                if id.contains(":free") {
                    free_models.push(id.to_string());
                    continue;
                }
                
                // 2. Check if "free" field is true
                if let Some(true) = model["free"].as_bool() {
                    free_models.push(id.to_string());
                    continue;
                }
                
                // 3. Check if "free_tokens" is greater than 0
                if let Some(tokens) = model["free_tokens"].as_u64() {
                    if tokens > 0 {
                        free_models.push(id.to_string());
                        continue;
                    }
                }
                
                // 4. Check if pricing is 0 for both prompt and completion
                if let Some(pricing) = model["pricing"].as_object() {
                    let prompt_price = pricing.get("prompt")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    
                    let completion_price = pricing.get("completion")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    
                    if prompt_price == 0.0 && completion_price == 0.0 {
                        free_models.push(id.to_string());
                        continue;
                    }
                }
            }
        }
        
        // If no free models found, try a second pass with more relaxed criteria
        if free_models.is_empty() {
            // Look for models with very low pricing (<= 0.0001)
            for model in data {
                if let Some(id) = model["id"].as_str() {
                    if let Some(pricing) = model["pricing"].as_object() {
                        let prompt_price = pricing.get("prompt")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0);
                        
                        let completion_price = pricing.get("completion")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0);
                        
                        // Consider very low-priced models as "effectively free"
                        if prompt_price <= 0.0001 && completion_price <= 0.0001 {
                            free_models.push(id.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // If we still found no free models, fall back to predefined list
    if free_models.is_empty() {
        println!("Warning: No free models found from OpenRouter API");
        println!("Using predefined free models as fallback...");
        return fallback_to_preferred_models();
    }
    
    Ok(free_models)
}

// Helper function to fall back to predefined models
fn fallback_to_preferred_models() -> Result<Vec<String>, String> {
    let mut models = Vec::new();
    
    // Add all predefined free models
    for model in PREFERRED_FREE_MODELS {
        models.push(model.to_string());
    }
    
    if models.is_empty() {
        return Err("No free models available, and fallback model list is empty".to_string());
    }
    
    Ok(models)
}

// Function to find best available model
fn find_best_available_model(available_models: &[String], config: &SimpleFreeOpenRouterConfig) -> Option<String> {
    // Start by using the previously successful model if it's still available
    if let Some(last_model) = &config.last_used_model {
        if available_models.contains(last_model) {
            let stats = config.model_stats.get(last_model);
            if is_model_available(&stats) {
                return Some(last_model.clone());
            }
        }
    }

    // Filter models that are not in jail or blacklisted
    let available_candidates: Vec<&String> = available_models
        .iter()
        .filter(|model| {
            let stats = config.model_stats.get(*model);
            is_model_available(&stats)
        })
        .collect();
    
    // First try our curated list of preferred models in order
    for preferred in PREFERRED_FREE_MODELS {
        let preferred_str = preferred.to_string();
        if available_candidates.contains(&&preferred_str) {
            return Some(preferred_str);
        }
    }
    
    // If none of our preferred models are available, use an intelligent fallback approach
    // by analyzing model names for parameter sizes (like 70b, 32b, etc.)
    if !available_candidates.is_empty() {
        // Sort by estimated parameter count (highest first)
        let mut sorted_candidates = available_candidates.clone();
        sorted_candidates.sort_by(|a, b| {
            let a_size = extract_model_size(a);
            let b_size = extract_model_size(b);
            b_size.cmp(&a_size) // Reverse order (largest first)
        });
        
        // Return the largest available model
        return Some(sorted_candidates[0].clone());
    }
    
    // If all models are jailed or blacklisted, try the least recently jailed one
    if !available_models.is_empty() {
        // Get all jailed but not blacklisted models
        let mut jailed_models: Vec<(String, chrono::DateTime<chrono::Utc>)> = Vec::new();
        
        for model in available_models {
            if let Some(stats) = config.model_stats.get(model) {
                if !stats.blacklisted && stats.jail_until.is_some() {
                    jailed_models.push((model.clone(), stats.jail_until.unwrap()));
                }
            }
        }
        
        // Sort by jail expiry time (soonest first)
        if !jailed_models.is_empty() {
            jailed_models.sort_by_key(|x| x.1);
            return Some(jailed_models[0].0.clone());
        }
        
        // Last resort: just use any model, even blacklisted ones
        return Some(available_models[0].clone());
    }
    
    None
}

// Helper function to extract model size from name (e.g., "llama-70b" -> 70)
fn extract_model_size(model_name: &str) -> u32 {
    let lower_name = model_name.to_lowercase();
    
    // Look for patterns like "70b", "32b", "7b", etc.
    let patterns = [
        "253b", "235b", "200b", "124b",
        "70b", "80b", "90b", "72b", "65b", 
        "40b", "32b", "30b", "24b", "20b",
        "16b", "14b", "13b", "12b", "11b", "10b",
        "9b", "8b", "7b", "6b", "5b", "4b", "3b", "2b", "1b"
    ];
    
    for pattern in patterns {
        if lower_name.contains(pattern) {
            // Extract the number from the pattern (e.g., "70b" -> 70)
            if let Ok(size) = pattern.trim_end_matches(|c| c == 'b' || c == 'B').parse::<u32>() {
                return size;
            }
        }
    }
    
    // Default size if no pattern matches
    // Check for specific keywords that might indicate a more powerful model
    if lower_name.contains("large") || lower_name.contains("ultra") {
        return 15; // Assume it's a medium-large model
    } else if lower_name.contains("medium") {
        return 10;
    } else if lower_name.contains("small") || lower_name.contains("tiny") {
        return 5;
    }
    
    // Default fallback
    0
}

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
        model_used: Some(config.model.clone()),
    };

    Ok((message, usage))
}

async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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

async fn generate_openai_compatible_commit_message(config: &OpenAICompatibleConfig, diff: &str, cli: &Cli) -> Result<(String, UsageInfo), String> {
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
        model_used: Some(config.model.clone()),
    };

    Ok((message, usage))
}

async fn generate_simple_free_commit_message(
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
    
    let message = response_data.choices
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

/// Helper function to save SimpleFreeOpenRouterConfig changes to disk
fn save_simple_free_config(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?
        .join(".aicommit.json");
        
    let mut full_config = Config::load()?;
    
    // Update the provider in the full config
    for provider in &mut full_config.providers {
        if let ProviderConfig::SimpleFreeOpenRouter(simple_config) = provider {
            if simple_config.id == config.id {
                *simple_config = config.clone();
                break;
            }
        }
    }
    
    let content = serde_json::to_string_pretty(&full_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

// Helper constants for model management
const MAX_CONSECUTIVE_FAILURES: usize = 3;
const INITIAL_JAIL_HOURS: i64 = 24;
const JAIL_TIME_MULTIPLIER: i64 = 2;
const MAX_JAIL_HOURS: i64 = 168; // 7 days
const BLACKLIST_AFTER_JAIL_COUNT: usize = 3;
const BLACKLIST_RETRY_DAYS: i64 = 7;

/// Decides if a model should be used based on its jail/blacklist status
fn is_model_available(model_stats: &Option<&ModelStats>) -> bool {
    match model_stats {
        None => true, // No stats yet, model is available
        Some(stats) => {
            // Check if blacklisted but should be retried
            if stats.blacklisted {
                if let Some(blacklisted_since) = stats.blacklisted_since {
                    let retry_duration = chrono::Duration::days(BLACKLIST_RETRY_DAYS);
                    let now = chrono::Utc::now();
                    
                    // If blacklisted for more than retry period, give it another chance
                    if now - blacklisted_since > retry_duration {
                        return true;
                    }
                    return false;
                }
                return false;
            }
            
            // Check if currently in jail
            if let Some(jail_until) = stats.jail_until {
                if chrono::Utc::now() < jail_until {
                    return false;
                }
            }
            
            true
        }
    }
}

/// Update model stats with success result
fn record_model_success(model_stats: &mut ModelStats) {
    model_stats.success_count += 1;
    model_stats.last_success = Some(chrono::Utc::now());
    
    // Reset consecutive failures if successful
    if model_stats.last_failure.is_none() || 
       model_stats.last_success.unwrap() > model_stats.last_failure.unwrap() {
        // The model is working now, remove any jail time
        model_stats.jail_until = None;
    }
}

/// Update model stats with failure result and determine if it should be jailed
fn record_model_failure(model_stats: &mut ModelStats) {
    let now = chrono::Utc::now();
    model_stats.failure_count += 1;
    model_stats.last_failure = Some(now);
    
    // Check if we have consecutive failures
    let has_consecutive_failures = match model_stats.last_success {
        None => true, // Never had a success
        Some(last_success) => {
            // If last success is older than last failure, we have consecutive failures
            model_stats.last_failure.unwrap() > last_success
        }
    };
    
    if has_consecutive_failures {
        // Count consecutive failures by comparing timestamps
        let consecutive_failures = if let Some(last_success) = model_stats.last_success {
            let hours_since_success = (now - last_success).num_hours();
            // If it's been more than a day since last success, count as consecutive failures
            if hours_since_success > 24 {
                model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
            } else {
                // Count failures since last success
                1 // This is at least 1 consecutive failure
            }
        } else {
            // No success ever, count all failures
            model_stats.failure_count.min(MAX_CONSECUTIVE_FAILURES)
        };
        
        // Jail if we hit the threshold
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            // Calculate jail duration based on recidivism
            let jail_hours = INITIAL_JAIL_HOURS * JAIL_TIME_MULTIPLIER.pow(model_stats.jail_count as u32);
            let jail_hours = jail_hours.min(MAX_JAIL_HOURS); // Cap at maximum
            
            // Set jail expiration time
            model_stats.jail_until = Some(now + chrono::Duration::hours(jail_hours));
            model_stats.jail_count += 1;
            
            // Blacklist if consistently problematic
            if model_stats.jail_count >= BLACKLIST_AFTER_JAIL_COUNT {
                model_stats.blacklisted = true;
                model_stats.blacklisted_since = Some(now);
            }
        }
    }
}

/// Show formatted model jail status
fn format_model_status(model: &str, stats: &ModelStats) -> String {
    let status = if stats.blacklisted {
        "BLACKLISTED".to_string()
    } else if let Some(jail_until) = stats.jail_until {
        if chrono::Utc::now() < jail_until {
            let remaining = jail_until - chrono::Utc::now();
            format!("JAILED ({}h remaining)", remaining.num_hours())
        } else {
            "ACTIVE".to_string()
        }
    } else {
        "ACTIVE".to_string()
    };
    
    let last_success = stats.last_success.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    let last_failure = stats.last_failure.map_or("Never".to_string(), |ts| {
        let ago = chrono::Utc::now() - ts;
        if ago.num_days() > 0 {
            format!("{} days ago", ago.num_days())
        } else if ago.num_hours() > 0 {
            format!("{} hours ago", ago.num_hours())
        } else {
            format!("{} minutes ago", ago.num_minutes())
        }
    });
    
    format!("{}: {} (Success: {}, Failure: {}, Last success: {}, Last failure: {})",
            model, status, stats.success_count, stats.failure_count, last_success, last_failure)
}

/// Display jail status for all models
fn display_model_jail_status(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
    if config.model_stats.is_empty() {
        println!("No model statistics available yet.");
        return Ok(());
    }
    
    println!("\nModel Status Report:");
    println!("===================");
    
    // Group models by status
    let mut active_models = Vec::new();
    let mut jailed_models = Vec::new();
    let mut blacklisted_models = Vec::new();
    
    for (model, stats) in &config.model_stats {
        if stats.blacklisted {
            blacklisted_models.push(format_model_status(model, stats));
        } else if let Some(jail_until) = stats.jail_until {
            if chrono::Utc::now() < jail_until {
                jailed_models.push(format_model_status(model, stats));
            } else {
                active_models.push(format_model_status(model, stats));
            }
        } else {
            active_models.push(format_model_status(model, stats));
        }
    }
    
    // Sort and display each group
    if !active_models.is_empty() {
        println!("\nACTIVE MODELS:");
        active_models.sort();
        for model in active_models {
            println!("  {}", model);
        }
    }
    
    if !jailed_models.is_empty() {
        println!("\nJAILED MODELS:");
        jailed_models.sort();
        for model in jailed_models {
            println!("  {}", model);
        }
    }
    
    if !blacklisted_models.is_empty() {
        println!("\nBLACKLISTED MODELS:");
        blacklisted_models.sort();
        for model in blacklisted_models {
            println!("  {}", model);
        }
    }
    
    Ok(())
}

/// Release a specific model from jail/blacklist
fn unjail_model(config: &mut SimpleFreeOpenRouterConfig, model_id: &str) -> Result<(), String> {
    let model_found = if model_id == "*" {
        // Reset all models
        for (_, stats) in config.model_stats.iter_mut() {
            stats.jail_until = None;
            stats.blacklisted = false;
            stats.jail_count = 0;
        }
        true
    } else {
        // Reset specific model
        if let Some(stats) = config.model_stats.get_mut(model_id) {
            stats.jail_until = None;
            stats.blacklisted = false;
            stats.jail_count = 0;
            true
        } else {
            false
        }
    };
    
    if !model_found {
        return Err(format!("Model '{}' not found in statistics", model_id));
    }
    
    // Save updated config
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?
        .join(".aicommit.json");
        
    let mut full_config = Config::load()?;
    
    // Update the provider in the full config
    for provider in &mut full_config.providers {
        if let ProviderConfig::SimpleFreeOpenRouter(simple_config) = provider {
            if simple_config.id == config.id {
                *simple_config = config.clone();
                break;
            }
        }
    }
    
    let content = serde_json::to_string_pretty(&full_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

/// Release all models from jail/blacklist
fn unjail_all_models(config: &mut SimpleFreeOpenRouterConfig) -> Result<(), String> {
    unjail_model(config, "*")
}