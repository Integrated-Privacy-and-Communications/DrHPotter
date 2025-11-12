//! Configuration management for DrHPotter

mod defaults;
mod file;
mod validation;

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

pub use defaults::*;
pub use file::ConfigLoader;
pub use validation::*;

use crate::Result;

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Shell configuration
    pub shell: ShellConfig,
    /// Capture configuration
    pub capture: CaptureConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Listen address (e.g., "0.0.0.0")
    pub listen_addr: String,
    /// Port to bind to
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Authentication delay in seconds
    pub auth_delay_secs: u64,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Enable rate limiting
    pub rate_limit_enabled: bool,
    /// Max connections per IP
    pub max_connections_per_ip: usize,
    /// Rate limit window in seconds
    pub rate_limit_window_secs: u64,
    /// IP whitelist (never rate limit)
    #[serde(default)]
    pub whitelist_ips: Vec<String>,
    /// IP blacklist (immediately reject)
    #[serde(default)]
    pub blacklist_ips: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    pub level: String,
    /// Log format: json, pretty
    pub format: String,
    /// Log output: stdout, file
    pub output: String,
    /// Log file path (if output = file)
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Enable persistent storage
    pub enabled: bool,
    /// Storage backend: file, sqlite
    pub backend: String,
    /// File storage configuration
    #[serde(default)]
    pub file: Option<FileStorageConfig>,
}

/// File storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileStorageConfig {
    /// Base path for storage
    pub base_path: String,
    /// Sessions directory
    pub sessions_dir: String,
    /// Captured files directory
    pub files_dir: String,
}

/// Shell configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellConfig {
    /// Hostname to present
    pub hostname: String,
    /// Enable command history
    pub history_enabled: bool,
    /// Maximum commands to track
    pub max_history: usize,
    /// Welcome banner
    pub banner: String,
}

/// Capture configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaptureConfig {
    /// Capture downloaded files
    pub capture_downloads: bool,
    /// Maximum file size to capture (bytes)
    pub max_file_size_bytes: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            shell: ShellConfig::default(),
            capture: CaptureConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        ConfigLoader::new().load()
    }

    /// Load configuration from specific path
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        ConfigLoader::new().from_file(path.into())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        Validator::new().validate(self)
    }

    /// Get the socket address for binding
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let ip: IpAddr = self.server.listen_addr.parse()
            .map_err(|e| format!("Invalid listen address: {}", e))?;
        Ok(SocketAddr::new(ip, self.server.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 2222);
        assert_eq!(config.server.listen_addr, "0.0.0.0");
    }

    #[test]
    fn test_socket_addr() {
        let config = Config::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), 2222);
    }

    #[test]
    fn test_invalid_socket_addr() {
        let mut config = Config::default();
        config.server.listen_addr = "invalid".to_string();
        assert!(config.socket_addr().is_err());
    }

    #[test]
    fn test_validation_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_port() {
        let mut config = Config::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }
}
