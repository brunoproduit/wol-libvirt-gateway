//! Libvirt domain management utilities for VM operations.
//!
//! This module provides functionality to interact with libvirt domains (VMs),
//! including starting VMs by UUID or MAC address and managing domain states.

use log::{debug, error, info};
use uuid::Uuid;
use virt::connect::Connect;
use virt::domain::Domain;

use crate::error::WolGatewayError;

/// Represents the various states a libvirt domain (VM) can be in.
///
/// This enum maps to the libvirt domain state codes and provides
/// a type-safe way to handle VM state information.
#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
enum DomainState {
    /// Domain state is unknown or not set
    NoState = 0,
    /// Domain is running and active
    Running = 1,
    /// Domain is blocked on a resource
    Blocked = 2,
    /// Domain is paused by user
    Paused = 3,
    /// Domain is being shut down
    Shutdown = 4,
    /// Domain is shut off
    Shutoff = 5,
    /// Domain has crashed
    Crashed = 6,
    /// Domain is suspended to disk (power management)
    PmSuspended = 7,
    /// Last state marker
    Last = 8,
}

impl From<u32> for DomainState {
    /// Converts a libvirt domain state code to a `DomainState` enum variant.
    ///
    /// # Arguments
    ///
    /// * `state_code` - The numeric state code from libvirt
    ///
    /// # Returns
    ///
    /// The corresponding `DomainState` variant, or `NoState` for unknown codes
    fn from(state_code: u32) -> Self {
        match state_code {
            1 => DomainState::Running,
            2 => DomainState::Blocked,
            3 => DomainState::Paused,
            4 => DomainState::Shutdown,
            5 => DomainState::Shutoff,
            6 => DomainState::Crashed,
            7 => DomainState::PmSuspended,
            8 => DomainState::Last,
            _ => DomainState::NoState,
        }
    }
}

/// Attempts to start a libvirt domain (VM) by its UUID.
///
/// This function handles different VM states appropriately:
/// - For shut off, shutdown, or crashed VMs: attempts to start them
/// - For paused VMs: attempts to resume them
/// - For other states: logs the current state and takes no action
///
/// # Arguments
///
/// * `conn` - The libvirt connection handle
/// * `vm_uuid` - The UUID of the VM to start
///
/// # Returns
///
/// * `Ok(())` - VM was successfully started, resumed, or was already in a non-startable state
/// * `Err(WolGatewayError)` - An error occurred during the operation
///
/// # Errors
///
/// Returns various `WolGatewayError` variants for different failure modes:
/// - `DomainLookupError` - Failed to find domain with the given UUID
/// - `DomainNameError` - Failed to retrieve domain name
/// - `DomainStateError` - Failed to retrieve domain state
/// - `DomainStartError` - Failed to start the domain
/// - `DomainResumeError` - Failed to resume a paused domain
async fn start_vm_libvirt(conn: &Connect, vm_uuid: Uuid) -> Result<(), WolGatewayError> {
    let domain = Domain::lookup_by_uuid(conn, vm_uuid).map_err(|e| {
        error!("Failed to lookup VM with UUID {}: {:?}", vm_uuid, e);
        WolGatewayError::DomainLookupError(e)
    })?;

    let vm_name = domain.get_name().map_err(|e| {
        error!("Failed to get name for VM with UUID {}: {:?}", vm_uuid, e);
        WolGatewayError::DomainNameError(e)
    })?;
    info!(
        "Attempting to start VM via libvirt: {} {}",
        vm_name, vm_uuid
    );

    let state_tuple = domain.get_state().map_err(|e| {
        error!("Failed to get state for VM {}: {:?}", vm_name, e);
        WolGatewayError::DomainStateError(e)
    })?;
    let state = DomainState::from(state_tuple.0);

    match state {
        DomainState::Shutoff | DomainState::Shutdown | DomainState::Crashed => {
            domain.create().map_err(|e| {
                error!("Failed to start VM {} via libvirt: {:?}", vm_name, e);
                WolGatewayError::DomainStartError(e)
            })?;
            info!(
                "Successfully commanded VM {} to start via libvirt.",
                vm_name
            );
        }
        DomainState::Paused => {
            domain.resume().map_err(|e| {
                error!("Failed to resume VM {} via libvirt: {:?}", vm_name, e);
                WolGatewayError::DomainResumeError(e)
            })?;
            info!(
                "Successfully commanded VM {} to resume (it was paused) via libvirt.",
                vm_name
            );
        }
        _ => {
            info!(
                "VM {} is not in a startable state (current: {:?}). No action taken.",
                vm_name, state
            );
        }
    }

    Ok(())
}

/// Finds a VM by its MAC address and attempts to start it if found.
///
/// This function searches through all libvirt domains to find one with a network
/// interface matching the specified MAC address. If found, it attempts to start
/// the VM using the appropriate method based on its current state.
///
/// # Arguments
///
/// * `conn` - The libvirt connection handle
/// * `target_mac` - The MAC address to search for (case-insensitive)
///
/// # Returns
///
/// * `Ok(())` - VM was found and successfully started/resumed
/// * `Err(WolGatewayError)` - An error occurred during the operation or VM was not found
///
/// # Errors
///
/// Returns various `WolGatewayError` variants for different failure modes:
/// - `VmNotFound` - No VM found with the specified MAC address
/// - `DomainListError` - Failed to list libvirt domains
/// - `DomainXmlError` - Failed to get domain XML description
/// - `MacExtractionError` - Failed to extract MAC addresses from XML
/// - `DomainUuidError` - Failed to get domain UUID
/// - Other errors propagated from `start_vm_libvirt`
///
/// Behavior
///
/// - Searches through all domains (both active and inactive)
/// - Performs case-insensitive MAC address comparison
/// - Extracts MAC addresses from domain XML descriptions
/// - Stops searching once a matching MAC is found
/// - Logs progress and results at appropriate levels
pub(crate) async fn find_and_start_vm_by_mac(
    conn: &Connect,
    target_mac: &str,
) -> Result<(), WolGatewayError> {
    info!("Searching for VM with MAC address: {}", target_mac);

    let target_mac_lower = target_mac.to_lowercase();

    let domains = conn
        .list_all_domains(0) // List all domains (both active and inactive)
        .map_err(|e| {
            error!("Failed to list all domains: {:?}", e);
            WolGatewayError::DomainListError(e)
        })?;

    for dom in domains {
        let xml_desc = dom.get_xml_desc(0).map_err(|e| {
            let domain_name = dom.get_name().unwrap_or_else(|_| "unknown".to_string());
            error!(
                "Failed to get XML description for domain {}: {:?}",
                domain_name, e
            );
            WolGatewayError::DomainXmlError(e)
        })?;

        let mac_addresses = crate::domain_xml::get_mac_addresses(&xml_desc)?;

        for mac in mac_addresses {
            debug!("Checking MAC address: {}", mac);
            if mac.to_lowercase() == target_mac_lower {
                let uuid = dom.get_uuid().map_err(|e| {
                    error!(
                        "Failed to get UUID for domain with matching MAC {}: {:?}",
                        target_mac, e
                    );
                    WolGatewayError::DomainUuidError(e)
                })?;

                info!(
                    "Found VM with matching MAC address: {} ({})",
                    target_mac, uuid
                );
                return start_vm_libvirt(conn, uuid).await;
            }
        }
    }

    info!("No VM found with MAC address: {}", target_mac);
    Err(WolGatewayError::VmNotFound(target_mac.to_string()))
}
