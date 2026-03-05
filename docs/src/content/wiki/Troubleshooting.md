# Troubleshooting

<cite>
**Referenced Files in This Document **   
- [main.rs](file://src/main.rs)
- [Cargo.toml](file://Cargo.toml)
- [readme.md](file://readme.md)
- [get_free_models.py](file://bin/get_free_models.py)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Authentication Failures](#authentication-failures)
3. [Connectivity Issues](#connectivity-issues)
4. [Configuration Problems](#configuration-problems)
5. [Operational Errors](#operational-errors)
6. [Model Jail Management](#model-jail-management)
7. [Platform-Specific Quirks](#platform-specific-quirks)
8. [Systematic Debugging Approach](#systematic-debugging-approach)

## Introduction
This troubleshooting guide addresses common issues encountered when using aicommit, a CLI tool that generates git commit messages using Large Language Models (LLMs). The guide is organized by problem categories including authentication failures, connectivity issues, configuration problems, and operational errors. Each section provides diagnostic steps using debug logs enabled via RUST_LOG, interpretation of error messages from src/main.rs, and verified solutions. Special attention is given to model jail status checking, clearing blacklisted models, and platform-specific quirks.

**Section sources**
- [main.rs](file://src/main.rs#L0-L3191)
- [readme.md](file://readme.md#L0-L734)

## Authentication Failures
Authentication failures in aicommit typically occur with invalid API keys or expired tokens when using LLM providers like OpenRouter. These issues prevent the tool from accessing the required AI services for generating commit messages.

To diagnose authentication issues, enable debug logging by setting the RUST_LOG environment variable:
```bash
RUST_LOG=debug aicommit --add-provider
```

Common error messages include "API request failed: 401 Unauthorized" which indicates an invalid API key. In src/main.rs, this is handled in the generate_openrouter_commit_message function where HTTP status codes are checked after API requests. For OpenRouter integration, ensure your API key has proper permissions and hasn't been revoked.

For Simple Free OpenRouter mode, verify your API key by testing it directly with the OpenRouter API:
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" https://openrouter.ai/api/v1/models
```

If you're using OpenAI-compatible endpoints like LM Studio, note that some local servers may accept any non-empty API key. Check your provider's documentation for specific authentication requirements.

The most common cause of authentication failure is incorrect API key entry during provider setup. Always copy-paste API keys rather than typing them manually to avoid typos. If issues persist, regenerate your API key from the provider's dashboard and reconfigure aicommit.

**Section sources**
- [main.rs](file://src/main.rs#L1100-L1150)
- [main.rs](file://src/main.rs#L2450-L2500)
- [readme.md](file://readme.md#L150-L200)

## Connectivity Issues
Connectivity issues in aicommit manifest as timeout errors or DNS resolution failures when communicating with LLM providers. These problems can stem from network restrictions, firewall settings, or temporary outages of the AI service endpoints.

Enable detailed network diagnostics by setting verbose logging:
```bash
RUST_LOG=trace aicommit --verbose
```

In src/main.rs, connectivity is managed through reqwest client configurations with explicit timeouts set to 10 seconds for regular requests and 15 seconds for model listing operations. When these timeouts are exceeded, you'll see errors like "Request timed out after 30 seconds" in the console output.

For OpenRouter connections, verify basic connectivity:
```bash
curl -v https://openrouter.ai/api/v1/models
```

If you encounter DNS resolution issues, try using alternative DNS servers like Google's 8.8.8.8 or Cloudflare's 1.1.1.1. Corporate networks often restrict outbound connections to specific domains; ensure that openrouter.ai, api.deep-foundation.tech, and other LLM provider domains are whitelisted.

The tool includes a sophisticated failover mechanism that automatically switches to predefined free models when network connectivity fails. This behavior is controlled by the get_available_free_models function in main.rs, which first attempts to fetch current models from OpenRouter API and falls back to a hardcoded list of preferred free models if the network request fails.

For local providers like Ollama running on http://localhost:11434, ensure the service is running and accessible:
```bash
curl http://localhost:11434/api/tags
```

If behind a proxy, configure your environment variables:
```bash
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=https://proxy.company.com:8080
```

**Section sources**
- [main.rs](file://src/main.rs#L1800-L1900)
- [main.rs](file://src/main.rs#L2600-L2650)
- [readme.md](file://readme.md#L300-L350)

## Configuration Problems
Configuration problems in aicommit typically involve malformed JSON in the configuration file (~/.aicommit.json) or incorrect paths specified in command-line arguments. These issues prevent the tool from loading provider settings or accessing necessary files.

To diagnose configuration issues, use the built-in config editor:
```bash
aicommit --config
```

This opens the configuration file in your default editor (determined by the EDITOR environment variable). The expected JSON structure includes providers array, active_provider ID, and retry_attempts count. A valid configuration follows this schema:
```json
{
  "providers": [{
    "id": "uuid-here",
    "provider": "openrouter",
    "api_key": "your-key",
    "model": "mistralai/mistral-tiny"
  }],
  "active_provider": "uuid-here",
  "retry_attempts": 3
}
```

Malformed JSON errors appear as "Failed to parse config file" in the console output. Common mistakes include trailing commas, unquoted keys, or mismatched brackets. Use a JSON validator to check syntax before saving.

For path-related issues, ensure that version files specified with --version-file exist and are readable:
```bash
aicommit --version-file ./version.txt --version-iterate
```

Relative paths are resolved relative to the current working directory. On Windows systems, use forward slashes or escaped backslashes in paths to avoid parsing issues.

The configuration system automatically creates a default .gitignore file if none exists, but this can be skipped with the --no-gitignore-check flag. If you encounter permission errors when writing configuration files, verify write access to your home directory.

Use the list command to verify your configuration:
```bash
aicommit --list
```

This displays all configured providers and helps confirm that your changes were saved correctly.

**Section sources**
- [main.rs](file://src/main.rs#L500-L600)
- [main.rs](file://src/main.rs#L800-L900)
- [readme.md](file://readme.md#L238-L287)

## Operational Errors
Operational errors in aicommit include empty commits and failed pushes, which typically occur due to misconfigured workflows or repository state issues. These problems prevent successful completion of the commit process despite successful AI message generation.

Empty commit errors appear as "No changes to commit" when running aicommit. This occurs when there are no staged changes in the Git repository. By default, aicommit only commits staged changes. To automatically stage all changes, use the --add flag:
```bash
aicommit --add
```

In src/main.rs, the get_git_diff function checks for both staged and unstaged changes. When --add is specified, it executes git add . before attempting to generate the commit message.

Failed push operations often result from missing upstream branch configuration. aicommit automatically handles upstream setup by checking if the current branch has a tracking relationship. If not, it uses git push --set-upstream origin <branch> to establish the connection. You can observe this process in verbose mode:
```bash
aicommit --push --verbose
```

Watch mode issues may occur when file monitoring doesn't trigger commits. This is often due to filesystem event limitations on certain platforms. The watch_and_commit function in main.rs uses polling every 500ms as a fallback when filesystem watchers aren't available.

For version management operations (--version-iterate, --version-cargo, etc.), ensure the specified version files exist and are properly formatted. The increment_version function expects semantic versioning format (e.g., "1.0.0").

When using dry-run mode, remember that no actual commits are created:
```bash
aicommit --dry-run
```

This only displays the generated message without creating a commit, which might be mistaken for a failure if expected to modify the repository.

**Section sources**
- [main.rs](file://src/main.rs#L3000-L3100)
- [main.rs](file://src/main.rs#L1600-L1700)
- [readme.md](file://readme.md#L500-L550)

## Model Jail Management
Model jail management in aicommit refers to the system that tracks and manages the reliability of different LLMs used for commit message generation. Models can be temporarily jailed or permanently blacklisted based on their performance history.

Check model status using:
```bash
aicommit --jail-status
```

This displays the current status of all models, showing whether they are ACTIVE, JAILED, or BLACKLISTED. The information comes from the model_stats field in the SimpleFreeOpenRouterConfig struct defined in src/main.rs.

Models are automatically jailed after three consecutive failures, with increasing jail times for repeat offenders. The jail duration starts at 24 hours and doubles with each subsequent offense, up to a maximum of 168 hours (7 days). This logic is implemented in the record_model_failure function.

To release a specific model from jail:
```bash
aicommit --unjail "meta-llama/llama-4-maverick:free"
```

To clear all model restrictions:
```bash
aicommit --unjail-all
```

The system distinguishes between network errors and model-specific errors to avoid unfairly penalizing reliable models during temporary connectivity issues. Network timeouts don't count toward the failure threshold if the model had recent successes.

Blacklisting occurs after a model has been jailed three times, indicating persistent unreliability. Blacklisted models are retried weekly to allow for recovery if the underlying issues have been resolved.

You can also use the get_free_models.py script to verify which models are currently available on OpenRouter:
```bash
python bin/get_free_models.py
```

This updates the local cache of free models and can help identify if a preferred model has become unavailable.

**Section sources**
- [main.rs](file://src/main.rs#L2000-L2200)
- [main.rs](file://src/main.rs#L2800-L2900)
- [get_free_models.py](file://bin/get_free_models.py#L0-L161)

## Platform-Specific Quirks
Platform-specific quirks in aicommit include Windows path handling issues and macOS security prompts that can interfere with normal operation. These differences stem from variations in filesystem behavior, security policies, and shell environments across operating systems.

On Windows systems, path separators in configuration files should use forward slashes (/) or escaped backslashes (\\) to avoid parsing issues. PowerShell users should be aware that command substitution works differently than in Unix shells:
```powershell
aicommit --add --push
```

Windows Defender or other antivirus software may flag the binary as suspicious during installation. Users may need to approve the application through Windows Security prompts.

On macOS, Gatekeeper may display security warnings when running newly downloaded binaries. To bypass this:
```bash
xattr -d com.apple.quarantine $(which aicommit)
```

Terminal applications on macOS may require accessibility permissions to interact with Git operations. Grant these through System Preferences > Security & Privacy > Privacy > Accessibility.

Linux users should ensure that the execute bit is set:
```bash
chmod +x $(which aicommit)
```

File permission issues are more common on Linux due to stricter umask settings. Ensure your home directory and ~/.aicommit.json have appropriate read/write permissions.

All platforms support the same core functionality, but shell-specific escaping rules apply when passing arguments containing special characters. Use single quotes on Unix systems and double quotes on Windows when specifying complex parameters.

The tool uses cross-platform compatible commands (sh -c) for Git operations to maintain consistency across different operating systems.

**Section sources**
- [main.rs](file://src/main.rs#L1600-L1650)
- [readme.md](file://readme.md#L600-L650)

## Systematic Debugging Approach
A systematic debugging approach for aicommit involves isolating problems between network, configuration, and code states through a structured diagnostic process. This method ensures comprehensive troubleshooting while minimizing unnecessary interventions.

Begin with verbose logging to capture detailed execution information:
```bash
RUST_LOG=debug aicommit --verbose
```

First, verify network connectivity independently of aicommit:
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" https://openrouter.ai/api/v1/models
```

Next, validate configuration integrity:
```bash
aicommit --config
aicommit --list
```

Check repository state:
```bash
git status
git diff --cached
```

Test basic functionality with dry-run mode:
```bash
aicommit --dry-run
```

If issues persist, follow this isolation hierarchy:

1. **Network Layer**: Verify DNS resolution, firewall rules, and API endpoint accessibility
2. **Configuration Layer**: Validate JSON syntax, API keys, and file paths
3. **Repository Layer**: Confirm Git repository state and staging area contents  
4. **Application Layer**: Test with minimal configuration and default settings

Use the --simulate-offline flag to test fallback behavior:
```bash
aicommit --simulate-offline --add
```

This bypasses network calls and uses predefined free models, helping determine if issues are network-related.

For persistent problems, reset to a known good state:
```bash
rm ~/.aicommit.json
aicommit --add-provider
```

Then reconfigure step by step, testing after each change. This incremental approach helps identify the specific configuration element causing issues.

Monitor the retry mechanism in action by intentionally using an invalid API key and observing the retry attempts with their 5-second intervals between attempts, as configured in the retry_attempts setting.

**Section sources**
- [main.rs](file://src/main.rs#L0-L3191)
- [readme.md](file://readme.md#L0-L734)