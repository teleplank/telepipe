# Platform Support

Telepipe platform compatibility and requirements.

---

## Fully Supported

### macOS (ARM64)

**Status:** ✅ Production-ready

**Verified on:**
- Apple Silicon (M1, M2, M3, M4)
- macOS 12 Monterey and later

**Installation:**
```bash
npm install -g telepipe
```

**Tests:** All three tiers pass (100%, 100%, ≥95%)

**Notes:**
- Uses `kqueue` for event handling
- Default FD limit: 256 (increase with `ulimit -n`)

---

### macOS (x86_64)

**Status:** ✅ Production-ready

**Verified on:**
- Intel Macs
- macOS 11 Big Sur and later

**Installation:**
```bash
npm install -g telepipe
```

**Tests:** All three tiers pass

**Notes:**
- Same behavior as ARM64
- Rosetta 2 not required (native binary)

---

### Linux

**Status:** ✅ Production-ready

**Verified on:**
- Ubuntu 22.04, 24.04
- Debian 11, 12
- Fedora 38+
- RHEL 8+
- Amazon Linux 2

**Requirements:**
- glibc 2.31+
- Linux kernel 5.4+

**Installation:**
```bash
npm install -g telepipe
```

**Tests:** All three tiers pass

**Notes:**
- Uses `epoll` for event handling
- FD limits in `/proc/sys/fs/file-max`
- May need sudo for npm global install

---

## Limited Support

### Windows (Native)

**Status:** ❌ Not currently supported

**Limitations:**
- Unix-specific syscalls used
- Path handling differences
- Process model differences

**Future:** Native Windows support planned

---

### Windows (WSL2)

**Status:** ✅ Supported via WSL2

**How to use:**
1. Install WSL2: `wsl --install`
2. Install Ubuntu from Microsoft Store
3. Open Ubuntu terminal
4. Follow Linux installation

**Installation:**
```bash
# Inside WSL2 Ubuntu
npm install -g telepipe
```

**Notes:**
- Full Linux compatibility
- Network access to host via `localhost`
- Chrome on Windows accessible from WSL2

---

### FreeBSD / OpenBSD

**Status:** ⚠️ Untested

**Expected:**
- Should work (POSIX-compliant)
- May need minor adjustments

**Contribution welcome!**

---

## Build Requirements

### macOS

**Development tools:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Build:**
```bash
cargo build --release
```

**Output:**
```
target/release/telepipe
```

---

### Linux

**Development tools:**
```bash
# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Build:**
```bash
cargo build --release
```

---

## Platform-Specific Notes

### macOS

**Event loop:** Uses `kqueue`
- Efficient event notification
- Native to BSD/macOS

**Process supervision:** Via `waitpid`
- Standard POSIX signal handling

**FD limits:**
```bash
# Check current limit
ulimit -n

# Increase (session)
ulimit -n 4096

# Increase (permanent) - add to /etc/launchd.conf
limit maxfiles 4096 unlimited
```

**Networking:**
- Binds to `127.0.0.1` by default
- Ephemeral ports: 49152-65535

---

### Linux

**Event loop:** Uses `epoll`
- High-performance event notification
- Scales to thousands of connections

**Process supervision:** Via `waitpid`
- Standard POSIX signal handling

**FD limits:**
```bash
# Check system limit
cat /proc/sys/fs/file-max

# Check user limit
ulimit -n

# Increase (session)
ulimit -n 4096

# Increase (permanent) - add to /etc/security/limits.conf
* soft nofile 4096
* hard nofile 65536
```

**Networking:**
- Binds to `127.0.0.1` by default
- Ephemeral ports: 49152-65535
- May need firewall rules for non-localhost

---

### WSL2

**Event loop:** Linux `epoll` (inside WSL2)

**Network access:**
```bash
# Access Windows services
curl http://localhost:9222  # Chrome on Windows

# Or use Windows IP
cat /etc/resolv.conf  # Get Windows IP
```

**Chrome from WSL2:**
```bash
# Start Chrome on Windows with debugging
# Then from WSL2:
WS_URL=$(curl -s http://localhost:9222/json/version | grep -o 'ws://[^"]*')
telepipe redirect --id chrome -- websocat --no-close --text "$WS_URL"
```

---

## Minimum Requirements

### All Platforms

| Requirement | Minimum |
|-------------|---------|
| Node.js | 14.0+ |
| npm | 6.0+ |
| Disk space | 100MB |
| RAM | 50MB base + ~5MB per session |

### Runtime Requirements

| Resource | Per Session |
|----------|-------------|
| File descriptors | 3 (redirect) / 1 (connect) |
| TCP ports | 3 (redirect) / 1 (connect) |
| Memory | ~5MB |

---

## Cross-Compilation

**Not recommended.** Build on the target platform for best results.

If you must cross-compile:

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build for target
cargo build --release --target x86_64-unknown-linux-gnu
```

**Targets available:**
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

---

## Docker

### Official Image

```dockerfile
FROM node:18-slim
RUN npm install -g telepipe
```

### With Chrome

```dockerfile
FROM node:18-slim

# Install Chrome
RUN apt-get update && apt-get install -y \
    chromium \
    websocat \
    && rm -rf /var/lib/apt/lists/*

# Install Telepipe
RUN npm install -g telepipe

# Start Chrome headless
CMD chromium --remote-debugging-port=9222 --headless=new --no-sandbox
```

### Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/telepipe /usr/local/bin/
CMD ["telepipe", "--help"]
```

---

## Troubleshooting Platform Issues

### macOS: "Operation not permitted"

**Cause:** Gatekeeper or SIP blocking

**Solution:**
```bash
# Allow execution
xattr -d com.apple.quarantine /path/to/telepipe
```

### Linux: "GLIBC not found"

**Cause:** System glibc too old

**Solution:**
- Upgrade to newer distro (Ubuntu 22.04+)
- Or build from source on target system

### WSL2: "Connection refused to localhost"

**Cause:** WSL2 network isolation

**Solution:**
```bash
# Use Windows IP instead of localhost
WIN_IP=$(cat /etc/resolv.conf | grep nameserver | cut -d' ' -f2)
curl http://$WIN_IP:9222/json/version
```

---

## Future Platforms

| Platform | Status | Timeline |
|----------|--------|----------|
| Windows native | Planned | TBD |
| FreeBSD | Possible | Community |
| OpenBSD | Possible | Community |
| Alpine Linux | Untested | May work |

---

## Platform Support Matrix

| Platform | Install | Build | Tests | Production |
|----------|---------|-------|-------|------------|
| macOS ARM64 | ✅ | ✅ | ✅ | ✅ |
| macOS x86_64 | ✅ | ✅ | ✅ | ✅ |
| Linux x86_64 | ✅ | ✅ | ✅ | ✅ |
| Linux ARM64 | ✅ | ✅ | ✅ | ✅ |
| Windows WSL2 | ✅ | ✅ | ✅ | ✅ |
| Windows native | ❌ | ❌ | ❌ | ❌ |
| FreeBSD | ⚠️ | ⚠️ | ⚠️ | ⚠️ |

✅ = Fully supported
⚠️ = Untested / community
❌ = Not supported
