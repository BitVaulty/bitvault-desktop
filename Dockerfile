# Stage 1: Build stage
FROM rust:bullseye AS builder

# Install system dependencies (as before)
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
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target list

WORKDIR /app

COPY . .

RUN cargo install tauri-cli
RUN cargo test
RUN cargo tauri dev

# Stage 2: Runtime stage (as before)
FROM debian:bullseye-slim

# Copy only the necessary artifacts from the build stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/bitvaulty /app/bitvaulty

WORKDIR /app

CMD ["./bitvaulty"] # Replace "bitvaulty" with your app name