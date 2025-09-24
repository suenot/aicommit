# Provider Configuration

<cite>
**Referenced Files in This Document**   
- [main.rs](file://src/main.rs)
- [readme.md](file://readme.md)
- [openrouter_models/free_models.json](file://openrouter_models/free_models.json)
- [bin/get_free_models.py](file://bin/get_free_models.py)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Provider Setup Procedures](#provider-setup-procedures)
3. [Simple Free Mode vs Custom Model Selection](#simple-free-mode-vs-custom-model-selection)
4. [Provider Trade-offs Comparison](#provider-trade-offs-comparison)
5. [Configuration Storage and Serialization](#configuration-storage-and-serialization)
6. [Code Examples for ProviderConfig Initialization](#code-examples-for-providerconfig-initialization)
7. [Common Issues and Troubleshooting](#common-issues-and-troubleshooting)
8. [Best Practices for API Key Security](#best-practices-for-api-key-security)

## Introduction
aicommit supports multiple LLM provider integrations, allowing users to generate git commit messages using various language models through different hosting options. The tool provides flexible configuration options for OpenRouter (cloud-based), Ollama (local execution), and generic OpenAI-compatible endpoints. Each provider offers distinct advantages in terms of cost, latency, privacy, and model quality, enabling users to select the optimal solution based on their specific requirements and constraints.

The system is designed with a sophisticated configuration management approach that stores provider settings in a JSON file at `~/.aicommit.json` using serde serialization. This configuration system supports advanced features like automatic model selection, failover mechanisms, and performance tracking across different providers.

**Section sources**
- [readme.md](file://readme.md#L1-L735)

## Provider Setup Procedures

### OpenRouter Configuration
To configure OpenRouter integration, users need to provide an API key and select a model. The setup can be performed interactively or non-interactively:

```bash
# Interactive setup
aicommit --add-provider
# Select "OpenRouter" from the menu

# Non-interactive setup
aicommit --add-openrouter --openrouter-api-key="your-api-key" --openrouter-model="mistralai/mistral-tiny"
```

The configuration includes standard parameters such as `max_tokens` (default: 200) and `temperature` (default: 0.3). For OpenRouter, token costs are automatically fetched from their API, providing accurate usage tracking. Users can also leverage the Simple Free mode, which automatically selects from available free models without manual configuration.

**Section sources**
- [readme.md](file://readme.md#L263-L287)
- [main.rs](file://src/main.rs#L510-L599)

### Ollama Configuration
Ollama integration allows running LLMs locally, providing enhanced privacy and offline capabilities. The setup requires configuring the local endpoint URL and selecting a model:

```bash
# Interactive setup
aicommit --add-provider
# Select "Ollama" from the menu

# Non-interactive setup
aicommit --add-ollama --ollama-url="http://localhost:11434" --ollama-model="llama2"
```

By default, the Ollama URL is set to `http://localhost:11434`, which corresponds to the standard Ollama server port. Users can pull models to their local Ollama instance using the Ollama CLI before configuring them in aicommit:

```bash
ollama pull llama2
ollama pull mistral
ollama pull codellama
```

This local execution model ensures complete data privacy as all processing occurs on the user's machine without transmitting code changes to external servers.

**Section sources**
- [readme.md](file://readme.md#L263-L287)
- [main.rs](file://src/main.rs#L510-L599)

### Generic OpenAI-Compatible Endpoints
aicommit supports any service that provides an OpenAI-compatible API endpoint, enabling integration with various local and cloud-based solutions:

```bash
# Interactive setup
aicommit --add-provider
# Select "OpenAI Compatible" from the menu

# Non-interactive setup
aicommit --add-openai-compatible \
  --openai-compatible-api-key="your-api-key" \
  --openai-compatible-api-url="https://api.example.com/v1/chat/completions" \
  --openai-compatible-model="gpt-3.5-turbo"
```

This configuration is particularly useful for integrating with local LLM servers like LM Studio, which runs a local OpenAI-compatible server. For local servers, the API key can often be any non-empty string as authentication may not be required. The flexibility of this approach allows users to connect to various services including DeepGPTBot, custom inference servers, or enterprise LLM deployments.

**Section sources**
- [readme.md](file://readme.md#L263-L287)
- [main.rs](file://src/main.rs#L510-L599)

## Simple Free Mode vs Custom Model Selection

### Simple Free Mode
The Simple Free mode provides an automated approach to using OpenRouter's free models without requiring manual model selection. When configured, the system automatically:

1. Queries OpenRouter's API for currently available free models
2. Selects the best available model based on an internally ranked preference list
3. Implements an advanced failover mechanism when models fail
4. Tracks model performance with a jail/blacklist system
5. Falls back to predefined models during network outages

```bash
# Set up Simple Free mode
aicommit --add-simple-free --openrouter-api-key="your-api-key"
```

This mode uses a curated ranking of preferred free models, prioritizing powerful models like Meta's Llama 4 series, NVIDIA's Nemotron Ultra models, and Qwen's large parameter models. The system intelligently handles model selection, making it ideal for users who want zero-cost operation without managing model configurations.

**Section sources**
- [readme.md](file://readme.md#L238-L287)
- [main.rs](file://src/main.rs#L15-L148)

### Custom Model Selection
Custom model selection gives users explicit control over which model is used for commit message generation. This approach is suitable when specific model characteristics are required:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "provider": "openrouter",
  "api_key": "sk-or-v1-...",
  "model": "mistralai/mistral-tiny",
  "max_tokens": 200,
  "temperature": 0.3
}
```

Users can select models based on specific requirements such as context length, parameter count, or specialized capabilities. This approach provides predictable behavior and allows optimization for specific use cases, though it requires more manual configuration and monitoring compared to the automated Simple Free mode.

**Section sources**
- [readme.md](file://readme.md#L263-L287)
- [main.rs](file://src/main.rs#L510-L599)

## Provider Trade-offs Comparison

### Cost Analysis
| Provider | Cost Structure | Pricing Details |
|---------|---------------|----------------|
| **OpenRouter** | Pay-per-use with free tier | Free models available; paid models charged per token |
| **Simple Free OpenRouter** | Completely free | Uses only free models from OpenRouter |
| **Ollama** | Free after initial setup | No API costs; hardware resources consumed locally |
| **OpenAI-Compatible** | Varies by service | Depends on the specific endpoint provider |

The Simple Free mode offers zero monetary cost by exclusively using OpenRouter's free models. Ollama is also free to use but requires local computational resources. Traditional OpenRouter usage incurs costs based on token consumption, while OpenAI-compatible endpoints vary depending on the specific service being used.

### Latency Comparison
| Provider | Typical Latency | Factors Affecting Performance |
|---------|----------------|------------------------------|
| **OpenRouter** | 1-5 seconds | Network latency, server load, model complexity |
| **Simple Free OpenRouter** | 1-8 seconds | Additional model discovery overhead |
| **Ollama** | 2-10 seconds | Local hardware capabilities, model size |
| **OpenAI-Compatible** | 1-6 seconds | Endpoint location and performance |

Local Ollama instances may have higher initial response times due to model loading, but subsequent requests are typically fast. Cloud-based providers like OpenRouter offer consistent performance but are subject to network conditions. The Simple Free mode has additional latency from model availability checking.

### Privacy Considerations
| Provider | Data Transmission | Privacy Level | Security Notes |
|---------|------------------|--------------|---------------|
| **OpenRouter** | Code diffs sent to cloud | Medium | Data processed by third-party servers |
| **Simple Free OpenRouter** | Code diffs sent to cloud | Medium | Same as OpenRouter |
| **Ollama** | No external transmission | High | All processing occurs locally |
| **OpenAI-Compatible** | Depends on endpoint | Variable | Local endpoints provide high privacy |

Ollama provides the highest privacy as no code leaves the local machine. Cloud-based providers transmit git diffs to external servers, which may be a concern for proprietary or sensitive codebases. Users should consider their organization's security policies when selecting a provider.

### Model Quality and Capabilities
| Provider | Model Variety | Context Length | Specialized Models |
|---------|--------------|---------------|-------------------|
| **OpenRouter** | Extensive | Up to 1M tokens | Wide range of specialized models |
| **Simple Free OpenRouter** | Limited to free models | Up to 1M tokens | Access to high-quality free models |
| **Ollama** | Depends on local downloads | Varies by model | User-selectable based on needs |
| **OpenAI-Compatible** | Depends on endpoint | Varies | Determined by endpoint capabilities |

OpenRouter provides access to the most diverse collection of models, including cutting-edge options. The Simple Free mode still offers access to powerful models like Meta's Llama 4 series and NVIDIA's large models. Ollama's capabilities depend on which models the user chooses to download and maintain locally.

**Section sources**
- [readme.md](file://readme.md#L238-L287)
- [main.rs](file://src/main.rs#L15-L148)

## Configuration Storage and Serialization

### Configuration File Structure
Provider configurations are stored in `~/.aicommit.json` using serde serialization for type-safe JSON handling. The configuration structure follows this schema:

```json
{
  "providers": [...],
  "active_provider": "provider-id",
  "retry_attempts": 3
}
```

Each provider is stored as a variant of the `ProviderConfig` enum, with specific fields serialized according to the provider type. The serde framework automatically handles the serialization and deserialization of complex Rust types to JSON format, ensuring data integrity and type safety.

### Provider-Specific Configuration
Different providers store distinct configuration parameters:

```rust
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
```

The configuration system uses UUIDs to uniquely identify each provider, allowing multiple configurations of the same provider type. The active provider is tracked by ID, enabling easy switching between different configurations.

### Advanced Configuration Features
The Simple Free mode includes additional tracking fields for its intelligent model management system:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SimpleFreeOpenRouterConfig {
    id: String,
    provider: String,
    api_key: String,
    max_tokens: i32,
    temperature: f32,
    failed_models: Vec<String>,
    model_stats: HashMap<String, ModelStats>,
    last_used_model: Option<String>,
    last_config_update: chrono::DateTime<Utc>,
}
```

These fields enable the system to track model performance over time, implement the jail/blacklist system, and optimize model selection based on historical success rates.

**Section sources**
- [main.rs](file://src/main.rs#L510-L599)
- [readme.md](file://readme.md#L238-L287)

## Code Examples for ProviderConfig Initialization

### ProviderConfig Enum Definition
The core configuration system is built around a Rust enum that represents different provider types:

```rust
#[derive(Debug, Serialize, Deserialize)]
enum ProviderConfig {
    OpenRouter(OpenRouterConfig),
    Ollama(OllamaConfig),
    OpenAICompatible(OpenAICompatibleConfig),
    SimpleFreeOpenRouter(SimpleFreeOpenRouterConfig),
}
```

This enum pattern allows the system to handle multiple provider types while maintaining type safety and proper serialization.

### Non-Interactive Configuration Setup
The configuration initialization process handles different provider types through conditional logic:

```rust
async fn setup_non_interactive(cli: &Cli) -> Result<Self, String> {
    let mut config = Config::load().unwrap_or_else(|_| Config::new());
    let provider_id = Uuid::new_v4().to_string();

    if cli.add_openrouter {
        let openrouter_config = OpenRouterConfig {
            id: provider_id.clone(),
            provider: "openrouter".to_string(),
            api_key: cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?,
            model: cli.openrouter_model.clone(),
            max_tokens: cli.max_tokens,
            temperature: cli.temperature,
        };
        config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
        config.active_provider = provider_id;
    } else if cli.add_simple_free {
        let simple_free_config = SimpleFreeOpenRouterConfig {
            id: provider_id.clone(),
            provider: "simple_free_openrouter".to_string(),
            api_key: cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?,
            max_tokens: cli.max_tokens,
            temperature: cli.temperature,
            failed_models: Vec::new(),
            model_stats: std::collections::HashMap::new(),
            last_used_model: None,
            last_config_update: chrono::Utc::now(),
        };
        config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
        config.active_provider = provider_id;
    }
    // Similar patterns for Ollama and OpenAICompatible...
}
```

This approach demonstrates how the ProviderConfig enum variants are initialized with appropriate parameters based on command-line arguments.

### Interactive Configuration Flow
The interactive setup uses dialoguer to guide users through provider selection:

```rust
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
            // Simple Free OpenRouter setup
            let api_key: String = Input::new()
                .with_prompt("Enter OpenRouter API key")
                .interact_text()
                .map_err(|e| format!("Failed to get API key: {}", e))?;
            
            let simple_free_config = SimpleFreeOpenRouterConfig {
                id: provider_id.clone(),
                provider: "simple_free_openrouter".to_string(),
                api_key,
                max_tokens: 200,
                temperature: 0.2,
                failed_models: Vec::new(),
                model_stats: std::collections::HashMap::new(),
                last_used_model: None,
                last_config_update: chrono::Utc::now(),
            };

            config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
            config.active_provider = provider_id;
        }
        // Additional match arms for other providers...
    }
}
```

This implementation shows how the same ProviderConfig enum is populated differently based on user choices during interactive setup.

**Section sources**
- [main.rs](file://src/main.rs#L510-L599)

## Common Issues and Troubleshooting

### Unreachable Servers
When servers cannot be reached, common causes and solutions include:

**For Ollama:**
- Ensure the Ollama server is running: `ollama serve`
- Verify the correct URL is configured (default: `http://localhost:11434`)
- Check firewall settings that might block the connection

**For OpenRouter and OpenAI-Compatible:**
- Verify internet connectivity
- Check API endpoint URLs for correctness
- Test API access independently using curl or similar tools
- Ensure no network restrictions (corporate firewalls, etc.)

The system implements retry logic with a configurable number of attempts (default: 3) and automatically falls back to predefined models in offline mode when possible.

### Invalid Credentials
Authentication issues typically manifest as authorization errors:

**For OpenRouter:**
- Verify the API key format (should start with "sk-or-v1-")
- Ensure the key has not been revoked or expired
- Check for copy-paste errors when entering the key

**For OpenAI-Compatible:**
- Some local servers accept any non-empty API key
- Verify the authentication requirements of the specific endpoint
- Check for typos in the API key

The configuration system validates required fields and provides clear error messages when credentials are missing or invalid.

### Model Compatibility Issues
Model-related problems can occur due to various factors:

**Unavailable Models:**
- The requested model may have been removed or renamed
- Free models may become temporarily unavailable
- Rate limits may prevent access to certain models

**Performance Problems:**
- Large models may exceed memory capacity (especially with Ollama)
- Slow response times may trigger timeouts
- Context length limitations may affect performance

The Simple Free mode includes sophisticated model management to handle these issues automatically, including:
- Tracking model success/failure rates
- Implementing a jail system for repeatedly failing models
- Maintaining blacklists for consistently problematic models
- Falling back to alternative models seamlessly

Users can monitor model status with `aicommit --jail-status` and reset models with `aicommit --unjail` or `aicommit --unjail-all`.

**Section sources**
- [main.rs](file://src/main.rs#L1431-L1431)
- [readme.md](file://readme.md#L238-L287)

## Best Practices for API Key Security

### Secure Storage Methods
To protect API keys, follow these security practices:

**Environment Variables:**
Store API keys in environment variables rather than configuration files:
```bash
export OPENROUTER_API_KEY="your-key-here"
```

**File Permissions:**
Ensure the configuration file has restrictive permissions:
```bash
chmod 600 ~/.aicommit.json
```

This prevents other users on the system from reading the file.

### Key Rotation Strategy
Implement regular key rotation to minimize exposure:

**Scheduled Rotation:**
- Rotate keys monthly or quarterly
- Update configurations after generating new keys
- Revoke old keys through the provider's dashboard

**Compromise Response:**
- Immediately revoke suspected compromised keys
- Generate new keys and update configurations
- Monitor for unauthorized usage

### Usage Monitoring
Track API usage to detect potential security issues:

**Regular Audits:**
- Review usage patterns for anomalies
- Check for unexpected spikes in token consumption
- Verify requests originate from expected locations

**Alerting:**
- Set up usage threshold alerts with your provider
- Monitor for failed authentication attempts
- Track geographic patterns of API access

### Additional Security Measures
Enhance overall security with these practices:

**Least Privilege:**
- Use API keys with minimal required permissions
- Avoid using personal account keys when possible
- Utilize service-specific keys when available

**Network Security:**
- Use encrypted connections (HTTPS) for all API communications
- Consider using API gateways or proxies in enterprise environments
- Implement rate limiting to prevent abuse

Following these best practices helps protect API keys from unauthorized access and minimizes potential damage if keys are compromised.

**Section sources**
- [readme.md](file://readme.md#L238-L287)
- [main.rs](file://src/main.rs#L510-L599)