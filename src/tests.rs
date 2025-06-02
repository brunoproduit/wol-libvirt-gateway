#[cfg(test)]
use crate::error::WolGatewayError;

#[test]
fn test_valid_wol_packet_without_password() {
    let mut packet = vec![0xFF; 6]; // Sync stream
    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    // Add MAC address 16 times
    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }

    let wol = crate::wakeonlan::WakeOnLanPacket::parse(&packet).unwrap();
    assert_eq!(wol.target_mac_string(), "aa:bb:cc:dd:ee:ff");
}

#[test]
fn test_valid_wol_packet_with_6_byte_password() {
    let mut packet = vec![0xFF; 6]; // Sync stream
    let mac = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let password = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

    // Add MAC address 16 times
    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }

    // Add password
    packet.extend_from_slice(&password);

    let wol = crate::wakeonlan::WakeOnLanPacket::parse(&packet).unwrap();
    assert_eq!(wol.target_mac_string(), "12:34:56:78:9a:bc");
}

#[test]
fn test_valid_wol_packet_with_4_byte_password() {
    let mut packet = vec![0xFF; 6]; // Sync stream
    let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let password_4byte = [0xAA, 0xBB, 0xCC, 0xDD];

    // Add MAC address 16 times
    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }

    // Add 4-byte password
    packet.extend_from_slice(&password_4byte);

    let wol = crate::wakeonlan::WakeOnLanPacket::parse(&packet).unwrap();
    assert_eq!(wol.target_mac_string(), "00:11:22:33:44:55");
}

#[test]
fn test_packet_too_short() {
    let packet = vec![0xFF; 50]; // Too short
    let result = crate::wakeonlan::WakeOnLanPacket::parse(&packet);
    let _error_msg = format!(
        "Packet too short for WOL: {} bytes, expected at least {}",
        packet.len(),
        crate::wakeonlan::WOL_PACKET_MIN_SIZE
    );
    assert!(matches!(
        result,
        Err(WolGatewayError::WakeOnLanParseError(_error_msg))
    ));
}

#[test]
fn test_invalid_sync_stream() {
    let mut packet = vec![0xFE; 6]; // Wrong sync bytes
    let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }

    let result = crate::wakeonlan::WakeOnLanPacket::parse(&packet);
    let _error_msg = "Packet does not start with 6 FF bytes (sync stream)".to_string();
    assert!(matches!(
        result,
        Err(WolGatewayError::WakeOnLanParseError(_error_msg))
    ));
}

#[test]
fn test_mac_repetition_mismatch() {
    let mut packet = vec![0xFF; 6]; // Sync stream
    let mac1 = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let mac2 = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

    let repetition_error = 8;
    // Add first MAC
    packet.extend_from_slice(&mac1);

    // Add 15 more repetitions, but make one different
    for i in 1..16 {
        if i == repetition_error {
            packet.extend_from_slice(&mac2); // Different MAC
        } else {
            packet.extend_from_slice(&mac1);
        }
    }

    let result = crate::wakeonlan::WakeOnLanPacket::parse(&packet);
    let _error_msg = format!(
        "MAC address repetition check failed at repetition {}",
        repetition_error
    );
    assert!(matches!(
        result,
        Err(WolGatewayError::WakeOnLanParseError(_error_msg))
    ));
}

#[test]
fn test_mac_to_string_formatting() {
    // Test various MAC addresses to ensure proper formatting
    let test_cases = vec![
        ([0x00, 0x00, 0x00, 0x00, 0x00, 0x00], "00:00:00:00:00:00"),
        ([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], "ff:ff:ff:ff:ff:ff"),
        ([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC], "12:34:56:78:9a:bc"),
        ([0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6], "a1:b2:c3:d4:e5:f6"),
    ];

    for (mac_bytes, expected_string) in test_cases {
        let mut packet = vec![0xFF; 6]; // Sync stream

        // Add MAC address 16 times
        for _ in 0..16 {
            packet.extend_from_slice(&mac_bytes);
        }

        let wol = crate::wakeonlan::WakeOnLanPacket::parse(&packet).unwrap();
        assert_eq!(wol.target_mac_string(), expected_string);
    }
}

#[test]
fn test_domain_xml_mac_extraction() {
    let xml = r#"
        <domain>
            <devices>
                <interface type='network'>
                    <mac address='52:54:00:12:34:56'/>
                    <source network='default'/>
                </interface>
                <interface type='bridge'>
                    <mac address='52:54:00:ab:cd:ef'/>
                    <source bridge='virbr0'/>
                </interface>
            </devices>
        </domain>
        "#;

    let macs = crate::domain_xml::get_mac_addresses(xml).unwrap();
    assert_eq!(macs.len(), 2);
    assert!(macs.contains(&"52:54:00:12:34:56".to_string()));
    assert!(macs.contains(&"52:54:00:ab:cd:ef".to_string()));
}

#[test]
fn test_domain_xml_no_interfaces() {
    let xml = r#"
        <domain>
            <devices>
                <disk type='file' device='disk'>
                    <source file='/var/lib/libvirt/images/vm.qcow2'/>
                </disk>
            </devices>
        </domain>
        "#;

    let macs = crate::domain_xml::get_mac_addresses(xml).unwrap();
    assert_eq!(macs.len(), 0);
}

#[test]
fn test_domain_xml_empty_interface() {
    let xml = r#"
        <domain>
            <devices>
                <interface type='network'/>
            </devices>
        </domain>
        "#;

    let macs = crate::domain_xml::get_mac_addresses(xml);
    assert!(macs.is_err());
}

#[test]
fn test_domain_xml_self_closing_mac_tag() {
    let xml = r#"
        <domain>
            <devices>
                <interface type='network'>
                    <mac address='aa:bb:cc:dd:ee:ff'/>
                </interface>
            </devices>
        </domain>
        "#;

    let macs = crate::domain_xml::get_mac_addresses(xml).unwrap();
    assert_eq!(macs.len(), 1);
    assert_eq!(macs[0], "aa:bb:cc:dd:ee:ff");
}
