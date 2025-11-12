# SPEC-002: Configuration System

**Status**: Draft
**Author**: Claude
**Created**: 2025-11-12
**Target**: v0.2.0
**Depends On**: SPEC-001

## Overview

Add TOML-based configuration file support to DrHPotter, enabling runtime configuration without recompilation. This allows operators to customize honeypot behavior, logging, storage, and security settings through a simple configuration file.

## Goals

1. Support TOML configuration files for all runtime settings
2. Provide sensible defaults when config file is missing
3. Allow environment variable overrides for sensitive settings
4. Validate configuration on startup with clear error messages
5. Support hot-reload of non-critical settings (future enhancement)

## Non-Goals (v0.2.0)

- GUI configuration editor
- Remote configuration management
- Multi-honeypot orchestration
- Configuration encryption (use file permissions)

## Configuration File Format

### Location Priority
1. Path specified via `--config` CLI argument
2. `./drhpotter.toml` (current directory)
3. `~/.config/drhpotter/config.toml` (user config)
4. `/etc/drhpotter/config.toml` (system config)
5. Built-in defaults if no file found

### TOML Structure

```toml
# drhpotter.toml - DrHPotter Configuration

[server]
# SSH server bind address and port
listen_addr = "0.0.0.0"
port = 2222

# Maximum concurrent connections
max_connections = 100

# Session timeout in seconds
session_timeout_secs = 1800  # 30 minutes

# Authentication rejection delay (prevents fingerprinting)
auth_delay_secs = 2

[security]
# Rate limiting
rate_limit_enabled = true
max_connections_per_ip = 10
rate_limit_window_secs = 60

# IP whitelist (never rate limit these)
whitelist_ips = []

# IP blacklist (immediately reject)
blacklist_ips = []

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: json, pretty
format = "json"

# Log output: stdout, file
output = "stdout"

# Log file path (if output = "file")
file_path = "/var/log/drhpotter/honeypot.log"

# Rotate logs
rotate = true
max_size_mb = 100
max_files = 10

[storage]
# Enable persistent session storage
enabled = true

# Storage backend: file, sqlite, postgres
backend = "file"

# File storage configuration
[storage.file]
base_path = "./data"
sessions_dir = "./data/sessions"
files_dir = "./data/captured_files"

# SQLite configuration (when backend = "sqlite")
[storage.sqlite]
database_path = "./data/honeypot.db"

# PostgreSQL configuration (when backend = "postgres")
[storage.postgres]
host = "localhost"
port = 5432
database = "honeypot"
username = "honeypot"
# Password via env var: DRHPOTTER_DB_PASSWORD

[shell]
# Hostname to present
hostname = "honeypot"

# Enable command history
history_enabled = true

# Maximum commands to track per session
max_history = 1000

# Custom welcome banner
banner = """
Welcome to Ubuntu 22.04.1 LTS (GNU/Linux 5.15.0-58-generic x86_64)

Last login: Sat Nov  9 10:30:15 2025 from 192.168.1.1
"""

[capture]
# Capture downloaded files
capture_downloads = true

# Maximum file size to capture (bytes)
max_file_size_bytes = 10485760  # 10MB

# Capture network reconnaissance attempts
capture_port_scans = true

[alerting]
# Enable real-time alerting
enabled = false

# Alert channels: webhook, email, slack
channels = ["webhook"]

# Webhook configuration
[alerting.webhook]
url = "https://example.com/webhook"
timeout_secs = 5
retry_count = 3

# Email configuration
[alerting.email]
smtp_host = "smtp.example.com"
smtp_port = 587
from = "honeypot@example.com"
to = ["security@example.com"]
# SMTP password via env var: DRHPOTTER_SMTP_PASSWORD

# Slack configuration
[alerting.slack]
webhook_url = "https://hooks.slack.com/services/..."
channel = "#security-alerts"
```

## Behavior Specifications

### Configuration Loading

```
GIVEN the honeypot starts
WHEN a config file exists at ./drhpotter.toml
THEN the honeypot loads settings from the file
AND validates all settings
AND reports any errors with clear messages
AND falls back to defaults for missing values
```

### Environment Variable Override

```
GIVEN a config file specifies port = 2222
AND environment variable DRHPOTTER_SERVER_PORT=3333 is set
WHEN the honeypot starts
THEN it binds to port 3333 (env var takes precedence)
```

### Configuration Validation

```
GIVEN a config file with port = 99999
WHEN the honeypot loads the config
THEN it returns a validation error
AND reports "Invalid port: 99999 (must be 1-65535)"
AND refuses to start
```

### Missing Configuration

```
GIVEN no config file exists
WHEN the honeypot starts
THEN it uses built-in defaults
AND logs "No config file found, using defaults"
AND continues normal operation
```

## Implementation Plan

### 1. Configuration Module (`src/config/`)

```
src/config/
├── mod.rs           # Public API
├── file.rs          # TOML file loading
├── validation.rs    # Config validation
└── defaults.rs      # Default values
```

### 2. Configuration Struct

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub storage: StorageConfig,
    pub shell: ShellConfig,
    pub capture: CaptureConfig,
    pub alerting: AlertingConfig,
}

impl Default for Config {
    fn default() -> Self {
        // Sensible defaults
    }
}
```

### 3. Environment Variable Mapping

```
DRHPOTTER_SERVER_PORT         -> server.port
DRHPOTTER_SERVER_LISTEN_ADDR  -> server.listen_addr
DRHPOTTER_LOG_LEVEL           -> logging.level
DRHPOTTER_DB_PASSWORD         -> storage.postgres.password
DRHPOTTER_SMTP_PASSWORD       -> alerting.email.password
```

### 4. CLI Arguments

```bash
drhpotter --config /path/to/config.toml
drhpotter --port 3333
drhpotter --log-level debug
drhpotter --help
drhpotter --version
```

## Testing Requirements

### Unit Tests
- Config file parsing (valid TOML)
- Config validation (invalid values)
- Default value population
- Environment variable override
- Config file priority (CLI > file > env > defaults)

### Integration Tests
- Load config and start server with custom port
- Reject invalid configuration
- Log appropriate messages for missing config

## Success Criteria

1. Honeypot can run without config file (defaults)
2. Config file is loaded from standard locations
3. All settings are configurable via TOML
4. Sensitive settings can use environment variables
5. Invalid configs are rejected with clear errors
6. CLI arguments override config file
7. Zero breaking changes to existing SPEC-001 behavior
8. Documentation includes example config file

## Dependencies

### New Crates
- `toml = "0.8"` - TOML parsing
- `serde = "1.0"` - Serialization (already have)
- `clap = { version = "4.5", features = ["derive"] }` - CLI parsing
- `config = "0.14"` - Config management (optional, evaluate)

## Migration Path

1. Add config support as optional (defaults maintain current behavior)
2. Update README with configuration examples
3. Provide `drhpotter.example.toml` template
4. Update `main.rs` to load and use config
5. All existing deployments continue working unchanged

## Future Enhancements (Post v0.2.0)

- Hot-reload for non-critical settings (SIGHUP handler)
- Configuration schema validation (JSON Schema)
- Configuration templates for common scenarios
- Configuration wizard CLI tool

## Security Considerations

1. **File Permissions**: Config files may contain sensitive data
   - Recommend 0600 permissions
   - Warn if world-readable

2. **Password Storage**: Never store plaintext passwords in config
   - Use environment variables
   - Consider external secret management

3. **Validation**: Strictly validate all inputs
   - Prevent path traversal in file paths
   - Validate IP addresses and ports
   - Sanitize all user-provided strings

## Documentation Updates

- `README.md`: Add configuration section
- `docs/configuration.md`: Detailed configuration guide
- `drhpotter.example.toml`: Full example config
- `docs/deployment.md`: Production deployment guide

## Example Usage

```bash
# Run with default config
drhpotter

# Run with custom config
drhpotter --config /etc/drhpotter/production.toml

# Override port via CLI
drhpotter --port 3333

# Override via environment
export DRHPOTTER_SERVER_PORT=3333
drhpotter

# Show current config
drhpotter --show-config
```

## Related Specs

- SPEC-001: SSH Honeypot Core (foundation)
- SPEC-003: Persistent Storage Backend (uses storage config)
- SPEC-004: Real-Time Alerting (uses alerting config)
