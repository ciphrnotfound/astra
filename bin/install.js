#!/usr/bin/env node

const os = require('os');
const path = require('path');
const fs = require('fs');
const https = require('https');
const { spawnSync } = require('child_process');

const root = path.join(__dirname, '..');
const pkg = require(path.join(root, 'package.json'));
const version = pkg.version;
const binDir = path.join(os.homedir(), '.astra', 'bin', version);
const binaryName = process.platform === 'win32' ? 'astra-cli.exe' : 'astra-cli';
const binaryPath = path.join(binDir, binaryName);

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

function platformKey() {
  const arch = process.arch;
  const platform = process.platform;
  return `${platform}-${arch}`;
}

function releaseBaseUrl() {
  if (process.env.ASTRA_DOWNLOAD_BASE_URL) return process.env.ASTRA_DOWNLOAD_BASE_URL;
  const tag = process.env.ASTRA_RELEASE_TAG || `v${version}`;
  return `https://github.com/ciphrnotfound/astra/releases/download/${tag}`;
}

function assetUrl() {
  const ext = process.platform === 'win32' ? 'exe' : 'bin';
  const name = `astra-cli-${version}-${platformKey()}.${ext}`;
  return `${releaseBaseUrl()}/${name}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, (res) => {
      if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume();
        download(res.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      if (res.statusCode !== 200) {
        res.resume();
        reject(new Error(`HTTP ${res.statusCode} downloading ${url}`));
        return;
      }
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on('finish', () => file.close(resolve));
      file.on('error', reject);
    });
    request.on('error', reject);
    request.setTimeout(30_000, () => request.destroy(new Error('Download timeout')));
  });
}

async function main() {
  if (process.env.ASTRA_SKIP_INSTALL === '1') return;

  ensureDir(binDir);

  if (fs.existsSync(binaryPath)) return;

  const url = assetUrl();
  process.stdout.write(`[astra-cli] downloading binary for ${platformKey()}...\n`);
  try {
    await download(url, binaryPath);
    if (process.platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }
    process.stdout.write(`[astra-cli] installed ${binaryName} to ${binaryPath}\n`);
  } catch (e) {
    process.stderr.write(
      `[astra-cli] install warning: could not download prebuilt binary.\n` +
        `Reason: ${e && e.message ? e.message : String(e)}\n` +
        `Expected asset name: astra-cli-${version}-${platformKey()}.${process.platform === 'win32' ? 'exe' : 'bin'}\n` +
        `You can set ASTRA_DOWNLOAD_BASE_URL to your release host, or set ASTRA_BUILD_FROM_SOURCE=1 to compile locally.\n`
    );
    if (process.env.ASTRA_BUILD_FROM_SOURCE === '1') {
      const targetDir = process.env.ASTRA_CARGO_TARGET_DIR || path.join(os.homedir(), '.astra', 'target');
      process.stdout.write(`[astra-cli] building from source into ${targetDir}...\n`);
      const r = spawnSync('cargo', ['build', '-p', 'astra-cli', '--release'], {
        cwd: root,
        stdio: 'inherit',
        shell: process.platform === 'win32',
        env: { ...process.env, CARGO_TARGET_DIR: targetDir },
      });
      if (r.status !== 0) {
        process.stderr.write('[astra-cli] build from source failed.\n');
        if (process.env.ASTRA_STRICT_INSTALL === '1') process.exit(1);
      }
    }
    if (process.env.ASTRA_STRICT_INSTALL === '1') {
      process.exit(1);
    }
  }
}

main();
