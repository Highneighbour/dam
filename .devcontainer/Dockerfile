# Devcontainer for Anchor/Solana
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential pkg-config libssl-dev curl git jq \
    ca-certificates sudo python3 python3-pip \
    nodejs npm \
  && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.73.0 && \
    /root/.cargo/bin/rustup component add rustfmt clippy
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Solana
ARG SOLANA_VERSION=v1.18.18
RUN sh -c "curl -sSfL https://release.solana.com/${SOLANA_VERSION}/install | sh" && \
    echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> /root/.bashrc
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor CLI
ARG ANCHOR_VERSION=0.30.1
RUN npm install -g @coral-xyz/anchor-cli@${ANCHOR_VERSION}

# Yarn
RUN npm install -g yarn

# Create workspace directory
WORKDIR /workspace
COPY . /workspace

# Pre-cache anchor deps
RUN anchor --version && solana --version && rustc --version

# Default command keeps container alive
CMD ["bash"]
