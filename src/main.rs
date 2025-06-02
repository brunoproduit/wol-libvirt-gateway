//! # WOL Libvirt Gateway
//!
//! A gateway service that provides Wake-on-LAN functionality for libvirt virtual machines.
//! This service allows you to wake up virtual machines by sending Wake-on-LAN packets
//! to their configured MAC addresses through a REST API interface.
//!
//! ## Usage
//!
//! ```bash
//! wol-libvirt-gateway -a 0.0.0.0:8080 -l qemu:///system
//! ```

use clap::Parser;
use log::info;

mod domain_xml;
mod error;
mod libvirt;
mod server;
mod tests;
mod wakeonlan;

/// Command line arguments for the WOL Libvirt Gateway service.
///
/// This struct defines the configuration options that can be passed to the service
/// when starting it from the command line.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The address and port to bind the HTTP server to.
    ///
    /// Format: `IP:PORT` (e.g., "127.0.0.1:8080" or "0.0.0.0:9090")
    /// Default: "127.0.0.1:9"
    #[arg(short, long, default_value = "127.0.0.1:9")]
    address: String,

    /// The libvirt connection URI to use for connecting to the hypervisor.
    ///
    /// Common URIs:
    /// - `qemu:///system` - Local QEMU system connection
    /// - `qemu:///session` - Local QEMU user session
    /// - `qemu+ssh://user@host/system` - Remote QEMU over SSH
    ///
    /// Default: "qemu:///system"
    #[arg(short, long, default_value = "qemu:///system")]
    libvirt_uri: String,
}

/// Main entry point for the WOL Libvirt Gateway service.
///
/// This function initializes the logging system, parses command line arguments,
/// and starts the gateway that provides the Wake-on-LAN for libvirt VMs.
///
/// # Environment Variables
///
/// - `RUST_LOG`: Controls the logging level (e.g., "debug", "info", "warn", "error")
///   Default: "info"
///
/// # Examples
///
/// Start the service with default settings:
/// ```bash
/// wol-libvirt-gateway
/// ```
///
/// Start the service on all interfaces with custom port:
/// ```bash
/// wol-libvirt-gateway --address 0.0.0.0:8080
/// ```
///
/// Connect to a remote libvirt instance:
/// ```bash
/// wol-libvirt-gateway --libvirt-uri qemu+ssh://user@host/system
/// ```
#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args = Cli::parse();

    info!(
        "WOL Libvirt Gateway v{} starting...",
        env!("CARGO_PKG_VERSION")
    );
    server::serve(args).await;
}
