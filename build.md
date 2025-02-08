# Build code on Ubuntu

```
apt-get update
apt-get install -y \
    libglib2.0-dev \
    libgirepository1.0-dev \
    build-essential \
    pkg-config \
    libgdk-pixbuf2.0-dev \
    libgtk-3-dev \
    tmux \
    curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo install trunk tauri-cli
# ???
# cargo add javascriptcore-rs-sys
rustup target add wasm32-unknown-unknown
rustup target add x86_64-unknown-linux-gnu

cargo tauri build --target aarch64-unknown-linux-gnu
```