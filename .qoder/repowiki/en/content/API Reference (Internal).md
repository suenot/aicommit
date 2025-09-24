# API Reference (Internal)

<cite>
**Referenced Files in This Document **   
- [main.rs](file://src/main.rs)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Core Data Structures](#core-data-structures)
3. [Configuration and Provider Management](#configuration-and-provider-management)
4. [Model Selection and Failure Handling](#model-selection-and-failure-handling)
5. [Commit Execution Flow](#commit-execution-flow)
6. [Watch Mode Implementation](#watch-mode-implementation)
7. [Key Functions Reference](#key-functions-reference)

## Introduction

This document provides an internal API reference for the aicommit Rust codebase, focusing on the core types and functions defined in `src/main.rs`. The tool generates descriptive git commit messages using Large Language Models (LLMs) through various providers including OpenRouter, Ollama, and OpenAI-compatible endpoints.

The system is designed with extensibility in mind, allowing developers to add new provider types and modify the commit generation workflow. Key features include automatic version management, watch mode for continuous integration, and sophisticated model selection logic for the Simple Free OpenRouter mode.

This reference targets developers who are extending or maintaining the codebase, providing detailed documentation of public structs, enums, and key functions that form the core functionality of the application.

## Core Data Structures

### Config Structure
The `Config` struct manages the global configuration state for the application, storing provider configurations and the active provider identifier.

```mermaid
classDiagram
class Config {
+Vec<ProviderConfig> providers
+String active_provider
+u32 retry_attempts
+load() Result<Self, String>
+new() Self
+check_gitignore() Result<(), String>
+get_default_gitignore() Result<String, String>
+setup_interactive() Result<Self, String>
+setup_non_interactive(cli : &Cli) Result<(), String>
+edit() Result<(), String>
}
class ProviderConfig {
<<enum>>
+OpenRouter(OpenRouterConfig)
+Ollama(OllamaConfig)
+OpenAICompatible(OpenAICompatibleConfig)
+SimpleFreeOpenRouter(SimpleFreeOpenRouterConfig)
}
Config "1" *-- "0..*" ProviderConfig
```

**Diagram sources**
- [main.rs](file://src/main.rs#L189-L209)

**Section sources**
- [main.rs](file://src/main.rs#L189-L209)

### CLI Arguments Structure
The `Cli` struct uses clap derive macros to automatically generate a command-line argument parser from its field definitions. Each field corresponds to a command-line flag with appropriate attributes for help text and default values.

```mermaid
classDiagram
class Cli {
+bool add_provider
+bool add
+bool add_openrouter
+Option<String> openrouter_api_key
+String openrouter_model
+bool add_simple_free
+bool add_ollama
+String ollama_url
+String ollama_model
+bool add_openai_compatible
+Option<String> openai_compatible_api_key
+Option<String> openai_compatible_api_url
+String openai_compatible_model
+i32 max_tokens
+f32 temperature
+bool list
+Option<String> set
+bool config
+Option<String> version_file
+bool version_iterate
+bool version_cargo
+bool version_npm
+bool version_github
+bool dry_run
+bool pull
+bool watch
+Option<String> wait_for_edit
+bool push
+bool help
+bool version
+bool verbose
+bool no_gitignore_check
+Option<String> msg
+bool simulate_offline
+bool jail_status
+Option<String> unjail
+bool unjail_all
}
```

**Diagram sources**
- [main.rs](file://src/main.rs#L37-L200)

**Section sources**
- [main.rs](file://src/main.rs#L37-L200)

### Model Statistics Structure
The `ModelStats` struct tracks performance metrics for individual LLMs used in the Simple Free OpenRouter mode, enabling intelligent failover decisions based on historical success/failure patterns.

```mermaid
classDiagram
class ModelStats {
+usize success_count
+usize failure_count
+Option<DateTime<Utc>> last_success
+Option<DateTime<Utc>> last_failure
+Option<DateTime<Utc>> jail_until
+usize jail_count
+bool blacklisted
+Option<DateTime<Utc>> blacklisted_since
}
ModelStats : Default implementation
ModelStats : Serialization with chrono : : serde : : ts_seconds_option
```

**Diagram sources**
- [main.rs](file://src/main.rs#L134-L154)

**Section sources**
- [main.rs](file://src/main.rs#L134-L154)

### Usage Information Structure
The `UsageInfo` struct captures token usage and cost information from LLM API calls, providing transparency into resource consumption during commit message generation.

```mermaid
classDiagram
class UsageInfo {
+i32 input_tokens
+i32 output_tokens
+f32 total_cost
+Option<String> model_used
}
```

**Diagram sources**
- [main.rs](file://src/main.rs#L1178-L1182)

**Section sources**
- [main.rs](file://src/main.rs#L1178-L1182)

## Configuration and Provider Management

### Provider Configuration Types
The system supports multiple provider types through a sum type (`ProviderConfig`) that encapsulates configuration for different LLM services. Each provider has specific configuration requirements reflected in their respective structs.

```mermaid
classDiagram
class OpenRouterConfig {
+String id
+String provider
+String api_key
+String model
+i32 max_tokens
+f32 temperature
}
class OllamaConfig {
+String id
+String provider
+String model
+String url
+i32 max_tokens
+f32 temperature
}
class OpenAICompatibleConfig {
+String id
+String provider
+String api_key
+String api_url
+String model
+i32 max_tokens
+f32 temperature
}
class SimpleFreeOpenRouterConfig {
+String id
+String provider
+String api_key
+i32 max_tokens
+f32 temperature
+Vec<String> failed_models
+HashMap<String, ModelStats> model_stats
+Option<String> last_used_model
+DateTime<Utc> last_config_update
}
ProviderConfig <|-- OpenRouterConfig
ProviderConfig <|-- OllamaConfig
ProviderConfig <|-- OpenAICompatibleConfig
ProviderConfig <|-- SimpleFreeOpenRouterConfig
```

**Diagram sources**
- [main.rs](file://src/main.rs#L110-L187)

**Section sources**
- [main.rs](file://src/main.rs#L110-L187)

### Configuration Loading Process
The configuration system follows a hierarchical approach to loading settings, with sensible defaults when configuration files don't exist. The process involves checking for the existence of the configuration file, parsing JSON content, and falling back to default values when necessary.

```mermaid
flowchart TD
A[Start Config::load] --> B{Config file exists?}
B --> |No| C[Return new Config with defaults]
B --> |Yes| D[Read file contents]
D --> E{Parse JSON successfully?}
E --> |No| F[Return error with parse details]
E --> |Yes| G[Deserialize into Config struct]
G --> H[Return Config instance]
```

**Diagram sources**
- [main.rs](file://src/main.rs#L198-L208)

**Section sources**
- [main.rs](file://src/main.rs#L198-L208)

## Model Selection and Failure Handling

### Model Availability Decision Tree
The system implements a sophisticated decision-making process for selecting which LLM to use when multiple options are available, particularly in the Simple Free OpenRouter mode where automatic model selection occurs.

```mermaid
flowchart TD
A[Find Best Available Model] --> B{Last used model available?}
B --> |Yes| C{Is model available?}
C --> |Yes| D[Use last successful model]
C --> |No| E[Filter available models]
B --> |No| E
E --> F[Filter by availability status]
F --> G{Any active models?}
G --> |Yes| H[Select best from preferred list]
G --> |No| I{Any jailed but not blacklisted?}
I --> |Yes| J[Select least recently jailed]
I --> |No| K[Use any model as last resort]
H --> L[Return selected model]
J --> L
K --> L
D --> L
```

**Diagram sources**
- [main.rs](file://src/main.rs#L2200-L2300)

**Section sources**
- [main.rs](file://src/main.rs#L2200-L2300)

### Model Failure Consequences
When a model fails to generate a commit message, the system records this failure and may apply progressive penalties including temporary jailing and eventual blacklisting if failures persist.

```mermaid
stateDiagram-v2
[*] --> ModelOperation
ModelOperation --> FailureDetected : API call fails
FailureDetected --> UpdateStats : Increment failure count
UpdateStats --> CheckConsecutiveFailures : Has it failed 3 times consecutively?
CheckConsecutiveFailures --> |Yes| ApplyJail : Set jail_until timestamp
ApplyJail --> CheckJailCount : jail_count >= 3?
CheckJailCount --> |Yes| ApplyBlacklist : Set blacklisted = true
CheckJailCount --> |No| ReturnToPool
ApplyBlacklist --> ReturnToPool
ApplyJail --> ReturnToPool
CheckConsecutiveFailures --> |No| ReturnToPool
ReturnToPool --> [*]
```

**Diagram sources**
- [main.rs](file://src/main.rs#L3000-L3191)

**Section sources**
- [main.rs](file://src/main.rs#L3000-L3191)

## Commit Execution Flow

### Main Execution Sequence
The primary execution flow handles all command-line arguments and routes to the appropriate functionality based on the provided flags, forming the central control structure of the application.

```mermaid
sequenceDiagram
participant Main as main()
participant Args as Cli : : parse()
participant Config as Config : : load()
participant Watch as watch_and_commit()
participant Run as run_commit()
Main->>Args : Parse command line
Args-->>Main : Return Cli struct
Main->>Main : Match on arguments
alt --help flag
Main->>Main : Display help text
else --version flag
Main->>Main : Display version
else --add-provider flag
Main->>Config : setup_interactive()
else --watch flag
Main->>Watch : Start monitoring loop
else standard mode
Main->>Run : Execute commit process
end
```

**Diagram sources**
- [main.rs](file://src/main.rs#L1200-L1500)

**Section sources**
- [main.rs](file://src/main.rs#L1200-L1500)

### Standard Commit Process
The standard commit process orchestrates the sequence of operations required to generate a commit message, create the commit, and optionally perform additional operations like pulling or pushing.

```mermaid
flowchart TD
A[run_commit] --> B[Update versions if requested]
B --> C[Stage changes if needed]
C --> D[Get git diff]
D --> E{Diff empty?}
E --> |Yes| F[Return error]
E --> |No| G[Generate commit message]
G --> H{Message valid?}
H --> |No| I[Return error]
H --> |Yes| J[Create git commit]
J --> K{Pull requested?}
K --> |Yes| L[Execute git pull]
K --> |No| M{Push requested?}
L --> N
M --> |Yes| O[Execute git push]
M --> |No| N
O --> N
N --> P[Return success]
```

**Diagram sources**
- [main.rs](file://src/main.rs#L2000-L2500)

**Section sources**
- [main.rs](file://src/main.rs#L2000-L2500)

## Watch Mode Implementation

### File Monitoring Architecture
The watch mode implements a continuous monitoring system that detects file changes and can automatically stage and commit them after a configurable delay period.

```mermaid
classDiagram
class FileWatcher {
+HashMap<String, Instant> waiting_files
+HashMap<String, String> file_hashes
+watch_and_commit(config : &Config, cli : &Cli) Result<(), String>
}
class FileChangeDetector {
+Command : : new("sh") with git ls-files -m -o
+Process modified files
+Check real content changes via git hash-object
}
class TimerManager {
+Duration wait_for_edit
+Check elapsed time since last modification
+Reset timer on subsequent edits
}
FileWatcher --> FileChangeDetector : Uses
FileWatcher --> TimerManager : Uses
```

**Diagram sources**
- [main.rs](file://src/main.rs#L1200-L1500)

**Section sources**
- [main.rs](file://src/main.rs#L1200-L1500)

### Edit Delay Logic
The edit delay feature prevents premature commits during active editing sessions by requiring a period of stability before committing changes.

```mermaid
flowchart TD
A[File Change Detected] --> B{Wait-for-edit specified?}
B --> |No| C[Immediate staging]
B --> |Yes| D{File in waiting list?}
D --> |Yes| E[Reset timer]
D --> |No| F[Add to waiting list]
E --> G[Continue monitoring]
F --> G
G --> H{Timer expired?}
H --> |No| G
H --> |Yes| I[Stage and commit]
```

**Diagram sources**
- [main.rs](file://src/main.rs#L1200-L1500)

**Section sources**
- [main.rs](file://src/main.rs#L1200-L1500)

## Key Functions Reference

### setup_openrouter_provider Function
Initializes an OpenRouter provider configuration through interactive user input, collecting API credentials and model preferences.

**Function Signature**
```
async fn setup_openrouter_provider() -> Result<OpenRouterConfig, String>
```

**Parameters**: None (uses interactive prompts)

**Return Values**
- Success: `Ok(OpenRouterConfig)` with populated fields
- Error: `Err(String)` with descriptive error message

**Error Conditions**
- Failed to read API key input
- Failed to parse max_tokens as integer
- Failed to parse temperature as float

**Section sources**
- [main.rs](file://src/main.rs#L953-L1000)

### execute_watch_mode Function
Implements continuous file monitoring with optional delay before committing, allowing for automatic version control during development.

**Function Signature**
```
async fn watch_and_commit(config: &Config, cli: &Cli) -> Result<(), String>
```

**Parameters**
- `config`: Reference to current configuration
- `cli`: Reference to parsed command-line arguments

**Return Values**
- Success: `Ok(())` indicating ongoing monitoring
- Error: `Err(String)` with descriptive error message

**Behavior**
- Monitors for file changes using `git ls-files -m -o`
- Tracks file modification timestamps
- Implements configurable delay via `wait-for-edit`
- Automatically stages and commits stable changes
- Continues monitoring until interrupted

**Section sources**
- [main.rs](file://src/main.rs#L1200-L1500)

### handle_version_bump Function
Coordinates version updates across multiple project files, ensuring consistency between different versioning systems.

**Function Signature**
```
async fn run_commit(config: &Config, cli: &Cli) -> Result<(), String>
```

**Parameters**
- `config`: Reference to current configuration
- `cli`: Reference to parsed command-line arguments

**Return Values**
- Success: `Ok(())` after completing all requested operations
- Error: `Err(String)` with descriptive error message

**Operations Performed**
- Increments version in specified version file
- Updates Cargo.toml and Cargo.lock if requested
- Updates package.json if requested
- Creates GitHub release tag if requested
- Stages all version changes for commit

**Error Conditions**
- Version file not specified when version update flags are used
- Failed to read or write version files
- Failed to execute git commands for version updates
- Cargo update command fails

**Section sources**
- [main.rs](file://src/main.rs#L2000-L2500)

### Data Flow Between Components
The application follows a clear data flow pattern from configuration loading through provider initialization to commit execution, with well-defined interfaces between components.

```mermaid
flowchart LR
A[CLI Arguments] --> B[Config::load]
B --> C[Provider Initialization]
C --> D[Git Diff Extraction]
D --> E[LLM Request Preparation]
E --> F[Commit Message Generation]
F --> G[Git Commit Creation]
G --> H[Post-commit Operations]
subgraph Configuration
A
B
C
end
subgraph Execution
D
E
F
end
subgraph Finalization
G
H
end
```

**Diagram sources**
- [main.rs](file://src/main.rs#L1200-L1500)

**Section sources**
- [main.rs](file://src/main.rs#L1200-L1500)

### Model Failure to Jail Decision Process
The system implements a progressive penalty system for failing models, moving from temporary restrictions to permanent blacklisting based on persistent failure patterns.

```mermaid
flowchart TD
A[Model Failure Detected] --> B[Increment failure_count]
B --> C[Set last_failure timestamp]
C --> D{Consecutive failures ≥ 3?}
D --> |No| E[Continue with other models]
D --> |Yes| F[Calculate jail duration]
F --> G[Set jail_until timestamp]
G --> H[Increment jail_count]
H --> I{jail_count ≥ 3?}
I --> |Yes| J[Set blacklisted = true]
I --> |No| K[Apply temporary restriction]
J --> L[Record blacklisted_since]
K --> M[Allow retry after jail expires]
L --> M
```

**Diagram sources**
- [main.rs](file://src/main.rs#L3000-L3191)

**Section sources**
- [main.rs](file://src/main.rs#L3000-L3191)