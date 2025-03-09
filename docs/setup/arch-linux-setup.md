# Development Environment Setup for Multi-Signature Wallet on Arch Linux

## Prerequisites
This guide assumes you have:
- A fresh Arch Linux installation
- Basic familiarity with the terminal

## Step 1: Install Required Dependencies

Install essential development tools and libraries:
```bash
sudo pacman -S base-devel git cmake nodejs npm openssl pkg-config
```

This installs:
- Base development tools (compilers, make, etc.)
- Git for version control
- CMake for build configuration
- Node.js and npm (for Figma integration)
- OpenSSL and pkg-config (for cryptographic functions)

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

## Step 4: Install Trunk for Web Development

Trunk is a WASM web application bundler for Rust:
```bash
cargo install trunk
```

## Step 5: Choose and Install Your IDE

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

Or use Flatpak:
```bash
flatpak install flathub sh.cursor.Cursor
flatpak run sh.cursor.Cursor
```

## Step 6: Configure Your IDE for Rust Development

Install these helpful extensions from the marketplace:
- Rust Analyzer: For Rust language support
- Even Better TOML: For better TOML file editing
- WebAssembly: For WASM support

### For VSCode:
```bash
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension ms-vscode.wasm-wasi-core
code --install-extension eamodio.gitlens
```

### For VSCodium:
```bash
codium --install-extension rust-lang.rust-analyzer
codium --install-extension tamasfe.even-better-toml
codium --install-extension ms-vscode.wasm-wasi-core
codium --install-extension GitHub.copilot
codium --install-extension GitHub.copilot-chat
codium --install-extension eamodio.gitlens
```

Note: GitHub Copilot requires a subscription and authentication with your GitHub account.

### For Cursor IDE: (not working currently due to cursor bug)
```bash
cursor --install-extension rust-lang.rust-analyzer
cursor --install-extension tamasfe.even-better-toml
cursor --install-extension ms-vscode.wasm-wasi-core
cursor --install-extension eamodio.gitlens
```

Note: Cursor IDE comes with built-in AI capabilities that can assist with coding, debugging, and understanding the codebase.

## Step 7: Project Setup

For instructions on setting up SSH access and cloning the project repository, refer to the [SSH Access Setup Guide](./ssh-access-setup.md) in the same directory.

Once you have cloned the repository, build the project:
```bash
cargo build --workspace
```

This will download and compile all dependencies specified in your `Cargo.toml` files.

## Step 8: Run the Development Server

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

## Step 9: Development Workflow

1. Edit your Rust code in your chosen IDE
2. When you save, Trunk will automatically rebuild and refresh the browser
3. Check the terminal for any compile errors

### Cursor IDE-Specific Features
If using Cursor IDE, you can take advantage of its AI features:
- Use Cmd+K / Ctrl+K to get AI assistance with code understanding
- Generate code explanations and documentation
- Get help with implementing complex algorithms
- Debug issues with AI assistance

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
