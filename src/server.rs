//! Wake-on-LAN server module for handling incoming WOL packets and managing virtual machines.

use crate::{
    error::WolGatewayError, libvirt::find_and_start_vm_by_mac, wakeonlan::WakeOnLanPacket, Cli,
};
use log::{debug, error, info, warn};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use virt::connect::Connect;

/// Maximum expected size for a WOL packet (102 bytes minimum + 6 bytes password).
const WOL_BUFFER_SIZE: usize = 108;

/// Starts the WOL gateway server that listens for Wake-on-LAN packets and manages VMs.
///
/// This function establishes a connection to libvirt, binds a UDP socket to listen for
/// Wake-on-LAN packets, and processes incoming packets by attempting to start the
/// corresponding virtual machines identified by MAC address.
///
/// # Arguments
///
/// * `args` - CLI arguments containing the libvirt URI and listen address configuration
///
/// # Behavior
///
/// The function runs in an infinite loop, processing incoming UDP packets:
/// 1. Validates each packet as a proper WOL magic packet
/// 2. Extracts the target MAC address from valid packets
/// 3. Searches for a VM with a matching MAC address in libvirt
/// 4. Attempts to start the VM if found
///
/// # Errors
///
/// The function will log errors and exit early on:
/// - Failed libvirt connection
/// - Invalid listen address parsing
/// - UDP socket binding failures
/// - Critical UDP receive errors
///
/// Non-critical errors (invalid packets, VM not found) are logged but don't stop the server.
pub(crate) async fn serve(args: Cli) {
    info!("Attempting to connect to libvirt URI: {}", args.libvirt_uri);

    // Establish libvirt connection
    let conn = match Connect::open(Some(&args.libvirt_uri)) {
        Ok(conn) => {
            let hostname = conn.get_hostname().unwrap_or_else(|_| "N/A".to_string());
            info!("Successfully connected to libvirt host: {}", hostname);
            conn
        }
        Err(e) => {
            error!("{}", WolGatewayError::LibvirtConnectError(e));
            return;
        }
    };

    // Parse the listen address
    let listen_addr: SocketAddr = match args.address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!("{}", WolGatewayError::AddressParseError(e));
            return;
        }
    };

    // Bind UDP socket for receiving WOL packets
    let socket = match UdpSocket::bind(listen_addr).await {
        Ok(socket) => socket,
        Err(e) => {
            error!("{}", WolGatewayError::SocketBindError(e));
            return;
        }
    };
    info!("Listening for WOL packets on {}", listen_addr);

    // Buffer to hold incoming packet data
    let mut buf = [0_u8; WOL_BUFFER_SIZE];

    // Main packet processing loop
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src_addr)) => {
                debug!("Received {} bytes from {}", len, src_addr);

                // Process the received packet
                handle_packet(&conn, &buf[..len]).await;
            }
            Err(e) => {
                error!(
                    "Critical UDP receive error: {}",
                    WolGatewayError::UdpReceiveError(e)
                );
                return;
            }
        }
    }
}

/// Handles a single incoming packet by parsing it as a WOL packet and starting the target VM.
///
/// # Arguments
///
/// * `conn` - Reference to the libvirt connection
/// * `packet` - Raw packet data received from UDP socket
async fn handle_packet(conn: &Connect, packet: &[u8]) {
    match WakeOnLanPacket::parse(packet) {
        Ok(wol) => {
            let mac_address_str = wol.target_mac_string();
            info!("Received valid WOL packet for MAC: {}", mac_address_str);

            // Attempt to find and start the VM with the target MAC address
            match find_and_start_vm_by_mac(conn, &mac_address_str).await {
                Ok(()) => {
                    info!("Successfully started VM with MAC: {}", mac_address_str);
                }
                Err(e) => {
                    warn!("Failed to start VM for MAC {}: {}", mac_address_str, e);
                }
            }
        }
        Err(e) => {
            warn!("Received invalid WOL packet: {}", e);
        }
    }
}
