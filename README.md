# BitVault Desktop

Bitcoin wallet desktop application built with Rust + egui.

## Architecture

- **UI**: egui (immediate mode GUI)
- **Bitcoin**: BDK (used directly, no wrappers)
- **Structure**: Clean separation between app logic and Bitcoin operations

## Quick Start

```bash
# Build and run
cargo run --bin bitvault-app

# Check compilation
cargo check --workspace

# Run tests
cargo test --workspace
```

## Project Structure

```
bitvault-desktop/
├── bitvault-app/      # Main egui application
│   ├── src/
│   │   ├── app.rs     # Main app struct
│   │   ├── ui/        # UI screens
│   │   ├── services/  # Service layer (uses BDK directly)
│   │   ├── models/    # Data models
│   │   ├── state/     # App state management
│   │   └── utils/     # Utilities (UR/QR, etc.)
│   └── Cargo.toml
│
└── bitvault-common/   # App-specific shared code
    ├── src/
    │   ├── config.rs  # Configuration
    │   ├── events.rs  # Event system
    │   ├── error.rs   # Error types
    │   └── ...
    └── Cargo.toml
```

## Key Principles

1. **Use BDK directly** - No wrappers around BDK
2. **App-specific code in common** - Only non-Bitcoin logic
3. **Clean architecture** - Clear separation of concerns
4. **Mirror mobile structure** - Match Swift app organization

## Development

See `ARCHITECTURE_PLAN.md` for detailed architecture documentation.
