# commit

[![Crates.io](https://img.shields.io/crates/v/commit.svg)](https://crates.io/crates/commit)
[![Documentation](https://docs.rs/commit/badge.svg)](https://docs.rs/commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A CLI tool that generates concise and descriptive git commit messages using LLMs (Large Language Models).

## Features

- 🤖 Uses LLMs to generate meaningful commit messages from your changes
- 🔄 Supports multiple LLM providers (OpenRouter, Ollama)
- ⚡ Fast and efficient - works directly from your terminal
- 🛠️ Easy configuration and customization
- 💰 Transparent token usage and cost tracking

## Installation

Install via cargo:

```bash
cargo install commit
```

Or build from source:

```bash
git clone https://github.com/yourusername/commit
cd commit
cargo install --path .
```

## Quick Start

1. Add a provider:
```bash
commit --add
```

2. Make some changes to your code

3. Create a commit:
```bash
commit
```

## Configuration

The configuration file is stored at `~/.commit.json`. You can edit it directly with:

```bash
commit --config
```

### Provider Configuration

Each provider can be configured with the following settings:

- `max_tokens`: Maximum number of tokens in the response (default: 50)
- `temperature`: Controls randomness in the response (0.0-1.0, default: 0.3)

For OpenRouter, token costs are automatically fetched from their API. For Ollama, you can specify your own costs if you want to track usage.

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
    "input_cost_per_1k_tokens": 0.25,
    "output_cost_per_1k_tokens": 0.25
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
    "input_cost_per_1k_tokens": 0.0,
    "output_cost_per_1k_tokens": 0.0
  }],
  "active_provider": "67e55044-10b1-426f-9247-bb680e5fe0c8"
}
```

## Usage Information

When generating a commit message, the tool will display:
- Number of tokens used (input and output)
- Total API cost (calculated separately for input and output tokens)

Example output:
```
Generated commit message: Add support for multiple LLM providers
Tokens: 8↑ 32↓
API Cost: $0.0100
```

You can have multiple providers configured and switch between them by changing the `active_provider` field to match the desired provider's `id`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
