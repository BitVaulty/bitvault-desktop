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

**bitvault-common** is an external dependency (https://github.com/BitVaulty/bitvault-common). For local development, use a path override in `.cargo/config.toml` (gitignored).

## Development

- **Arch Linux**: [docs/setup/arch-linux-setup.md](docs/setup/arch-linux-setup.md)
- **Architecture**: [docs/design/architecture-overview.md](docs/design/architecture-overview.md)
- **Makefile**: `make dev`, `make test`, `make lint`

