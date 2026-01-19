// Utility functions

use std::fs;
use std::env;
use crate::types::*;

// From: 016_function_default_retry_attempts.rs
// Used by serde(default) in types.rs
#[allow(dead_code)]
pub fn default_retry_attempts() -> u32 {
    3
}

// From: 021_function_get_safe_slice_length.rs
pub fn get_safe_slice_length(s: &str, max_len: usize) -> usize {
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

// From: 027_function_parse_duration.rs
pub fn parse_duration(duration_str: &str) -> Result<std::time::Duration, String> {
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

// From: 045_function_save_simple_free_config.rs
pub fn save_simple_free_config(config: &SimpleFreeOpenRouterConfig) -> Result<(), String> {
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

/// HTTP client settings combining CLI arguments, config file, and environment variables
pub struct HttpClientSettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub timeout_secs: u64,
}

impl HttpClientSettings {
    /// Create settings from CLI arguments, config, and environment variables
    /// Priority: CLI args > Environment variables > Config file
    pub fn from_cli_and_config(cli: &Cli, config: Option<&Config>) -> Self {
        // Get proxy settings with priority: CLI > ENV > Config
        let http_proxy = cli.http_proxy.clone()
            .or_else(|| env::var("HTTP_PROXY").ok())
            .or_else(|| env::var("http_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.http_proxy.clone()));

        let https_proxy = cli.https_proxy.clone()
            .or_else(|| env::var("HTTPS_PROXY").ok())
            .or_else(|| env::var("https_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.https_proxy.clone()));

        let no_proxy = cli.no_proxy.clone()
            .or_else(|| env::var("NO_PROXY").ok())
            .or_else(|| env::var("no_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.no_proxy.clone()));

        // Get timeout: CLI (if non-default) > Config > Default
        let timeout_secs = if cli.timeout != 30 {
            cli.timeout
        } else {
            config.map(|c| c.request_timeout).unwrap_or(30)
        };

        HttpClientSettings {
            http_proxy,
            https_proxy,
            no_proxy,
            timeout_secs,
        }
    }

    /// Create settings from config only (for cases without CLI access)
    pub fn from_config(config: Option<&Config>) -> Self {
        // Get proxy settings from ENV > Config
        let http_proxy = env::var("HTTP_PROXY").ok()
            .or_else(|| env::var("http_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.http_proxy.clone()));

        let https_proxy = env::var("HTTPS_PROXY").ok()
            .or_else(|| env::var("https_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.https_proxy.clone()));

        let no_proxy = env::var("NO_PROXY").ok()
            .or_else(|| env::var("no_proxy").ok())
            .or_else(|| config.and_then(|c| c.proxy.as_ref()).and_then(|p| p.no_proxy.clone()));

        let timeout_secs = config.map(|c| c.request_timeout).unwrap_or(30);

        HttpClientSettings {
            http_proxy,
            https_proxy,
            no_proxy,
            timeout_secs,
        }
    }

    /// Check if any proxy is configured
    pub fn has_proxy(&self) -> bool {
        self.http_proxy.is_some() || self.https_proxy.is_some()
    }
}

/// Build an HTTP client with proxy and timeout configuration
///
/// This function creates a reqwest::Client configured with:
/// - HTTP/HTTPS proxy settings (from CLI, environment variables, or config file)
/// - NO_PROXY bypass rules
/// - Request timeout
///
/// Priority for proxy settings: CLI args > Environment variables > Config file
pub fn build_http_client(settings: &HttpClientSettings) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(settings.timeout_secs));

    // Configure proxy if any proxy settings are provided
    if settings.has_proxy() {
        // Build proxy configuration
        if let Some(ref https_proxy) = settings.https_proxy {
            let mut proxy = reqwest::Proxy::https(https_proxy)
                .map_err(|e| format!("Invalid HTTPS proxy URL '{}': {}", https_proxy, e))?;

            // Apply NO_PROXY rules
            if let Some(ref no_proxy) = settings.no_proxy {
                proxy = proxy.no_proxy(reqwest::NoProxy::from_string(no_proxy));
            }

            builder = builder.proxy(proxy);
        }

        if let Some(ref http_proxy) = settings.http_proxy {
            let mut proxy = reqwest::Proxy::http(http_proxy)
                .map_err(|e| format!("Invalid HTTP proxy URL '{}': {}", http_proxy, e))?;

            // Apply NO_PROXY rules
            if let Some(ref no_proxy) = settings.no_proxy {
                proxy = proxy.no_proxy(reqwest::NoProxy::from_string(no_proxy));
            }

            builder = builder.proxy(proxy);
        }
    }

    builder.build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Build an HTTP client with a specific timeout override
pub fn build_http_client_with_timeout(settings: &HttpClientSettings, timeout_secs: u64) -> Result<reqwest::Client, String> {
    let modified_settings = HttpClientSettings {
        http_proxy: settings.http_proxy.clone(),
        https_proxy: settings.https_proxy.clone(),
        no_proxy: settings.no_proxy.clone(),
        timeout_secs,
    };
    build_http_client(&modified_settings)
}

/// Display proxy configuration for verbose output
pub fn display_proxy_config(settings: &HttpClientSettings, verbose: bool) {
    if verbose && settings.has_proxy() {
        println!("\n=== Proxy Configuration ===");
        if let Some(ref http_proxy) = settings.http_proxy {
            println!("HTTP Proxy: {}", http_proxy);
        }
        if let Some(ref https_proxy) = settings.https_proxy {
            println!("HTTPS Proxy: {}", https_proxy);
        }
        if let Some(ref no_proxy) = settings.no_proxy {
            println!("NO_PROXY: {}", no_proxy);
        }
        println!("Request Timeout: {}s", settings.timeout_secs);
    }
}

