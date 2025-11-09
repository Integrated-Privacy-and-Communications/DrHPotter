//! Security and isolation module

mod rate_limit;

pub use rate_limit::RateLimiter;

// Future security features:
// - Resource limits per session
// - Sandbox/seccomp filters
// - Connection timeout enforcement
