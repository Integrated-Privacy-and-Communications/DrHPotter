//! Session logging implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use super::SessionLog;

/// Logger for SSH sessions
pub struct SessionLogger {
    log: Arc<Mutex<SessionLog>>,
}

impl SessionLogger {
    /// Create a new session logger
    pub fn new(addr: Option<SocketAddr>) -> Self {
        Self {
            log: Arc::new(Mutex::new(SessionLog::new(addr))),
        }
    }

    /// Log an authentication attempt
    pub async fn log_auth(&self, username: &str, password: &str, success: bool) {
        let mut log = self.log.lock().await;
        log.add_auth(username, password, success);

        info!(
            session_id = %log.session_id,
            username = username,
            password = password,
            success = success,
            "Authentication attempt"
        );
    }

    /// Log a command execution
    pub async fn log_command(&self, input: &str, output: &str) {
        let mut log = self.log.lock().await;
        log.add_command(input, output);

        info!(
            session_id = %log.session_id,
            command = input,
            "Command executed"
        );
    }

    /// Log a file download
    pub async fn log_download(&self, url: &str, sha256: &str, size: usize, path: &str) {
        let mut log = self.log.lock().await;
        log.add_download(url, sha256, size, path);

        info!(
            session_id = %log.session_id,
            url = url,
            sha256 = sha256,
            size = size,
            "File downloaded"
        );
    }

    /// Log a generic event
    pub async fn log_event(&self, event_type: &str, data: &str) {
        let mut log = self.log.lock().await;
        log.add_event(event_type, data);

        info!(
            session_id = %log.session_id,
            event_type = event_type,
            data = data,
            "Session event"
        );
    }

    /// End the session and write final log
    pub async fn end_session(&self) -> SessionLog {
        let mut log = self.log.lock().await;
        log.end();

        info!(
            session_id = %log.session_id,
            duration_seconds = (log.timestamp_end.unwrap() - log.timestamp_start).num_seconds(),
            commands = log.commands.len(),
            downloads = log.downloads.len(),
            "Session ended"
        );

        // TODO: Write to persistent storage
        // For now, return a clone
        log.clone()
    }

    /// Get the session ID
    pub async fn session_id(&self) -> uuid::Uuid {
        let log = self.log.lock().await;
        log.session_id
    }

    /// Get current session log (for testing)
    pub async fn get_log(&self) -> SessionLog {
        let log = self.log.lock().await;
        log.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logger_creation() {
        let logger = SessionLogger::new(None);
        let log = logger.get_log().await;
        assert_eq!(log.auth_attempts.len(), 0);
    }

    #[tokio::test]
    async fn test_log_auth() {
        let logger = SessionLogger::new(None);
        logger.log_auth("root", "toor", true).await;
        let log = logger.get_log().await;
        assert_eq!(log.auth_attempts.len(), 1);
    }

    #[tokio::test]
    async fn test_log_command() {
        let logger = SessionLogger::new(None);
        logger.log_command("whoami", "root\n").await;
        let log = logger.get_log().await;
        assert_eq!(log.commands.len(), 1);
    }
}
