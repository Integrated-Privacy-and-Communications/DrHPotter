//! SSH Server implementation for the honeypot

mod handler;
mod session;

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn, error};

use crate::security::RateLimiter;
use crate::Result;

pub use handler::Handler;
pub use session::SessionInfo;

/// Main SSH honeypot server
pub struct SshHoneypot {
    listen_addr: String,
    server_key: russh_keys::key::KeyPair,
    rate_limiter: Arc<RateLimiter>,
}

impl SshHoneypot {
    /// Create a new SSH honeypot server
    pub async fn new(listen_addr: &str) -> Result<Self> {
        info!("Initializing SSH honeypot");

        // Generate ephemeral server key
        let server_key = russh_keys::key::KeyPair::generate_ed25519()
            .ok_or("Failed to generate server key")?;

        // Initialize rate limiter (10 connections per IP per minute)
        let rate_limiter = Arc::new(RateLimiter::new(10, 60));

        Ok(Self {
            listen_addr: listen_addr.to_string(),
            server_key,
            rate_limiter,
        })
    }

    /// Run the honeypot server
    pub async fn run(self) -> Result<()> {
        let config = Arc::new(russh::server::Config {
            auth_rejection_time: std::time::Duration::from_secs(2),
            ..Default::default()
        });

        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("SSH honeypot listening on {}", self.listen_addr);

        let rate_limiter = self.rate_limiter.clone();

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);

                    // Check rate limiting
                    if !rate_limiter.check_and_record(addr.ip()).await {
                        warn!("Rate limit exceeded for {}, dropping connection", addr);
                        continue;
                    }

                    let config = config.clone();
                    let handler = Handler::new(Some(addr));

                    tokio::spawn(async move {
                        if let Err(e) = russh::server::run_stream(
                            config,
                            stream,
                            handler,
                        ).await {
                            error!("Connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_honeypot_creation() {
        let honeypot = SshHoneypot::new("127.0.0.1:0").await;
        assert!(honeypot.is_ok());
    }
}
