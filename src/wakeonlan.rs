//! Wake-on-LAN packet parsing utilities.
//!
//! This module provides functionality to parse Wake-on-LAN (WOL) magic packets,
//! which are used to remotely wake up network devices. A valid WOL packet consists
//! of 6 bytes of 0xFF followed by 16 repetitions of the target MAC address,
//! optionally followed by a 4 or 6-byte password.

use crate::error::WolGatewayError;
use std::string::String;

/// Minimum size of a valid WOL packet in bytes (6 sync bytes + 16 * 6 MAC bytes).
pub(crate) const WOL_PACKET_MIN_SIZE: usize = 102;

/// Length of a MAC address in bytes.
const MAC_ADDR_LEN: usize = 6;

/// Type alias for a 6-byte MAC address.
pub(crate) type MacAddress = [u8; 6];

/// Represents a parsed Wake-on-LAN magic packet.
///
/// A WOL packet contains a synchronization stream of 6 0xFF bytes,
/// followed by 16 repetitions of the target MAC address, and an
/// optional password field.
#[derive(Debug)]
pub(crate) struct WakeOnLanPacket {
    /// The 6-byte synchronization stream (should be all 0xFF).
    _sync_stream: [u8; 6],
    /// Array of 16 identical MAC addresses.
    mac_addresses: [MacAddress; 16],
    /// Optional 6-byte password (4-byte passwords are padded with zeros).
    _password: Option<[u8; 6]>,
}

/// Converts MAC address bytes to a colon-separated hexadecimal string.
///
/// # Arguments
///
/// * `mac` - A reference to a 6-byte MAC address
///
/// # Returns
///
/// A string representation of the MAC address in the format "xx:xx:xx:xx:xx:xx"
pub(crate) fn mac_to_string(mac: &MacAddress) -> String {
    mac.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join(":")
}

/// Parses a MAC address string and returns a MacAddress.
///
/// # Arguments
///
/// * `mac_str` - A string representation of a MAC address in the format "xx:xx:xx:xx:xx:xx"
///
/// # Returns
///
/// `Result<MacAddress, WolGatewayError>` if the string is valid, error otherwise
pub(crate) fn parse_mac_address_string(mac_str: &str) -> Result<MacAddress, WolGatewayError> {
    let parts: Vec<&str> = mac_str.split(':').collect();

    if parts.len() != 6 {
        return Err(WolGatewayError::WakeOnLanParseError(format!(
            "Invalid MAC address format: expected 6 parts separated by colons, got {}",
            parts.len()
        )));
    }

    let mut mac = [0u8; 6];

    for (i, part) in parts.iter().enumerate() {
        if part.len() != 2 {
            return Err(WolGatewayError::WakeOnLanParseError(format!(
                "Invalid MAC address part '{}': each part must be exactly 2 hex characters",
                part
            )));
        }

        mac[i] = u8::from_str_radix(part, 16).map_err(|_| {
            WolGatewayError::WakeOnLanParseError(format!(
                "Invalid hex digit in MAC address part '{}'",
                part
            ))
        })?;
    }

    Ok(mac)
}

impl WakeOnLanPacket {
    /// Parses a raw packet and attempts to construct a `WakeOnLanPacket`.
    ///
    /// This method validates that the packet follows the WOL magic packet format:
    /// - At least 102 bytes long
    /// - Starts with 6 bytes of 0xFF (sync stream)
    /// - Contains 16 identical repetitions of a MAC address
    /// - Optionally contains a 4 or 6-byte password at the end
    ///
    /// # Arguments
    ///
    /// * `packet` - A byte slice containing the raw packet data
    ///
    /// # Returns
    ///
    /// `Result<WakeOnLanPacket, WolGatewayError>` if the packet is valid, error otherwise
    pub(crate) fn parse(packet: &[u8]) -> Result<Self, WolGatewayError> {
        if packet.len() < WOL_PACKET_MIN_SIZE {
            let error_msg = format!(
                "Packet too short for WOL: {} bytes, expected at least {}",
                packet.len(),
                WOL_PACKET_MIN_SIZE
            );
            return Err(WolGatewayError::WakeOnLanParseError(error_msg));
        }

        // Extract and validate sync header
        let sync_stream = packet.get(0..6).ok_or_else(|| {
            WolGatewayError::WakeOnLanParseError("Failed to get sync stream bytes".to_string())
        })?;

        // Check for 6 leading 0xFF bytes (sync stream)
        if !sync_stream.iter().all(|&b| b == 0xFF) {
            return Err(WolGatewayError::WakeOnLanParseError(
                "Packet does not start with 6 FF bytes (sync stream)".to_string(),
            ));
        }

        let mut sync_bytes = [0_u8; 6];
        sync_bytes.copy_from_slice(sync_stream);

        // Extract the first instance of the MAC address (bytes 6-11)
        let first_mac_bytes = packet.get(6..(6 + MAC_ADDR_LEN)).ok_or_else(|| {
            WolGatewayError::WakeOnLanParseError(
                "Failed to get first MAC address bytes".to_string(),
            )
        })?;

        // Get the MAC address portion of the packet
        let mac_portion = packet.get(6..(6 + (MAC_ADDR_LEN * 16))).ok_or_else(|| {
            WolGatewayError::WakeOnLanParseError(
                "Packet too short for MAC address portion".to_string(),
            )
        })?;

        // Create chunks iterator for MAC addresses
        let mac_chunks: Vec<_> = mac_portion.chunks_exact(MAC_ADDR_LEN).take(16).collect();

        // Ensure we have exactly 16 MAC address chunks
        if mac_chunks.len() != 16 {
            return Err(WolGatewayError::WakeOnLanParseError(format!(
                "Packet too short for 16 MAC repetitions, found {}",
                mac_chunks.len()
            )));
        }

        let mut mac_addresses = [MacAddress::default(); 16];

        for (i, mac_chunk) in mac_chunks.iter().enumerate() {
            // Verify this MAC matches the first one
            if *mac_chunk != first_mac_bytes {
                let error_msg = format!("MAC address repetition check failed at repetition {}", i);
                return Err(WolGatewayError::WakeOnLanParseError(error_msg));
            }

            // Copy the MAC into our array
            mac_addresses[i].copy_from_slice(mac_chunk);
        }
        Ok(WakeOnLanPacket {
            _sync_stream: sync_bytes,
            mac_addresses,
            _password: None,
        })
    }

    /// Returns the target MAC address as a formatted string.
    ///
    /// # Returns
    ///
    /// A string representation of the MAC address in the format "xx:xx:xx:xx:xx:xx"
    pub(crate) fn target_mac_string(&self) -> String {
        mac_to_string(&self.mac_addresses[0])
    }
}
