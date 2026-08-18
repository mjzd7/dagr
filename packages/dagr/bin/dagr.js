#!/usr/bin/env node
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

const BIN_NAME = process.platform === 'win32' ? 'dagr-native.exe' : 'dagr-native';
const LOCAL_BIN = path.join(__dirname, BIN_NAME);

// 1. Look for downloaded/installed local binary
let executable = LOCAL_BIN;

if (!fs.existsSync(executable)) {
  // 2. Check if dagr is in system PATH
  try {
    const { execSync } = require('child_process');
    const systemPath = execSync(process.platform === 'win32' ? 'where dagr' : 'which dagr', { stdio: 'pipe' })
      .toString()
      .trim();
    if (systemPath && fs.existsSync(systemPath)) {
      executable = systemPath;
    }
  } catch (e) {
    // Check fallback build locations
    const releaseBin = path.join(__dirname, '..', '..', '..', 'target', 'release', 'dagr');
    const debugBin = path.join(__dirname, '..', '..', '..', 'target', 'debug', 'dagr');
    if (fs.existsSync(releaseBin)) {
      executable = releaseBin;
    } else if (fs.existsSync(debugBin)) {
      executable = debugBin;
    } else {
      console.error('❌ [DAGR] Native binary not found. Run "npm run postinstall" or "cargo install dagr".');
      process.exit(1);
    }
  }
}

// Spawn native binary with full stdio/MCP pass-through
const child = spawn(executable, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env,
});

child.on('error', (err) => {
  console.error(`❌ [DAGR] Execution failed: ${err.message}`);
  process.exit(1);
});

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code || 0);
  }
});
