//! Configuration file loading

use super::*;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Configuration loader
pub struct ConfigLoader {
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Current directory
        search_paths.push(PathBuf::from("./drhpotter.toml"));

        // User config directory
        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("drhpotter/config.toml"));
        }

        // System config directory
        search_paths.push(PathBuf::from("/etc/drhpotter/config.toml"));

        Self { search_paths }
    }

    /// Load configuration from the first found file or use defaults
    pub fn load(&self) -> Result<Config> {
        // Try to find a config file
        for path in &self.search_paths {
            if path.exists() {
                info!("Loading configuration from {:?}", path);
                return self.from_file(path.clone());
            }
        }

        // No config file found, use defaults
        warn!("No configuration file found, using defaults");
        info!("Searched paths: {:?}", self.search_paths);

        let config = Config::default();
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn from_file(&self, path: PathBuf) -> Result<Config> {
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", path, e))?;

        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file {:?}: {}", path, e))?;

        // Apply environment variable overrides
        self.apply_env_overrides(&mut config);

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&self, config: &mut Config) {
        // Server overrides
        if let Ok(port) = std::env::var("DRHPOTTER_SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                info!("Overriding server port from environment: {}", port_num);
                config.server.port = port_num;
            }
        }

        if let Ok(addr) = std::env::var("DRHPOTTER_SERVER_LISTEN_ADDR") {
            info!("Overriding listen address from environment: {}", addr);
            config.server.listen_addr = addr;
        }

        // Logging overrides
        if let Ok(level) = std::env::var("DRHPOTTER_LOG_LEVEL") {
            info!("Overriding log level from environment: {}", level);
            config.logging.level = level;
        }

        if let Ok(format) = std::env::var("DRHPOTTER_LOG_FORMAT") {
            info!("Overriding log format from environment: {}", format);
            config.logging.format = format;
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_defaults() {
        let loader = ConfigLoader::new();
        let config = loader.load();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server.port, 2222);
    }

    #[test]
    fn test_load_from_valid_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[server]
listen_addr = "127.0.0.1"
port = 3333
max_connections = 50
session_timeout_secs = 900
auth_delay_secs = 1

[security]
rate_limit_enabled = true
max_connections_per_ip = 5
rate_limit_window_secs = 30
whitelist_ips = []
blacklist_ips = []

[logging]
level = "debug"
format = "json"
output = "stdout"

[storage]
enabled = true
backend = "file"

[storage.file]
base_path = "./data"
sessions_dir = "./data/sessions"
files_dir = "./data/captured_files"

[shell]
hostname = "test-honeypot"
history_enabled = true
max_history = 500
banner = "Test Banner\n"

[capture]
capture_downloads = true
max_file_size_bytes = 5242880
"#
        )
        .unwrap();

        let loader = ConfigLoader::new();
        let config = loader.from_file(file.path().to_path_buf());
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.server.port, 3333);
        assert_eq!(config.server.listen_addr, "127.0.0.1");
        assert_eq!(config.server.max_connections, 50);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.shell.hostname, "test-honeypot");
        assert_eq!(config.shell.max_history, 500);
    }

    #[test]
    fn test_load_from_invalid_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid toml content [[[").unwrap();

        let loader = ConfigLoader::new();
        let config = loader.from_file(file.path().to_path_buf());
        assert!(config.is_err());
    }

    #[test]
    fn test_load_from_invalid_values() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[server]
listen_addr = "0.0.0.0"
port = 0

[security]
rate_limit_enabled = true
max_connections_per_ip = 10
rate_limit_window_secs = 60
whitelist_ips = []
blacklist_ips = []

[logging]
level = "info"
format = "json"
output = "stdout"

[storage]
enabled = true
backend = "file"

[storage.file]
base_path = "./data"
sessions_dir = "./data/sessions"
files_dir = "./data/captured_files"

[shell]
hostname = "honeypot"
history_enabled = true
max_history = 1000
banner = "Test\n"

[capture]
capture_downloads = true
max_file_size_bytes = 1048576
"#
        )
        .unwrap();

        let loader = ConfigLoader::new();
        let config = loader.from_file(file.path().to_path_buf());
        // Should fail validation due to port = 0
        assert!(config.is_err());
    }

    #[test]
    fn test_env_override_port() {
        std::env::set_var("DRHPOTTER_SERVER_PORT", "4444");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[server]
listen_addr = "0.0.0.0"
port = 2222
max_connections = 100
session_timeout_secs = 1800
auth_delay_secs = 2

[security]
rate_limit_enabled = true
max_connections_per_ip = 10
rate_limit_window_secs = 60
whitelist_ips = []
blacklist_ips = []

[logging]
level = "info"
format = "json"
output = "stdout"

[storage]
enabled = true
backend = "file"

[storage.file]
base_path = "./data"
sessions_dir = "./data/sessions"
files_dir = "./data/captured_files"

[shell]
hostname = "honeypot"
history_enabled = true
max_history = 1000
banner = "Test\n"

[capture]
capture_downloads = true
max_file_size_bytes = 10485760
"#
        )
        .unwrap();

        let loader = ConfigLoader::new();
        let config = loader.from_file(file.path().to_path_buf()).unwrap();

        // Port should be overridden by environment variable
        assert_eq!(config.server.port, 4444);

        std::env::remove_var("DRHPOTTER_SERVER_PORT");
    }
}
