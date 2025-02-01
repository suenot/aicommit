#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

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
        throw new Error(`Unsupported platform (${platform}) or architecture (${arch})`);
    }
    
    // Linux only supports x64
    if (platform === 'linux' && arch !== 'x64') {
        throw new Error('Linux builds are only available for x64 architecture');
    }

    const binaryName = platform === 'win32' ? 'aicommit.exe' : 'aicommit';
    const binaryPath = path.join(__dirname, 'bin', `${platformKey}-${archKey}`, binaryName);
    
    if (!fs.existsSync(binaryPath)) {
        throw new Error(`Binary not found at ${binaryPath}`);
    }
    
    return binaryPath;
}

try {
    const binaryPath = getBinaryPath();
    
    // Make sure the binary is executable
    if (os.platform() !== 'win32') {
        fs.chmodSync(binaryPath, '755');
    }
    
    // Execute the binary with all arguments
    const binary = spawn(binaryPath, process.argv.slice(2), {
        stdio: 'inherit'
    });
    
    binary.on('error', (err) => {
        console.error('Failed to start binary:', err);
        process.exit(1);
    });
    
    binary.on('exit', (code) => {
        process.exit(code);
    });
} catch (err) {
    console.error('Error:', err.message);
    process.exit(1);
}
