# Multisig Wallet Testing Guide

**Purpose**: Test the complete multisig wallet setup flow with multiple devices/wallets

---

## Overview

BitVault uses a **2-of-3 multisig** setup:
- **Owner** (Main Device): Your primary wallet
- **Co-owner** (Second Device): Backup device or hardware wallet
- **Convenience Service**: Third party for transaction signing

This guide covers testing the complete flow from vault creation to transaction signing with multiple devices.

---

## Prerequisites

### Required Setup

1. **Two Separate Wallet Instances**
   - Main device (owner)
   - Second device (co-owner)
   - Can be same machine with different data directories

2. **Test Network**
   - Use **Testnet** for testing (recommended)
   - Mainnet requires active subscription

3. **Convenience Service**
   - Must be running and accessible
   - Testnet convenience service URL configured

4. **Test Bitcoin**
   - Get testnet Bitcoin from faucet
   - Needed for transaction testing

---

## Testing Setup Options

### Option 1: Two Separate Data Directories (Same Machine)

**Best for**: Local development and testing

```bash
# Main device (owner)
BITVAULT_DATA_DIR=/tmp/bitvault-main cargo run

# Second device (co-owner) - in separate terminal
BITVAULT_DATA_DIR=/tmp/bitvault-coowner cargo run
```

### Option 2: Two Separate Machines

**Best for**: Real-world scenario testing

1. Run BitVault on Machine A (main device)
2. Run BitVault on Machine B (second device)
3. Transfer QR codes between machines

### Option 3: Desktop + Mobile

**Best for**: Cross-platform testing

1. Use desktop app as main device
2. Use mobile app as co-owner device
3. Transfer QR codes via camera/display

---

## Step-by-Step Testing Procedure

### Step 1: Main Device Setup (Owner)

#### Step 1: Start Main Device

```bash
# Set data directory for main device
export BITVAULT_DATA_DIR=/tmp/bitvault-main

# Run application
cargo run
```

#### Step 2: Create Vault on Main Device

1. **Select Network**: Choose **Testnet** (recommended for testing)
2. **Create New Vault**: Click "Create New Vault"
3. **Generate Mnemonic**: 
   - Generate new mnemonic
   - **IMPORTANT**: Save the mnemonic securely for testing
4. **Display Seed Phrase**: Verify seed phrase is displayed correctly
5. **Verify Seed Phrase**: Enter seed phrase to verify
6. **Set Time Delay**: 
   - Set time delay (e.g., 1 day for testing)
   - Note: Shorter delays = faster testing
7. **Set PIN**: Create a PIN for the vault
8. **Generate Co-owner QR**: 
   - This step generates QR code with co-owner public keys
   - **SAVE THIS QR CODE** - You'll need it for the second device

#### Step 3: Save QR Code

**Option A: Save QR Code Image**
- Screenshot the QR code
- Save to file: `main-device-qr.png`

**Option B: Copy QR Code String**
- If displayed as text, copy the QR string
- Save to file: `main-device-qr.txt`

**Option C: Export QR Data**
- The QR contains co-owner public keys
- Format: Compressed QR string with keys and timelock

---

### Step 2: Second Device Setup (Co-owner)

#### Step 1: Start Second Device

```bash
# Set data directory for second device
export BITVAULT_DATA_DIR=/tmp/bitvault-coowner

# Run application (in separate terminal)
cargo run
```

#### Step 2: Import as Co-owner

1. **Select Network**: Choose **Testnet** (must match main device)
2. **Import Vault**: Click "Import Vault"
3. **Select "This is a coowner device"**: Check the co-owner option
4. **Enter Mnemonic**: 
   - Generate NEW mnemonic for co-owner device
   - **IMPORTANT**: This is different from main device mnemonic
5. **Scan QR Code**: 
   - Use camera to scan QR from main device
   - OR load QR image file
   - OR paste QR string manually
6. **Verify Setup**: 
   - Verify co-owner keys are decoded correctly
   - Verify timelock matches main device

#### Step 3: Complete Co-owner Setup

1. **Set Time Delay**: Should match main device (from QR)
2. **Set PIN**: Create PIN for co-owner device
3. **Link Co-owner**: 
   - Enter email (same as main device)
   - Enter auth code (from email)
   - Link co-owner to vault

---

### Step 3: Verify Multisig Setup

#### Check 1: Vault Addresses Match

**On Main Device**:
- Note the vault address (should be displayed after creation)

**On Second Device**:
- Note the vault address (should match main device)

**Verify**: Both addresses should be **identical**

#### Check 2: Descriptors Match

**On Main Device**:
- Check vault descriptor (if accessible in UI)
- Should show 2-of-3 multisig descriptor

**On Second Device**:
- Check vault descriptor
- Should show same 2-of-3 multisig descriptor

**Verify**: Descriptors should match (same script, same keys)

#### Check 3: Convenience Service Registration

**Check Logs**:
- Both devices should show successful registration
- Vault should be registered on convenience service

**Verify**: 
- Vault registered on convenience service
- Both owner and co-owner linked

---

### Step 4: Test Transactions

#### Test 1: Receive Transaction

1. **Get Testnet Bitcoin**:
   - Use testnet faucet: https://testnet-faucet.mempool.co/
   - Send to vault address

2. **Check Balance on Both Devices**:
   - Main device: Check balance
   - Second device: Check balance
   - **Verify**: Both show same balance

#### Test 2: Send Transaction (Main Device)

1. **On Main Device**:
   - Navigate to "Send Transaction"
   - Enter recipient address
   - Enter amount
   - Review transaction
   - Sign transaction

2. **Transaction Flow**:
   - Main device signs PSBT
   - PSBT sent to convenience service
   - Convenience service adds signature
   - Transaction broadcast

3. **Verify**:
   - Transaction appears in history on both devices
   - Balance updates on both devices

#### Test 3: Send Transaction (Second Device)

1. **On Second Device**:
   - Navigate to "Send Transaction"
   - Enter recipient address
   - Enter amount
   - Review transaction
   - Sign transaction

2. **Verify**:
   - Transaction works from co-owner device
   - Both devices show updated balance

---

## Testing Scenarios

### Scenario 1: Standard Two-Device Setup

**Setup**:
- Main device: Desktop app
- Second device: Desktop app (different data dir)

**Test**:
- ✅ Create vault on main device
- ✅ Import as co-owner on second device
- ✅ Send transaction from main device
- ✅ Send transaction from second device
- ✅ Verify both devices show same balance

### Scenario 2: Desktop + Mobile

**Setup**:
- Main device: Desktop app
- Second device: Mobile app

**Test**:
- ✅ Create vault on desktop
- ✅ Import as co-owner on mobile
- ✅ Cross-platform transaction signing
- ✅ Verify synchronization

### Scenario 3: Hardware Wallet as Co-owner

**Setup**:
- Main device: Desktop app
- Second device: Hardware wallet (via mobile/desktop)

**Test**:
- ✅ Create vault on main device
- ✅ Link hardware wallet as co-owner
- ✅ Sign transactions with hardware wallet
- ✅ Verify hardware wallet integration

### Scenario 4: Time Delay Testing

**Setup**:
- Set time delay to 1 hour (for testing)

**Test**:
- ✅ Create transaction
- ✅ Wait for time delay
- ✅ Verify transaction completes after delay
- ✅ Test time delay cancellation

---

## Troubleshooting

### Issue: QR Code Not Scanning

**Solutions**:
1. **Use File Import**: Save QR as image, import via file
2. **Manual Entry**: Copy QR string, paste manually
3. **Check Format**: Verify QR format matches expected

### Issue: Vault Addresses Don't Match

**Causes**:
- Different networks (mainnet vs testnet)
- Different descriptors
- QR code not decoded correctly

**Solutions**:
1. Verify both devices use same network
2. Verify QR code decoded correctly
3. Check descriptors match

### Issue: Transactions Not Appearing

**Causes**:
- Convenience service not responding
- Network connectivity issues
- Transaction not broadcast

**Solutions**:
1. Check convenience service logs
2. Verify network connectivity
3. Check transaction on block explorer

### Issue: Balance Not Syncing

**Causes**:
- Wallet not syncing
- Different networks
- Esplora API issues

**Solutions**:
1. Force wallet sync
2. Verify network matches
3. Check Esplora API status

---

## Automated Testing Script

### Manual Testing Checklist

```bash
# Main Device Setup
[ ] Start main device with separate data dir
[ ] Create vault on testnet
[ ] Generate mnemonic
[ ] Verify seed phrase
[ ] Set time delay
[ ] Set PIN
[ ] Generate co-owner QR
[ ] Save QR code

# Second Device Setup
[ ] Start second device with separate data dir
[ ] Import vault as co-owner
[ ] Generate new mnemonic
[ ] Scan/import QR from main device
[ ] Verify co-owner keys
[ ] Set PIN
[ ] Link co-owner

# Verification
[ ] Verify vault addresses match
[ ] Verify descriptors match
[ ] Verify convenience service registration

# Transaction Testing
[ ] Receive testnet Bitcoin
[ ] Verify balance on both devices
[ ] Send transaction from main device
[ ] Verify transaction on both devices
[ ] Send transaction from second device
[ ] Verify transaction on both devices
```

---

## Test Data Directory Structure

```
/tmp/
├── bitvault-main/          # Main device data
│   ├── vaults/
│   │   └── <vault-id>/
│   │       ├── descriptor_mainnet.txt
│   │       ├── descriptor_testnet.txt
│   │       └── mnemonic.txt (encrypted)
│   └── settings.json
│
└── bitvault-coowner/        # Second device data
    ├── vaults/
    │   └── <vault-id>/
    │       ├── descriptor_mainnet.txt
    │       ├── descriptor_testnet.txt
    │       └── mnemonic.txt (encrypted)
    └── settings.json
```

---

## Environment Variables

```bash
# Main Device
export BITVAULT_DATA_DIR=/tmp/bitvault-main
export BITVAULT_NETWORK=testnet

# Second Device
export BITVAULT_DATA_DIR=/tmp/bitvault-coowner
export BITVAULT_NETWORK=testnet
```

---

## Quick Test Script

```bash
#!/bin/bash

# Test multisig wallet setup

echo "=== Multisig Wallet Testing ==="

# Main device setup
echo "1. Starting main device..."
BITVAULT_DATA_DIR=/tmp/bitvault-main cargo run &
MAIN_PID=$!

sleep 5

echo "2. Main device started (PID: $MAIN_PID)"
echo "   - Create vault on main device"
echo "   - Generate co-owner QR"
echo "   - Save QR code to /tmp/main-qr.png"

read -p "Press Enter when main device setup is complete..."

# Second device setup
echo "3. Starting second device..."
BITVAULT_DATA_DIR=/tmp/bitvault-coowner cargo run &
COOWNER_PID=$!

sleep 5

echo "4. Second device started (PID: $COOWNER_PID)"
echo "   - Import vault as co-owner"
echo "   - Scan QR from main device"
echo "   - Complete co-owner setup"

read -p "Press Enter when second device setup is complete..."

echo "5. Testing complete!"
echo "   - Main device PID: $MAIN_PID"
echo "   - Second device PID: $COOWNER_PID"
echo "   - Test transactions from both devices"
```

---

## Expected Results

### Successful Setup

✅ **Main Device**:
- Vault created successfully
- QR code generated
- Vault address displayed
- Convenience service registered

✅ **Second Device**:
- QR code scanned successfully
- Co-owner keys decoded
- Vault imported successfully
- Convenience service linked

✅ **Both Devices**:
- Same vault address
- Same descriptors
- Same balance
- Transactions sync

---

## Next Steps

After successful multisig setup testing:

1. **Test Recovery Scenarios**:
   - Test recovery with one device
   - Test recovery with time delay
   - Test recovery with convenience service

2. **Test Edge Cases**:
   - Network switching (mainnet/testnet)
   - Time delay expiration
   - Transaction cancellation

3. **Test Integration**:
   - Hardware wallet integration
   - Mobile app integration
   - Cross-platform testing

---

## Notes

- **Always use Testnet** for initial testing
- **Save mnemonics securely** for testing
- **Use short time delays** for faster testing
- **Keep QR codes** for reference
- **Test both devices** can send transactions
- **Verify synchronization** between devices

---
