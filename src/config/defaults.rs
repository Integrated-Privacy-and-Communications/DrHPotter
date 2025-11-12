//! Default configuration values

use super::*;

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0".to_string(),
            port: 2222,
            max_connections: 100,
            session_timeout_secs: 1800, // 30 minutes
            auth_delay_secs: 2,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_enabled: true,
            max_connections_per_ip: 10,
            rate_limit_window_secs: 60,
            whitelist_ips: Vec::new(),
            blacklist_ips: Vec::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_path: None,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "file".to_string(),
            file: Some(FileStorageConfig::default()),
        }
    }
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            base_path: "./data".to_string(),
            sessions_dir: "./data/sessions".to_string(),
            files_dir: "./data/captured_files".to_string(),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            hostname: "honeypot".to_string(),
            history_enabled: true,
            max_history: 1000,
            banner: "Welcome to Ubuntu 22.04.1 LTS (GNU/Linux 5.15.0-58-generic x86_64)\n\n\
                     Last login: Sat Nov  9 10:30:15 2025 from 192.168.1.1\n".to_string(),
        }
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            capture_downloads: true,
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_defaults() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.port, 2222);
        assert_eq!(cfg.listen_addr, "0.0.0.0");
        assert_eq!(cfg.max_connections, 100);
    }

    #[test]
    fn test_security_defaults() {
        let cfg = SecurityConfig::default();
        assert!(cfg.rate_limit_enabled);
        assert_eq!(cfg.max_connections_per_ip, 10);
    }

    #[test]
    fn test_logging_defaults() {
        let cfg = LoggingConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, "json");
        assert_eq!(cfg.output, "stdout");
    }

    #[test]
    fn test_storage_defaults() {
        let cfg = StorageConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.backend, "file");
        assert!(cfg.file.is_some());
    }

    #[test]
    fn test_shell_defaults() {
        let cfg = ShellConfig::default();
        assert_eq!(cfg.hostname, "honeypot");
        assert!(cfg.history_enabled);
        assert_eq!(cfg.max_history, 1000);
    }

    #[test]
    fn test_capture_defaults() {
        let cfg = CaptureConfig::default();
        assert!(cfg.capture_downloads);
        assert_eq!(cfg.max_file_size_bytes, 10 * 1024 * 1024);
    }
}
