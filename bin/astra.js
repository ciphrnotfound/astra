#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Path to the Rust binary
// In a real NPM package, we would package the compiled binaries for different platforms
// and select the correct one here. For local testing/dev, we use the build target.
const binaryName = process.platform === 'win32' ? 'astra-cli.exe' : 'astra-cli';
let binaryPath = path.join(__dirname, '..', 'target', 'release', binaryName);

if (!fs.existsSync(binaryPath)) {
  // Fallback to debug if release isn't built
  binaryPath = path.join(__dirname, '..', 'target', 'debug', binaryName);
}

if (!fs.existsSync(binaryPath)) {
  console.error('\n\x1b[31m[Error]\x1b[0m Astra binary not found. Please run "cargo build" first.');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('exit', (code) => {
  process.exit(code);
});
