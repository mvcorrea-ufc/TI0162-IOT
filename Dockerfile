FROM debian:bullseye-slim

# Set the APT frontend to non-interactive
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libudev-dev \
    curl git neovim \
    usbutils \
    openssh-server \
    && rm -rf /var/lib/apt/lists/*

# Note: Serial port access handled via OCI annotation in compose file

# Create workspace directory
RUN mkdir -p /workspace

# Create SSH daemon directory (critical for SSH to work)
RUN mkdir -p /run/sshd

# SSH configuration
RUN sed -i 's/^#*PermitRootLogin .*/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    sed -i 's/^#*PasswordAuthentication .*/PasswordAuthentication yes/' /etc/ssh/sshd_config && \
    sed -i 's/^#*PubkeyAuthentication .*/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Set password for root
RUN echo 'root:rootpass' | chpasswd

# Generate SSH host keys (important for SSH to start properly)
RUN ssh-keygen -A

# --- Rust Installation ---
# Install Rust via rustup.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Add the Cargo bin directory to the user's PATH.
ENV PATH="/root/.cargo/bin:${PATH}"

# Install the cross-compilation target for the ESP32-C3.
RUN rustup target add riscv32imc-unknown-none-elf
# for esp32-c6 integration (need to change the project files)
#RUN rustup target add riscv32imac-unknown-none-elf

# Install espflash and ldproxy for the user.
RUN cargo install espflash ldproxy probe-rs-tools

WORKDIR /workspace
EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
