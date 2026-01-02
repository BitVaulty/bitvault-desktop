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

# Note: egui/eframe doesn't require trunk or tauri
# Removed trunk and tauri-cli installation as we use egui/eframe, not Tauri

WORKDIR /app

COPY . .

RUN cargo test
RUN cargo build --release -p bitvault-app

# Stage 2: Runtime stage (as before)
FROM debian:bullseye-slim

# Copy only the necessary artifacts from the build stage
COPY --from=builder /app/target/release/bitvault-app /app/bitvault-app

WORKDIR /app

CMD ["./bitvault-app"]