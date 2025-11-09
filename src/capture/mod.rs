//! Data capture and logging module

mod logger;
mod storage;

pub use logger::SessionLogger;
pub use storage::FileStorage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// Captured authentication attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAttempt {
    pub timestamp: DateTime<Utc>,
    pub username: String,
    pub password: String,
    pub success: bool,
}

/// Captured command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub timestamp: DateTime<Utc>,
    pub input: String,
    pub output: String,
}

/// Captured file download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDownload {
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub sha256: String,
    pub size_bytes: usize,
    pub path: String,
}

/// Complete session log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: Uuid,
    pub timestamp_start: DateTime<Utc>,
    pub timestamp_end: Option<DateTime<Utc>>,
    pub source_ip: Option<String>,
    pub source_port: Option<u16>,
    pub auth_attempts: Vec<AuthAttempt>,
    pub commands: Vec<CommandExecution>,
    pub downloads: Vec<FileDownload>,
    pub events: Vec<SessionEvent>,
}

/// Generic session event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub data: String,
}

impl SessionLog {
    /// Create a new session log
    pub fn new(addr: Option<SocketAddr>) -> Self {
        let (source_ip, source_port) = addr
            .map(|a| (Some(a.ip().to_string()), Some(a.port())))
            .unwrap_or((None, None));

        Self {
            session_id: Uuid::new_v4(),
            timestamp_start: Utc::now(),
            timestamp_end: None,
            source_ip,
            source_port,
            auth_attempts: Vec::new(),
            commands: Vec::new(),
            downloads: Vec::new(),
            events: Vec::new(),
        }
    }

    /// End the session
    pub fn end(&mut self) {
        self.timestamp_end = Some(Utc::now());
    }

    /// Add an authentication attempt
    pub fn add_auth(&mut self, username: &str, password: &str, success: bool) {
        self.auth_attempts.push(AuthAttempt {
            timestamp: Utc::now(),
            username: username.to_string(),
            password: password.to_string(),
            success,
        });
    }

    /// Add a command execution
    pub fn add_command(&mut self, input: &str, output: &str) {
        self.commands.push(CommandExecution {
            timestamp: Utc::now(),
            input: input.to_string(),
            output: output.to_string(),
        });
    }

    /// Add a file download
    pub fn add_download(&mut self, url: &str, sha256: &str, size: usize, path: &str) {
        self.downloads.push(FileDownload {
            timestamp: Utc::now(),
            url: url.to_string(),
            sha256: sha256.to_string(),
            size_bytes: size,
            path: path.to_string(),
        });
    }

    /// Add a generic event
    pub fn add_event(&mut self, event_type: &str, data: &str) {
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            data: data.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_log_creation() {
        let log = SessionLog::new(None);
        assert_eq!(log.auth_attempts.len(), 0);
        assert_eq!(log.commands.len(), 0);
    }

    #[test]
    fn test_add_auth() {
        let mut log = SessionLog::new(None);
        log.add_auth("root", "password123", true);
        assert_eq!(log.auth_attempts.len(), 1);
        assert_eq!(log.auth_attempts[0].username, "root");
    }

    #[test]
    fn test_add_command() {
        let mut log = SessionLog::new(None);
        log.add_command("ls -la", "total 48\ndrwxr-xr-x...");
        assert_eq!(log.commands.len(), 1);
        assert_eq!(log.commands[0].input, "ls -la");
    }
}
