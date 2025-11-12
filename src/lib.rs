//! DrHPotter - A minimal, secure SSH honeypot
//!
//! This library provides the core functionality for running an SSH honeypot
//! that logs attacker behavior while maintaining strict security isolation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod capture;
pub mod config;
pub mod security;
pub mod server;
pub mod shell;

pub use config::Config;
pub use server::SshHoneypot;

/// Result type for DrHPotter operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
