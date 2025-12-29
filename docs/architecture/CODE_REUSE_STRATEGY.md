# Code Reuse Strategy - Shared Rust Core Library

**Goal**: Extract shared business logic into `bitvault-core` that both mobile (Swift) and desktop (Rust) apps can use.

---

## Architecture

```
bitvault-core/ (Rust library)
├── wallet/          # Core wallet operations (uses BDK)
├── psbt/            # PSBT creation/signing/validation
├── ur/              # UR encoding/decoding
├── derivation/      # Key derivation (BIP32/BIP44)
├── models/          # Shared data models
├── convenience/     # Convenience service client
└── utils/           # Shared utilities

bitvault-mobile/ (Swift)
├── Uses bitvault-core via FFI (C bindings)
└── SwiftUI UI layer

bitvault-desktop/ (Rust + egui)
├── Uses bitvault-core as Rust dependency
└── egui UI layer
```

---

## What Gets Shared

### ✅ Shared in bitvault-core:
1. **Wallet Operations** - VaultService logic (using BDK)
2. **PSBT Handling** - Creation, signing, validation
3. **UR Encoding/Decoding** - Crypto-Account, Crypto-PSBT
4. **Key Derivation** - BIP32/BIP44 paths, mnemonic handling
5. **Convenience Service Client** - HTTP client for API calls
6. **Transaction Building** - Using BDK's build_tx
7. **Descriptor Handling** - Parsing, validation
8. **Shared Models** - Vault, Transaction, Subscription types

### ❌ NOT Shared (Platform-Specific):
1. **UI Code** - SwiftUI vs egui
2. **Storage** - Keychain vs platform secure storage
3. **Biometrics** - Touch ID vs Windows Hello
4. **Backup** - iCloud vs desktop file system
5. **Notifications** - iOS notifications vs desktop notifications

---

## Implementation Approach

### Phase 1: Build Desktop App with Shared Logic in Mind
- Implement services in desktop app
- Design them to be extractable later
- Use BDK directly (already shared foundation)

### Phase 2: Extract to bitvault-core
- Create `bitvault-core` crate
- Move shared logic from desktop app
- Keep it as pure Rust library

### Phase 3: Desktop App Uses Core
- Refactor desktop app to use `bitvault-core`
- Desktop app becomes thin UI layer

### Phase 4: Mobile App Uses Core via FFI
- Create C bindings for `bitvault-core`
- Create Swift Package wrapper
- Refactor mobile app to use core

---

## FFI Strategy for Mobile

### Option 1: C Bindings (Recommended)
- Use `cbindgen` to generate C headers
- Swift imports C functions
- Simple, well-supported

### Option 2: Swift Package with C Interop
- Create Swift Package that wraps C bindings
- Cleaner Swift API
- More setup but better DX

---

## Benefits

1. **Single Source of Truth** - Business logic in one place
2. **Bug Fixes** - Fix once, benefits both apps
3. **Feature Parity** - Easier to keep in sync
4. **Testing** - Test core logic once
5. **Security** - Security-critical code in one well-audited place

---

## Next Steps

1. Continue building desktop app services
2. Design them to be extractable
3. Once desktop app works, extract shared parts to `bitvault-core`
4. Then add FFI for mobile app



