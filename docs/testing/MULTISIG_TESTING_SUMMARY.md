# Multisig Wallet Testing Summary

**Quick Reference**: How to test multisig wallet setup with multiple devices

---

## The Problem

You've set up the multisig wallet creation flow, but haven't tested it with **multiple actual wallets/devices**. You need to verify:

1. ✅ Main device can create vault
2. ✅ Second device can import as co-owner
3. ✅ Both devices share same vault
4. ✅ Transactions work from both devices
5. ✅ Balance syncs between devices

---

## Solution: Two Separate Data Directories

**Key Insight**: You can run two instances of the app on the same machine using different data directories.

### Quick Start

```bash
# Terminal 1 - Main Device (Owner)
export BITVAULT_DATA_DIR=/tmp/bitvault-main
cargo run

# Terminal 2 - Second Device (Co-owner)
export BITVAULT_DATA_DIR=/tmp/bitvault-coowner
cargo run
```

---

## Complete Testing Flow

### Phase 1: Main Device (Owner)

1. **Start App**: `BITVAULT_DATA_DIR=/tmp/bitvault-main cargo run`
2. **Select Network**: Testnet (recommended)
3. **Create Vault**:
   - Generate mnemonic
   - Verify seed phrase
   - Set time delay
   - Set PIN
4. **Generate Co-owner QR**:
   - This creates QR with co-owner public keys
   - **SAVE THIS QR** (screenshot or copy string)

### Phase 2: Second Device (Co-owner)

1. **Start App**: `BITVAULT_DATA_DIR=/tmp/bitvault-coowner cargo run`
2. **Select Network**: Testnet (must match)
3. **Import Vault**:
   - Check "This is a coowner device"
   - Generate NEW mnemonic (different from main)
   - Scan/import QR from main device
   - Set PIN
   - Link co-owner (email + auth code)

### Phase 3: Verification

**Check Both Devices**:
- ✅ Vault addresses match
- ✅ Descriptors match
- ✅ Both show same balance
- ✅ Both can send transactions

---

## Testing Transactions

### 1. Receive Testnet Bitcoin

- Use faucet: https://testnet-faucet.mempool.co/
- Send to vault address
- Check balance on both devices

### 2. Send from Main Device

- Create transaction on main device
- Sign and broadcast
- Verify on both devices

### 3. Send from Second Device

- Create transaction on second device
- Sign and broadcast
- Verify on both devices

---

## Key Testing Points

### ✅ Must Match

- **Network**: Both devices must use same network
- **Vault Address**: Should be identical
- **Descriptors**: Should match exactly
- **Balance**: Should sync between devices

### ✅ Should Work

- **QR Scanning**: Camera or file import
- **Transaction Signing**: From both devices
- **Balance Updates**: Real-time sync
- **Convenience Service**: Both devices linked

---

## Troubleshooting

### QR Code Issues

**Problem**: Can't scan QR code
**Solution**: 
- Use file import
- Copy QR string manually
- Verify QR format

### Address Mismatch

**Problem**: Vault addresses don't match
**Solution**:
- Check network matches (testnet/testnet)
- Verify QR decoded correctly
- Check descriptors

### Transaction Issues

**Problem**: Transactions not working
**Solution**:
- Check convenience service running
- Verify network connectivity
- Check transaction on block explorer

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

## Files Created

- `MULTISIG_WALLET_TESTING.md` - Complete detailed guide
- `QUICK_MULTISIG_TEST.md` - Quick reference
- `MULTISIG_TESTING_SUMMARY.md` - This summary

---

## Next Steps

1. **Run Quick Test**: Follow `QUICK_MULTISIG_TEST.md`
2. **Full Testing**: Follow `MULTISIG_WALLET_TESTING.md`
3. **Verify Results**: Check all verification points
4. **Test Transactions**: Send from both devices

---

**You're ready to test!** Start with the quick test guide.
