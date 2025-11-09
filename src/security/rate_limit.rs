//! Rate limiting for connections

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Connection record for rate limiting
#[derive(Debug, Clone)]
struct ConnectionRecord {
    count: usize,
    window_start: Instant,
}

/// Rate limiter to prevent abuse
pub struct RateLimiter {
    connections: Arc<Mutex<HashMap<IpAddr, ConnectionRecord>>>,
    max_connections: usize,
    window_seconds: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_connections` - Maximum connections allowed per IP
    /// * `window_seconds` - Time window in seconds
    pub fn new(max_connections: usize, window_seconds: u64) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
            window_seconds,
        }
    }

    /// Check if IP is allowed to connect and record the connection
    pub async fn check_and_record(&self, ip: IpAddr) -> bool {
        let mut connections = self.connections.lock().await;
        let now = Instant::now();

        // Clean up old entries periodically
        connections.retain(|_, record| {
            now.duration_since(record.window_start).as_secs() < self.window_seconds
        });

        // Check current IP
        match connections.get_mut(&ip) {
            Some(record) => {
                let elapsed = now.duration_since(record.window_start).as_secs();

                if elapsed >= self.window_seconds {
                    // Window expired, reset
                    record.count = 1;
                    record.window_start = now;
                    true
                } else if record.count < self.max_connections {
                    // Still within limits
                    record.count += 1;
                    true
                } else {
                    // Rate limit exceeded
                    false
                }
            }
            None => {
                // New IP, allow
                connections.insert(
                    ip,
                    ConnectionRecord {
                        count: 1,
                        window_start: now,
                    },
                );
                true
            }
        }
    }

    /// Get current connection count for an IP
    pub async fn get_count(&self, ip: IpAddr) -> usize {
        let connections = self.connections.lock().await;
        connections.get(&ip).map(|r| r.count).unwrap_or(0)
    }

    /// Clear all rate limit records (for testing)
    #[cfg(test)]
    pub async fn clear(&self) {
        let mut connections = self.connections.lock().await;
        connections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_rate_limiter_allows_initial() {
        let limiter = RateLimiter::new(5, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        assert!(limiter.check_and_record(ip).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_excess() {
        let limiter = RateLimiter::new(3, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // First 3 should succeed
        assert!(limiter.check_and_record(ip).await);
        assert!(limiter.check_and_record(ip).await);
        assert!(limiter.check_and_record(ip).await);

        // 4th should be blocked
        assert!(!limiter.check_and_record(ip).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, 60);
        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Different IPs should have independent limits
        assert!(limiter.check_and_record(ip1).await);
        assert!(limiter.check_and_record(ip1).await);
        assert!(!limiter.check_and_record(ip1).await);

        // ip2 should still be allowed
        assert!(limiter.check_and_record(ip2).await);
    }

    #[tokio::test]
    async fn test_get_count() {
        let limiter = RateLimiter::new(10, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        assert_eq!(limiter.get_count(ip).await, 0);

        limiter.check_and_record(ip).await;
        assert_eq!(limiter.get_count(ip).await, 1);

        limiter.check_and_record(ip).await;
        assert_eq!(limiter.get_count(ip).await, 2);
    }
}
