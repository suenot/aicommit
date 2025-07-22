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
- ✅ Supports multiple LLM providers:
  - [OpenRouter](https://openrouter.ai/) (cloud)
  - [Simple Free OpenRouter](#simple-free-mode) (automatically uses best available free models)
  - [Ollama](https://ollama.ai/) (local)
  - OpenAI-compatible endpoints (LM Studio, local OpenAI proxy, etc.)
- ✅ Automatically stages changes with `--add` option
- ✅ Pushes commits automatically with `--push`
- ✅ Interactive mode with `--dry-run`
- ✅ Watch mode with `--watch`
- ✅ Verbose mode with `--verbose`
- ✅ Version control helpers:
  - Automatic version bumping (`--version-iterate`)
  - Cargo.toml version sync (`--version-cargo`)
  - package.json version sync (`--version-npm`)
  - GitHub version update (`--version-github`)
- ✅ Smart retry mechanism for API failures
- ✅ Easy configuration management
- ✅ VS Code extension available

## Simple Free Mode

The Simple Free mode allows you to use OpenRouter's free models without having to manually select a model. You only need to provide an OpenRouter API key, and the system will:

1. Automatically query OpenRouter for currently available free models
2. Select the best available free model based on an internally ranked list
3. Automatically switch to alternative models using an advanced failover mechanism
4. Track model performance with a sophisticated jail/blacklist system
5. Fall back to predefined free models if network connectivity is unavailable

To set up Simple Free mode:

```bash
# Interactive setup
aicommit --add-provider
# Select "Simple Free OpenRouter" from the menu

# Or non-interactive setup
aicommit --add-simple-free --openrouter-api-key=<YOUR_API_KEY>
```

### Advanced Failover Mechanism

The Simple Free mode uses a sophisticated failover mechanism to ensure optimal model selection:

- **Three-Tier Model Status**: Models are categorized as `Active`, `Jailed` (temporary restriction), or `Blacklisted` (long-term ban).
- **Counter-Based System**: Tracks success/failure ratio for each model; 3 consecutive failures move a model to `Jailed` status.
- **Time-Based Jail**: Models are temporarily jailed for 24 hours after repeated failures, with increasing jail time for recidivism.
- **Blacklist Management**: Models with persistent failures over multiple days are blacklisted but retried weekly.
- **Success Rate Tracking**: Records performance history to prioritize more reliable models.
- **Smart Reset**: Models get fresh chances daily, and users can manually reset with `--unjail` and `--unjail-all` commands.
- **Network Error Handling**: Distinguishes between model errors and connection issues to avoid unfair penalties.

Model management commands:
```bash
# Show status of all model jails/blacklists
aicommit --jail-status

# Release specific model from restrictions
aicommit --unjail <model-id>

# Release all models from restrictions
aicommit --unjail-all
```

### Benefits of Simple Free Mode

- **Zero Cost**: Uses only free models from OpenRouter
- **Automatic Selection**: No need to manually choose the best free model
- **Resilient Operation**: If one model fails, it automatically switches to the next best model
- **Advanced Failover**: Uses a sophisticated system to track model performance over time
- **Learning Algorithm**: Adapts to changing model reliability by tracking success rates
- **Self-Healing**: Automatically retries previously failed models after a cooling-off period
- **Network Resilience**: Works even when network connectivity to OpenRouter is unavailable by using predefined models
- **Always Up-to-Date**: Checks for currently available free models each time
- **Best Quality First**: Uses a predefined ranking of models, prioritizing the most powerful ones
- **Future-Proof**: Intelligently handles new models by analyzing model names for parameter counts

The ranked list includes powerful models like:
- Meta's Llama 4 Maverick and Scout
- NVIDIA's Nemotron Ultra models (253B parameters)
- Qwen's massive 235B parameter models 
- Many large models from the 70B+ parameter family
- And dozens of other high-quality free options of various sizes

Even if the preferred models list becomes outdated over time, the system will intelligently identify the best available models based on their parameter size by analyzing model names (e.g., models with "70b" or "32b" in their names).

For developers who want to see all available free models, a utility script is included:

```bash
python bin/get_free_models.py
```

This script will:
- Fetch all available models from OpenRouter
- Identify which ones are free
- Save the results to JSON and text files for reference
- Display a summary of available options

## BingX Position Manager

An additional Python script for automated BingX futures position management:

```bash
python bin/setup_bingx.py    # Quick setup
python bin/bingx_monitor.py start  # Start the manager
```

Features:
- ✅ Automatically adds stop loss to positions without one
- ✅ Partial position closing at 1% profit (without leverage)
- ✅ Moves stop loss to breakeven after partial close
- ✅ Comprehensive logging and error handling
- ✅ Risk management and safety features

See `bin/QUICKSTART_bingx.md` for quick start guide or `bin/README_bingx.md` for full documentation.

## Installation

To install aicommit, use the following npm command:

```
npm install -g @suenot/aicommit
```

For Rust users, you can install using cargo:

```
cargo install aicommit
```

## Quick Start

1. **Set up a provider:**
```bash
aicommit --add-provider
```

2. **Generate a commit message:**
```bash
git add .
aicommit
```

3. **Or stage and commit in one step:**
```bash
aicommit --add
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

### Simple Free OpenRouter
```json
{
  "providers": [{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "provider": "simple_free_openrouter",
    "api_key": "sk-or-v1-...",
    "max_tokens": 50,
    "temperature": 0.3,
    "failed_models": [],
    "model_stats": {},
    "last_used_model": null,
    "last_config_update": "2023-10-15T12:00:00Z"
  }],
  "active_provider": "550e8400-e29b-41d4-a716-446655440000"
}
```

The Simple Free mode offers a hassle-free way to use OpenRouter's free models:

- **Automatic Model Selection**: No need to specify a model - the system queries OpenRouter's API for all available models and filters for free ones
- **Intelligent Ranking**: Uses an internal ranked list of preferred models (maintained in the codebase) to select the best available free model
- **Advanced Model Management**: Tracks success and failure rates for each model, with a sophisticated jail/blacklist system
- **Smart Jail System**: Models with repeated failures go to "jail" temporarily, with increasing penalties for repeat offenders
- **Fallback System**: Automatically falls back to the next best available model if the preferred one fails
- **Network Resilience**: Can operate even when your network connection to OpenRouter is unavailable by using predefined models
- **Free Usage**: Takes advantage of OpenRouter's free models that have free quotas or free access
- **Future-Proof Design**: Even when new models appear that aren't in the preferred list, the system can intelligently identify high-quality models by analyzing model names for parameter counts (e.g., models with "70b" in their name are prioritized over those with "7b")
- **Smart Model Analysis**: Uses a sophisticated algorithm to extract parameter counts from model names and prioritize larger models when none of the preferred models are available

This approach ensures that your `aicommit` installation will continue to work effectively even years later, as it can adapt to the changing landscape of available free models on OpenRouter.

This is the recommended option for most users who want to use aicommit without worrying about model selection or costs.

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
    B -->|--simulate-offline| Offline[Simulate offline mode]
    B -->|--jail-status| JailStatus[Display model jail status]
    B -->|--unjail| Unjail[Release specific model]
    B -->|--unjail-all| UnjailAll[Release all models]
    
    %% Provider addition
    E -->|interactive| E1[Interactive setup]
    E -->|--add-openrouter| E2[Add OpenRouter]
    E -->|--add-ollama| E3[Add Ollama]
    E -->|--add-openai-compatible| E4[Add OpenAI compatible API]
    E -->|--add-simple-free| E_Free[Add Simple Free OpenRouter]
    E1 --> E5[Save configuration]
    E2 --> E5
    E3 --> E5
    E4 --> E5
    E_Free --> E5
    
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
    N1 --> N_Truncate["Smart diff processing (truncate large files only)"]
    N -->|only staged changes| N_Truncate["Smart diff processing (truncate large files only)"]
    N_Truncate --> O["Generate commit message (using refined prompt)"]
    
    %% Simple Free OpenRouter branch
    O -->|Simple Free OpenRouter| SF1["Query OpenRouter API for available free models"]
    SF1 --> SF_Network{Network available?}
    SF_Network -->|Yes| SF2["Filter for free models"]
    SF_Network -->|No| SF3["Use fallback predefined free models list"]
    SF2 --> SF4["Advanced Model Selection"]
    SF3 --> SF4
    
    %% Advanced Model Selection subgraph
    SF4 --> SF_Last{Last successful model available?}
    SF_Last -->|Yes| SF_LastJailed{Is model jailed or blacklisted?}
    SF_Last -->|No| SF_Sort["Sort by model capabilities"]
    SF_LastJailed -->|Yes| SF_Sort
    SF_LastJailed -->|No| SF_UseLastModel["Use last successful model"]
    
    SF_Sort --> SF_Active{Any active models available?}
    SF_Active -->|Yes| SF_SelectBest["Select best active model"]
    SF_Active -->|No| SF_Jailed{Any jailed models (not blacklisted)?}
    
    SF_Jailed -->|Yes| SF_SelectJailed["Select least recently jailed model"]
    SF_Jailed -->|No| SF_Desperate["Use any model as last resort"]
    
    SF_UseLastModel --> SF_Use["Use selected model"]
    SF_SelectBest --> SF_Use
    SF_SelectJailed --> SF_Use
    SF_Desperate --> SF_Use
    
    SF_Use --> SF6["Generate commit using selected model"]
    SF6 --> SF_Success{Model worked?}
    SF_Success -->|Yes| SF_RecordSuccess["Record success & update model stats"]
    SF_Success -->|No| SF_RecordFailure["Record failure & potentially jail model"]
    
    SF_RecordSuccess --> SF7["Display which model was used"]
    SF_RecordFailure --> SF_Retry{Retry attempt limit reached?}
    SF_Retry -->|No| SF4
    SF_Retry -->|Yes| SF_Fail["Display error and exit"]
    
    %% Normal provider branch
    O -->|Other providers| P{Success?}
    
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
    
    K12 --> K13["Start commit process (includes smart diff processing & message generation)"]
    K13 --> K14[Remove committed files from waiting list]
    K14 --> K8
    
    K7 --> K13
    
    %% Dry run
    I --> I1[Load configuration]
    I1 --> I2[Get git diff]
    I2 --> I3_Truncate["Smart diff processing (truncate large files only)"]
    I3_Truncate --> I3["Generate commit message (using refined prompt)"]
    I3 --> I4[Display result without creating commit]
    
    %% Offline mode simulation
    Offline --> Offline1[Skip network API calls]
    Offline1 --> Offline2[Use predefined model list]
    Offline2 --> J
    
    %% Jail management commands
    JailStatus --> JailStatus1[Display all model statuses]
    Unjail --> Unjail1[Release specific model from jail/blacklist]
    UnjailAll --> UnjailAll1[Reset all models to active status]
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Advanced Model Management

To help you manage and optimize the model selection process, aicommit provides several commands for working with model jails and blacklists:

```bash
# Show current status of all models in the system
aicommit --jail-status

# Release a specific model from jail or blacklist
aicommit --unjail="meta-llama/llama-4-maverick:free"

# Release all models from jail and blacklist
aicommit --unjail-all
```

These commands can be especially useful when:
1. You want to understand why certain models aren't being selected
2. You need to manually reset a model after a temporary issue
3. You want to give blacklisted models another chance

The jail system distinguishes between network errors and model errors, and only penalizes models for their own failures, not for connectivity issues. This ensures that good models don't end up blacklisted due to temporary network problems.
