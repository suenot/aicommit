/**
 * Comprehensive Error Handling Framework for AICommit
 *
 * This module provides custom error types, consistent error formatting,
 * and error recovery mechanisms for the aicommit application.
 */

/**
 * Base error class for all AICommit errors
 */
class AICommitError extends Error {
  constructor(message, code = 'AICOMMIT_ERROR', details = {}) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.details = details;
    this.timestamp = new Date().toISOString();

    // Maintain proper stack trace for where our error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }

  /**
   * Convert error to a structured object for logging/reporting
   */
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      details: this.details,
      timestamp: this.timestamp,
      stack: this.stack
    };
  }

  /**
   * Get user-friendly error message
   */
  getUserMessage() {
    return this.message;
  }
}

/**
 * Platform/Binary related errors
 */
class PlatformError extends AICommitError {
  constructor(message, platform = null, architecture = null) {
    super(message, 'PLATFORM_ERROR', { platform, architecture });
  }

  getUserMessage() {
    return `Platform Error: ${this.message}. Please check your system compatibility.`;
  }
}

/**
 * Binary execution errors
 */
class BinaryError extends AICommitError {
  constructor(message, binaryPath = null, exitCode = null) {
    super(message, 'BINARY_ERROR', { binaryPath, exitCode });
  }

  getUserMessage() {
    return `Binary Execution Error: ${this.message}. Please check the installation.`;
  }
}

/**
 * VSCode integration errors
 */
class VSCodeError extends AICommitError {
  constructor(message, operation = null) {
    super(message, 'VSCODE_ERROR', { operation });
  }

  getUserMessage() {
    return `VSCode Integration Error: ${this.message}`;
  }
}

/**
 * Git operation errors
 */
class GitError extends AICommitError {
  constructor(message, command = null, repository = null) {
    super(message, 'GIT_ERROR', { command, repository });
  }

  getUserMessage() {
    return `Git Error: ${this.message}. Please check your git repository status.`;
  }
}

/**
 * Configuration errors
 */
class ConfigError extends AICommitError {
  constructor(message, configKey = null) {
    super(message, 'CONFIG_ERROR', { configKey });
  }

  getUserMessage() {
    return `Configuration Error: ${this.message}. Please check your settings.`;
  }
}

/**
 * Network/API errors
 */
class NetworkError extends AICommitError {
  constructor(message, url = null, statusCode = null) {
    super(message, 'NETWORK_ERROR', { url, statusCode });
  }

  getUserMessage() {
    return `Network Error: ${this.message}. Please check your internet connection.`;
  }
}

module.exports = {
  AICommitError,
  PlatformError,
  BinaryError,
  VSCodeError,
  GitError,
  ConfigError,
  NetworkError
};