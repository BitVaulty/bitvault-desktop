# BitVault UI

A desktop Bitcoin wallet application built with Rust and egui.

## Overview

BitVault is a secure, self-custody Bitcoin wallet that provides a simple and intuitive interface for managing your Bitcoin. The application is built using Rust for performance and security, and egui for the user interface.

## Features

- Create a new wallet with a 12-word recovery phrase
- Restore an existing wallet using a recovery phrase
- PIN protection for wallet access
- Secure storage of wallet data
- Simple and intuitive user interface

## Development

### Prerequisites

- Rust (stable)
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/BitVaultWallet.git
cd BitVaultWallet/bitvault-ui

# Build the application
cargo build --release
```

### Running

```bash
# Run the application
./start.sh
```

Or directly:

```bash
cargo run --release
```

## Architecture

The application is built using the following components:

- **egui**: A simple, immediate mode GUI library for Rust
- **eframe**: The egui framework for creating desktop applications
- **bip39**: For generating and managing BIP-39 mnemonic phrases
- **aes-gcm**: For secure encryption of wallet data
- **argon2**: For key derivation from user PINs

## License

This project is licensed under the MIT License - see the LICENSE file for details.
