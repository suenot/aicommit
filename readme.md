# Install package using cargo
cargo install --path .

# Build using cargo
cargo build --release

# Configuration
The tool uses `~/.commit.json` for configuration. On first run, it will guide you through an interactive setup process.

You can edit the configuration directly using:
```bash
commit --config
```
This will open your default editor ($EDITOR or vim) with the configuration file.

## Supported LLM Providers

### OpenRouter
```json
{
  "providers": [{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "provider": "openrouter",
    "api_key": "sk-or-v1-...",
    "model": "mistralai/mistral-tiny"
  }],
  "active_provider": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Ollama
```json
{
  "providers": [{
    "id": "67e55044-10b1-426f-9247-bb680e5fe0c8",
    "provider": "ollama",
    "url": "http://localhost:11434",
    "model": "llama2"
  }],
  "active_provider": "67e55044-10b1-426f-9247-bb680e5fe0c8"
}
```

You can have multiple providers configured and switch between them by changing the `active_provider` field to match the desired provider's `id`.
