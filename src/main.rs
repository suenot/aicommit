use std::env;
use std::path::Path;
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use dialoguer::{Input, Select};
use console::Term;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterConfig {
    id: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaConfig {
    id: String,
    model: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "provider")]
enum ProviderConfig {
    #[serde(rename = "openrouter")]
    OpenRouter(OpenRouterConfig),
    #[serde(rename = "ollama")]
    Ollama(OllamaConfig),
}

impl ProviderConfig {
    fn get_id(&self) -> &str {
        match self {
            ProviderConfig::OpenRouter(config) => &config.id,
            ProviderConfig::Ollama(config) => &config.id,
        }
    }
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
            .join(".commit.json");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))
        } else {
            Ok(Config::new())
        }
    }

    fn save(&self) -> Result<(), String> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".commit.json");

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    async fn setup_interactive() -> Result<Self, String> {
        let term = Term::stdout();
        let mut config = Config::new();

        println!("Welcome to commit setup!");
        
        let provider_options = &["OpenRouter", "Ollama"];
        let provider_selection = Select::new()
            .with_prompt("Select LLM provider")
            .items(provider_options)
            .interact()
            .map_err(|e| format!("Failed to get provider selection: {}", e))?;

        let provider_id = Uuid::new_v4().to_string();

        match provider_selection {
            0 => {
                let api_key: String = Input::new()
                    .with_prompt("Enter OpenRouter API key")
                    .interact_text()
                    .map_err(|e| format!("Failed to get API key: {}", e))?;

                let model: String = Input::new()
                    .with_prompt("Enter model name (default: mistralai/mistral-tiny)")
                    .default("mistralai/mistral-tiny".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get model name: {}", e))?;

                config.providers.push(ProviderConfig::OpenRouter(OpenRouterConfig {
                    id: provider_id.clone(),
                    api_key,
                    model,
                }));
                config.active_provider = provider_id;
            }
            1 => {
                let url: String = Input::new()
                    .with_prompt("Enter Ollama URL (default: http://localhost:11434)")
                    .default("http://localhost:11434".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get URL: {}", e))?;

                let model: String = Input::new()
                    .with_prompt("Enter model name (default: llama2)")
                    .default("llama2".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get model name: {}", e))?;

                config.providers.push(ProviderConfig::Ollama(OllamaConfig {
                    id: provider_id.clone(),
                    url,
                    model,
                }));
                config.active_provider = provider_id;
            }
            _ => unreachable!(),
        }

        config.save()?;
        println!("Configuration saved to ~/.commit.json");
        Ok(config)
    }
}

#[tokio::main]
async fn main() {
    // Check for --config flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--config" {
        edit_config();
        return;
    }

    let config = match Config::load() {
        Ok(config) if !config.providers.is_empty() => config,
        _ => match Config::setup_interactive().await {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to setup configuration: {}", e);
                return;
            }
        },
    };

    // Stage all changes
    if let Err(e) = run_command("git add .") {
        eprintln!("Error staging changes: {}", e);
        return;
    }

    // Get git diff
    let diff = match run_command("git diff --cached") {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error getting git diff: {}", e);
            return;
        }
    };

    if diff.trim().is_empty() {
        println!("No changes to commit.");
        return;
    }

    // Generate commit message based on active provider
    let commit_message = match &config.providers.iter().find(|p| p.get_id() == config.active_provider) {
        Some(ProviderConfig::OpenRouter(config)) => {
            generate_openrouter_commit_message(config, &diff).await
        }
        Some(ProviderConfig::Ollama(config)) => {
            generate_ollama_commit_message(config, &diff).await
        }
        None => {
            eprintln!("No active provider configured");
            return;
        }
    };

    let commit_message = match commit_message {
        Ok(message) => message,
        Err(e) => {
            eprintln!("Error generating commit message: {}", e);
            return;
        }
    };

    println!("Generated commit message: {}", commit_message);

    // Commit changes with properly escaped message
    if let Err(e) = run_command(&format!("git commit -m '{}'", commit_message.replace("'", "'\\''"))) {
        eprintln!("Error committing changes: {}", e);
    } else {
        println!("Commit successfully created.");
    }
}

async fn generate_openrouter_commit_message(config: &OpenRouterConfig, diff: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": config.model,
        "prompt": format!("Write a clear and concise git commit message (one line, no technical terms) that describes these changes:\n\n{}", diff),
        "max_tokens": 50,
        "temperature": 0.3
    });

    let response = client
        .post("https://openrouter.ai/api/v1/completions")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", "https://github.com/")
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

    let commit_message = json["choices"][0]["text"]
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

    Ok(commit_message)
}

async fn generate_ollama_commit_message(config: &OllamaConfig, diff: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": config.model,
        "prompt": format!("Write a clear and concise git commit message (one line, no technical terms) that describes these changes:\n\n{}", diff),
        "stream": false
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

    Ok(commit_message)
}

// Helper function to run shell commands
fn run_command(command: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn edit_config() {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let config_path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".commit.json");

    if !config_path.exists() {
        // Create default config if it doesn't exist
        let default_config = Config::new();
        let content = serde_json::to_string_pretty(&default_config)
            .expect("Failed to serialize default config");
        fs::write(&config_path, content)
            .expect("Failed to write default config file");
    }

    let status = Command::new(&editor)
        .arg(config_path)
        .status()
        .expect("Failed to open editor");

    if !status.success() {
        eprintln!("Failed to edit configuration");
    }
}
