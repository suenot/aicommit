#!/usr/bin/env node

/**
 * Error Handling Framework Usage Examples
 *
 * This file demonstrates how to use the comprehensive error handling
 * framework implemented for AICommit.
 */

// Import the error handling framework
const { AICommitError, PlatformError, BinaryError, VSCodeError, GitError, ConfigError, NetworkError } = require('../lib/errors');
const { ErrorHandler, ErrorRecovery, setupGlobalErrorHandler } = require('../lib/error-handler');

console.log('📚 AICommit Error Handling Framework Usage Examples\n');

// Example 1: Basic Error Creation and Handling
console.log('Example 1: Basic Error Creation');
console.log('===============================');

function exampleBasicErrorCreation() {
  try {
    // Create a custom error with context
    throw new BinaryError('aicommit binary not found', '/usr/local/bin/aicommit');
  } catch (error) {
    console.log('✅ Caught error:', error.message);
    console.log('📋 Error code:', error.code);
    console.log('🆔 Error details:', JSON.stringify(error.details, null, 2));
    console.log('👤 User message:', error.getUserMessage());
  }
}

exampleBasicErrorCreation();
console.log('');

// Example 2: Error Handler Setup
console.log('Example 2: Error Handler Setup');
console.log('==============================');

function exampleErrorHandlerSetup() {
  // Create an error handler with custom configuration
  const errorHandler = new ErrorHandler({
    logLevel: 'debug',           // Show debug information
    logToFile: true,             // Log errors to file
    logFilePath: '/tmp/aicommit-errors.log',
    exitOnError: false           // Don't exit on error (useful for testing)
  });

  const error = new GitError('Repository not initialized', 'git status');

  console.log('📝 Logging error with context...');
  errorHandler.logError(error, {
    operation: 'statusCheck',
    userId: 'example-user',
    timestamp: new Date().toISOString()
  });

  console.log('✅ Error logged successfully');
}

exampleErrorHandlerSetup();
console.log('');

// Example 3: Global Error Handler
console.log('Example 3: Global Error Handler');
console.log('===============================');

function exampleGlobalErrorHandler() {
  // Setup global error handling for uncaught exceptions and promise rejections
  const globalHandler = setupGlobalErrorHandler({
    logLevel: 'info',
    exitOnError: false  // Set to false for this example
  });

  console.log('🌐 Global error handler setup complete');
  console.log('   - Uncaught exceptions will be handled');
  console.log('   - Unhandled promise rejections will be caught');

  // Example of promise rejection (commented out to avoid actual rejection)
  // Promise.reject(new Error('Example unhandled rejection'));
}

exampleGlobalErrorHandler();
console.log('');

// Example 4: Error Recovery Suggestions
console.log('Example 4: Error Recovery');
console.log('=========================');

function exampleErrorRecovery() {
  const platformError = new PlatformError('Unsupported architecture', 'linux', 'arm32');

  console.log('🔧 Getting recovery suggestions for platform error:');
  const suggestions = ErrorRecovery.getRecoverySuggestions(platformError);

  suggestions.forEach((suggestion, index) => {
    console.log(`   ${index + 1}. ${suggestion}`);
  });

  console.log('\n📢 Displaying recovery suggestions:');
  ErrorRecovery.displayRecoverySuggestions(platformError);
}

exampleErrorRecovery();

// Example 5: Function Wrapping
console.log('Example 5: Function Wrapping');
console.log('============================');

function exampleFunctionWrapping() {
  const errorHandler = new ErrorHandler({ exitOnError: false });

  // Original risky function
  function riskyAsyncOperation() {
    return new Promise((resolve, reject) => {
      // Simulate random failure
      if (Math.random() > 0.5) {
        reject(new BinaryError('Random failure in async operation'));
      } else {
        resolve('Success!');
      }
    });
  }

  // Wrap the function with error handling
  const safeAsyncOperation = errorHandler.wrapAsync(riskyAsyncOperation, {
    module: 'example',
    operation: 'riskyOperation'
  });

  console.log('🔄 Attempting wrapped async operation...');

  safeAsyncOperation()
    .then(result => console.log('✅ Operation succeeded:', result))
    .catch(error => console.log('❌ Operation failed but was handled gracefully'));
}

exampleFunctionWrapping();
console.log('');

// Example 6: Different Error Types
console.log('Example 6: Different Error Types');
console.log('================================');

function exampleDifferentErrorTypes() {
  const errors = [
    new PlatformError('Platform not supported', 'freebsd', 'x64'),
    new BinaryError('Binary execution failed', '/bin/aicommit', 127),
    new VSCodeError('Extension initialization failed', 'activate'),
    new GitError('Merge conflict detected', 'git merge', '/path/to/repo'),
    new ConfigError('Invalid configuration value', 'api.provider'),
    new NetworkError('API request failed', 'https://api.example.com', 500)
  ];

  errors.forEach((error, index) => {
    console.log(`${index + 1}. ${error.constructor.name}: ${error.getUserMessage()}`);
  });
}

exampleDifferentErrorTypes();
console.log('');

console.log('🎯 Key Benefits of This Framework:');
console.log('==================================');
console.log('• Consistent error handling across all modules');
console.log('• Structured error information with context');
console.log('• User-friendly error messages');
console.log('• Automatic recovery suggestions');
console.log('• Comprehensive logging and debugging');
console.log('• Global error catching for robustness');
console.log('• Easy integration with existing code');
console.log('');
console.log('📖 For more information, see:');
console.log('   - lib/errors.js (Custom error types)');
console.log('   - lib/error-handler.js (Error handling utilities)');
console.log('   - test/error-framework.test.js (Test examples)');
console.log('');