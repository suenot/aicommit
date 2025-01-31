#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

// Get the binary name based on the platform and architecture
function getBinaryName() {
    const platform = os.platform();
    const arch = os.arch();
    
    const platformMap = {
        win32: 'aicommit-windows',
        linux: 'aicommit-linux',
        darwin: 'aicommit-macos'
    };
    
    const archMap = {
        x64: 'x86_64',
        arm64: 'aarch64'
    };
    
    const base = platformMap[platform];
    const archSuffix = archMap[arch];
    
    // Linux only supports x86_64
    if (platform === 'linux' && arch !== 'x64') {
        throw new Error('Linux builds are only available for x86_64 architecture');
    }
    
    if (!base || !archSuffix) {
        throw new Error(`Unsupported platform (${platform}) or architecture (${arch})`);
    }
    
    return `${base}-${archSuffix}${platform === 'win32' ? '.exe' : ''}`;
}

// Path to the binary in the package
const binaryPath = path.join(__dirname, 'bin', getBinaryName());

// Check if binary exists and is executable
if (!fs.existsSync(binaryPath)) {
    console.error(`Binary not found: ${binaryPath}`);
    process.exit(1);
}

// Make binary executable (not needed on Windows)
if (os.platform() !== 'win32') {
    try {
        fs.chmodSync(binaryPath, 0o755);
    } catch (err) {
        console.error(`Failed to make binary executable: ${err}`);
        process.exit(1);
    }
}

// Run the binary with all arguments passed through
const binary = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' });

binary.on('error', (err) => {
    console.error(`Failed to start binary: ${err}`);
    process.exit(1);
});

binary.on('exit', (code) => {
    process.exit(code);
});
