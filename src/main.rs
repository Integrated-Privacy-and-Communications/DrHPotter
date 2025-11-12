//! DrHPotter SSH Honeypot
//!
//! A minimal, secure SSH honeypot implementation in Rust.

use clap::Parser;
use drhpotter::{Config, SshHoneypot};
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Port to bind to (overrides config)
    #[arg(short, long)]
    port: Option<u16>,

    /// Log level (overrides config)
    #[arg(long, value_name = "LEVEL")]
    log_level: Option<String>,

    /// Show current configuration and exit
    #[arg(long)]
    show_config: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        info!("Loading configuration from {:?}", config_path);
        Config::from_file(config_path.clone())?
    } else {
        Config::load()?
    };

    // Apply CLI overrides
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(log_level) = &args.log_level {
        config.logging.level = log_level.clone();
    }

    // Initialize logging based on config
    let log_level = config.logging.level.clone();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("drhpotter={}", log_level).into()),
        )
        .with(
            if config.logging.format == "json" {
                tracing_subscriber::fmt::layer().json().boxed()
            } else {
                tracing_subscriber::fmt::layer().pretty().boxed()
            }
        )
        .init();

    info!("Starting DrHPotter SSH Honeypot v{}", env!("CARGO_PKG_VERSION"));

    // Show config and exit if requested
    if args.show_config {
        println!("Current configuration:");
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    // Validate configuration
    config.validate()?;

    let socket_addr = config.socket_addr()?;
    info!("Configuration loaded successfully");
    info!("Binding to {}", socket_addr);

    // Create and run honeypot
    let honeypot = SshHoneypot::new(&socket_addr.to_string()).await?;

    if let Err(e) = honeypot.run().await {
        error!("Honeypot error: {}", e);
        return Err(e);
    }

    Ok(())
}
