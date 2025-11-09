# DrHPotter

A minimal, secure SSH honeypot written in Rust with zero unsafe code.

## Overview

DrHPotter is a high-interaction SSH honeypot designed to capture and analyze attacker behavior while maintaining strict security isolation. Built from scratch using Rust for memory safety and performance, it provides a realistic Linux-like environment to deceive attackers and collect valuable threat intelligence.

## Features

### ✨ Core Capabilities
- **High-Interaction SSH Honeypot** - Full SSH protocol support with realistic shell environment
- **Fake Shell Environment** - Emulates common Linux commands (ls, cd, cat, wget, curl, etc.)
- **Comprehensive Logging** - Structured JSON logs of all authentication attempts and commands
- **File Capture** - Downloads and stores malware/scripts for analysis with SHA256 hashing
- **Rate Limiting** - Prevents abuse with configurable connection limits per IP
- **Memory Safe** - Zero `unsafe` code blocks, leveraging Rust's safety guarantees

### 🔒 Security Design
- **Isolated Execution** - In-memory fake filesystem, no real system access
- **Resource Limits** - Per-session memory and timeout enforcement
- **No Network Access** - Downloaded files are captured but not executed
- **Defensive Coding** - All input treated as malicious

### 📊 Observability
- **Structured Logging** - JSON-formatted logs with tracing support
- **Session Recording** - Complete session replay capability
- **Attack Pattern Detection** - Capture and analyze attacker techniques
- **MITRE ATT&CK Mapping** - (Planned) Correlate attacks to frameworks

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              DrHPotter SSH Honeypot                  │
├─────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────┐  │
│  │    SSH Server (russh 0.45)                   │  │
│  │  • Authentication (always accepts)            │  │
│  │  • Channel management                         │  │
│  │  • Rate limiting (10 conn/min/IP)            │  │
│  └────────────────┬─────────────────────────────┘  │
│                   │                                 │
│  ┌────────────────▼─────────────────────────────┐  │
│  │    Fake Shell Environment                    │  │
│  │  • Command emulation (20+ commands)          │  │
│  │  • In-memory fake filesystem                 │  │
│  │  • Download capture (wget/curl)              │  │
│  └────────────────┬─────────────────────────────┘  │
│                   │                                 │
│  ┌────────────────▼─────────────────────────────┐  │
│  │    Logging & Capture Engine                  │  │
│  │  • Structured JSON logs                      │  │
│  │  • Session replay data                       │  │
│  │  • SHA256-indexed malware storage            │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites
- Rust 1.70+ (2021 edition)
- Linux/macOS/Windows

### Build
```bash
# Clone the repository
git clone https://github.com/Integrated-Privacy-and-Communications/DrHPotter
cd DrHPotter

# Build in release mode
cargo build --release

# Run tests (23 tests, all passing)
cargo test
```

### Run
```bash
# Run on default port 2222
cargo run

# Or run the binary directly
./target/release/drhpotter

# With custom logging level
RUST_LOG=drhpotter=debug cargo run
```

### Connect to Honeypot
```bash
# From another terminal
ssh root@localhost -p 2222
# Any password will be accepted!

# Try some commands
$ whoami
root
$ ls -la
total 48
drwxr-xr-x 2 root root 4096 Nov  9 10:30 .
drwxr-xr-x 2 root root 4096 Nov  9 10:30 ..
...
```

## Project Structure

```
DrHPotter/
├── docs/
│   └── specs/
│       └── SPEC-001-ssh-honeypot-core.md   # Feature specification
├── src/
│   ├── server/
│   │   ├── mod.rs           # SSH server implementation
│   │   ├── handler.rs       # Connection/session handler
│   │   └── session.rs       # Session info tracking
│   ├── shell/
│   │   ├── mod.rs           # Fake shell
│   │   ├── commands.rs      # Command implementations
│   │   └── filesystem.rs    # In-memory fake FS
│   ├── capture/
│   │   ├── mod.rs           # Data structures
│   │   ├── logger.rs        # Session logger
│   │   └── storage.rs       # File storage
│   ├── security/
│   │   ├── mod.rs           # Security module
│   │   └── rate_limit.rs    # Rate limiting
│   ├── lib.rs               # Library exports
│   └── main.rs              # Binary entry point
├── Cargo.toml               # Dependencies
└── README.md                # This file
```

## Development

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_shell_creation

# Run tests with output
cargo test -- --nocapture

# Check test coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Code Quality
```bash
# Check for issues
cargo clippy

# Format code
cargo fmt

# Security audit
cargo audit
```

### Spec-Driven Development
1. **Write Specification** - Document feature in `docs/specs/`
2. **Write Tests** - Create failing tests (RED)
3. **Implement** - Write minimal code to pass (GREEN)
4. **Refactor** - Improve code quality (REFACTOR)
5. **Commit** - With spec reference in message

## Configuration

Currently configured via code (config file support planned):

```rust
// src/main.rs
let listen_addr = "0.0.0.0:2222";  // Bind address
let max_conn_per_ip = 10;           // Rate limit
let window_seconds = 60;            // Rate limit window
```

## Logging

Logs are output as structured JSON to stdout:

```json
{
  "timestamp": "2025-11-09T10:30:15Z",
  "level": "INFO",
  "fields": {
    "message": "Authentication attempt",
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "root",
    "password": "password123",
    "success": true
  }
}
```

Redirect to file:
```bash
cargo run 2>&1 | tee honeypot.log
```

## Captured Data

### Session Logs
Located in: `logs/` (when persistent storage is enabled)

### Downloaded Files
Located in: `captured_files/` (SHA256 as filename)

Example:
```
captured_files/
└── abc123def456... (SHA256 hash)
```

## Supported Commands

The fake shell currently supports:

- **Navigation**: `pwd`, `cd`, `ls`
- **File Operations**: `cat`, `touch`, `rm`, `cp`, `mv`, `mkdir`
- **System Info**: `whoami`, `id`, `uname`, `hostname`, `ps`, `env`
- **Network**: `ifconfig`, `ip`, `netstat`, `wget`, `curl`
- **Permissions**: `chmod`, `chown`
- **Utilities**: `echo`, `history`, `exit`

## Roadmap

### v0.1.0 (Current)
- [x] SSH honeypot core
- [x] Fake shell with 20+ commands
- [x] Rate limiting
- [x] Structured logging
- [x] File capture
- [x] 90%+ test coverage

### v0.2.0 (Planned)
- [ ] Persistent storage backend
- [ ] Configuration file support
- [ ] Real-time alerting (webhooks)
- [ ] Enhanced command coverage
- [ ] Public key authentication
- [ ] SFTP/SCP honeypot

### v0.3.0 (Future)
- [ ] Web dashboard for visualization
- [ ] Machine learning for attack classification
- [ ] Multi-node deployment support
- [ ] WASM-based log analyzer
- [ ] Integration with SIEM platforms

## Security Considerations

### Deployment
- **Run as non-root** - Use port forwarding (e.g., `iptables -t nat -A PREROUTING -p tcp --dport 22 -j REDIRECT --to-port 2222`)
- **Isolate Network** - Deploy in DMZ or separate VLAN
- **Monitor Resources** - Set system-level limits (ulimit, cgroups)
- **Legal Compliance** - Ensure honeypot deployment complies with local laws

### Limitations
- Not a replacement for real security measures
- Intended for research and threat intelligence
- No warranty - use at your own risk

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests first (TDD)
4. Commit your changes (`git commit -m '[SPEC-XXX] Add feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Authors

- **DrHPotter Contributors** - [Integrated-Privacy-and-Communications](https://github.com/Integrated-Privacy-and-Communications)

## Acknowledgments

- Built with [russh](https://github.com/warp-tech/russh) - Pure Rust SSH implementation
- Inspired by [Cowrie](https://github.com/cowrie/cowrie) and [Dionaea](https://github.com/DinoTools/dionaea)
- Follows [OWASP Honeypot Best Practices](https://owasp.org/)

## References

- [SPEC-001: SSH Honeypot Core](docs/specs/SPEC-001-ssh-honeypot-core.md)
- [Honeypot Research Papers](docs/research/) (Coming soon)
- [Attack Pattern Analysis](docs/analysis/) (Coming soon)

---

**⚠️ WARNING**: This is a honeypot. It intentionally accepts all authentication attempts and allows attacker interaction. Only deploy in controlled, isolated environments.
