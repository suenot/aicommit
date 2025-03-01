#!/bin/bash

# Build script for the AICommit VS Code extension

# Make sure required tools are installed
echo "Checking for required tools..."
if ! command -v npm &> /dev/null; then
    echo "npm could not be found. Please install Node.js and npm."
    exit 1
fi

# Get version from package.json
PACKAGE_VERSION=$(node -p "require('./package.json').version")
echo "Building AICommit VS Code extension v${PACKAGE_VERSION}..."

# Install dependencies
echo "Installing dependencies..."
npm install

# Install vsce if not already installed
if ! command -v vsce &> /dev/null; then
    echo "Installing vsce globally..."
    npm install -g @vscode/vsce
fi

# Package the extension
echo "Packaging extension..."
vsce package

# Check if the package was created successfully
if [ -f "aicommit-vscode-${PACKAGE_VERSION}.vsix" ]; then
    echo "Extension packaged successfully: aicommit-vscode-${PACKAGE_VERSION}.vsix"
    echo ""
    echo "To install the extension in VS Code, run:"
    echo "code --install-extension aicommit-vscode-${PACKAGE_VERSION}.vsix"
else
    echo "Error: Failed to package extension."
    exit 1
fi
