# Contributing to Telepipe

Thanks for your interest in contributing! Here's how to get started.

---

## Development Setup

### Prerequisites

- **Rust 1.70+** - Core implementation
- **Cargo** - Rust package manager
- **Node.js 14+** - npm wrapper
- **Git** - Version control

### Clone and Build

```bash
git clone https://github.com/teleplank/telepipe.git
cd telepipe
cargo build --release
```

### Run Tests

```bash
cargo test
```

All three test tiers must pass:
- **Level 1: Constitutional** - 100% required
- **Level 2: Production** - 100% required
- **Level 3: Chaos** - ≥95% required

### Verify Build

```bash
./target/release/telepipe --help
```

---

## Understanding the Codebase

### Project Structure

```
telepipe/
├── src/              # Rust source code
├── target/           # Build output
├── docs/             # Public documentation (uses "telepipe")
├── governance/       # Constitutional specs (uses "telepipe")
├── docs/for-agents/README.md    # AI agent instructions
├── README.md         # Main readme
├── package.json      # npm package
└── Cargo.toml        # Rust package
```

### Constitutional Governance

Telepipe uses a novel governance model:

- **Constitution defines behavior** - Specs in `governance/`
- **Tests are legislative truth** - Must match constitution
- **Code must match both** - Zero drift allowed

This means:
1. Read the constitution first
2. Tests enforce the constitution
3. Code implements the tests

See: [governance.md](../about/governance.md)

---

## Making Changes

### Code Style

```bash
# Format code
cargo fmt

# Check for warnings
cargo clippy

# Run all checks
cargo fmt && cargo clippy && cargo test
```

### Testing Requirements

**All changes must pass:**
1. `cargo test` - Unit tests
2. Level 1 tests (100%) - Constitutional compliance
3. Level 2 tests (100%) - Production scenarios
4. Level 3 tests (≥95%) - Chaos engineering

### Adding Features

1. **Update constitution** (if behavior changes)
2. **Write tests first** (TDAL - Test-Driven Architecture by Law)
3. **Implement feature**
4. **Verify all tests pass**

### Documentation

When updating docs, ensure consistency with the `telepipe` branding throughout.

### Commit Messages

```
type: short description

Longer explanation if needed.
```

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Test changes
- `refactor:` - Code refactoring
- `chore:` - Build, CI, etc.

Examples:
```
feat: add --timeout flag to exec command

fix: handle TCP reset during streaming

docs: update Chrome DevTools setup guide

test: add chaos tests for port exhaustion
```

---

## Pull Request Process

### Before Submitting

- [ ] All tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Commit messages clear

### PR Template

```markdown
## Summary

Brief description of what changed.

## Changes

- Added X
- Fixed Y
- Updated Z

## Testing

- [ ] cargo test passes
- [ ] Level 1-3 tests pass
- [ ] Manually tested

## Related Issues

Fixes #123
```

### Review Process

1. **Automated tests run** - CI must pass
2. **Code review** - Maintainer reviews
3. **Constitutional review** - If spec changed
4. **Merge** - After approval

---

## Testing Guidelines

### Unit Tests

```rust
#[test]
fn test_session_creation() {
    // Arrange
    let config = SessionConfig::new("myapp");

    // Act
    let session = Session::create(config);

    // Assert
    assert!(session.is_ok());
}
```

### Integration Tests

```rust
#[test]
fn test_redirect_exec_flow() {
    // Create session
    let result = Command::new("telepipe")
        .args(["redirect", "--id", "test", "--", "echo", "hello"])
        .output();

    // Verify success
    assert!(result.unwrap().status.success());

    // Exec
    let output = Command::new("telepipe")
        .args(["exec", "--id", "test"])
        .output();

    // Verify output
    assert!(output.unwrap().stdout.contains("hello"));
}
```

### Chaos Tests

Chaos tests verify behavior under stress:
- Port exhaustion
- FD exhaustion
- Concurrent operations
- Process crashes
- Network failures

---

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────┐
│              CLI Parser                  │
│  (redirect, exec, connect, etc.)        │
└────────────────┬────────────────────────┘
                 │
┌────────────────┴────────────────────────┐
│           Session Manager                │
│  (dictionary, lifecycle, cleanup)       │
└────────────────┬────────────────────────┘
                 │
┌────────────────┴────────────────────────┐
│            Supervisor                    │
│  (I/O multiplexing, TCP, child mgmt)   │
└─────────────────────────────────────────┘
```

### Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, CLI |
| `src/session.rs` | Session management |
| `src/supervisor.rs` | I/O supervision |
| `src/tcp.rs` | TCP handling |
| `src/error.rs` | Error types |

---

## Constitutional Documents

Read these to understand the system:

| Document | Purpose |
|----------|---------|
| `501 core spec` | Core operations |
| `515 cli reference` | CLI specification |
| `530 error model` | Error codes |
| `550 concurrency` | Locking model |
| `555 security` | Security model |
| `575 supervisor` | Supervisor architecture |

Location: `governance/the telepipe constitution/`

---

## Questions?

### GitHub Discussions
https://github.com/teleplank/telepipe/discussions

### GitHub Issues
https://github.com/teleplank/telepipe/issues

### Documentation
- [Architecture](../about/architecture.md)
- [Philosophy](../about/philosophy.md)
- [Governance](../about/governance.md)

---

## License

By contributing, you agree to license your contributions under the MIT license.

---

## Thank You!

Every contribution helps make Telepipe better for AI agents everywhere. We appreciate your time and effort!
