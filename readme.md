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
    "model": "mistralai/mistral-tiny",
    "max_tokens": 50,
    "temperature": 0.3,
    "cost_per_1k_tokens": 0.2
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
    "model": "llama2",
    "max_tokens": 50,
    "temperature": 0.3,
    "cost_per_1k_tokens": 0.0
  }],
  "active_provider": "67e55044-10b1-426f-9247-bb680e5fe0c8"
}
```

## Provider Settings

Each provider supports the following settings:

- `max_tokens`: Maximum number of tokens in the response (default: 50)
- `temperature`: Controls randomness in the response (0.0-1.0, default: 0.3)
- `cost_per_1k_tokens`: Cost per 1,000 tokens in USD (default: 0.0)

## Usage Information

When generating a commit message, the tool will display:
- Number of tokens used (input and output)
- API cost (if cost_per_1k_tokens is set)

Example output:
```
Generated commit message: Add support for multiple LLM providers
Tokens: 8↑ 32↓
API Cost: $0.0080
```

You can have multiple providers configured and switch between them by changing the `active_provider` field to match the desired provider's `id`.
