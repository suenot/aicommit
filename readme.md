# Install package using cargo
cargo install --path .

# Build using cargo
cargo build --release

# Configuration
The tool uses `~/.commit.json` for configuration. On first run, it will guide you through an interactive setup process.

## Supported LLM Providers

### OpenRouter
```json
{
  "providers": [{
    "provider": "openrouter",
    "api_key": "sk-or-v1-...",
    "model": "mistralai/mistral-tiny"
  }],
  "active_provider": "openrouter"
}
```

### Ollama
```json
{
  "providers": [{
    "provider": "ollama",
    "url": "http://localhost:11434",
    "model": "llama2"
  }],
  "active_provider": "ollama"
}
```

You can have multiple providers configured and switch between them by changing the `active_provider` field.
