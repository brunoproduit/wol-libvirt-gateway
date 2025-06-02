FROM rust:1.87-slim-bullseye

# Install libvirt development libraries
RUN apt-get update && apt-get install -y \
    libvirt-dev \
    libvirt0 \
    ca-certificates \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Expose the WOL port
EXPOSE 9/udp

VOLUME /var/run/libvirt/libvirt-sock
# Start the server
CMD ["./target/release/wol-libvirt-gateway", "--address", "127.0.0.1:9", "--libvirt-uri", "qemu:///system"]
