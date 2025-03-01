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

## Known Issues

- Currently only supports the first repository when multiple repositories are open

## Release Notes

### 0.1.0

Initial release of AICommit for VS Code
