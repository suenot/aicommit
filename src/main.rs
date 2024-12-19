use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use uuid::Uuid;
use serde_json::json;
use std::env;

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
enum ProviderConfig {
    OpenRouter(OpenRouterConfig),
    Ollama(OllamaConfig),
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    providers: Vec<ProviderConfig>,
    active_provider: String,
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

        if !config_path.exists() {
            return Err("Configuration file not found".to_string());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    async fn setup_interactive() -> Result<Self, String> {
        let mut config = Config::new();

        println!("No configuration found. Let's set up a provider.");
        let provider_options = &["OpenRouter", "Ollama"];
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
            _ => return Err("Invalid provider selection".to_string()),
        }

        // Save the configuration
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".commit.json");

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
        .header("HTTP-Referer", "https://github.com/")
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

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    match command {
        Some("--add") => {
            // Только настройка нового провайдера
            let config = Config::setup_interactive().await?;
            println!("Provider successfully configured and set as default.");
        }
        _ => {
            // Делаем коммит с текущей конфигурацией
            let config = Config::load().unwrap_or_else(|_| {
                println!("No configuration found. Run 'commit --add' to set up a provider.");
                std::process::exit(1);
            });
            run_commit(&config).await?;
        }
    }

    Ok(())
}

async fn run_commit(config: &Config) -> Result<(), String> {
    // Получаем активного провайдера
    let active_provider = config.providers.iter().find(|p| match p {
        ProviderConfig::OpenRouter(c) => c.id == config.active_provider,
        ProviderConfig::Ollama(c) => c.id == config.active_provider,
    }).ok_or("No active provider found")?;

    // Получаем git diff
    let diff = get_git_diff()?;
    if diff.is_empty() {
        return Err("No changes to commit".to_string());
    }

    // Генерируем сообщение коммита
    let (message, usage) = match active_provider {
        ProviderConfig::OpenRouter(c) => generate_openrouter_commit_message(c, &diff).await?,
        ProviderConfig::Ollama(c) => generate_ollama_commit_message(c, &diff).await?,
    };

    println!("Generated commit message: \"{}\"\n", message);
    println!("{}", message);
    println!("Tokens: {}↑ {}↓", usage.input_tokens, usage.output_tokens);
    println!("API Cost: ${:.4}", usage.total_cost);

    // Создаем коммит
    create_git_commit(&message)?;
    println!("Commit successfully created.");

    Ok(())
}

fn get_git_diff() -> Result<String, String> {
    // Сначала добавляем все изменения в staging
    let add_output = Command::new("sh")
        .arg("-c")
        .arg("git add .")
        .output()
        .map_err(|e| format!("Failed to execute git add: {}", e))?;

    if !add_output.status.success() {
        return Err(String::from_utf8_lossy(&add_output.stderr).to_string());
    }

    // Затем получаем diff
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
