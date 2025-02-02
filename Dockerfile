FROM rust:latest

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    pkg-config \
    nodejs \
    npm \
    git \
    lld \
    clang

RUN cargo install trunk

RUN cargo install tauri-cli

# Install wasm target
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add x86_64-unknown-linux-gnu

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Configure cargo for Linux build
RUN mkdir -p .cargo && \
    echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=lld"]' > .cargo/config.toml

# Build command
CMD ["cargo", "tauri", "build"]
