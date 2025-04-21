# aicommit
![logo](./docs/aicommit-logo.png)

[![Crates.io](https://img.shields.io/crates/v/aicommit.svg)](https://crates.io/crates/aicommit)
[![Documentation](https://docs.rs/aicommit/badge.svg)](https://docs.rs/aicommit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/suenot.suenot-aicommit-vscode?label=VS%20Code%20extension&color=23a2f0)](https://marketplace.visualstudio.com/items?itemName=suenot.suenot-aicommit-vscode)

📚 [Website & Documentation](https://suenot.github.io/aicommit/)

A CLI tool that generates concise and descriptive git commit messages using LLMs (Large Language Models).

## Features

### Implemented Features
- ✅ Uses LLMs to generate meaningful commit messages from your changes
- ✅ Supports multiple LLM providers (OpenRouter, Ollama, LM Studio and any other OpenAI compatible api)
- ✅ Custom api keys for services through open router api (for google aistudio and etc) - go to https://openrouter.ai/settings/integrations and paste key from any of them: AI21, Amazon BedRock, Anthropic, AnyScale, Avian.io, Cloudflare, Cohere, DeepInfra, **DeepSeek**, Fireworks, **Google AI Studio**, Google Vertex, Hyperbolic, Infermatic, Inflection, Lambda, Lepton, Mancer, Mistral, NovitaAI, OpenAI, Perplexity, Recursal, SambaNova, SF Compute, Together, xAI
- ✅ Fast and efficient - works directly from your terminal
- ✅ Easy configuration and customization
- ✅ Transparent token usage and cost tracking
- ✅ Automatic retry on provider errors (configurable attempts with 5s delay)
- ✅ Version management with automatic incrementation
- ✅ Version synchronization with Cargo.toml
- ✅ Version synchronization with package.json
- ✅ GitHub version management
- ✅ VS Code integration for generating commit messages directly in the editor
- ✅ Provider management (add, list, set active)
- ✅ Interactive configuration setup
- ✅ Configuration file editing
- ✅ Add all to stash functionality (`aicommit --add`)
- ✅ Auto push functionality (`aicommit --push`)
- ✅ Auto pull functionality (`aicommit --pull`)
- ✅ Automatic upstream branch setup for new branches
- ✅ Interactive commit message generation (`aicommit --dry-run`)
- ✅ Basic .gitignore file checks and management (create ~/.default_gitignore and use it as template if there is no .gitignore in this directory)
- ✅ Help information display (`aicommit --help`)
- ✅ Publication in npm
- ✅ Support for cross-compilation (ARM, AARCH64, etc.)
- ✅ Installation from binary
- ✅ --verbose mode (show context for LLM)
- ✅ Watch mode for automatic commits

### Planned Features
- 🚧 MCP
- 🚧 Support github issues (sync, auto open, auto close)
- 🚧 Tests for each feature to prevent breaking changes
- 🚧 Split commits by file (`aicommit --by-file`)
- 🚧 Split commits by feature (`aicommit --by-feature`)
- 🚧 Version management for multiple languages (requirements.txt, etc.)
- 🚧 Branch safety checks for push operations
- 🚧 Publication management
- 🚧 Publication in brew/macports
- 🚧 Publication in apt/apk/yum/pacman
- 🚧 Publication in other package managers
- 🚧 Support for submodules
- 🚧 Support for mercurial
- 🚧 Langchain support for multiple providers and custom logic
- 🚧 Using priority for providers (if one of provider broken)

Legend:
- ✅ Implemented
- 🚧 Planned
- 🧪 Has tests

## Installation

There are several ways to install aicommit:

### Using Cargo (Rust package manager)

If you have Rust installed:
```bash
cargo install aicommit
```

### Using npm/npx

```bash
# Run without installation
npx @suenot/aicommit

# Or install globally
npm install -g @suenot/aicommit
aicommit
```

### Using brew
```bash
# Install Homebrew if you haven't already
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Add aicommit tap and install
brew tap suenot/tap
brew install suenot/tap/aicommit
```

### Manual Installation

#### Download Pre-built Binaries

You can download pre-built binaries from the [GitHub Releases](https://github.com/suenot/aicommit/releases) page.

Available builds:
- Linux (x86_64, ARM64)
- macOS (Intel x86_64, Apple Silicon ARM64)
- Windows (x86_64, ARM64)

#### Linux/macOS:
```bash
# 1. Download and extract (replace VERSION and ARCH with appropriate values)
# wget https://github.com/suenot/aicommit/releases/download/vVERSION/aicommit-<ARCH>
# chmod +x aicommit-<ARCH>
# mv aicommit-<ARCH> aicommit
# sudo mv aicommit /usr/local/bin/

# Example for Linux x86_64:
wget https://github.com/suenot/aicommit/releases/download/v0.1.72/aicommit-linux-x86_64
mv aicommit-linux-x86_64 aicommit
chmod +x aicommit
sudo mv aicommit /usr/local/bin/

# Example for macOS ARM64:
wget https://github.com/suenot/aicommit/releases/download/v0.1.72/aicommit-macos-aarch64
mv aicommit-macos-aarch64 aicommit
chmod +x aicommit
sudo mv aicommit /usr/local/bin/

# Example for macOS x86_64:
wget https://github.com/suenot/aicommit/releases/download/v0.1.72/aicommit-macos-x86_64
mv aicommit-macos-x86_64 aicommit
chmod +x aicommit
sudo mv aicommit /usr/local/bin/


# 2. Make it executable
chmod +x aicommit-<ARCH>


# 3. Move to a directory in your PATH (optional)

```

#### Windows:
1. Download the ZIP file for your architecture
2. Extract the executable
3. Add the directory to your PATH or move the executable to a directory in your PATH

### Build from Source

If you want to build the latest version from source:

```bash
# 1. Clone the repository
git clone https://github.com/suenot/aicommit
cd aicommit

# 2. Build and install
cargo install --path .
```

Requirements for building from source:
- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- A C compiler (gcc, clang, or MSVC)
- OpenSSL development packages (on Linux)

## Quick Start

1. Add a provider (choose one method):

   Interactive mode:
   ```bash
   aicommit --add-provider
   ```

   Non-interactive mode (example with OpenRouter):
   ```bash
   aicommit --add-provider --add-openrouter --openrouter-api-key "your-api-key"
   ```

2. Make some changes to your code

3. Create a commit:
   ```bash
   # Commit only staged changes (files added with git add)
   aicommit

   # Automatically stage and commit all changes
   aicommit --add

   # Stage all changes, commit, and push
   aicommit --add --push
   ```

## Provider Management

Add a provider in interactive mode:
```bash
aicommit --add-provider
```

Add providers in non-interactive mode:
```bash
# Add OpenRouter provider
aicommit --add-provider --add-openrouter --openrouter-api-key "your-api-key" --openrouter-model "mistralai/mistral-tiny"

# Add Ollama provider
aicommit --add-provider --add-ollama --ollama-url "http://localhost:11434" --ollama-model "llama2"

# Add OpenAI compatible provider
aicommit --add-provider --add-openai-compatible \
  --openai-compatible-api-key "your-api-key" \
  --openai-compatible-api-url "https://api.deep-foundation.tech/v1/chat/completions" \
  --openai-compatible-model "gpt-4o-mini"
```

Optional parameters for non-interactive mode:
- `--max-tokens` - Maximum number of tokens (default: 50)
- `--temperature` - Controls randomness (default: 0.3)

List all configured providers:
```bash
aicommit --list
```

Set active provider:
```bash
aicommit --set <provider-id>
```

## Version Management

aicommit supports automatic version management with the following features:

1. Automatic version incrementation using a version file:
```bash
aicommit --version-file version --version-iterate
```

2. Synchronize version with Cargo.toml:
```bash
aicommit --version-file version --version-iterate --version-cargo
```

3. Synchronize version with package.json:
```bash
aicommit --version-file version --version-iterate --version-npm
```

4. Update version on GitHub (creates a new tag):
```bash
aicommit --version-file version --version-iterate --version-github
```

You can combine these flags to update multiple files at once:
```bash
aicommit --version-file version --version-iterate --version-cargo --version-npm --version-github
```

## VS Code Extension

aicommit now includes a VS Code extension for seamless integration with the editor:

1. Navigate to the vscode-extension directory
```bash
cd vscode-extension
```

2. Install the extension locally for development
```bash
code --install-extension aicommit-vscode-0.1.0.vsix
```

Or build the extension package manually:
```bash
# Install vsce if not already installed
npm install -g @vscode/vsce

# Package the extension
vsce package
```

Once installed, you can generate commit messages directly from the Source Control view in VS Code by clicking the "AICommit: Generate Commit Message" button.

See the [VS Code Extension README](./vscode-extension/README.md) for more details.

## Configuration

The configuration file is stored at `~/.aicommit.json`. You can edit it directly with:

```bash
aicommit --config
```

### Global Configuration

The configuration file supports the following global settings:

```json
{
  "providers": [...],
  "active_provider": "provider-id",
  "retry_attempts": 3  // Number of attempts to generate commit message if provider fails
}
```

- `retry_attempts`: Number of retry attempts if provider fails (default: 3)
  - Waits 5 seconds between attempts
  - Shows informative messages about retry progress
  - Can be adjusted based on your needs (e.g., set to 5 for less stable providers)

### Provider Configuration

Each provider can be configured with the following settings:

- `max_tokens`: Maximum number of tokens in the response (default: 200)
- `temperature`: Controls randomness in the response (0.0-1.0, default: 0.3)

Example configuration with all options:
```json
{
  "providers": [{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "provider": "openrouter",
    "api_key": "sk-or-v1-...",
    "model": "mistralai/mistral-tiny",
    "max_tokens": 200,
    "temperature": 0.3
  }],
  "active_provider": "550e8400-e29b-41d4-a716-446655440000",
  "retry_attempts": 3
}
```

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

#### Recommended Providers through OpenRouter

- 🌟 **Google AI Studio** - 1000000 tokens for free
  - "google/gemini-2.0-flash-exp:free"
- 🌟 **DeepSeek**
  - "deepseek/deepseek-chat"


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

### OpenAI-compatible API

You can use any service that provides an OpenAI-compatible API endpoint.

#### Example: DeepGPTBot

For example, you can use DeepGPTBot's OpenAI-compatible API for generating commit messages. Here's how to set it up:

1. Get your API key from Telegram:
   - Open [@DeepGPTBot](https://t.me/DeepGPTBot) in Telegram
   - Use the `/api` command to get your API key

2. Configure aicommit (choose one method):

   Interactive mode:
   ```bash
   aicommit --add-provider
   ```
   Select "OpenAI Compatible" and enter:
   - API Key: Your key from @DeepGPTBot
   - API URL: https://api.deep-foundation.tech/v1/chat/completions
   - Model: gpt-4o-mini
   - Max tokens: 50 (default)
   - Temperature: 0.3 (default)

   Non-interactive mode:
   ```bash
   aicommit --add-provider --add-openai-compatible \
     --openai-compatible-api-key "your-api-key" \
     --openai-compatible-api-url "https://api.deep-foundation.tech/v1/chat/completions" \
     --openai-compatible-model "gpt-4o-mini"
   ```

3. Start using it:
   ```bash
   aicommit
   ```

#### Example: LM Studio

LM Studio runs a local server that is OpenAI-compatible. Here's how to configure `aicommit` to use it:

1.  **Start LM Studio**: Launch the LM Studio application.
2.  **Load a Model**: Select and load the model you want to use (e.g., Llama 3, Mistral).
3.  **Start the Server**: Navigate to the "Local Server" tab (usually represented by `<->`) and click "Start Server".
![How turn on server](./docs/telegram-cloud-photo-size-2-5202061790916241349-y.jpg)
4.  **Note the URL**: LM Studio will display the server URL, typically `http://localhost:1234/v1/chat/completions`.
5.  **Configure aicommit** (choose one method):

    **Interactive mode:**
    ```bash
    aicommit --add-provider
    ```
    Select "OpenAI Compatible" and enter:
    - API Key: `lm-studio` (or any non-empty string, as it's often ignored by the local server)
    - API URL: `http://localhost:1234/v1/chat/completions` (or the URL shown in LM Studio)
    - Model: `lm-studio-model` (or any descriptive name; the actual model used is determined by what's loaded in LM Studio)
    - Max tokens: 50 (or adjust as needed)
    - Temperature: 0.3 (or adjust as needed)

    **Important**: The `Model` field here is just a label for `aicommit`. The actual LLM used (e.g., `llama-3.2-1b-instruct`) is determined by the model you have loaded and selected within the LM Studio application's server tab.

    **Non-interactive mode:**
    ```bash
    aicommit --add-provider --add-openai-compatible \
      --openai-compatible-api-key "lm-studio" \
      --openai-compatible-api-url "http://localhost:1234/v1/chat/completions" \
      --openai-compatible-model "mlx-community/Llama-3.2-1B-Instruct-4bit"
    ```

6.  **Select the Provider**: If this isn't your only provider, make sure it's active using `aicommit --set <provider-id>`. You can find the ID using `aicommit --list`.
7.  **Start using it**:
    ```bash
    aicommit
    ```

    Keep the LM Studio server running while using `aicommit`.

## Upcoming Features
- ⏳ Hooks for Git systems (pre-commit, post-commit)
- ⏳ Support for more LLM providers
- ⏳ Integration with IDEs and editors
- ⏳ EasyCode: VS Code integration for commit message generation directly from editor
- ⏳ Command history and reuse of previous messages
- ⏳ Message templates and customization options

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

### Staging Changes

By default, aicommit will only commit changes that have been staged using `git add`. To automatically stage all changes before committing, use the `--add` flag:

```bash
# Only commit previously staged changes
aicommit

# Automatically stage and commit all changes
aicommit --add

# Stage all changes, commit, and push (automatically sets up upstream if needed)
aicommit --add --push

# Stage all changes, pull before commit, and push after (automatically sets up upstream if needed)
aicommit --add --pull --push
```

### Automatic Upstream Branch Setup

When using `--pull` or `--push` flags, aicommit automatically handles upstream branch configuration:

- If the current branch has no upstream set:
  ```bash
  # Automatically runs git push --set-upstream origin <branch> when needed
  aicommit --push

  # Automatically sets up tracking and pulls changes
  aicommit --pull
  ```

- For new branches:
  - With `--push`: Creates the remote branch and sets up tracking
  - With `--pull`: Skips pull if remote branch doesn't exist yet
  - No manual `git push --set-upstream origin <branch>` needed

This makes working with new branches much easier, as you don't need to manually configure upstream tracking.

## Watch Mode

The watch mode allows you to automatically commit changes when files are modified. This is useful for:
- Automatic backups of your work
- Maintaining a detailed history of changes
- Not forgetting to commit your changes

### Basic Watch Mode

```bash
aicommit --watch      # Monitor files continuously and commit on changes
```

### Watch with Edit Delay

You can add a delay after the last edit before committing. This helps avoid creating commits while you're still actively editing files:

```bash
aicommit --watch --wait-for-edit 30s   # Monitor files continuously, but wait 30s after last edit before committing
```

### Time Units for wait-for-edit
- `s`: seconds
- `m`: minutes
- `h`: hours

### Additional Options
You can combine watch mode with other flags:
```bash
# Watch with auto-push
aicommit --watch --push

# Watch with version increment
aicommit --watch --add --version-file version --version-iterate

# Interactive mode with watch
aicommit --watch --dry-run
```

### Tips
- Use `--wait-for-edit` when you want to avoid partial commits
- For active editing, set longer wait times (e.g., `--wait-for-edit 1m`)
- For quick commits after small changes, don't use `--wait-for-edit`
- Use `Ctrl+C` to stop watching

## Algorithm of Operation

Below is a flowchart diagram of the aicommit program workflow:

```mermaid
flowchart TD
    A[Start aicommit] --> B{Check parameters}
    
    %% Main flags processing
    B -->|--help| C[Show help]
    B -->|--version| D[Show version]
    B -->|--add-provider| E[Add new provider]
    B -->|--list| F[List providers]
    B -->|--set| G[Set active provider]
    B -->|--config| H[Edit configuration]
    B -->|--dry-run| I[Message generation mode without commit]
    B -->|standard mode| J[Standard commit mode]
    B -->|--watch| K[File change monitoring mode]
    
    %% Provider addition
    E -->|interactive| E1[Interactive setup]
    E -->|--add-openrouter| E2[Add OpenRouter]
    E -->|--add-ollama| E3[Add Ollama]
    E -->|--add-openai-compatible| E4[Add OpenAI compatible API]
    E1 --> E5[Save configuration]
    E2 --> E5
    E3 --> E5
    E4 --> E5
    
    %% Main commit process
    J --> L[Load configuration]
    L --> M{Versioning}
    M -->|--version-iterate| M1[Update version]
    M -->|--version-cargo| M2[Update in Cargo.toml]
    M -->|--version-npm| M3[Update in package.json]
    M -->|--version-github| M4[Create GitHub tag]
    M1 --> N
    M2 --> N
    M3 --> N
    M4 --> N
    M -->|no versioning options| N[Get git diff]
    
    %% Git operations
    N -->|--add| N1[git add .]
    N1 --> O
    N -->|only staged changes| O[Generate commit message]
    
    O --> P{Success?}
    P -->|Yes| Q[Create commit]
    P -->|No| P1{Retry limit reached?}
    P1 -->|Yes| P2[Generation error]
    P1 -->|No| P3[Retry after 5 sec]
    P3 --> O
    
    Q --> R{Additional operations}
    R -->|--pull| R1[Sync with remote repository]
    R -->|--push| R2[Push changes to remote]
    R1 --> S[Done]
    R2 --> S
    R -->|no additional options| S
    
    %% Improved watch mode with timer reset logic
    K --> K1[Initialize file monitoring system]
    K1 --> K2[Start monitoring for changes]
    K2 --> K3{File change detected?}
    K3 -->|Yes| K4[Log change to terminal]
    K3 -->|No| K2
    
    K4 --> K5{--wait-for-edit specified?}
    K5 -->|No| K7[git add changed file]
    K5 -->|Yes| K6[Check if file is already in waiting list]
    
    K6 --> K6A{File in waiting list?}
    K6A -->|Yes| K6B[Reset timer for this file]
    K6A -->|No| K6C[Add file to waiting list with current timestamp]
    
    K6B --> K2
    K6C --> K2
    
    %% Parallel process for waiting list with timer reset logic
    K1 --> K8[Check waiting list every second]
    K8 --> K9{Any files in waiting list?}
    K9 -->|No| K8
    K9 -->|Yes| K10[For each file in waiting list]
    
    K10 --> K11{Time since last modification >= wait-for-edit time?}
    K11 -->|No| K8
    K11 -->|Yes| K12[git add stable files]
    
    K12 --> K13[Start commit process]
    K13 --> K14[Remove committed files from waiting list]
    K14 --> K8
    
    K7 --> K13
    
    %% Dry run
    I --> I1[Load configuration]
    I1 --> I2[Get git diff]
    I2 --> I3[Generate commit message]
    I3 --> I4[Display result without creating commit]
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
