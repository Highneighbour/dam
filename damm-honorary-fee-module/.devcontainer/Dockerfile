FROM ubuntu:22.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    llvm \
    clang \
    make \
    git \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Solana CLI
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor CLI
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN /root/.cargo/bin/avm install 0.29.0
RUN /root/.cargo/bin/avm use 0.29.0

# Set up workspace
WORKDIR /workspace

# Copy setup scripts
COPY scripts/ /workspace/scripts/

# Make scripts executable
RUN chmod +x /workspace/scripts/*.sh

# Default command
CMD ["/bin/bash"]