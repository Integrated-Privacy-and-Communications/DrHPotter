//! Session information tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// Information about an SSH session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session identifier
    pub session_id: Uuid,
    /// Client address
    pub client_addr: Option<SocketAddr>,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Session end time
    pub end_time: Option<DateTime<Utc>>,
    /// Username used for authentication
    pub username: Option<String>,
    /// Password used for authentication
    pub password: Option<String>,
    /// Number of authentication attempts
    pub auth_attempts: u32,
    /// Whether authentication succeeded
    pub auth_success: bool,
}

impl SessionInfo {
    /// Create new session info
    pub fn new(addr: Option<SocketAddr>) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            client_addr: addr,
            start_time: Utc::now(),
            end_time: None,
            username: None,
            password: None,
            auth_attempts: 0,
            auth_success: false,
        }
    }

    /// Mark session as ended
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }

    /// Get session duration in seconds
    pub fn duration_seconds(&self) -> i64 {
        let end = self.end_time.unwrap_or_else(Utc::now);
        (end - self.start_time).num_seconds()
    }
}
