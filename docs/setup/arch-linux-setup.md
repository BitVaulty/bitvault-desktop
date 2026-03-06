# BitVault Desktop - Arch Linux Setup

## Overview

This guide helps you set up a development environment for BitVault Desktop on Arch Linux. BitVault Desktop is a Bitcoin multisig wallet built with Rust and egui.

## Prerequisites

- Arch Linux (or derivative)
- Basic familiarity with the terminal

## Step 1: Install System Dependencies

Install the libraries required for egui/eframe (native backend):

```bash
sudo pacman -S base-devel git pkg-config \
  webkit2gtk gtk3 libayatana-appindicator librsvg \
  openssl libxcb libxkbcommon
```

This provides:
- **webkit2gtk** - WebView backend for egui
- **gtk3** - GTK bindings
- **libayatana-appindicator** - System tray
- **librsvg** - SVG rendering
- **libxcb, libxkbcommon** - X11/Wayland support

## Step 2: Install Rust

```bash
sudo pacman -S rustup
rustup default stable
```

Verify:
```bash
rustc --version
cargo --version
```

## Step 3: Clone and Build

```bash
git clone https://github.com/BitVaulty/bitvault-desktop.git
cd bitvault-desktop
cargo build
```

## Step 4: Run

```bash
cargo run
```

Or use the Makefile:
```bash
make dev
```

## Step 5: Run Tests

```bash
cargo test
```

## Step 6: IDE Setup (Optional)

### Cursor / VSCode / VSCodium

Install Rust Analyzer for language support:
```bash
cursor --install-extension rust-lang.rust-analyzer
# or: code --install-extension rust-lang.rust-analyzer
# or: codium --install-extension rust-lang.rust-analyzer
```

## Project Structure

BitVault Desktop is a single Rust crate at repo root:
- `src/` - Application source (app, ui, services, state, utils)
- `tests/` - Integration and E2E tests
- `resources/` - Icons and assets
- **bitvault-common** - Shared library (external git dependency)

## Local Development with bitvault-common

To develop against a local copy of bitvault-common, create `.cargo/config.toml` (gitignored):

```toml
[patch."https://github.com/BitVaulty/bitvault-common.git"]
bitvault-common = { path = "../bitvault-common" }
```

## Troubleshooting

### Missing libraries
If build fails with "library not found":
```bash
sudo pacman -S gcc
# Re-run the install from Step 1
```

### PATH
Ensure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### SSL/TLS
```bash
sudo pacman -S ca-certificates
```

## Additional Resources

- [README](../../README.md) - Quick start
- [CONTRIBUTING](../../CONTRIBUTING.md) - Contribution guidelines
- [architecture-overview.md](../design/architecture-overview.md) - Architecture
