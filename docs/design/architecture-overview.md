# BitVault Desktop - Architecture Overview

This document describes the architecture of BitVault Desktop and its relationship to bitvault-common.

## Architecture

### Structure

```
bitvault-desktop/          # Single Rust crate
├── src/
│   ├── app.rs             # Main application, egui integration
│   ├── ui/                # Screens and components
│   │   ├── vault_creation/
│   │   ├── send_transaction/
│   │   ├── recovery/
│   │   └── ...
│   ├── services/          # KeyService, network_check, etc.
│   ├── state/             # AppState, Navigation
│   └── utils/             # QR, icons, camera
├── tests/
└── resources/

bitvault-common/           # External dependency (separate repo)
├── wallet/                # VaultService, descriptor building
├── convenience/           # API client, connectivity
├── derivation/            # Key derivation, CoownerKeys
├── ur/                    # UR encoding/decoding
└── ...
```

### Module Responsibilities

**bitvault-desktop (this repo)**
- UI rendering (egui/eframe)
- User workflows (vault creation, send, recovery)
- Platform integration (keyring, file dialogs)
- State management
- Uses BDK via bitvault-common's VaultService

**bitvault-common** (https://github.com/BitVaulty/bitvault-common)
- VaultService (wallet ops, PSBT, descriptors)
- Key derivation, UR encoding
- ConvenienceService (backend API)
- PinService, secure storage
- No UI; shared by desktop and mobile targets

### Design Principles

- **BDK directly** - No wrappers; VaultService wraps BDK for vault operations
- **Separation of concerns** - UI in desktop, wallet logic in common
- **2-of-3 multisig** - Primary wallet type with time delay

## Technology Stack

- **Rust** - Application language
- **egui/eframe** - GUI framework
- **BDK** - Bitcoin operations
- **bitvault-common** - Shared wallet logic (git dependency)
