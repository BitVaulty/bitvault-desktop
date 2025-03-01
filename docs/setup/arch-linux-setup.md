# Development Environment Setup for Multi-Signature Wallet on Arch Linux

## Prerequisites
This guide assumes you have:
- A fresh Arch Linux installation
- VSCode/VSCodium installed
- Basic familiarity with the terminal

## Step 1: Install Required Dependencies

Install essential development tools and libraries:
```bash
sudo pacman -S base-devel git cmake nodejs npm openssl pkg-config webkit2gtk-4.1
```

This installs:
- Base development tools (compilers, make, etc.)
- Git for version control
- CMake for build configuration
- Node.js and npm (for Figma integration)
- OpenSSL and pkg-config (for cryptographic functions)
- WebKit2GTK 4.1 (required for Tauri applications)

## Step 2: Install Rust
Arch Linux provides rustup for managing Rust installations:

```bash
sudo pacman -S rustup
```

Initialize rustup and install the stable toolchain:
```bash
rustup default stable
```

Add WebAssembly target:
```bash
rustup target add wasm32-unknown-unknown
```

Verify the installation:
```bash
rustc --version
cargo --version
```

## Step 3: Set Up WebAssembly Support

Install the WebAssembly tools:
```bash
cargo install wasm-pack
```

## Step 4: Install Trunk and Tauri CLI

Trunk is a WASM web application bundler for Rust:
```bash
cargo install trunk
```

Install the Tauri CLI for desktop and mobile app development:
```bash
cargo install tauri-cli
```

## Step 5: Configure VSCode/VSCodium for Rust Development

Install these helpful extensions from the marketplace:
- Rust Analyzer: For Rust language support
- Even Better TOML: For better TOML file editing
- WebAssembly: For WASM support
- Tauri: For Tauri app development
- Tailwind CSS IntelliSense: For CSS support

Install from terminal:
```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension ms-vscode.wasm-wasi-core
code --install-extension tauri-apps.tauri-vscode
code --install-extension bradlc.vscode-tailwindcss
```

For VSCodium, replace `code` with `codium` in the commands above.

## Step 6: Project Setup

For instructions on setting up SSH access and cloning the project repository, refer to the [SSH Access Setup Guide](./ssh-access-setup.md) in the same directory.

Once you have cloned the repository, build the project:
```bash
cargo build --workspace
```

This will download and compile all dependencies specified in your `Cargo.toml` files.

## Step 7: Run the Development Server

Start the development server:
```bash
make dev
```

For Android development:
```bash
make android
```

This will:
1. Compile your Rust code
2. Start the development environment
3. Launch the application in development mode

## Step 8: Development Workflow

1. Edit your Rust code in VSCodium
2. When you save, Trunk will automatically rebuild and refresh the browser
3. Check the terminal for any compile errors

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
For better WASM optimization:
```bash
sudo pacman -S binaryen
```

### Rust Toolchain Issues
If you encounter problems with the Rust toolchain:
```bash
rustup update
rustup target add wasm32-unknown-unknown
```