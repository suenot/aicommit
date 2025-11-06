impl Config {
    fn new() -> Self {
        Config {
            providers: Vec::new(),
            active_provider: String::new(),
            retry_attempts: default_retry_attempts(),
        }
    }

    fn load() -> Result<Self, String> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    fn check_gitignore() -> Result<(), String> {
        if !Cli::parse().no_gitignore_check {
            // Check if .gitignore exists in current directory
            if !std::path::Path::new(".gitignore").exists() {
                // Get default gitignore content
                let default_content = Self::get_default_gitignore()?;
                fs::write(".gitignore", default_content)
                    .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
                info!("Created default .gitignore file");
            }
        }
        Ok(())
    }

    fn get_default_gitignore() -> Result<String, String> {
        let default_gitignore_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".default_gitignore");

        // If default gitignore doesn't exist, create it with default content
        if !default_gitignore_path.exists() {
            let default_content = r#"*.DS_Store
.AppleDouble
.LSOverride
._*

Thumbs.db
Thumbs.db:encryptable
ehthumbs.db
ehthumbs_vista.db
Desktop.ini
$RECYCLE.BIN/

.directory
.dropbox
.dropbox.attr

*.log
*.log.*
*.sql
*.sqlite
*.jar
*.war
*.ear
*.zip
*.tar.gz
*.rar
*.exe
*.dll
*.so
*.dylib
*.bak
*.swp
*~
*.tmp

__pycache__/
*.py[cod]
*$py.class
.Python
env/
venv/
ENV/
env.bak/
venv.bak/
.pytest_cache/
.coverage
.coverage.*
coverage.xml
*.cover

bin/
pkg/
*.test
*.prof

## Rust
target/
*.rs.bk

build/
*.o
*.obj
*.out
*.a
*.lib
*.pdb

target/
pom.xml.tag
pom.xml.releaseBackup
pom.xml.versionsBackup
dependency-reduced-pom.xml
release.properties
tomcat*/
*.class

bin/
obj/
*.user
*.suo
*.csproj.bak
*.cache
*.ilk
*.meta
*.ncx
*.nupkg

_build/
deps/
*.ez

.Rhistory
.RData
.Rproj.user/
*.Rout

node_modules/
dist/
build/
*.min.*
npm-debug.log*
yarn-debug.log*
yarn-error.log*
*.tsbuildinfo

DerivedData/
*.pbxuser
!default.pbxuser
*.mode1v3
!default.mode1v3
*.mode2v3
!default.mode2v3
*.perspectivev3
!default.perspectivev3
*.xccheckout
*.moved-aside
*.xcuserstate
*.xcworkspace
Pods/

.gradle/
build/
*.apk
*.ap_
*.aab
local.properties
*.idea/
*.iml

.idea/
*.iml
.vscode/
*.swp
*.swo
nbproject/
*.code-workspace

.env
.env.local
.env.*.local
*.cache
*.lock
*.pid
"#;
            fs::write(&default_gitignore_path, default_content)
                .map_err(|e| format!("Failed to create default gitignore template: {}", e))?;
        }

        fs::read_to_string(&default_gitignore_path)
            .map_err(|e| format!("Failed to read default gitignore template: {}", e))
    }

    async fn setup_interactive() -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());

        info!("Setting up a provider");
        let provider_options = &["Free OpenRouter (recommended)", "OpenRouter", "Ollama", "OpenAI Compatible", "Claude Code", "OpenCode"];
        let provider_selection = Select::new()
            .with_prompt("Select a provider")
            .items(provider_options)
            .default(0)
            .interact()
            .map_err(|e| format!("Failed to get provider selection: {}", e))?;

        let provider_id = Uuid::new_v4().to_string();

        match provider_selection {
            0 => {
                let api_key: String = Input::new()
                    .with_prompt("Enter OpenRouter API key")
                    .interact_text()
                    .map_err(|e| format!("Failed to get API key: {}", e))?;

                let max_tokens: String = Input::new()
                    .with_prompt("Enter max tokens")
                    .default("200".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get max tokens: {}", e))?;
                let max_tokens: i32 = max_tokens.parse()
                    .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

                let temperature: String = Input::new()
                    .with_prompt("Enter temperature")
                    .default("0.2".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get temperature: {}", e))?;
                let temperature: f32 = temperature.parse()
                    .map_err(|e| format!("Failed to parse temperature: {}", e))?;

                let simple_free_config = SimpleFreeOpenRouterConfig {
                    id: provider_id.clone(),
                    provider: "simple_free_openrouter".to_string(),
                    api_key,
                    max_tokens,
                    temperature,
                    failed_models: Vec::new(),
                    model_stats: std::collections::HashMap::new(),
                    last_used_model: None,
                    last_config_update: chrono::Utc::now(),
                };

                config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
                config.active_provider = provider_id;
            }
            1 => {
                let mut openrouter_config = setup_openrouter_provider().await?;
                openrouter_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
                config.active_provider = provider_id;
            }
            2 => {
                let url: String = Input::new()
                    .with_prompt("Enter Ollama API URL")
                    .default("http://localhost:11434".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get URL: {}", e))?;

                let model: String = Input::new()
                    .with_prompt("Enter model name")
                    .default("llama2".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get model name: {}", e))?;

                let max_tokens: String = Input::new()
                    .with_prompt("Enter max tokens")
                    .default("200".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get max tokens: {}", e))?;
                let max_tokens: i32 = max_tokens.parse()
                    .map_err(|e| format!("Failed to parse max tokens: {}", e))?;

                let temperature: String = Input::new()
                    .with_prompt("Enter temperature")
                    .default("0.3".into())
                    .interact_text()
                    .map_err(|e| format!("Failed to get temperature: {}", e))?;
                let temperature: f32 = temperature.parse()
                    .map_err(|e| format!("Failed to parse temperature: {}", e))?;

                config.providers.push(ProviderConfig::Ollama(OllamaConfig {
                    id: provider_id.clone(),
                    provider: "ollama".to_string(),
                    model,
                    url,
                    max_tokens,
                    temperature,
                }));
                config.active_provider = provider_id;
            }
            3 => {
                let mut openai_compatible_config = setup_openai_compatible_provider().await?;
                openai_compatible_config.id = provider_id.clone();
                config.providers.push(ProviderConfig::OpenAICompatible(openai_compatible_config));
                config.active_provider = provider_id;
            }
            4 => {
                config.providers.push(ProviderConfig::ClaudeCode(ClaudeCodeConfig {
                    id: provider_id.clone(),
                    provider: "claude_code".to_string(),
                }));
                config.active_provider = provider_id;
            }
            5 => {
                config.providers.push(ProviderConfig::OpenCode(OpenCodeConfig {
                    id: provider_id.clone(),
                    provider: "opencode".to_string(),
                }));
                config.active_provider = provider_id;
            }
            _ => return Err("Invalid provider selection".to_string()),
        }

        // Save the configuration
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(config)
    }

    fn edit() -> Result<(), String> {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        if !config_path.exists() {
            let default_config = Config::new();
            let content = serde_json::to_string_pretty(&default_config)
                .map_err(|e| format!("Failed to serialize default config: {}", e))?;
            fs::write(&config_path, content)
                .map_err(|e| format!("Failed to write default config: {}", e))?;
        }

        let status = Command::new(editor)
            .arg(&config_path)
            .status()
            .map_err(|e| format!("Failed to open editor: {}", e))?;

        if !status.success() {
            return Err("Editor exited with error".to_string());
        }

        Ok(())
    }

    async fn setup_non_interactive(cli: &Cli) -> Result<Self, String> {
        let mut config = Config::load().unwrap_or_else(|_| Config::new());
        let provider_id = Uuid::new_v4().to_string();

        if cli.add_openrouter {
            let api_key = cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?;

            let openrouter_config = OpenRouterConfig {
                id: provider_id.clone(),
                provider: "openrouter".to_string(),
                api_key,
                model: cli.openrouter_model.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::OpenRouter(openrouter_config));
            config.active_provider = provider_id;
        } else if cli.add_simple_free {
            let api_key = cli.openrouter_api_key.clone()
                .ok_or_else(|| "OpenRouter API key is required".to_string())?;

            let simple_free_config = SimpleFreeOpenRouterConfig {
                id: provider_id.clone(),
                provider: "simple_free_openrouter".to_string(),
                api_key,
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
                failed_models: Vec::new(),
                model_stats: std::collections::HashMap::new(),
                last_used_model: None,
                last_config_update: chrono::Utc::now(),
            };
            config.providers.push(ProviderConfig::SimpleFreeOpenRouter(simple_free_config));
            config.active_provider = provider_id;
        } else if cli.add_ollama {
            let ollama_config = OllamaConfig {
                id: provider_id.clone(),
                provider: "ollama".to_string(),
                model: cli.ollama_model.clone(),
                url: cli.ollama_url.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::Ollama(ollama_config));
            config.active_provider = provider_id;
        } else if cli.add_openai_compatible {
            let api_key = cli.openai_compatible_api_key.clone()
                .ok_or_else(|| "OpenAI compatible API key is required".to_string())?;
            let api_url = cli.openai_compatible_api_url.clone()
                .ok_or_else(|| "OpenAI compatible API URL is required".to_string())?;

            let openai_compatible_config = OpenAICompatibleConfig {
                id: provider_id.clone(),
                provider: "openai_compatible".to_string(),
                api_key,
                api_url,
                model: cli.openai_compatible_model.clone(),
                max_tokens: cli.max_tokens,
                temperature: cli.temperature,
            };
            config.providers.push(ProviderConfig::OpenAICompatible(openai_compatible_config));
            config.active_provider = provider_id;
        }

        // Save the configuration
        let config_path = dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".aicommit.json");

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(config)
    }
}