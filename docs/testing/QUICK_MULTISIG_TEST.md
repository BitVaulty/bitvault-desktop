# Quick Multisig Testing Guide

**Quick reference for testing multisig wallet setup**

---

## Quick Setup (5 Minutes)

### 1. Terminal 1 - Main Device

```bash
# Set data directory
export BITVAULT_DATA_DIR=/tmp/bitvault-main

# Run app
cd bitvault-desktop/bitvault-app
cargo run
```

**Steps in UI**:
1. Select **Testnet**
2. Click **"Create New Vault"**
3. Generate mnemonic → Verify → Set time delay → Set PIN
4. **Generate Co-owner QR** → **SAVE QR CODE** (screenshot or copy string)

### 2. Terminal 2 - Second Device

```bash
# Set different data directory
export BITVAULT_DATA_DIR=/tmp/bitvault-coowner

# Run app
cd bitvault-desktop/bitvault-app
cargo run
```

**Steps in UI**:
1. Select **Testnet** (must match main device)
2. Click **"Import Vault"**
3. Check **"This is a coowner device"**
4. Generate NEW mnemonic → Verify → Set PIN
5. **Scan/Import QR** from main device
6. Enter email + auth code → Link co-owner

### 3. Verify Setup

**Check Both Devices**:
- ✅ Vault addresses should match
- ✅ Both show same balance
- ✅ Both can send transactions

---

## Testing Transactions

### Receive Testnet Bitcoin

1. Get testnet Bitcoin: https://testnet-faucet.mempool.co/
2. Send to vault address
3. Check balance on both devices (should match)

### Send Transaction

**From Main Device**:
- Send Transaction → Enter address → Sign → Broadcast
- Verify transaction appears on both devices

**From Second Device**:
- Send Transaction → Enter address → Sign → Broadcast
- Verify transaction appears on both devices

---

## Troubleshooting

**QR Code Issues**:
- Use file import instead of camera
- Copy QR string manually
- Verify QR format

**Address Mismatch**:
- Check both devices use same network (testnet)
- Verify QR decoded correctly
- Check descriptors match

**Transaction Issues**:
- Check convenience service is running
- Verify network connectivity
- Check transaction on block explorer

---

## Clean Up

```bash
# Remove test data
rm -rf /tmp/bitvault-main
rm -rf /tmp/bitvault-coowner
```

---

**That's it!** You now have a working multisig wallet with two devices.
