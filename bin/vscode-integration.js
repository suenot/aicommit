#!/usr/bin/env node

/**
 * VSCode integration for aicommit
 * This script is intended to be used with VSCode's source control integration
 * to generate commit messages automatically using aicommit CLI
 *
 * Usage from VSCode commands:
 * - Run "aicommit: Generate Commit Message" to create a commit message
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Import error handling framework
const { BinaryError, GitError } = require('../lib/errors');
const { ErrorHandler, ErrorRecovery } = require('../lib/error-handler');

// Setup error handler for VSCode integration
const errorHandler = new ErrorHandler({
  logLevel: 'info',
  exitOnError: false // Don't exit on error in VSCode integration
});

function generateCommitMessage() {
  console.log('Generating commit message with aicommit...');

  try {
    // Execute aicommit CLI with --dry-run to just generate the message without committing
    const result = execSync('aicommit --dry-run', { encoding: 'utf8' });

    // Return the message to VS Code
    console.log('Generated message:');
    console.log(result.trim());
    return result.trim();
  } catch (error) {
    let aicommitError;

    if (error.code === 'ENOENT') {
      aicommitError = new BinaryError('aicommit binary not found in PATH');
    } else if (error.status && error.status !== 0) {
      const stderr = error.stderr ? error.stderr.toString() : '';
      if (stderr.includes('git') || stderr.includes('repository')) {
        aicommitError = new GitError(`Git operation failed: ${stderr}`, 'aicommit --dry-run');
      } else {
        aicommitError = new BinaryError(`aicommit exited with code ${error.status}: ${stderr}`, null, error.status);
      }
    } else {
      aicommitError = new BinaryError(`Failed to execute aicommit: ${error.message}`);
    }

    errorHandler.handleError(aicommitError, { operation: 'generateCommitMessage' });
    ErrorRecovery.displayRecoverySuggestions(aicommitError);

    return null;
  }
}

// Main function - can be extended to handle CLI args when called from VS Code
function main() {
  const args = process.argv.slice(2);
  
  if (args.includes('--generate')) {
    const message = generateCommitMessage();
    if (message) {
      // Output in a format that VS Code extension can parse
      console.log(JSON.stringify({ success: true, message }));
    } else {
      console.log(JSON.stringify({ success: false, error: 'Failed to generate commit message' }));
    }
  } else {
    console.log('VS Code integration for aicommit');
    console.log('Usage:');
    console.log('  --generate: Generate a commit message');
  }
}

main();
