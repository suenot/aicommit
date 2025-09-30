#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

// Import error handling framework
const { PlatformError, BinaryError } = require('./lib/errors');
const { ErrorHandler, ErrorRecovery, setupGlobalErrorHandler } = require('./lib/error-handler');

// Setup global error handler
const errorHandler = setupGlobalErrorHandler({
  logLevel: process.env.AICOMMIT_LOG_LEVEL || 'info',
  logToFile: process.env.AICOMMIT_LOG_TO_FILE === 'true',
  exitOnError: true
});

// Get the binary path based on the platform and architecture
function getBinaryPath() {
    const platform = os.platform();
    const arch = os.arch();
    
    const platformMap = {
        win32: 'win32',
        linux: 'linux',
        darwin: 'darwin'
    };
    
    const archMap = {
        x64: 'x64',
        arm64: 'arm64'
    };
    
    const platformKey = platformMap[platform];
    const archKey = archMap[arch];
    
    if (!platformKey || !archKey) {
        throw new PlatformError(`Unsupported platform (${platform}) or architecture (${arch})`, platform, arch);
    }

    // Linux only supports x64
    if (platform === 'linux' && arch !== 'x64') {
        throw new PlatformError('Linux builds are only available for x64 architecture', platform, arch);
    }

    const binaryName = platform === 'win32' ? 'aicommit.exe' : 'aicommit';
    const binaryPath = path.join(__dirname, 'bin', `${platformKey}-${archKey}`, binaryName);

    if (!fs.existsSync(binaryPath)) {
        throw new BinaryError(`Binary not found at ${binaryPath}`, binaryPath);
    }
    
    return binaryPath;
}

try {
    const binaryPath = getBinaryPath();

    // Make sure the binary is executable
    if (os.platform() !== 'win32') {
        try {
            fs.chmodSync(binaryPath, '755');
        } catch (chmodErr) {
            throw new BinaryError(`Failed to set binary permissions: ${chmodErr.message}`, binaryPath);
        }
    }

    // Execute the binary with all arguments
    const binary = spawn(binaryPath, process.argv.slice(2), {
        stdio: 'inherit'
    });

    binary.on('error', (err) => {
        const binaryError = new BinaryError(`Failed to start binary: ${err.message}`, binaryPath);
        errorHandler.handleError(binaryError, { operation: 'spawn', args: process.argv.slice(2) });
        ErrorRecovery.displayRecoverySuggestions(binaryError);
    });

    binary.on('exit', (code) => {
        if (code !== 0) {
            const binaryError = new BinaryError(`Binary exited with code ${code}`, binaryPath, code);
            errorHandler.logError(binaryError, { operation: 'exit', exitCode: code });
        }
        process.exit(code);
    });

} catch (err) {
    errorHandler.handleError(err, { operation: 'initialization' });
    ErrorRecovery.displayRecoverySuggestions(err);
}
