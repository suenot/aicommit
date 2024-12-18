use std::env;
use std::path::Path;
use std::process::Command;
use dotenv::from_path;
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    // Load .env from the current working directory
    if let Err(e) = load_env_from_current_directory() {
        eprintln!("Error loading .env file: {}", e);
        return;
    }

    let api_url = env::var("OPENROUTER_API_URL").expect("OPENROUTER_API_URL is not set in .env");
    let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY is not set in .env");
    let model = env::var("MODEL_NAME").unwrap_or_else(|_| "gpt-neo-125M".to_string());

    // Stage all changes
    if let Err(e) = run_command("git add .") {
        eprintln!("Error staging changes: {}", e);
        return;
    }

    // Get `git diff` for staged changes
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

    // Generate commit message
    let commit_message = match generate_commit_message(&api_url, &api_key, &model, &diff).await {
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

// Function to load .env file from the current working directory
fn load_env_from_current_directory() -> Result<(), String> {
    let current_dir = env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let env_path = current_dir.join(".env");
    if Path::new(&env_path).exists() {
        from_path(env_path).map_err(|e| format!("Failed to load .env file: {}", e))?;
        Ok(())
    } else {
        Err(".env file not found in the current directory".to_string())
    }
}

// Function to execute terminal commands
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

// Function to generate commit message using OpenRouter API
async fn generate_commit_message(api_url: &str, api_key: &str, model: &str, diff: &str) -> Result<String, String> {
    let client = Client::new();

    let request_body = json!({
        "model": model,
        "prompt": format!("Write a clear and concise git commit message (one line, no technical terms) that describes these changes:\n\n{}", diff),
        "max_tokens": 50,
        "temperature": 0.3
    });

    let response = client
        .post(format!("{}/completions", api_url))
        .header("Authorization", format!("Bearer {}", api_key))
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

    // Clean up the response
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
