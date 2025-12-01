//! WRAITH Protocol CLI
//!
//! Wire-speed Resilient Authenticated Invisible Transfer Handler

mod config;
mod progress;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use config::Config;
use progress::{TransferProgress, format_bytes};

/// WRAITH - Secure, fast, undetectable file transfer
#[derive(Parser)]
#[command(name = "wraith")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "~/.config/wraith/config.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file to a peer
    Send {
        /// File to send
        #[arg(required = true)]
        file: String,

        /// Recipient peer ID or address
        #[arg(required = true)]
        recipient: String,

        /// Obfuscation mode
        #[arg(long, default_value = "privacy")]
        mode: String,
    },

    /// Receive files from peers
    Receive {
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,

        /// Listen address
        #[arg(short, long, default_value = "0.0.0.0:0")]
        bind: String,
    },

    /// Run as background daemon
    Daemon {
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0:0")]
        bind: String,

        /// Enable relay mode
        #[arg(long)]
        relay: bool,
    },

    /// Show connection status
    Status,

    /// List connected peers
    Peers,

    /// Generate a new identity keypair
    Keygen {
        /// Output file for private key
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .init();

    // Load configuration
    let config_path = PathBuf::from(&cli.config);
    let config = if config_path.exists() {
        Config::load(&config_path)?
    } else if config_path == Config::default_path() {
        Config::load_or_default()?
    } else {
        Config::load(&config_path)? // Will fail with proper error
    };

    // Validate configuration
    config.validate()?;

    match cli.command {
        Commands::Send {
            file,
            recipient,
            mode,
        } => {
            send_file(PathBuf::from(file), recipient, mode, &config).await?;
        }
        Commands::Receive { output, bind } => {
            receive_files(PathBuf::from(output), bind, &config).await?;
        }
        Commands::Daemon { bind, relay } => {
            run_daemon(bind, relay, &config).await?;
        }
        Commands::Status => {
            show_status(&config).await?;
        }
        Commands::Peers => {
            list_peers(&config).await?;
        }
        Commands::Keygen { output } => {
            generate_keypair(output, &config).await?;
        }
    }

    Ok(())
}

/// Send a file to a recipient
async fn send_file(
    file: PathBuf,
    recipient: String,
    mode: String,
    config: &Config,
) -> anyhow::Result<()> {
    tracing::info!("Sending {:?} to {} (mode: {})", file, recipient, mode);

    // Verify file exists
    if !file.exists() {
        anyhow::bail!("File not found: {:?}", file);
    }

    let file_size = std::fs::metadata(&file)?.len();
    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    println!("File: {}", file.display());
    println!("Size: {}", format_bytes(file_size));
    println!("Recipient: {}", recipient);
    println!("Obfuscation: {}", mode);

    // Create progress bar
    let progress = TransferProgress::new(file_size, filename);

    // Placeholder: Full implementation requires protocol integration
    // This demonstrates the CLI structure and progress tracking

    tracing::warn!("Full send implementation requires Phase 7 protocol integration");
    tracing::info!(
        "Would send using chunk_size={}, obfuscation={}",
        config.transfer.chunk_size,
        mode
    );

    progress.finish_with_message(
        "Send command structured (full implementation pending Phase 7)".to_string(),
    );

    Ok(())
}

/// Receive files from peers
async fn receive_files(output: PathBuf, bind: String, config: &Config) -> anyhow::Result<()> {
    tracing::info!("Receiving files to {:?} (listening on {})", output, bind);

    // Create output directory if it doesn't exist
    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }

    println!("Output directory: {}", output.display());
    println!("Listening on: {}", bind);
    println!(
        "Chunk size: {}",
        format_bytes(config.transfer.chunk_size as u64)
    );
    println!("Resume enabled: {}", config.transfer.enable_resume);

    // Placeholder: Full implementation requires protocol integration
    tracing::warn!("Full receive implementation requires Phase 7 protocol integration");

    println!("\nReady to receive files (implementation pending Phase 7)...");
    println!("Press Ctrl+C to stop");

    // Keep alive
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");

    Ok(())
}

/// Run daemon mode
async fn run_daemon(bind: String, relay: bool, config: &Config) -> anyhow::Result<()> {
    tracing::info!("Starting WRAITH daemon on {} (relay: {})", bind, relay);

    println!("WRAITH Daemon");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Listen: {}", bind);
    println!("Relay mode: {}", relay);
    println!("XDP: {}", config.network.enable_xdp);

    if config.network.enable_xdp {
        if let Some(iface) = &config.network.xdp_interface {
            println!("XDP interface: {}", iface);
        }
    }

    // Placeholder: Full implementation requires protocol integration
    tracing::warn!("Full daemon implementation requires Phase 7 protocol integration");

    println!("\nDaemon ready (implementation pending Phase 7)...");
    println!("Press Ctrl+C to stop");

    // Keep alive
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");

    Ok(())
}

/// Show node status
async fn show_status(config: &Config) -> anyhow::Result<()> {
    println!("WRAITH Protocol Status");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    println!("Configuration:");
    println!("  Listen: {}", config.network.listen_addr);
    println!("  Obfuscation: {}", config.obfuscation.default_level);
    println!(
        "  Chunk size: {}",
        format_bytes(config.transfer.chunk_size as u64)
    );
    println!("  Max concurrent: {}", config.transfer.max_concurrent);
    println!();

    println!("Network:");
    println!("  XDP: {}", config.network.enable_xdp);
    println!("  UDP fallback: {}", config.network.udp_fallback);
    println!();

    println!("Discovery:");
    println!(
        "  Bootstrap nodes: {}",
        config.discovery.bootstrap_nodes.len()
    );
    println!("  Relay servers: {}", config.discovery.relay_servers.len());
    println!();

    // Placeholder: Show runtime status when protocol is integrated
    tracing::warn!("Runtime status display requires Phase 7 protocol integration");

    Ok(())
}

/// List connected peers
async fn list_peers(config: &Config) -> anyhow::Result<()> {
    println!("Connected Peers:");
    println!();

    // Placeholder: Full implementation requires protocol integration
    tracing::warn!("Peer listing requires Phase 7 protocol integration");

    println!("No peers connected (implementation pending Phase 7)");
    println!();
    println!("Discovery configured:");
    println!(
        "  Bootstrap nodes: {}",
        config.discovery.bootstrap_nodes.len()
    );
    println!("  Relay servers: {}", config.discovery.relay_servers.len());

    Ok(())
}

/// Generate a new identity keypair
async fn generate_keypair(output: Option<String>, _config: &Config) -> anyhow::Result<()> {
    use wraith_crypto::signatures::SigningKey;

    println!("Generating new Ed25519 identity keypair...");

    let mut rng = rand_core::OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();

    println!("Public key: {}", hex::encode(verifying_key.to_bytes()));

    if let Some(path) = output {
        let output_path = PathBuf::from(path);

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save private key (simplified serialization)
        let private_bytes = signing_key.to_bytes();
        std::fs::write(&output_path, private_bytes)?;

        println!("Private key saved to: {}", output_path.display());
        println!("\n⚠️  Keep this file secure! It contains your private key.");
    } else {
        println!("\n⚠️  Private key not saved (use --output to save)");
    }

    Ok(())
}
