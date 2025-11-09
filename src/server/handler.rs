//! SSH connection handler implementation

use async_trait::async_trait;
use russh::server::{Auth, Handler as RusshHandler, Msg, Session};
use russh::{Channel, ChannelId, MethodSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::capture::SessionLogger;
use crate::shell::FakeShell;
use super::SessionInfo;

/// Handler for individual SSH connections
pub struct Handler {
    addr: Option<SocketAddr>,
    session_info: Arc<Mutex<SessionInfo>>,
    shell: Arc<Mutex<FakeShell>>,
    logger: Arc<SessionLogger>,
}

impl Handler {
    /// Create a new handler for a client connection
    pub fn new(addr: Option<SocketAddr>) -> Self {
        let session_info = Arc::new(Mutex::new(SessionInfo::new(addr)));
        let logger = Arc::new(SessionLogger::new(addr));
        let shell = Arc::new(Mutex::new(FakeShell::new()));

        Self {
            addr,
            session_info,
            shell,
            logger,
        }
    }
}

#[async_trait]
impl RusshHandler for Handler {
    type Error = russh::Error;

    async fn auth_none(&mut self, user: &str) -> Result<Auth, Self::Error> {
        info!("Auth attempt (none) from {:?}, user: {}", self.addr, user);

        // Log the authentication attempt
        let mut session = self.session_info.lock().await;
        session.username = Some(user.to_string());

        // Reject 'none' auth, require password
        Ok(Auth::Reject {
            proceed_with_methods: Some(MethodSet::PASSWORD),
        })
    }

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error> {
        info!(
            "Auth attempt (password) from {:?}, user: {}, password: {}",
            self.addr, user, password
        );

        // Log authentication attempt
        let mut session = self.session_info.lock().await;
        session.username = Some(user.to_string());
        session.password = Some(password.to_string());
        session.auth_attempts += 1;

        self.logger
            .log_auth(user, password, true)
            .await;

        // Simulate realistic delay (prevent fingerprinting)
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Always accept (this is a honeypot!)
        info!("Accepting authentication for user: {}", user);
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        info!("Channel session opened from {:?}", self.addr);
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        _channel: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!(
            "PTY request from {:?}: term={}, cols={}, rows={}",
            self.addr, term, col_width, row_height
        );

        self.logger
            .log_event("pty_request", &format!("term={}, cols={}, rows={}", term, col_width, row_height))
            .await;

        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!("Shell request from {:?}", self.addr);

        self.logger.log_event("shell_request", "").await;

        // Send welcome banner
        let banner = b"Welcome to Ubuntu 22.04.1 LTS (GNU/Linux 5.15.0-58-generic x86_64)\n\n\
                      Last login: Sat Nov  9 10:30:15 2025 from 192.168.1.1\n$ ";
        session.data(channel, banner.to_vec().into());

        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data);
        info!("Exec request from {:?}: {}", self.addr, command);

        // Execute command in fake shell
        let mut shell = self.shell.lock().await;
        let output = shell.execute(&command).await;

        self.logger
            .log_command(&command, &output)
            .await;

        // Send output
        session.data(channel, output.as_bytes().to_vec().into());
        session.exit_status_request(channel, 0);
        session.eof(channel);
        session.close(channel);

        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let input = String::from_utf8_lossy(data);

        // Handle special characters
        if input.contains('\r') || input.contains('\n') {
            let command = input.trim();
            if !command.is_empty() {
                info!("Command from {:?}: {}", self.addr, command);

                // Execute command
                let mut shell = self.shell.lock().await;
                let output = shell.execute(command).await;

                self.logger
                    .log_command(command, &output)
                    .await;

                // Send output
                session.data(channel, output.as_bytes().to_vec().into());
                session.data(channel, b"$ ".to_vec().into());
            } else {
                // Just newline, re-prompt
                session.data(channel, b"$ ".to_vec().into());
            }
        } else {
            // Echo back the character (for interactive shell)
            session.data(channel, data.to_vec().into());
        }

        Ok(())
    }

    async fn channel_close(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!("Channel closed from {:?}", self.addr);
        self.logger.log_event("channel_close", "").await;
        Ok(())
    }
}
