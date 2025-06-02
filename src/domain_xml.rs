//! Module for parsing MAC addresses from XML domain configurations.
//!
//! This module provides functionality to extract and validate MAC addresses
//! from libvirt domain XML configurations, specifically targeting network
//! interface definitions.

use crate::error::WolGatewayError;
use crate::wakeonlan::{mac_to_string, parse_mac_address_string};
use serde::Deserialize;

/// Root structure for deserializing domain XML that contains MAC address information.
///
/// This represents the top-level domain element from a libvirt XML configuration.
#[derive(Debug, Deserialize)]
struct DomainMacs {
    /// The devices section containing network interfaces and their MAC addresses.
    devices: DevicesMacs,
}

/// Container for device-related information in a domain XML.
///
/// This structure specifically focuses on network interfaces within the devices section.
#[derive(Debug, Deserialize)]
struct DevicesMacs {
    /// List of network interfaces, each containing MAC address information.
    /// Maps to the "interface" XML elements within the devices section.
    #[serde(rename = "interface", default)]
    interfaces: Vec<InterfaceMac>,
}

/// Represents a single network interface with its MAC address.
///
/// This structure corresponds to an interface element in the libvirt domain XML.
#[derive(Debug, Deserialize)]
struct InterfaceMac {
    /// The MAC address information for this interface.
    mac: MacAddressString,
}

/// Container for a MAC address string from XML.
///
/// This structure handles the XML attribute containing the actual MAC address value.
#[derive(Debug, Deserialize)]
struct MacAddressString {
    /// The MAC address string, extracted from the "address" XML attribute.
    #[serde(rename = "@address")]
    address: String,
}

/// Extracts and validates MAC addresses from a libvirt domain XML string.
///
/// This function parses the provided XML string to extract all network interface
/// MAC addresses, validates each address format, and returns them as a vector of
/// standardized MAC address strings.
///
/// # Arguments
///
/// * `xml` - A string slice containing the libvirt domain XML configuration
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of validated MAC address strings in standard format
/// * `Err(WolGatewayError)` - If XML parsing fails or any MAC address is invalid
pub(crate) fn get_mac_addresses(xml: &str) -> Result<Vec<String>, WolGatewayError> {
    let domain: DomainMacs =
        serde_xml_rs::from_str(xml).map_err(WolGatewayError::MacExtractionError)?;
    domain
        .devices
        .interfaces
        .into_iter()
        .map(|iface| {
            // Make sure the content is a valid mac address
            let mac = parse_mac_address_string(&iface.mac.address)?;
            Ok(mac_to_string(&mac))
        })
        .collect()
}
