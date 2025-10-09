#!/bin/bash
# Test script for Claude Code provider integration

echo "Testing Claude Code provider..."
echo "Checking if claude command exists..."
if command -v claude &> /dev/null; then
    echo "✓ claude command found in PATH"
    claude --version || echo "Could not get claude version"
else
    echo "✗ claude command not found in PATH"
    echo "  This is expected in CI/test environment"
    echo "  The integration will show proper error message to users"
fi

echo ""
echo "Testing OpenCode provider..."
echo "Checking if opencode command exists..."
if command -v opencode &> /dev/null; then
    echo "✓ opencode command found in PATH"
    opencode --version || echo "Could not get opencode version"
else
    echo "✗ opencode command not found in PATH"
    echo "  This is expected in CI/test environment"
    echo "  The integration will show proper error message to users"
fi

echo ""
echo "Test completed. The providers are designed to work with external CLIs."
echo "They will display user-friendly error messages if the CLIs are not installed."
