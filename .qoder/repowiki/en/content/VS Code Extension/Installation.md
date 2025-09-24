# Installation

<cite>
**Referenced Files in This Document **   
- [package.json](file://vscode-extension/package.json)
- [build.sh](file://vscode-extension/build.sh)
- [README.md](file://vscode-extension/README.md)
- [extension.js](file://vscode-extension/extension.js)
- [index.js](file://index.js)
</cite>

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Installation from Visual Studio Code Marketplace](#installation-from-visual-studio-code-marketplace)
3. [Manual Build and Installation from Source](#manual-build-and-installation-from-source)
4. [Extension Configuration and Settings](#extension-configuration-and-settings)
5. [Verification of Installation](#verification-of-installation)
6. [Platform-Specific Considerations](#platform-specific-considerations)
7. [Integration with aicommit CLI Tool](#integration-with-aicommit-cli-tool)
8. [Troubleshooting Common Issues](#troubleshooting-common-issues)

## Prerequisites

Before installing the AICommit VS Code extension, ensure that the following prerequisites are met:

- **Node.js and npm**: Required for building the extension from source. The build script checks for npm availability and installs dependencies accordingly.
- **aicommit CLI tool**: Must be installed and accessible in the system PATH. This is essential for the extension to function properly as it relies on the CLI for generating commit messages.
- **Visual Studio Code**: Version 1.60.0 or higher, as specified in the `engines` field of package.json.

The extension leverages the aicommit CLI through Node.js child process execution, requiring the binary to be correctly installed and configured.

**Section sources**
- [package.json](file://vscode-extension/package.json#L1-L82)
- [extension.js](file://vscode-extension/extension.js#L0-L127)

## Installation from Visual Studio Code Marketplace

The recommended method for installing the AICommit extension is via the Visual Studio Code Marketplace:

1. Open Visual Studio Code.
2. Navigate to the Extensions view by clicking on the Extensions icon in the Activity Bar or pressing `Ctrl+Shift+X`.
3. Search for "AICommit for VS Code" or use the publisher name "suenot".
4. Click the Install button next to the "AICommit for VS Code" extension.
5. Once installed, reload VS Code if prompted.

This method automatically handles all dependencies and ensures you receive updates when new versions are published.

**Section sources**
- [package.json](file://vscode-extension/package.json#L1-L82)
- [README.md](file://vscode-extension/README.md#L0-L88)

## Manual Build and Installation from Source

For developers who wish to build the extension from source, follow these steps:

1. Clone the repository:
```bash
git clone https://github.com/suenot/aicommit.git
cd aicommit/vscode-extension
```

2. Run the build script:
```bash
bash build.sh
```

The build.sh script performs the following operations:
- Checks for npm availability
- Installs required npm dependencies
- Installs the vsce package globally if not already present
- Packages the extension into a .vsix file
- Provides instructions for installation

3. Install the packaged extension:
```bash
code --install-extension aicommit-vscode-0.1.0.vsix
```

Alternatively, use the VS Code UI: Extensions → ... → Install from VSIX.

**Section sources**
- [build.sh](file://vscode-extension/build.sh#L0-L39)
- [package.json](file://vscode-extension/package.json#L1-L82)

## Extension Configuration and Settings

The AICommit extension provides configurable settings through VS Code's settings interface:

- `aicommit.autoStage`: When enabled, automatically stages all changes before generating a commit message
- `aicommit.providerOverride`: Allows overriding the default provider specified in the aicommit configuration

These settings can be accessed via VS Code's Settings UI (Ctrl+,) or by editing settings.json directly. The extension integrates with the SCM view, adding a "Generate Commit Message" button when a git repository is detected.

**Section sources**
- [package.json](file://vscode-extension/package.json#L1-L82)
- [extension.js](file://vscode-extension/extension.js#L0-L127)

## Verification of Installation

To verify successful installation and functionality:

1. Open a git repository in VS Code
2. Make changes to tracked files
3. Open the Source Control view (Ctrl+Shift+G)
4. Verify the presence of the sparkle icon (✨) in the source control toolbar
5. Click the "Generate Commit Message" button
6. Confirm that a commit message appears in the commit input box

If the extension fails to generate a message, check the developer console (Help → Toggle Developer Tools) for error messages related to aicommit CLI execution.

**Section sources**
- [extension.js](file://vscode-extension/extension.js#L0-L127)
- [README.md](file://vscode-extension/README.md#L0-L88)

## Platform-Specific Considerations

The extension functions across macOS, Linux, and Windows environments with minor considerations:

- **Windows**: Ensure that the aicommit binary is in the system PATH and that command-line tools are accessible from PowerShell or Command Prompt.
- **macOS and Linux**: The index.js wrapper script ensures proper execution permissions are set on the binary. Verify that the binary has execute permissions (`chmod +x`).
- **Architecture Support**: The aicommit CLI supports x64 and arm64 architectures on all platforms, with Linux builds limited to x64 only.

The extension uses Node.js child_process to execute the aicommit CLI, abstracting platform-specific command execution differences.

**Section sources**
- [index.js](file://index.js#L0-L69)
- [extension.js](file://vscode-extension/extension.js#L0-L127)

## Integration with aicommit CLI Tool

The VS Code extension is tightly integrated with the aicommit CLI tool, which must be installed separately:

```bash
cargo install aicommit
```

Or via npm:
```bash
npm install -g @suenot/aicommit
```

After installation, configure a provider:
```bash
aicommit --add-provider
```

The extension executes the CLI with the `--dry-run` flag to generate commit messages without creating actual commits. It passes the repository root as the working directory and handles the output appropriately.

**Section sources**
- [extension.js](file://vscode-extension/extension.js#L0-L127)
- [README.md](file://vscode-extension/README.md#L0-L88)

## Troubleshooting Common Issues

### Missing aicommit CLI
**Symptom**: "Command failed: aicommit --dry-run" error in developer console
**Solution**: Install the aicommit CLI and ensure it's in the system PATH

### Permission Denied Errors
**Symptom**: EACCES errors when executing aicommit
**Solution**: On Unix-like systems, ensure the binary has execute permissions:
```bash
chmod +x $(which aicommit)
```

### Build Script Failures
**Symptom**: npm or vsce commands not found during build
**Solution**: Install Node.js and npm, then install vsce globally:
```bash
npm install -g @vscode/vsce
```

### PATH Configuration Issues
Ensure that the directory containing the aicommit binary is included in your system PATH environment variable. The location varies by installation method:
- Cargo: Typically `~/.cargo/bin`
- npm: Typically `~/.npm-global/bin` or `/usr/local/bin`

**Section sources**
- [build.sh](file://vscode-extension/build.sh#L0-L39)
- [extension.js](file://vscode-extension/extension.js#L0-L127)
- [index.js](file://index.js#L0-L69)