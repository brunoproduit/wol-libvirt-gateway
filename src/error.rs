use std::error::Error;
use std::fmt;

/// Error types that can occur during Wake-on-LAN gateway operations.
///
/// This enum represents various error conditions that may arise when operating
/// a Wake-on-LAN gateway, including network address parsing errors, socket
/// binding issues, libvirt connection problems, and UDP communication errors.
pub(crate) enum WolGatewayError {
    /// Error occurred while parsing a network address.
    ///
    /// This variant wraps `std::net::AddrParseError` which is returned when
    /// attempting to parse an invalid IP address or socket address string.
    AddressParseError(std::net::AddrParseError),

    /// Error occurred while binding to a socket.
    ///
    /// This variant wraps `std::io::Error` specifically for socket binding
    /// operations, such as when attempting to bind to an address that is
    /// already in use.
    SocketBindError(std::io::Error),

    /// Error occurred while connecting to libvirt.
    ///
    /// This variant wraps `virt::error::Error` which represents various
    /// libvirt connection and operation errors.
    LibvirtConnectError(virt::error::Error),

    /// Error occurred during UDP receive operations.
    ///
    /// This variant wraps `std::io::Error` specifically for UDP socket
    /// receive operations.
    UdpReceiveError(std::io::Error),

    /// No VM found with the specified MAC address.
    ///
    /// This variant indicates that no virtual machine was found with a network
    /// interface matching the requested MAC address.
    VmNotFound(String),

    /// Error occurred while listing libvirt domains.
    ///
    /// This variant wraps `virt::error::Error` for domain listing operations.
    DomainListError(virt::error::Error),

    /// Error occurred while retrieving domain XML description.
    ///
    /// This variant wraps `virt::error::Error` for domain XML retrieval operations.
    DomainXmlError(virt::error::Error),

    /// Error occurred while extracting MAC addresses from domain XML.
    ///
    /// This variant contains the error message from MAC address extraction failures.
    MacExtractionError(serde_xml_rs::Error),

    /// Error occurred while retrieving domain UUID.
    ///
    /// This variant wraps `virt::error::Error` for domain UUID retrieval operations.
    DomainUuidError(virt::error::Error),

    /// Error occurred while looking up a domain by UUID.
    ///
    /// This variant wraps `virt::error::Error` for domain lookup operations.
    DomainLookupError(virt::error::Error),

    /// Error occurred while retrieving domain name.
    ///
    /// This variant wraps `virt::error::Error` for domain name retrieval operations.
    DomainNameError(virt::error::Error),

    /// Error occurred while retrieving domain state.
    ///
    /// This variant wraps `virt::error::Error` for domain state retrieval operations.
    DomainStateError(virt::error::Error),

    /// Error occurred while starting a domain.
    ///
    /// This variant wraps `virt::error::Error` for domain start operations.
    DomainStartError(virt::error::Error),

    /// Error occurred while resuming a domain.
    ///
    /// This variant wraps `virt::error::Error` for domain resume operations.
    DomainResumeError(virt::error::Error),

    /// Error occurred while parsing a wake-on-lan packet.
    ///
    /// This variant contains the specific parsing error as a string
    WakeOnLanParseError(String),
}

impl fmt::Display for WolGatewayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WolGatewayError::AddressParseError(e) => write!(f, "Address parsing error: {}", e),
            WolGatewayError::SocketBindError(e) => write!(f, "Socket bind error: {}", e),
            WolGatewayError::LibvirtConnectError(e) => write!(f, "Libvirt connection error: {}", e),
            WolGatewayError::UdpReceiveError(e) => write!(f, "UDP receive error: {}", e),
            WolGatewayError::VmNotFound(mac) => write!(f, "No VM found with MAC address: {}", mac),
            WolGatewayError::DomainListError(e) => write!(f, "Failed to list domains: {}", e),
            WolGatewayError::DomainXmlError(e) => write!(f, "Failed to get domain XML: {}", e),
            WolGatewayError::MacExtractionError(e) => {
                write!(f, "Failed to extract MAC addresses: {}", e)
            }
            WolGatewayError::DomainUuidError(e) => write!(f, "Failed to get domain UUID: {}", e),
            WolGatewayError::DomainLookupError(e) => write!(f, "Failed to lookup domain: {}", e),
            WolGatewayError::DomainNameError(e) => write!(f, "Failed to get domain name: {}", e),
            WolGatewayError::DomainStateError(e) => write!(f, "Failed to get domain state: {}", e),
            WolGatewayError::DomainStartError(e) => write!(f, "Failed to start domain: {}", e),
            WolGatewayError::DomainResumeError(e) => write!(f, "Failed to resume domain: {}", e),
            WolGatewayError::WakeOnLanParseError(e) => write!(f, "Parsing error: {}", e),
        }
    }
}

impl fmt::Debug for WolGatewayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WolGatewayError::AddressParseError(e) => write!(f, "Address parsing error: {}", e),
            WolGatewayError::SocketBindError(e) => write!(f, "Socket bind error: {}", e),
            WolGatewayError::LibvirtConnectError(e) => write!(f, "Libvirt connection error: {}", e),
            WolGatewayError::UdpReceiveError(e) => write!(f, "UDP receive error: {}", e),
            WolGatewayError::VmNotFound(mac) => write!(f, "No VM found with MAC address: {}", mac),
            WolGatewayError::DomainListError(e) => write!(f, "Failed to list domains: {}", e),
            WolGatewayError::DomainXmlError(e) => write!(f, "Failed to get domain XML: {}", e),
            WolGatewayError::MacExtractionError(e) => {
                write!(f, "Failed to extract MAC addresses: {}", e)
            }
            WolGatewayError::DomainUuidError(e) => write!(f, "Failed to get domain UUID: {}", e),
            WolGatewayError::DomainLookupError(e) => write!(f, "Failed to lookup domain: {}", e),
            WolGatewayError::DomainNameError(e) => write!(f, "Failed to get domain name: {}", e),
            WolGatewayError::DomainStateError(e) => write!(f, "Failed to get domain state: {}", e),
            WolGatewayError::DomainStartError(e) => write!(f, "Failed to start domain: {}", e),
            WolGatewayError::DomainResumeError(e) => write!(f, "Failed to resume domain: {}", e),
            WolGatewayError::WakeOnLanParseError(e) => write!(f, "Parsing error: {}", e),
        }
    }
}

impl Error for WolGatewayError {}

impl From<std::net::AddrParseError> for WolGatewayError {
    fn from(err: std::net::AddrParseError) -> Self {
        WolGatewayError::AddressParseError(err)
    }
}

impl From<std::io::Error> for WolGatewayError {
    fn from(err: std::io::Error) -> Self {
        // Differentiate between bind and receive errors
        // A more robust way might be needed depending on io::Error variants
        if err.to_string().contains("address already in use") {
            WolGatewayError::SocketBindError(err)
        } else {
            WolGatewayError::UdpReceiveError(err)
        }
    }
}

impl From<virt::error::Error> for WolGatewayError {
    fn from(err: virt::error::Error) -> Self {
        WolGatewayError::LibvirtConnectError(err)
    }
}
