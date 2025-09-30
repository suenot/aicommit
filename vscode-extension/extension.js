// The module 'vscode' contains the VS Code extensibility API
const vscode = require('vscode');
const { execSync, exec } = require('child_process');
const { resolve } = require('path');

// Import error handling framework
const { AICommitError, VSCodeError, GitError, BinaryError, ConfigError } = require('../lib/errors');
const { ErrorHandler, ErrorRecovery } = require('../lib/error-handler');

// Setup error handler for VSCode extension
const errorHandler = new ErrorHandler({
  logLevel: 'info',
  exitOnError: false, // Don't exit on error in VSCode extension
  logToFile: true // Log to file for debugging VSCode issues
});

/**
 * Run a command in the given directory and return the output
 * @param {string} command The command to execute
 * @param {string} cwd The working directory
 * @returns {Promise<string>} Command output
 */
function execPromise(command, cwd) {
  return new Promise((resolve, reject) => {
    console.log(`Executing command: ${command} in directory: ${cwd}`);
    exec(command, { cwd, encoding: 'utf8' }, (error, stdout, stderr) => {
      if (error) {
        let aicommitError;

        if (error.code === 'ENOENT') {
          aicommitError = new BinaryError(`Command not found: ${command}`, command);
        } else if (command.includes('git')) {
          aicommitError = new GitError(`Git command failed: ${stderr || error.message}`, command, cwd);
        } else if (command.includes('aicommit')) {
          aicommitError = new BinaryError(`AICommit execution failed: ${stderr || error.message}`, command);
        } else {
          aicommitError = new VSCodeError(`Command execution failed: ${stderr || error.message}`, command);
        }

        errorHandler.logError(aicommitError, { command, cwd });
        console.error(`Command execution error: ${error.message}`);
        console.error(`Command stderr: ${stderr}`);
        reject(aicommitError);
        return;
      }
      console.log(`Command succeeded with output: ${stdout.trim()}`);
      resolve(stdout.trim());
    });
  });
}

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  // Register the command for generating commit messages
  let disposable = vscode.commands.registerCommand('aicommit.generateCommitMessage', async function () {
    try {
      // Get configuration
      const config = vscode.workspace.getConfiguration('aicommit');
      const autoStage = config.get('autoStage');
      const providerOverride = config.get('providerOverride');
      
      // Get git extension
      const gitExtension = vscode.extensions.getExtension('vscode.git').exports;
      const git = gitExtension.getAPI(1);
      
      // Get repository
      const repositories = git.repositories;
      if (!repositories.length) {
        const gitError = new GitError('No git repository found');
        errorHandler.logError(gitError, { operation: 'getRepository' });
        vscode.window.showErrorMessage(gitError.getUserMessage());
        ErrorRecovery.displayRecoverySuggestions(gitError);
        return;
      }
      
      const repo = repositories[0];
      const repoRoot = repo.rootUri.fsPath;
      console.log(`Repository root: ${repoRoot}`);
      
      // Auto-stage changes if configured
      if (autoStage) {
        await vscode.window.withProgress({
          location: vscode.ProgressLocation.Notification,
          title: "AICommit: Staging changes...",
          cancellable: false
        }, async () => {
          console.log('Staging changes...');
          await repo.add(['.']);
        });
      }
      
      // First check if there are any changes to commit
      try {
        console.log('Checking for changes...');
        // Check if there are any changes in the repo
        await execPromise('git status --porcelain', repoRoot).then(result => {
          if (!result) {
            vscode.window.showInformationMessage('No changes to commit');
            throw new GitError('No changes to commit', 'git status --porcelain', repoRoot);
          }
        });
      } catch (error) {
        if (error instanceof GitError && error.message.includes('No changes to commit')) {
          return;
        }
        throw error;
      }
      
      // Generate message
      const message = await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "AICommit: Generating commit message...",
        cancellable: false
      }, async () => {
        let cmd = 'aicommit --dry-run --no-gitignore-check';
        
        if (providerOverride) {
          cmd += ` --set ${providerOverride}`;
        }
        
        try {
          // Use proper working directory (repository root)
          console.log(`Generating commit message with command: ${cmd}`);
          return await execPromise(cmd, repoRoot);
        } catch (error) {
          console.error(`Error generating commit message: ${error}`);

          if (error instanceof AICommitError) {
            throw error;
          } else {
            throw new BinaryError(`AICommit command failed: ${error.message || error}`, cmd);
          }
        }
      });
      
      // Apply the message to the input box
      if (message) {
        console.log(`Setting commit message: ${message}`);
        repo.inputBox.value = message;
        vscode.window.showInformationMessage('Commit message generated successfully');
      }
    } catch (error) {
      let aicommitError;

      if (error instanceof AICommitError) {
        aicommitError = error;
      } else {
        aicommitError = new VSCodeError(`Unexpected error in aicommit extension: ${error.message}`);
      }

      errorHandler.handleError(aicommitError, { operation: 'generateCommitMessage', extension: 'vscode' });
      vscode.window.showErrorMessage(aicommitError.getUserMessage());
      ErrorRecovery.displayRecoverySuggestions(aicommitError);
    }
  });

  context.subscriptions.push(disposable);
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
