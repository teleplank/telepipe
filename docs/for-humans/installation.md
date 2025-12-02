# Installing Telepipe

Get Telepipe running in under a minute.

---

## Quick Install

```bash
npm install -g telepipe
```

That's it! Telepipe is now available globally.

---

## Platform-Specific Notes

### macOS

**Requirements:**
- Node.js 14+
- npm (comes with Node.js)

**Installation:**
```bash
npm install -g telepipe
```

**Verify:**
```bash
telepipe --help
```

**Install location:**
```bash
# Find where npm installed it
npm root -g
# Usually: /usr/local/lib/node_modules/telepipe/
```

**Install websocat (for Chrome CDP):**
```bash
brew install websocat
```

### Linux

**Requirements:**
- Node.js 14+
- npm
- glibc 2.31+
- Linux kernel 5.4+

**Installation:**
```bash
npm install -g telepipe
```

**May need sudo** depending on npm configuration:
```bash
sudo npm install -g telepipe
```

**Verify:**
```bash
telepipe --help
```

**Install websocat (for Chrome CDP):**
```bash
# Ubuntu/Debian
sudo apt install websocat

# Or via cargo
cargo install websocat

# Or download binary
# https://github.com/vi/websocat/releases
```

### Windows

**Status:** Not currently supported natively.

**Workaround:** Use WSL2 (Windows Subsystem for Linux)

**WSL2 Instructions:**
1. Install WSL2: `wsl --install`
2. Install Ubuntu from Microsoft Store
3. Open Ubuntu terminal
4. Follow Linux installation steps

---

## Verifying Installation

After install, verify Telepipe works:

```bash
telepipe --help
```

Should display:
```
Telepipe - TCP tunnels for AI agents

USAGE:
    telepipe <COMMAND>

COMMANDS:
    redirect    Spawn process with redirected I/O
    exec        Pipe and stream to a session
    connect     Attach to external TCP service
    stop        Terminate redirect session
    disconnect  Terminate connect session
    info        Show session state
```

---

## Finding the Binary

If you need to know where Telepipe is installed:

```bash
# Find npm global directory
npm root -g

# Telepipe is at:
# <that path>/telepipe/
```

Common locations:
- macOS: `/usr/local/lib/node_modules/telepipe/`
- Linux: `/usr/lib/node_modules/telepipe/`
- nvm: `~/.nvm/versions/node/<version>/lib/node_modules/telepipe/`

---

## Troubleshooting

### "command not found"

**Cause:** npm global bin is not in PATH.

**Solution:**
```bash
# Find npm prefix
npm config get prefix

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$(npm config get prefix)/bin:$PATH"
```

### Permission errors on npm install

**Cause:** npm trying to write to system directory.

**Solutions:**

**Option 1: Use nvm (recommended)**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install Node
nvm install node

# Now install telepipe (no sudo needed)
npm install -g telepipe
```

**Option 2: Configure npm prefix**
```bash
# Create local npm directory
mkdir ~/.npm-global

# Configure npm to use it
npm config set prefix '~/.npm-global'

# Add to PATH
export PATH=~/.npm-global/bin:$PATH

# Install telepipe
npm install -g telepipe
```

### Need specific version?

```bash
npm install -g telepipe@1.0.0
```

### Upgrading Telepipe

```bash
npm update -g telepipe
```

### Uninstalling

```bash
npm uninstall -g telepipe
```

---

## Dependencies

### Required
- **Node.js 14+** - Runtime
- **npm** - Package manager

### Optional (for Chrome CDP)
- **websocat** - WebSocket to TCP bridge
- **Chrome/Chromium** - Browser with DevTools Protocol

### Installing websocat

**macOS:**
```bash
brew install websocat
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install websocat
```

**Via Cargo (any platform):**
```bash
cargo install websocat
```

**Binary download:**
https://github.com/vi/websocat/releases

---

## Next Steps

Now that Telepipe is installed:

1. **Quick Start** - [quick-start.md](quick-start.md) - Connect to Chrome in 5 minutes
2. **Agent Setup** - [agent-setup.md](agent-setup.md) - Enable your AI
3. **Chrome DevTools** - [chrome-devtools.md](chrome-devtools.md) - Complete CDP setup
4. **Databases** - [database-connections.md](database-connections.md) - PostgreSQL, MySQL, Redis

---

## System Requirements

| Requirement | Minimum |
|-------------|---------|
| Node.js | 14+ |
| npm | 6+ |
| Disk space | 100MB |
| RAM | 50MB + ~5MB per session |

### Platform Support

| Platform | Status |
|----------|--------|
| macOS (ARM64) | ✅ Production-ready |
| macOS (x86_64) | ✅ Production-ready |
| Linux | ✅ Production-ready |
| Windows (WSL2) | ✅ Via WSL2 |
| Windows (native) | ❌ Not supported |

See [platform-support.md](../reference/platform-support.md) for details.
