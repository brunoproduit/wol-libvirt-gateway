# WOL Libvirt Gateway

A simple Wake-on-LAN (WOL) gateway service for starting libvirt VMs. It listens for WOL "magic packets" on UDP port 9 (configurable), identifies the target MAC address, and if that MAC address belongs to a defined libvirt VM, it attempts to start that VM using the libvirt API.

This service is designed to be run on the same host as the libvirt daemon and the VMs. It's particularly useful for triggering VM power-on from external tools like Guacamole, which can send WOL packets.

## Security Warning

**⚠️ SECURITY NOTICE:** This service is inherently insecure when exposed to the internet. Wake-on-LAN packets are plaintext and contain no authentication mechanism, besides an optional plaintext "password" that is intentionally ignored as it is insecure. Anyone who can send UDP packets to your service and knows (or can guess) a VM's MAC address can potentially start that VM. Only run this service on trusted local networks, if not localhost only.

## Features

*   Listens for WOL magic packets.
*   Queries libvirt for VM MAC addresses.
*   Uses the libvirt API directly to start VMs (no `virsh` command execution).
*   Configurable listen address and libvirt URI.

## Prerequisites

*   Rust (for building from source)
*   Libvirt installed and running (`libvirtd` daemon)
*   Your libvirt VMs must have their network interfaces configured with static MAC addresses that your WOL client will target

## Installation

## Usage

### Basic Usage

Run the service with default settings:
```bash
wol-libvirt-gateway
```

This will:
- Listen on `127.0.0.1:9` (UDP port 9)
- Connect to libvirt using `qemu:///system`

### Command Line Options

```bash
wol-libvirt-gateway --help
```

Common options:
- `--listen-address <IP:PORT>` - Address and port to listen on (default: `127.0.0.1:9`)
- `--libvirt-uri <URI>` - Libvirt connection URI (default: `qemu:///system`)

Examples:
```bash
# Listen only on localhost, port 9009
wol-libvirt-gateway --listen-address 127.0.0.1:9009

# Use session libvirt instead of system
wol-libvirt-gateway --libvirt-uri qemu:///session
```

### Running as a System Service

#### systemd Service

Enable and start the service:
```bash
sudo cp ./packaging/systemd/wol-libvirt-gateway.service /etc/systemd/system/wol-libvirt-gateway.service
sudo systemctl daemon-reload
sudo systemctl enable wol-libvirt-gateway.service
sudo systemctl start wol-libvirt-gateway.service
```

Check service status:
```bash
sudo systemctl status wol-libvirt-gateway.service
sudo journalctl -u wol-libvirt-gateway.service -f
```

## Development

### Setting up the Development Environment

1. **Install libvirt development libraries:**

   **Ubuntu/Debian:**
   ```bash
   sudo apt install libvirt-dev pkg-config
   ```

   **RHEL/CentOS/Fedora:**
   ```bash
   sudo dnf install libvirt-devel pkg-config
   # or: sudo yum install libvirt-devel pkg-config
   ```

   **Arch Linux:**
   ```bash
   sudo pacman -S libvirt pkg-config
   ```

2. **Clone and build:**
   ```bash
   git clone https://github.com/brunoproduit/wol-libvirt-gateway.git
   cd wol-libvirt-gateway
   cargo build
   ```

4. **Run in development mode:**
   ```bash
   cargo run -- --help
   cargo run -- --listen-address 127.0.0.1:9009
   ```

### Testing

Run tests:
```bash
cargo test
```

## Sending WOL Packets

You can use tools like `wakeonlan` or `etherwake` to send WOL packets. Many network management tools and virtualization platforms (like Guacamole) also have built-in WOL functionality.

### Using wakeonlan

Install wakeonlan:
```bash
# Ubuntu/Debian
sudo apt install wakeonlan

# RHEL/CentOS/Fedora
sudo dnf install wakeonlan

# Arch Linux
sudo pacman -S wakeonlan
```

Send a WOL packet:
```bash
wakeonlan AA:BB:CC:DD:EE:FF -i 127.0.0.1  # Replace with your VM's MAC address
```

## How it Works

1. The service binds to a UDP socket (default `127.0.0.1:9`).
2. When a UDP packet is received, it's checked to see if it's a valid WOL magic packet.
3. If valid, the MAC address is extracted from the packet.
4. The service connects to the specified libvirt URI.
5. It iterates through all defined libvirt domains (VMs).
6. For each domain, it parses the XML definition to extract network interface MAC addresses.
7. If the MAC from the WOL packet matches a MAC found in a libvirt domain:
   * The service checks the current state of that domain.
   * If the domain is `shutoff`, `shutdown`, or `crashed`, the service attempts to start it.
   * If the domain is already running or in another non-startable state, no action is taken.
8. If no domain matches the MAC, a warning is logged.

## Troubleshooting

### Common Issues

**Permission denied connecting to libvirt:**
- Ensure the user running the service is in the `libvirtd` group
- Check that `libvirtd` service is running: `sudo systemctl status libvirtd`
- Verify the libvirt URI is correct for your setup

**VM not starting:**
- Verify the MAC address in your WOL client matches the VM's network interface
- Check VM state: `virsh list --all`
- Ensure the VM is in a startable state (shutoff, shutdown, or crashed)
- Check libvirt logs: `sudo journalctl -u libvirtd.service`

### Getting VM MAC Addresses

To find the MAC address of your libvirt VMs:
```bash
sudo virsh domiflist <vm_name>
```

### Logs and Debugging

Check service logs:
```bash
# systemd service logs
sudo journalctl -u wol-libvirt-gateway.service -f

# If running manually, increase verbosity
RUST_LOG=debug wol-libvirt-gateway
```

## License

This project is licensed under the Apache 2.0 License. See the `LICENSE` file for details.
