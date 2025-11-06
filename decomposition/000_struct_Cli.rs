#[derive(Parser, Debug)]
#[command(name = "aicommit")]
#[command(about = "A CLI tool that generates concise and descriptive git commit messages using LLMs", long_about = None)]
#[command(disable_help_flag = true)]
#[command(bin_name = "aicommit")]
struct Cli {
    /// Add a new provider (interactive mode)
    #[arg(long = "add-provider")]
    add_provider: bool,

    /// Automatically stage all changes before commit
    #[arg(long = "add")]
    add: bool,

    /// Add OpenRouter provider non-interactively
    #[arg(long)]
    add_openrouter: bool,

    /// OpenRouter API key
    #[arg(long)]
    openrouter_api_key: Option<String>,

    /// OpenRouter model name
    #[arg(long, default_value = "mistralai/mistral-tiny")]
    openrouter_model: String,

    /// Add Simple Free OpenRouter provider (uses best available free models automatically)
    #[arg(long)]
    add_simple_free: bool,

    /// Add Ollama provider non-interactively
    #[arg(long)]
    add_ollama: bool,

    /// Ollama API URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Ollama model name
    #[arg(long, default_value = "llama2")]
    ollama_model: String,

    /// Add OpenAI compatible provider non-interactively (e.g., LM Studio, custom endpoints)
    #[arg(long)]
    add_openai_compatible: bool,

    /// OpenAI compatible API key
    #[arg(long)]
    openai_compatible_api_key: Option<String>,

    /// OpenAI compatible API URL
    #[arg(long)]
    openai_compatible_api_url: Option<String>,

    /// OpenAI compatible model name
    #[arg(long, default_value = "gpt-3.5-turbo")]
    openai_compatible_model: String,

    /// Max tokens for provider configuration
    #[arg(long, default_value = "200")]
    max_tokens: i32,

    /// Temperature for provider configuration
    #[arg(long, default_value = "0.2")]
    temperature: f32,

    /// List all providers
    #[arg(long)]
    list: bool,

    /// Set active provider
    #[arg(long)]
    set: Option<String>,

    /// Edit configuration file
    #[arg(long)]
    config: bool,

    /// Path to version file
    #[arg(long = "version-file")]
    version_file: Option<String>,

    /// Automatically increment version in version file
    #[arg(long = "version-iterate")]
    version_iterate: bool,

    /// Synchronize version with Cargo.toml
    #[arg(long = "version-cargo")]
    version_cargo: bool,

    /// Synchronize version with package.json
    #[arg(long = "version-npm")]
    version_npm: bool,

    /// Update version on GitHub
    #[arg(long = "version-github")]
    version_github: bool,

    /// Interactive commit message generation
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Pull changes before commit
    #[arg(long = "pull")]
    pull: bool,

    /// Watch for changes and auto-commit
    #[arg(long = "watch")]
    watch: bool,

    /// Wait for edit delay before committing (e.g. "30s" for 30 seconds)
    #[arg(long = "wait-for-edit")]
    wait_for_edit: Option<String>,

    /// Automatically push changes after commit
    #[arg(long = "push")]
    push: bool,

    /// Display help information
    #[arg(long = "help")]
    help: bool,

    /// Display version information
    #[arg(long = "version")]
    version: bool,

    /// Display verbose information
    #[arg(long = "verbose")]
    verbose: bool,
    
    /// Skip .gitignore check and creation
    #[arg(long = "no-gitignore-check")]
    no_gitignore_check: bool,

    /// Set the git commit message without using AI. (For CI/CD or offline use cases)
    #[arg(long)]
    msg: Option<String>,

    /// Force the use of offline mode (uses fallback model list) for testing purposes
    #[arg(long, hide = true)]
    simulate_offline: bool,
    
    /// Show status of all model jails and blacklists
    #[arg(long = "jail-status")]
    jail_status: bool,
    
    /// Release specific model from jail/blacklist (model ID as parameter)
    #[arg(long = "unjail")]
    unjail: Option<String>,
    
    /// Release all models from jail/blacklist
    #[arg(long = "unjail-all")]
    unjail_all: bool,
}