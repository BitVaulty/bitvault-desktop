# BitVault

A secure multi-signature wallet

## Architecture

BitVault uses a workspace structure with the following components:

- **bitvault-ui**: Leptos-based frontend (WASM)
- **bitvault-app**: Tauri application shell
- **bitvault-ipc**: IPC layer between app and core
- **bitvault-core**: Secure cryptographic core
- **bitvault-common**: Shared types and utilities

## Development Setup

### Prerequisites

Make sure you have installed the prerequisites for your OS: https://v2.tauri.app

For Arch Linux users, refer to the [Development Environment Setup](./docs/setup/arch-linux-setup.md) guide.

### Repository Access

To access the private repository, follow the [SSH setup instructions](./docs/setup/ssh-guide.md).

### Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Build and Run

The project uses a Rust workspace structure. You can build and run using the Makefile or from individual component directories.

### Using the Makefile

Run the desktop app in development mode:
```bash
make dev
```

Run the UI only:
```bash
make ui
```

Build the UI with trunk:
```bash
make trunk
```

Run the Android app in development mode:
```bash
make android
```

### From the Root Directory

Build all components:
```bash
cargo build --workspace
```

Run tests for all components:
```bash
cargo test --workspace
```

Then access at: http://localhost:1777