#!/usr/bin/env node

/**
 * Telepipe postinstall script
 * Selects the correct platform binary from the bundled binaries
 *
 * Binary naming:
 * - telepipe-darwin-arm64  (macOS Apple Silicon)
 * - telepipe-darwin-x64    (macOS Intel)
 * - telepipe-linux-x64     (Linux x86_64)
 * - telepipe-windows-x64.exe (Windows x86_64)
 *
 * Creates: bin/telepipe (or bin/telepipe.exe on Windows)
 */

const fs = require('fs');
const path = require('path');

// Platform mapping: Node.js platform/arch -> binary filename
const BINARY_MAP = {
  'darwin-arm64': 'telepipe-darwin-arm64',
  'darwin-x64': 'telepipe-darwin-x64',
  'linux-x64': 'telepipe-linux-x64',
  'win32-x64': 'telepipe-windows-x64.exe'
};

// Output binary name
const OUTPUT_BINARY = process.platform === 'win32' ? 'telepipe.exe' : 'telepipe';

function main() {
  const platformKey = `${process.platform}-${process.arch}`;
  const sourceBinary = BINARY_MAP[platformKey];

  if (!sourceBinary) {
    console.error(`\nUnsupported platform: ${platformKey}`);
    console.error('Supported platforms:');
    console.error('  - darwin-arm64  (macOS Apple Silicon)');
    console.error('  - darwin-x64    (macOS Intel)');
    console.error('  - linux-x64     (Linux x86_64)');
    console.error('  - win32-x64     (Windows x86_64)');
    console.error('');
    process.exit(1);
  }

  const binDir = path.join(__dirname, '..', 'bin');
  const sourcePath = path.join(binDir, sourceBinary);
  const destPath = path.join(binDir, OUTPUT_BINARY);

  // Check if source binary exists
  if (!fs.existsSync(sourcePath)) {
    console.error(`\nBinary not found: ${sourcePath}`);
    console.error('This may indicate a packaging issue.');
    console.error('');
    console.error('Expected binaries in bin/:');
    Object.values(BINARY_MAP).forEach(b => console.error(`  - ${b}`));
    console.error('');
    process.exit(1);
  }

  // Check if already set up (idempotent)
  if (fs.existsSync(destPath)) {
    const sourceStats = fs.statSync(sourcePath);
    const destStats = fs.statSync(destPath);

    // If dest is same size as source, probably already set up
    if (sourceStats.size === destStats.size) {
      console.log('Telepipe already configured for this platform.');
      return;
    }
  }

  try {
    // Copy the platform-specific binary to the generic name
    fs.copyFileSync(sourcePath, destPath);

    // Set executable permissions (Unix only)
    if (process.platform !== 'win32') {
      fs.chmodSync(destPath, 0o755);
    }

    console.log('');
    console.log('Telepipe installed successfully!');
    console.log(`Platform: ${platformKey}`);
    console.log('');
    console.log('Run "telepipe --help" to get started.');
    console.log('');

  } catch (error) {
    console.error(`\nFailed to set up binary: ${error.message}`);
    console.error('');

    // Provide manual instructions
    console.error('Manual setup:');
    if (process.platform === 'win32') {
      console.error(`  copy "${sourcePath}" "${destPath}"`);
    } else {
      console.error(`  cp "${sourcePath}" "${destPath}"`);
      console.error(`  chmod +x "${destPath}"`);
    }
    console.error('');
    process.exit(1);
  }
}

main();
