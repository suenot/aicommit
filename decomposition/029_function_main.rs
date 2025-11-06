async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    // Initialize logging system
    let mut logging_config = LoggingConfig::new();

    // Adjust logging level based on CLI flags
    if cli.verbose {
        logging_config.with_debug();
    }

    let _logging_guard = match init_logging(&logging_config) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            // Continue execution even if logging fails
            None
        }
    };

    info!("Starting aicommit version {}", get_version());

    // Check .gitignore at startup
    if !cli.no_gitignore_check {
        Config::check_gitignore()?;
    }

    match () {
        _ if cli.help => {
            // Получаем версию для отображения в справке
            let version = get_version();
            
            println!("aicommit v{} - A CLI tool that generates concise and descriptive git commit messages using LLMs", version);
            println!("\nUsage:");
            println!("  aicommit [OPTIONS]");
            println!("\nOptions:");
            println!("  --add-provider       Add a new provider (interactive mode)");
            println!("  --add                Automatically stage all changes before commit");
            println!("  --add-openrouter     Add OpenRouter provider non-interactively");
            println!("  --add-simple-free    Add Simple Free OpenRouter provider (uses best available free models)");
            println!("  --add-ollama         Add Ollama provider non-interactively");
            println!("  --add-openai-compatible Add OpenAI compatible provider non-interactively");
            println!("  --openrouter-api-key=<KEY> OpenRouter API key (required for --add-openrouter)");
            println!("  --openrouter-model=<MODEL> OpenRouter model (default: mistralai/mistral-tiny)");
            println!("  --ollama-url=<URL>    Ollama API URL (default: http://localhost:11434)");
            println!("  --ollama-model=<MODEL> Ollama model (default: llama2)");
            println!("  --openai-compatible-api-key=<KEY> OpenAI compatible API key");
            println!("  --openai-compatible-api-url=<URL> OpenAI compatible API URL");
            println!("  --openai-compatible-model=<MODEL> OpenAI compatible model (default: gpt-3.5-turbo)");
            println!("  --max-tokens=<TOKENS> Max tokens for response (default: 200)");
            println!("  --temperature=<TEMP>  Temperature for generation (default: 0.2)");
            println!("  --list                List all providers");
            println!("  --set=<ID>            Set active provider by ID");
            println!("  --config              Edit configuration file");
            println!("  --version-file=<FILE> Path to version file");
            println!("  --version-iterate     Automatically increment version in version file");
            println!("  --version-cargo       Synchronize version with Cargo.toml");
            println!("  --version-npm         Synchronize version with package.json");
            println!("  --version-github      Update version on GitHub");
            println!("  --dry-run             Interactive commit message generation");
            println!("  --pull                Pull changes before commit");
            println!("  --push                Automatically push changes after commit");
            println!("  --help                Display this help message");
            println!("  --version             Display version information");
            println!("  --verbose             Display verbose information");
            println!("  --no-gitignore-check  Skip .gitignore check and creation");
            println!("  --watch               Watch for changes and auto-commit");
            println!("  --wait-for-edit=<DURATION> Wait for edit delay before committing (e.g. \"30s\")");
            println!("  --jail-status         Show status of all model jails and blacklists");
            println!("  --unjail=<MODEL>      Release specific model from jail/blacklist (model ID as parameter)");
            println!("  --unjail-all          Release all models from jail/blacklist");
            println!("\nExamples:");
            println!("  aicommit --add-provider");
            println!("  aicommit --add");
            println!("  aicommit --add-openrouter --openrouter-api-key=<KEY>");
            println!("  aicommit --add-simple-free --openrouter-api-key=<KEY>");
            println!("  aicommit --list");
            println!("  aicommit --set=<ID>");
            println!("  aicommit --version-file=version.txt --version-iterate");
            println!("  aicommit --watch");
            println!("  aicommit");
            Ok(())
        }
        _ if cli.version => {
            println!("aicommit v{}", get_version());
            Ok(())
        }
        _ if cli.jail_status => {
            // Display jail status for all models
            let config = Config::load()?;
            
            // Find a Simple Free provider
            let mut found = false;
            
            for provider in config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    display_model_jail_status(&c)?;
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.unjail.is_some() => {
            // Unjail a specific model
            let model_id = cli.unjail.unwrap();
            let mut config = Config::load()?;
            let mut found = false;
            
            for provider in &mut config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    unjail_model(c, &model_id)?;
                    println!("Model '{}' successfully released from jail", model_id);
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.unjail_all => {
            // Unjail all models
            let mut config = Config::load()?;
            let mut found = false;
            
            for provider in &mut config.providers {
                if let ProviderConfig::SimpleFreeOpenRouter(c) = provider {
                    unjail_all_models(c)?;
                    println!("All models successfully released from jail");
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("No Simple Free OpenRouter configuration found. You can add one with 'aicommit --add-simple-free'");
            }
            
            Ok(())
        }
        _ if cli.add_provider => {
            Config::setup_interactive().await?;
            println!("Provider added successfully!");
            Ok(())
        }
        _ if cli.add_openrouter || cli.add_ollama || cli.add_openai_compatible || cli.add_simple_free => {
            Config::setup_non_interactive(&cli).await?;
            println!("Provider added successfully!");
            Ok(())
        }
        _ if cli.list => {
            // Список всех провайдеров
            let config = Config::load()?;
            for provider in config.providers {
                match provider {
                    ProviderConfig::OpenRouter(c) => println!("OpenRouter: {}", c.id),
                    ProviderConfig::Ollama(c) => println!("Ollama: {}", c.id),
                    ProviderConfig::OpenAICompatible(c) => println!("OpenAI Compatible: {}", c.id),
                    ProviderConfig::SimpleFreeOpenRouter(c) => println!("Simple Free OpenRouter: {}", c.id),
                }
            }
            Ok(())
        }
        _ if cli.set.is_some() => {
            // Установка активного провайдера
            let mut config = Config::load()?;
            let new_active_provider = cli.set.unwrap();
            let mut found = false;

            for provider in &config.providers {
                match provider {
                    ProviderConfig::OpenRouter(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::Ollama(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::OpenAICompatible(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                    ProviderConfig::SimpleFreeOpenRouter(c) => {
                        if c.id == new_active_provider {
                            config.active_provider = c.id.clone();
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                return Err(format!("Provider '{}' not found", new_active_provider));
            }

            let config_path = dirs::home_dir()
                .ok_or_else(|| "Could not find home directory".to_string())?
                .join(".aicommit.json");

            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;

            fs::write(&config_path, content)
                .map_err(|e| format!("Failed to write config file: {}", e))?;

            println!("Active provider set to {}", new_active_provider);
            Ok(())
        }
        _ if cli.config => {
            Config::edit()?;
            println!("Configuration updated.");
            Ok(())
        }
        _ if cli.dry_run => {
            // Special handling for --dry-run to provide better error messages
            match dry_run(&cli).await {
                Ok(message) => {
                    println!("{}", message);
                    Ok(())
                }
                Err(e) => {
                    // Provide more detailed error message
                    eprintln!("Error in dry-run mode: {}", e);
                    Err(e)
                }
            }
        }
        _ => {
            let config = Config::load().unwrap_or_else(|_| {
                println!("No configuration found. Run 'aicommit --add-provider' to set up a provider.");
                std::process::exit(1);
            });

            if config.active_provider.is_empty() || config.providers.is_empty() {
                println!("No active provider found. Please run 'aicommit --add-provider' to configure a provider.");
                std::process::exit(1);
            }

            // Check if we're in watch mode
            if cli.watch {
                watch_and_commit(&config, &cli).await?
            } else {
                run_commit(&config, &cli).await?;
            }
            Ok(())
        }
    }
}