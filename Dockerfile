# Stage 1: Build stage
FROM rust:1.84.1-slim-bullseye AS builder

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
RUN rustup target add wasm32-unknown-unknown

# Install wasm target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

COPY . .

# Build the frontend using Trunk (NO --target needed here)
RUN trunk build

# Configure cargo for Linux build
RUN mkdir -p .cargo && \
    echo '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=lld"]' > .cargo/config.toml

# Build the Tauri application for x86_64
RUN cargo tauri build --target x86_64-unknown-linux-gnu


# Stage 2: Runtime stage
FROM debian:bullseye-slim

# Copy only the necessary artifacts from the build stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/bitvaulty /app/bitvaulty # Replace "bitvaulty" with your app name

# Install runtime dependencies if any
# Example (adjust as needed):
# RUN apt-get update && apt-get install -y libwebkit2gtk-4.0-2

WORKDIR /app

CMD ["./bitvaulty"] # Replace "bitvaulty" with your app name