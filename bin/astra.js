#!/usr/bin/env node

const { spawn, spawnSync } = require('child_process');
const os = require('os');
const path = require('path');
const fs = require('fs');

const binaryName = process.platform === 'win32' ? 'astra-cli.exe' : 'astra-cli';
const root = path.join(__dirname, '..');
const pkg = require(path.join(root, 'package.json'));
const version = pkg.version;
const binDir = path.join(os.homedir(), '.astra', 'bin', version);
const installedBinary = path.join(binDir, binaryName);
const cargoTargetDir = process.env.ASTRA_CARGO_TARGET_DIR || path.join(os.homedir(), '.astra', 'target');
const candidates = [installedBinary, path.join(root, 'target', 'release', binaryName), path.join(root, 'target', 'debug', binaryName)];
let binaryPath = candidates.find((p) => fs.existsSync(p));

if (!binaryPath) {
  const installer = path.join(__dirname, 'install.js');
  spawnSync(process.execPath, [installer], { stdio: 'inherit' });
  binaryPath = candidates.find((p) => fs.existsSync(p));
}

if (!binaryPath && process.env.ASTRA_BUILD_FROM_SOURCE === '1') {
  const install = spawnSync('cargo', ['build', '-p', 'astra-cli', '--release'], {
    cwd: root,
    stdio: 'inherit',
    shell: process.platform === 'win32',
    env: { ...process.env, CARGO_TARGET_DIR: cargoTargetDir },
  });
  if (install.status === 0) {
    const built = path.join(cargoTargetDir, 'release', binaryName);
    if (fs.existsSync(built)) {
      binaryPath = built;
    }
  }
}

if (!binaryPath) {
  console.error('\n[astra-cli] binary not available.');
  console.error(`- Expected downloaded binary: ${installedBinary}`);
  console.error('- To fix: re-run install, or set ASTRA_DOWNLOAD_BASE_URL to your release host.');
  console.error('- Dev fallback: set ASTRA_BUILD_FROM_SOURCE=1 and ensure Rust is installed.');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('exit', (code) => {
  process.exit(code);
});
