# Architecture Plan - Best of Both Approaches

**Date**: 2025-12-20  
**Goal**: Combine egui (main) + clean architecture (feature branch) without duplicating BDK

---

## 🎯 Design Principles

1. **Use BDK Directly** - No wrappers, use BDK for all Bitcoin operations
2. **egui UI** - Immediate mode GUI (from main branch)
3. **App-Specific Common Code** - Only non-Bitcoin logic in shared modules
4. **Mirror Mobile Structure** - Match Swift app organization for consistency
5. **Security Boundaries** - Clear separation between UI and wallet operations

---

## 📁 Proposed Structure

```
bitvault-desktop/
├── Cargo.toml                    # Workspace root
├── bitvault-app/                  # Main application (egui)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                # Entry point
│       ├── app.rs                 # Main App struct (eframe::App)
│       ├── ui/                    # UI modules (mirrors Features/)
│       │   ├── mod.rs
│       │   ├── dashboard/         # Dashboard screens
│       │   ├── vault_creation/    # Vault creation flows
│       │   ├── send_transaction/   # Send flow
│       │   ├── transaction_detail/
│       │   ├── recovery/
│       │   ├── settings/
│       │   └── components/         # Shared UI components
│       ├── services/              # Service layer (mirrors Service/)
│       │   ├── mod.rs
│       │   ├── vault_service.rs   # Uses BDK directly
│       │   ├── key_service.rs
│       │   ├── mempool_service.rs
│       │   ├── convenience_client.rs
│       │   ├── telegram_service.rs
│       │   ├── pcloud_backup.rs
│       │   └── tx_storage.rs
│       ├── models/                # Data models
│       │   ├── vault.rs
│       │   ├── transaction.rs
│       │   └── subscription.rs
│       ├── utils/                 # Utilities
│       │   ├── ur/                # UR encoding/decoding
│       │   ├── qr/                # QR code handling
│       │   └── derivation.rs
│       └── state/                 # App state
│           ├── app_state.rs
│           └── navigation.rs
│
└── bitvault-common/               # App-specific shared code (NOT Bitcoin ops)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── types.rs               # App-specific types (WalletInfo, etc.)
        ├── error.rs                # Error types
        ├── config.rs               # Configuration management
        ├── events.rs               # Event system (for UI updates)
        ├── localization.rs         # i18n support
        ├── logging.rs              # Security-aware logging
        ├── address_book.rs         # Address book feature
        ├── platform/               # Platform abstractions
        │   ├── mod.rs
        │   ├── paths.rs            # File paths
        │   └── storage.rs          # Secure storage locations
        └── utils.rs                # Minimal utilities (dust threshold, constants)
```

---

## ✅ What Goes Where

### bitvault-app (Main Application)
- **egui UI** - All UI code
- **Services** - Use BDK directly, no wrappers
- **Models** - App-specific data structures
- **State** - App state management
- **Utils** - UR/QR, derivation helpers

### bitvault-common (Shared App Logic)
- **Types** - App-specific types (not Bitcoin types)
- **Config** - Settings management
- **Events** - Event bus for UI updates
- **Platform** - Cross-platform abstractions
- **Localization** - i18n
- **Logging** - Security-aware logging
- **Address Book** - App feature
- **Utils** - Only app-specific utilities (dust threshold constant)

### ❌ NOT in bitvault-common
- ❌ UTXO selection (use BDK)
- ❌ Transaction building (use BDK)
- ❌ Fee estimation (use BDK)
- ❌ Address validation (use bitcoin crate)
- ❌ Wallet operations (use BDK directly)

---

## 🔧 Key Dependencies

### bitvault-app
```toml
[dependencies]
# GUI
eframe = "0.27"
egui = "0.27"
egui_extras = "0.27"

# Bitcoin - Use directly, no wrappers
bdk = "0.30"
bitcoin = "0.32"
secp256k1 = "0.28"

# UR/QR
ur = "0.4.1"
qrcode = "0.14"
quircs = "0.1"
ciborium = "0.2"

# Infrastructure
reqwest = "0.12"
tokio = { version = "1", features = ["full"] }
rusqlite = "0.31"
keyring = "2.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
notify-rust = "5.0"
chrono = "0.4"

# Internal
bitvault-common = { path = "../bitvault-common" }
```

### bitvault-common
```toml
[dependencies]
# Minimal dependencies - app-specific only
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
toml = "0.8"
dirs = "5.0"
zeroize = "1.7"  # For sensitive data wrappers
fluent = "0.15"  # Localization
unic-langid = "0.9"
```

---

## 🎨 Service Layer Pattern

### Example: VaultService (uses BDK directly)

```rust
// bitvault-app/src/services/vault_service.rs

use bdk::Wallet;
use bdk::database::SqliteDatabase;
use bdk::blockchain::EsploraBlockchain;
use bitcoin::Network;

pub struct VaultService {
    wallet: Wallet<SqliteDatabase>,
    // ... other state
}

impl VaultService {
    // Use BDK directly - no wrapper
    pub fn create_transaction(
        &self,
        recipient: &str,
        amount: u64,
    ) -> Result<bdk::psbt::PartiallySignedTransaction, bdk::Error> {
        let address = bitcoin::Address::from_str(recipient)?;
        
        // Use BDK's build_tx directly
        let mut builder = self.wallet.build_tx();
        builder
            .add_recipient(address.script_pubkey(), amount)
            .fee_rate(bdk::FeeRate::from_sat_per_vb(1.0));
        
        let (psbt, _details) = builder.finish()?;
        Ok(psbt)
    }
    
    // BDK handles coin selection automatically
    // No need for custom UTXO selection
}
```

---

## 📋 Implementation Phases

### Phase 1: Foundation (Days 1-3)
- [ ] Set up workspace structure
- [ ] Initialize bitvault-app with egui
- [ ] Create bitvault-common with minimal modules
- [ ] Set up basic app state and navigation

### Phase 2: Core Services (Days 4-10)
- [ ] VaultService using BDK directly
- [ ] KeyService
- [ ] MempoolService
- [ ] ConvenienceClient
- [ ] TransactionStorage (SQLite)

### Phase 3: UI Screens (Days 11-20)
- [ ] Dashboard
- [ ] Vault creation flows
- [ ] Send transaction
- [ ] Transaction history
- [ ] Settings

### Phase 4: Advanced Features (Days 21-29)
- [ ] Hardware wallet integration (UR/QR)
- [ ] Backup/recovery
- [ ] Telegram integration
- [ ] pCloud backup

### Phase 5: Polish & Testing (Days 30-43)
- [ ] Testing
- [ ] Error handling
- [ ] UI polish
- [ ] Documentation

---

## 🚫 Anti-Patterns to Avoid

1. ❌ **Don't wrap BDK** - Use it directly
2. ❌ **Don't duplicate coin selection** - BDK has it
3. ❌ **Don't reimplement fee estimation** - Use BDK's
4. ❌ **Don't create Bitcoin type wrappers** - Use bitcoin crate directly
5. ❌ **Don't put Bitcoin logic in common** - Only app-specific code

---

## ✅ Patterns to Follow

1. ✅ **Use BDK directly** in services
2. ✅ **App-specific types** in common
3. ✅ **Event system** for UI updates
4. ✅ **Platform abstractions** for cross-platform code
5. ✅ **Security-aware logging** in common
6. ✅ **Clear service boundaries** - each service has a clear purpose

---

## 🎯 Success Criteria

- ✅ Uses egui for UI
- ✅ Uses BDK directly (no wrappers)
- ✅ Clean architecture with clear boundaries
- ✅ No duplication of BDK functionality
- ✅ App-specific code in common module
- ✅ Mirrors mobile app structure
- ✅ Ready for future shared library extraction

---

**Next**: Set up dev branch with this structure



