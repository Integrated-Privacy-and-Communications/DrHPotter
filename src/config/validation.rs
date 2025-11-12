//! Configuration validation

use super::*;

/// Configuration validator
pub struct Validator;

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a configuration
    pub fn validate(&self, config: &Config) -> Result<()> {
        self.validate_server(&config.server)?;
        self.validate_security(&config.security)?;
        self.validate_logging(&config.logging)?;
        self.validate_storage(&config.storage)?;
        self.validate_shell(&config.shell)?;
        self.validate_capture(&config.capture)?;
        Ok(())
    }

    fn validate_server(&self, config: &ServerConfig) -> Result<()> {
        // Validate port
        if config.port == 0 {
            return Err("Invalid port: 0 (must be 1-65535)".into());
        }

        // Validate listen address
        config.listen_addr.parse::<IpAddr>()
            .map_err(|_| format!("Invalid listen address: {}", config.listen_addr))?;

        // Validate max connections
        if config.max_connections == 0 {
            return Err("max_connections must be greater than 0".into());
        }

        Ok(())
    }

    fn validate_security(&self, config: &SecurityConfig) -> Result<()> {
        // Validate rate limit settings
        if config.rate_limit_enabled {
            if config.max_connections_per_ip == 0 {
                return Err("max_connections_per_ip must be greater than 0".into());
            }
            if config.rate_limit_window_secs == 0 {
                return Err("rate_limit_window_secs must be greater than 0".into());
            }
        }

        // Validate IP addresses in whitelist
        for ip in &config.whitelist_ips {
            ip.parse::<IpAddr>()
                .map_err(|_| format!("Invalid whitelist IP: {}", ip))?;
        }

        // Validate IP addresses in blacklist
        for ip in &config.blacklist_ips {
            ip.parse::<IpAddr>()
                .map_err(|_| format!("Invalid blacklist IP: {}", ip))?;
        }

        Ok(())
    }

    fn validate_logging(&self, config: &LoggingConfig) -> Result<()> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.level.as_str()) {
            return Err(format!(
                "Invalid log level: {} (must be one of: {})",
                config.level,
                valid_levels.join(", ")
            )
            .into());
        }

        // Validate log format
        let valid_formats = ["json", "pretty"];
        if !valid_formats.contains(&config.format.as_str()) {
            return Err(format!(
                "Invalid log format: {} (must be one of: {})",
                config.format,
                valid_formats.join(", ")
            )
            .into());
        }

        // Validate log output
        let valid_outputs = ["stdout", "file"];
        if !valid_outputs.contains(&config.output.as_str()) {
            return Err(format!(
                "Invalid log output: {} (must be one of: {})",
                config.output,
                valid_outputs.join(", ")
            )
            .into());
        }

        // If output is file, file_path must be set
        if config.output == "file" && config.file_path.is_none() {
            return Err("file_path must be set when output is 'file'".into());
        }

        Ok(())
    }

    fn validate_storage(&self, config: &StorageConfig) -> Result<()> {
        if config.enabled {
            // Validate backend
            let valid_backends = ["file", "sqlite"];
            if !valid_backends.contains(&config.backend.as_str()) {
                return Err(format!(
                    "Invalid storage backend: {} (must be one of: {})",
                    config.backend,
                    valid_backends.join(", ")
                )
                .into());
            }

            // If backend is file, file config must be set
            if config.backend == "file" && config.file.is_none() {
                return Err("file storage config must be set when backend is 'file'".into());
            }
        }

        Ok(())
    }

    fn validate_shell(&self, config: &ShellConfig) -> Result<()> {
        // Validate hostname (basic check)
        if config.hostname.is_empty() {
            return Err("hostname cannot be empty".into());
        }

        // Validate max history
        if config.max_history == 0 {
            return Err("max_history must be greater than 0".into());
        }

        Ok(())
    }

    fn validate_capture(&self, config: &CaptureConfig) -> Result<()> {
        // Validate max file size
        if config.max_file_size_bytes == 0 {
            return Err("max_file_size_bytes must be greater than 0".into());
        }

        // Reasonable upper limit: 100MB
        if config.max_file_size_bytes > 100 * 1024 * 1024 {
            return Err("max_file_size_bytes cannot exceed 100MB".into());
        }

        Ok(())
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        let validator = Validator::new();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = Config::default();
        config.server.port = 0;
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_invalid_listen_addr() {
        let mut config = Config::default();
        config.server.listen_addr = "invalid".to_string();
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_invalid_log_level() {
        let mut config = Config::default();
        config.logging.level = "invalid".to_string();
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_file_output_without_path() {
        let mut config = Config::default();
        config.logging.output = "file".to_string();
        config.logging.file_path = None;
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_invalid_whitelist_ip() {
        let mut config = Config::default();
        config.security.whitelist_ips.push("invalid".to_string());
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_empty_hostname() {
        let mut config = Config::default();
        config.shell.hostname = "".to_string();
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validate_file_size_too_large() {
        let mut config = Config::default();
        config.capture.max_file_size_bytes = 200 * 1024 * 1024; // 200MB
        let validator = Validator::new();
        assert!(validator.validate(&config).is_err());
    }
}
