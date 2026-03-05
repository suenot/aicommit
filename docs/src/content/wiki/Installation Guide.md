# Installation Guide

<cite>
**Referenced Files in This Document**   
- [package.json](file://package.json)
- [index.js](file://index.js)
- [Cargo.toml](file://Cargo.toml)
- [readme.md](file://readme.md)
- [get_free_models.py](file://bin/get_free_models.py)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Prerequisites](#prerequisites)
3. [Installation Methods](#installation-methods)
4. [Post-Installation Verification](#post-installation-verification)
5. [Environment Setup and Configuration](#environment-setup-and-configuration)
6. [Common Installation Issues and Troubleshooting](#common-installation-issues-and-troubleshooting)
7. [Conclusion](#conclusion)

## Introduction
This guide provides comprehensive instructions for installing `aicommit`, a CLI tool that generates concise and descriptive git commit messages using Large Language Models (LLMs). The document covers all supported installation methods including npm, Cargo, and direct binary usage. It also details prerequisite software, platform-specific considerations, environment configuration, and troubleshooting guidance for common issues encountered during setup.

The `aicommit` tool supports multiple platforms (macOS, Linux, Windows) and architectures (x64, arm64), with different installation approaches available depending on your development environment and preferences. Whether you're using Node.js, Rust, or prefer standalone binaries, this guide will help you get set up correctly.

**Section sources**
- [readme.md](file://readme.md#L0-L734)

## Prerequisites
Before installing `aicommit`, ensure your system meets the necessary prerequisites based on your chosen installation method.

For **npm installation**, you must have:
- Node.js (version 14 or higher recommended)
- npm (Node Package Manager) - typically included with Node.js installations

For **Cargo installation**, you need:
- Rust programming language toolchain
- Cargo package manager (installed automatically with Rust via rustup)

For **utility scripts** included with the package:
- Python 3.6+ (required to run the `get_free_models.py` script for fetching OpenRouter model information)

All installation methods require:
- Git (for version control operations)
- Internet connectivity (for initial installation and API access)

The application itself can interface with various LLM providers, which may have additional requirements:
- For cloud-based providers like OpenRouter: Valid API key
- For local models via Ollama: Ollama server running locally
- For OpenAI-compatible endpoints: Access to compatible service (e.g., LM Studio, DeepGPTBot)

**Section sources**
- [readme.md](file://readme.md#L0-L734)
- [package.json](file://package.json#L0-L57)
- [get_free_models.py](file://bin/get_free_models.py#L0-L161)

## Installation Methods
### npm Installation
The primary installation method for `aicommit` is through npm (Node Package Manager). This approach leverages the Node.js ecosystem and provides cross-platform compatibility.

To install globally using npm:
```bash
npm install -g @suenot/aicommit
```

This command installs the package globally, making the `aicommit` command available system-wide. The installation process automatically handles executable permissions through the postinstall script defined in package.json (`"postinstall": "chmod +x index.js"`).

The npm package uses a JavaScript wrapper (`index.js`) that determines the appropriate pre-compiled binary for your platform and architecture, then executes it with the provided arguments. This allows distribution of a single npm package that works across different operating systems.

**Section sources**
- [package.json](file://package.json#L0-L57)
- [index.js](file://index.js#L0-L70)
- [readme.md](file://readme.md#L0-L734)

### Building from Source with Cargo
For Rust developers or those who prefer compiling from source, `aicommit` can be built using Cargo, Rust's package manager and build system.

First, ensure you have the Rust toolchain installed by following the official installation guide at https://rust-lang.org. Then execute:
```bash
cargo install aicommit
```

Alternatively, if you've cloned the repository, you can build directly from the source directory:
```bash
git clone https://github.com/suenot/aicommit.git
cd aicommit
cargo install --path .
```

Building from source compiles the Rust binary (`src/main.rs`) natively for your system, potentially offering performance benefits and allowing customization of build features. The resulting binary is statically linked and self-contained.

The Cargo.toml file specifies dependencies including tokio for async runtime, reqwest for HTTP requests, serde for serialization, and clap for command-line argument parsing, among others.

**Section sources**
- [Cargo.toml](file://Cargo.toml#L0-L27)
- [readme.md](file://readme.md#L0-L734)

### Direct Binary Usage
While not explicitly documented as a standalone installation method, the architecture supports direct binary usage through the npm package structure. The package includes pre-compiled binaries for different platforms stored in the `bin/` directory organized by platform-architecture combinations.

The `index.js` launcher script automatically detects your system's platform and architecture, then executes the corresponding binary. Supported combinations include:
- darwin-x64 (macOS Intel)
- darwin-arm64 (macOS Apple Silicon)
- linux-x64 (Linux x86_64)
- win32-x64 (Windows 64-bit)

Linux builds are currently only available for x64 architecture. The launcher script validates the existence of the appropriate binary before execution and throws an error if unsupported platform or architecture is detected.

**Section sources**
- [index.js](file://index.js#L8-L43)
- [package.json](file://package.json#L0-L57)

## Post-Installation Verification
After installation, verify that `aicommit` is properly installed and accessible from your command line.

Run the version command to confirm successful installation:
```bash
aicommit --version
```

This should display the current version number (e.g., 0.1.139). If you encounter a "command not found" error, check your system's PATH environment variable to ensure the installation directory is included.

You can also test basic functionality without making actual commits:
```bash
aicommit --help
```

This displays the help menu with available commands and options, confirming that the executable is working correctly.

For npm installations specifically, you can verify the package was installed globally by checking:
```bash
npm list -g @suenot/aicommit
```

Additionally, test that the binary execution mechanism works by attempting to run the command in your repository with staged changes:
```bash
git add .
aicommit --dry-run
```

The `--dry-run` flag shows what commit message would be generated without actually creating a commit, allowing safe verification of the complete workflow.

**Section sources**
- [readme.md](file://readme.md#L0-L734)
- [index.js](file://index.js#L0-L70)

## Environment Setup and Configuration
### Executable Linking Mechanism
The npm package configuration in `package.json` maps the `aicommit` command to the `index.js` file through the bin field:
```json
"bin": {
  "aicommit": "./index.js"
}
```

When npm installs the package globally, it creates a symbolic link (or Windows equivalent) from the global bin directory to the `index.js` file. On Unix-like systems, this typically creates `/usr/local/bin/aicommit` pointing to the installed package's `index.js`. The shebang `#!/usr/bin/env node` at the top of `index.js` ensures it's executed with Node.js.

npm automatically handles the executable permissions during installation, but the package includes an explicit `postinstall` script (`"postinstall": "chmod +x index.js"`) to ensure the script has execute permissions across different environments.

### Provider Configuration
Before using `aicommit` effectively, you need to configure at least one LLM provider. The configuration file is stored at `~/.aicommit.json`.

Set up a provider interactively:
```bash
aicommit --add-provider
```

Or configure non-interactively. For example, to set up Simple Free OpenRouter mode:
```bash
aicommit --add-simple-free --openrouter-api-key=<YOUR_API_KEY>
```

Available provider types include:
- Simple Free OpenRouter (automatically selects best free models)
- OpenRouter (specific model selection)
- Ollama (local LLM server)
- OpenAI-compatible endpoints (LM Studio, DeepGPTBot, etc.)

### Utility Scripts
The package includes utility scripts that require additional setup:

The `get_free_models.py` script fetches information about available free models from OpenRouter:
```bash
python bin/get_free_models.py
```

This script requires Python 3.6+ and reads your existing `~/.aicommit.json` configuration to extract the OpenRouter API key. It saves results to the `openrouter_models/` directory in JSON and text formats, providing visibility into available free model options.

**Section sources**
- [package.json](file://package.json#L0-L57)
- [index.js](file://index.js#L0-L70)
- [readme.md](file://readme.md#L0-L734)
- [get_free_models.py](file://bin/get_free_models.py#L0-L161)

## Common Installation Issues and Troubleshooting
### Permission Errors
On Unix-like systems (macOS, Linux), permission errors may occur during global npm installation. Resolve this by:
- Using a Node.js version manager like nvm, fnm, or Volta that installs packages in user directories
- Fixing npm permissions by changing the default directory
- Running npm with sudo (not recommended for security reasons)

If encountering "EACCES" errors:
```bash
# Check npm's global prefix
npm config get prefix

# Ensure you have write permissions to this directory
# Or use nvm to manage Node.js versions in your home directory
```

### PATH Configuration Issues
If the `aicommit` command is not recognized after installation, your PATH likely doesn't include npm's global bin directory.

Find npm's global bin directory:
```bash
npm bin -g
```

Add this directory to your shell profile (`.zshrc`, `.bashrc`, etc.):
```bash
export PATH="$PATH:$(npm bin -g)"
```

Reload your shell configuration or restart your terminal.

### Dependency Conflicts
For npm installations, ensure you're using a compatible Node.js version. The tool has been tested with Node.js 14+.

For Cargo installations, ensure your Rust toolchain is up to date:
```bash
rustup update
```

### Platform-Specific Issues
**Windows**: Some users may encounter issues with file associations or execution policies. Ensure PowerShell execution policy allows script execution:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Apple Silicon (M1/M2)**: The package includes native arm64 binaries for macOS. If experiencing issues, ensure you're not running in Rosetta emulation mode unnecessarily.

**Linux**: Only x64 architecture is supported. ARM-based Linux systems (like Raspberry Pi) are not currently supported.

### Network and API Issues
The `get_free_models.py` script requires internet connectivity and a valid OpenRouter API key in your configuration. If this script fails:
- Verify your internet connection
- Ensure `~/.aicommit.json` exists and contains a valid OpenRouter API key
- Check that your API key has sufficient permissions

### Verification Steps
If installation appears successful but the tool doesn't work:
1. Confirm the binary exists in the expected location
2. Check that the index.js has execute permissions
3. Verify network connectivity for API-dependent features
4. Test with `--help` and `--version` flags
5. Examine any error messages carefully

**Section sources**
- [readme.md](file://readme.md#L0-L734)
- [index.js](file://index.js#L0-L70)
- [get_free_models.py](file://bin/get_free_models.py#L0-L161)

## Conclusion
This installation guide has covered all supported methods for setting up `aicommit`, including npm installation, building from source with Cargo, and understanding the underlying binary execution mechanism. We've detailed the prerequisites, step-by-step installation procedures for different platforms, post-installation verification steps, and comprehensive troubleshooting guidance for common issues.

The tool's flexible installation options accommodate various development environments and preferences, whether you're primarily a JavaScript/Node.js developer using npm, a Rust developer preferring Cargo, or someone who needs to understand the underlying execution model for debugging purposes.

With `aicommit` successfully installed and configured, you can now leverage AI-powered commit message generation to improve your Git workflow, ensuring consistent, descriptive, and meaningful commit messages across your projects.