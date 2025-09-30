/**
 * Error Handler Utilities for AICommit
 *
 * Provides consistent error formatting, logging, and recovery mechanisms
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

/**
 * Error formatting and reporting utilities
 */
class ErrorHandler {
  constructor(options = {}) {
    this.logLevel = options.logLevel || 'info';
    this.logToFile = options.logToFile || false;
    this.logFilePath = options.logFilePath || path.join(os.tmpdir(), 'aicommit-errors.log');
    this.exitOnError = options.exitOnError !== false; // Default true
  }

  /**
   * Format error for console output
   */
  formatError(error, includeStack = false) {
    const timestamp = new Date().toISOString();
    let formatted = `[${timestamp}] ERROR: ${error.message}`;

    if (error.code) {
      formatted += ` (${error.code})`;
    }

    if (error.details && Object.keys(error.details).length > 0) {
      formatted += `\nDetails: ${JSON.stringify(error.details, null, 2)}`;
    }

    if (includeStack && error.stack) {
      formatted += `\nStack:\n${error.stack}`;
    }

    return formatted;
  }

  /**
   * Format error for user display (user-friendly)
   */
  formatUserError(error) {
    if (error.getUserMessage && typeof error.getUserMessage === 'function') {
      return error.getUserMessage();
    }
    return error.message || 'An unexpected error occurred';
  }

  /**
   * Log error to console and optionally to file
   */
  logError(error, context = {}) {
    const errorInfo = {
      ...error.toJSON ? error.toJSON() : {
        name: error.name || 'Error',
        message: error.message,
        stack: error.stack
      },
      context,
      pid: process.pid,
      platform: os.platform(),
      nodeVersion: process.version
    };

    // Console logging
    console.error(this.formatError(error, this.logLevel === 'debug'));

    // File logging
    if (this.logToFile) {
      try {
        const logEntry = JSON.stringify(errorInfo, null, 2) + '\n';
        fs.appendFileSync(this.logFilePath, logEntry);
      } catch (logErr) {
        console.error('Failed to write to error log file:', logErr.message);
      }
    }
  }

  /**
   * Handle error with appropriate response
   */
  handleError(error, context = {}) {
    this.logError(error, context);

    // Show user-friendly message
    const userMessage = this.formatUserError(error);
    console.error(`\nError: ${userMessage}`);

    if (this.exitOnError) {
      process.exit(1);
    }
  }

  /**
   * Wrap async functions with error handling
   */
  wrapAsync(fn, context = {}) {
    return async (...args) => {
      try {
        return await fn(...args);
      } catch (error) {
        this.handleError(error, { ...context, function: fn.name, args });
        throw error;
      }
    };
  }

  /**
   * Wrap sync functions with error handling
   */
  wrapSync(fn, context = {}) {
    return (...args) => {
      try {
        return fn(...args);
      } catch (error) {
        this.handleError(error, { ...context, function: fn.name, args });
        throw error;
      }
    };
  }
}

/**
 * Recovery mechanisms for different error types
 */
class ErrorRecovery {
  /**
   * Attempt to recover from platform errors
   */
  static attemptPlatformRecovery(error) {
    const suggestions = [];

    if (error.code === 'PLATFORM_ERROR') {
      if (error.details.platform === 'linux' && error.details.architecture !== 'x64') {
        suggestions.push('Try using an x64 system for Linux compatibility');
      }

      if (!error.details.platform || !error.details.architecture) {
        suggestions.push('Your platform may not be supported. Check the documentation for supported platforms');
      }
    }

    return suggestions;
  }

  /**
   * Attempt to recover from binary errors
   */
  static attemptBinaryRecovery(error) {
    const suggestions = [];

    if (error.code === 'BINARY_ERROR') {
      suggestions.push('Try reinstalling the package: npm uninstall -g @suenot/aicommit && npm install -g @suenot/aicommit');
      suggestions.push('Check if the binary has proper execution permissions');

      if (error.details.binaryPath) {
        suggestions.push(`Verify the binary exists at: ${error.details.binaryPath}`);
      }
    }

    return suggestions;
  }

  /**
   * Attempt to recover from Git errors
   */
  static attemptGitRecovery(error) {
    const suggestions = [];

    if (error.code === 'GIT_ERROR') {
      suggestions.push('Ensure you are in a git repository');
      suggestions.push('Check if git is installed and accessible from command line');
      suggestions.push('Verify you have proper permissions for the repository');

      if (error.message.includes('No changes')) {
        suggestions.push('Stage some changes before generating a commit message');
      }
    }

    return suggestions;
  }

  /**
   * Attempt to recover from VSCode errors
   */
  static attemptVSCodeRecovery(error) {
    const suggestions = [];

    if (error.code === 'VSCODE_ERROR') {
      suggestions.push('Ensure VSCode is properly configured');
      suggestions.push('Check if the git extension is enabled in VSCode');
      suggestions.push('Verify workspace settings for aicommit extension');
    }

    return suggestions;
  }

  /**
   * Get recovery suggestions for any error
   */
  static getRecoverySuggestions(error) {
    let suggestions = [];

    switch (error.code) {
      case 'PLATFORM_ERROR':
        suggestions = this.attemptPlatformRecovery(error);
        break;
      case 'BINARY_ERROR':
        suggestions = this.attemptBinaryRecovery(error);
        break;
      case 'GIT_ERROR':
        suggestions = this.attemptGitRecovery(error);
        break;
      case 'VSCODE_ERROR':
        suggestions = this.attemptVSCodeRecovery(error);
        break;
      default:
        suggestions.push('Check the documentation or report this issue if it persists');
    }

    return suggestions;
  }

  /**
   * Display recovery suggestions to user
   */
  static displayRecoverySuggestions(error) {
    const suggestions = this.getRecoverySuggestions(error);

    if (suggestions.length > 0) {
      console.error('\nPossible solutions:');
      suggestions.forEach((suggestion, index) => {
        console.error(`  ${index + 1}. ${suggestion}`);
      });
      console.error('');
    }
  }
}

/**
 * Global error handler setup
 */
function setupGlobalErrorHandler(options = {}) {
  const errorHandler = new ErrorHandler(options);

  // Handle uncaught exceptions
  process.on('uncaughtException', (error) => {
    console.error('Uncaught Exception:');
    errorHandler.handleError(error, { type: 'uncaughtException' });
  });

  // Handle unhandled promise rejections
  process.on('unhandledRejection', (reason, promise) => {
    const error = reason instanceof Error ? reason : new Error(String(reason));
    console.error('Unhandled Promise Rejection:');
    errorHandler.handleError(error, { type: 'unhandledRejection', promise });
  });

  return errorHandler;
}

module.exports = {
  ErrorHandler,
  ErrorRecovery,
  setupGlobalErrorHandler
};