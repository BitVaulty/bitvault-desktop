# Simple Two-Instance Testing

**Yes!** You can run 2 instances of the app on the same machine.

---

## Quick Setup

### Terminal 1 - Main Device (Owner)

```bash
cd /home/user/src/bitvault-org/bitvault-desktop/bitvault-app
BITVAULT_DATA_DIR=/tmp/bitvault-main cargo run
```

### Terminal 2 - Second Device (Co-owner)

```bash
cd /home/user/src/bitvault-org/bitvault-desktop/bitvault-app
BITVAULT_DATA_DIR=/tmp/bitvault-coowner cargo run
```

**That's it!** Two separate instances, two separate data directories.

---

## What Happens

1. **First Instance** (Main Device):
   - Uses `/tmp/bitvault-main` for data
   - Creates vault
   - Generates QR code

2. **Second Instance** (Co-owner Device):
   - Uses `/tmp/bitvault-coowner` for data
   - Imports vault
   - Scans QR from first instance

3. **Both Instances**:
   - Run independently
   - Have separate vault storage
   - Can communicate via QR codes
   - Test multisig setup

---

## Testing Flow

### In First Instance (Main Device):

1. Select **Testnet**
2. Click **"Create New Vault"**
3. Generate mnemonic → Verify → Set time delay → Set PIN
4. **Generate Co-owner QR** → **SAVE QR** (screenshot or copy)

### In Second Instance (Co-owner Device):

1. Select **Testnet** (must match!)
2. Click **"Import Vault"**
3. Check **"This is a coowner device"**
4. Generate NEW mnemonic → Verify → Set PIN
5. **Scan/Import QR** from first instance
6. Enter email + auth code → Link co-owner

### Verify:

- ✅ Both instances show same vault address
- ✅ Both instances show same balance
- ✅ Can send transactions from both instances

---

## Tips

- **Use Testnet**: Safer for testing
- **Save QR Code**: Screenshot or copy the QR string
- **Same Network**: Both instances must use same network
- **Different Mnemonics**: Co-owner uses different mnemonic

---

## Clean Up

When done testing:

```bash
rm -rf /tmp/bitvault-main
rm -rf /tmp/bitvault-coowner
```

---

**That's all you need!** Just two terminal windows, two different data directories.
