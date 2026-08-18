#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = require('../package.json').version;
const REPO = 'mjzd7/dagr';
const BIN_DIR = path.join(__dirname, '..', 'bin');
const TARGET_EXE = path.join(BIN_DIR, process.platform === 'win32' ? 'dagr-native.exe' : 'dagr-native');

function getTargetTriple() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'darwin') {
    return arch === 'arm64' ? 'dagr-darwin-arm64' : 'dagr-darwin-x86_64';
  } else if (platform === 'linux') {
    return arch === 'arm64' ? 'dagr-linux-aarch64' : 'dagr-linux-x86_64';
  } else if (platform === 'win32') {
    return 'dagr-windows-x86_64.exe';
  }
  throw new Error(`Unsupported platform: ${platform} (${arch})`);
}

function downloadBinary() {
  // If local binary exists in cargo target or path, link/copy it
  const localTarget = path.join(__dirname, '..', '..', '..', 'target', 'release', 'dagr');
  const localTargetDebug = path.join(__dirname, '..', '..', '..', 'target', 'debug', 'dagr');
  
  if (fs.existsSync(localTarget)) {
    fs.copyFileSync(localTarget, TARGET_EXE);
    fs.chmodSync(TARGET_EXE, 0o755);
    console.log(`⚡ [DAGR-NPM] Linked local compiled release binary -> ${TARGET_EXE}`);
    return;
  } else if (fs.existsSync(localTargetDebug)) {
    fs.copyFileSync(localTargetDebug, TARGET_EXE);
    fs.chmodSync(TARGET_EXE, 0o755);
    console.log(`⚡ [DAGR-NPM] Linked local compiled debug binary -> ${TARGET_EXE}`);
    return;
  }

  const targetName = getTargetTriple();
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${targetName}`;
  console.log(`⚡ [DAGR-NPM] Downloading pre-compiled binary: ${url}...`);

  const file = fs.createWriteStream(TARGET_EXE);
  https.get(url, (response) => {
    if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
      https.get(response.headers.location, (redirectResponse) => {
        redirectResponse.pipe(file);
        file.on('finish', () => {
          file.close();
          fs.chmodSync(TARGET_EXE, 0o755);
          console.log('✅ [DAGR-NPM] Native binary successfully installed.');
        });
      });
    } else if (response.statusCode === 200) {
      response.pipe(file);
      file.on('finish', () => {
        file.close();
        fs.chmodSync(TARGET_EXE, 0o755);
        console.log('✅ [DAGR-NPM] Native binary successfully installed.');
      });
    } else {
      console.warn(`⚠️  Could not download pre-compiled release (${response.statusCode}). Will build from source if cargo is available.`);
      file.close();
      if (fs.existsSync(TARGET_EXE)) fs.unlinkSync(TARGET_EXE);
    }
  }).on('error', (err) => {
    console.warn(`⚠️  Download failed: ${err.message}`);
  });
}

try {
  downloadBinary();
} catch (e) {
  console.warn(`⚠️  Postinstall notice: ${e.message}`);
}
