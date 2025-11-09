//! DrHPotter SSH Honeypot
//!
//! A minimal, secure SSH honeypot implementation in Rust.

use drhpotter::SshHoneypot;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "drhpotter=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting DrHPotter SSH Honeypot v{}", env!("CARGO_PKG_VERSION"));

    // Configuration (TODO: load from config file)
    let listen_addr = "0.0.0.0:2222";

    info!("Binding to {}", listen_addr);

    // Create and run honeypot
    let honeypot = SshHoneypot::new(listen_addr).await?;

    if let Err(e) = honeypot.run().await {
        error!("Honeypot error: {}", e);
        return Err(e);
    }

    Ok(())
}
