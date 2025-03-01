# AICommit for VS Code

Generate concise and descriptive git commit messages using LLMs directly in VS Code.

## Features

- Seamlessly generate AI-powered commit messages within VS Code
- Integrates with the standard Source Control view
- Uses the powerful aicommit CLI tool with support for multiple LLM providers
- Customizable configuration

## Requirements

This extension requires the [aicommit](https://github.com/suenot/aicommit) CLI tool to be installed and configured on your system.

```bash
# Install aicommit CLI
cargo install aicommit

# Configure a provider (interactive)
aicommit --add-provider
```

## Extension Settings

This extension contributes the following settings:

* `aicommit.autoStage`: Enable/disable automatically staging all changes before generating commit message
* `aicommit.providerOverride`: Override the default provider specified in aicommit configuration

## Usage

1. Open a git repository in VS Code
2. Make changes to your files
3. Open the Source Control view (Ctrl+Shift+G)
4. Click the "AICommit: Generate Commit Message" button in the source control toolbar
5. The generated message will be inserted into the commit message input box
6. Review and edit the message if needed, then commit as usual

## Building from Source

If you want to build the extension from source, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/suenot/aicommit.git
   cd aicommit/vscode-extension
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Package the extension:
   ```bash
   npm install -g @vscode/vsce  # If you don't have vsce installed
   vsce package
   ```

4. Install the extension:
   ```bash
   # From VS Code UI:
   # Open VS Code → Extensions → ... (More Actions) → Install from VSIX → Select the .vsix file

   # Or using command line (if VS Code CLI is available):
   code --install-extension aicommit-vscode-0.1.0.vsix
   ```

## Development

For working on the extension:

1. Open the project in VS Code
2. Make your changes
3. Press F5 to launch a new VS Code window with the extension loaded
4. Test your changes in the new window
5. Make changes and press F5 again to reload

## Known Issues

- Currently only supports the first repository when multiple repositories are open

## Release Notes

### 0.1.0

Initial release of AICommit for VS Code
