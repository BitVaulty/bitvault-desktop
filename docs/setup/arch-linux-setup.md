# BitVault Development Environment Setup for Arch Linux

## Overview
This guide helps you set up a development environment for BitVault, a Bitcoin-only wallet with a focus on security through a 2-of-3 threshold signature scheme and strict security boundaries.

## Prerequisites
This guide assumes you have:
- A fresh Arch Linux installation
- Basic familiarity with the terminal

## Step 1: Install Required Dependencies

Install essential development tools and libraries:
```bash
sudo pacman -S base-devel git cmake openssl pkg-config libxcb xcb-util-keysyms libxkbcommon
```

This installs:
- Base development tools (compilers, make, etc.)
- Git for version control
- CMake for build configuration
- OpenSSL and pkg-config (for cryptographic functions)
- X11 libraries required for egui

## Step 2: Install Rust
Arch Linux provides rustup for managing Rust installations:

```bash
sudo pacman -S rustup
```

Initialize rustup and install the stable toolchain:
```bash
rustup default stable
```

Add WebAssembly target (for future web interface development):
```bash
rustup target add wasm32-unknown-unknown
```

Verify the installation:
```bash
rustc --version
cargo --version
```

## Step 3: Set Up WebAssembly Support

Install the WebAssembly tools (for future web interface development):
```bash
cargo install wasm-pack
```

## Step 4: Install Trunk for Web Development

Trunk is a WASM web application bundler for Rust (for future web interface):
```bash
cargo install trunk
```

## Step 5: Install Bitcoin Development Kit Tools

BitVault relies on the Bitcoin Development Kit (BDK) for Bitcoin operations:

```bash
# Install Bitcoin Core for testing (optional but recommended)
sudo pacman -S bitcoin

# Install SQLite for BDK storage
sudo pacman -S sqlite
```

## Step 6: Choose and Install Your IDE

You have three options for your development environment:

### Option A: VSCode
Install VSCode from the official repositories:
```bash
sudo pacman -S code
```

### Option B: VSCodium (Open Source Build of VSCode)
Install VSCodium from the AUR:
```bash
yay -S vscodium-bin
# or
paru -S vscodium-bin
```

### Option C: Cursor IDE (AI-enhanced Fork of VSCode)
Install Cursor IDE from the AUR:
```bash
yay -S cursor-bin
# or
paru -S cursor-bin
```

Alternatively, download the AppImage from the official website:
```bash
# Download the AppImage from the official website
wget https://download.cursor.sh/linux/appImage/x64 -O Cursor.AppImage

# Make it executable
chmod +x Cursor.AppImage

# Run the AppImage
./Cursor.AppImage
```

## Step 7: Configure Your IDE for Rust Development

Install these helpful extensions from the marketplace:
- Rust Analyzer: For Rust language support
- Even Better TOML: For better TOML file editing
- LLDB: For debugging
- WebAssembly: For WASM support (for future web interface)

### For VSCode:
```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension vadimcn.vscode-lldb
code --install-extension ms-vscode.wasm-wasi-core
```

### For VSCodium:
```bash
codium --install-extension rust-lang.rust-analyzer
codium --install-extension tamasfe.even-better-toml
codium --install-extension vadimcn.vscode-lldb
codium --install-extension ms-vscode.wasm-wasi-core
```

### For Cursor IDE:
```bash
cursor --install-extension rust-lang.rust-analyzer
cursor --install-extension tamasfe.even-better-toml
cursor --install-extension vadimcn.vscode-lldb
cursor --install-extension ms-vscode.wasm-wasi-core
```

## Step 8: Security-Specific Setup for Development

BitVault uses process isolation as a security boundary. Set up the necessary tools for security testing:

```bash
# Install tools for security isolation testing
sudo pacman -S strace ltrace valgrind

# Set up seccomp-bpf development tools
sudo pacman -S libseccomp linux-headers

# Optional: Install IPC testing tools
sudo pacman -S socat netcat
```

## Step 9: Project Setup

For instructions on setting up SSH access and cloning the project repository, refer to the [SSH Access Setup Guide](./ssh-guide.md) in the same directory.

Once you have cloned the repository, build the project:
```bash
# Build the entire workspace
cargo build --workspace

# Build specific crates
cargo build -p bitvault-core
cargo build -p bitvault-ui
```

## Step 10: Run the Development Environment

```bash
# Run the development build with security isolation
make dev

# Run tests for the core module
cargo test -p bitvault-core

# Run tests for the UI module
cargo test -p bitvault-ui

# Run security boundary tests
make security-test
```

For future web development:
```bash
# Run the web development server
trunk serve
```

## Step 11: Development Workflow

BitVault consists of multiple crates separated by security boundaries:
- `bitvault-core`: Security-critical operations within the secure boundary
- `bitvault-common`: Shared code that crosses boundaries
- `bitvault-ui`: User interface using egui
- `bitvault-app`: Platform integration and process management

When developing, be mindful of the security boundaries:
1. Never move security-critical code outside of `bitvault-core`
2. Be cautious when modifying IPC interfaces in `bitvault-common`
3. All key operations must remain within the secure process

## Troubleshooting

### Missing Libraries
If you encounter errors about missing libraries:
```bash
sudo pacman -S gcc
```

### SSL Certificate Issues
For SSL/TLS related errors:
```bash
sudo pacman -S ca-certificates
```

### Path Issues
If you get "command not found" errors, make sure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### WebAssembly Optimization Issues
For better WASM optimization (future web interface):
```bash
sudo pacman -S binaryen
```

### Security Boundary Testing Issues
If you encounter problems with process isolation or IPC:
```bash
# Check if IPC mechanisms are working properly
ls -la /run/user/$(id -u)/
```

### Permission Issues with Secure Process
If the secure process fails to start with permission errors:
```bash
# Check seccomp filter capabilities
grep SECCOMP /boot/config-$(uname -r)

# Ensure proper namespace permissions
sudo sysctl -w kernel.unprivileged_userns_clone=1
```

## Additional Resources

- Refer to the project documentation under `docs/design/` for comprehensive information about BitVault's architecture and security model
- See `docs/design/architecture-overview.md` for the codebase structure
- Review `docs/design/security-boundaries.md` for details on security isolation