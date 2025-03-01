// The module 'vscode' contains the VS Code extensibility API
const vscode = require('vscode');
const { execSync, exec } = require('child_process');
const { resolve } = require('path');

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
        console.error(`Command execution error: ${error.message}`);
        console.error(`Command stderr: ${stderr}`);
        reject(`Command failed: ${command}\n${stderr || error.message}`);
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
        vscode.window.showErrorMessage('No git repository found');
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
            throw new Error('No changes to commit');
          }
        });
      } catch (error) {
        if (error.message === 'No changes to commit') {
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
          throw new Error(`Command failed: ${error}`);
        }
      });
      
      // Apply the message to the input box
      if (message) {
        console.log(`Setting commit message: ${message}`);
        repo.inputBox.value = message;
        vscode.window.showInformationMessage('Commit message generated successfully');
      }
    } catch (error) {
      console.error(`Error in aicommit extension: ${error.message}`);
      console.error(`Stack trace: ${error.stack}`);
      vscode.window.showErrorMessage(`AICommit error: ${error.message}`);
    }
  });

  context.subscriptions.push(disposable);
}

function deactivate() {}

module.exports = {
  activate,
  deactivate
};
