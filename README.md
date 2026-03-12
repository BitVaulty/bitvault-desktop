# BitVault Desktop

Bitcoin multisig wallet desktop application built with Rust + egui.

## Architecture

- **UI**: egui/eframe (immediate mode GUI)
- **Bitcoin**: BDK (used directly, no wrappers)
- **Shared logic**: bitvault-common (separate repo, git dependency)

## Quick Start

```bash
# Build and run
cargo run

# Check compilation
cargo check

# Run tests
cargo test
```

## Project Structure

```
bitvault-desktop/
├── Cargo.toml
├── src/               # Application source
│   ├── app.rs         # Main app struct
│   ├── ui/            # UI screens
│   ├── services/      # Service layer
│   ├── state/         # App state management
│   └── utils/         # UR/QR, etc.
├── tests/
├── resources/
└── docs/
```

**bitvault-common** is an external dependency (https://github.com/BitVaulty/bitvault-common). By default Cargo uses the git dependency (branch `dev`). For local development when working on both repos:

1. Create `.cargo/config.toml` (this directory is gitignored so the override is not committed).
2. Add a path patch so the desktop uses your local common:

```toml
[patch."https://github.com/BitVaulty/bitvault-common.git"]
bitvault-common = { path = "../bitvault-common" }
```

Adjust the `path` if your bitvault-common clone lives elsewhere. Without this file, CI and fresh clones use the published git dependency.

## Development

- **Arch Linux**: [docs/setup/arch-linux-setup.md](docs/setup/arch-linux-setup.md)
- **Architecture**: [docs/design/architecture-overview.md](docs/design/architecture-overview.md)
- **Makefile**: `make dev`, `make test`, `make lint`

