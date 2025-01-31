#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const { platform, arch } = process;
const { chmodSync, renameSync, existsSync } = require('fs');
const { join } = require('path');

const binaryName = `aicommit-${platform}-${arch}`;
const binDir = join(__dirname, 'bin');

if (existsSync(join(binDir, binaryName))) {
  renameSync(join(binDir, binaryName), join(binDir, 'aicommit'));
  chmodSync(join(binDir, 'aicommit'), '755');
} else {
  console.warn(`Binary for ${binaryName} not found, using JS fallback`);
}

// Get the binary name based on the platform and architecture
function getBinaryName() {
    const platform = os.platform();
    const arch = os.arch();
    
    console.log('Debug: Platform:', platform);
    console.log('Debug: Architecture:', arch);
    
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
    
    const binaryName = `${base}-${archSuffix}${platform === 'win32' ? '.exe' : ''}`;
    console.log('Debug: Binary name:', binaryName);
    return binaryName;
}

// Path to the binary in the package
const binaryPath = path.join(__dirname, 'bin', getBinaryName());
console.log('Debug: Binary path:', binaryPath);

// Check if binary exists and is executable
if (!fs.existsSync(binaryPath)) {
    console.error(`Binary not found: ${binaryPath}`);
    console.log('Debug: Directory contents:', fs.readdirSync(path.join(__dirname, 'bin')));
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
