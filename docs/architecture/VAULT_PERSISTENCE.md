# Vault Persistence and Management System

## Overview

The BitVault desktop application now includes a complete vault persistence and management system that allows users to create, manage, and switch between multiple Bitcoin vaults.

## Features

### Core Persistence

1. **SQLite Database Storage**
   - Vaults are stored in SQLite databases using BDK's `SqliteDatabase`
   - Database paths follow mobile app structure: `{data_dir}/bitvault/vault_data/{vault_address}/wallet_{network}.sqlite`
   - Vaults persist across app restarts

2. **Vault Metadata Storage**
   - Metadata stored as JSON files in `{data_dir}/bitvault/vaults/`
   - Includes: name, network, address, database_path, descriptor, created_at
   - Automatic metadata saving during vault creation

### Vault Management Operations

1. **Create** - New vaults via vault creation flow
2. **Read** - List and view all vaults
3. **Update** - Rename vaults
4. **Delete** - Remove vaults completely (metadata + database)
5. **Switch** - Change between vaults
6. **Import** - Restore vault metadata from JSON
7. **Export** - Backup vault metadata as JSON

### Vault Validation

- Validates vault metadata integrity
- Checks database file existence
- Verifies descriptor presence
- Visual indicators show vault health status
- Prevents loading invalid vaults

## Architecture

### File Structure

```
{data_dir}/bitvault/
├── vaults/                    # Vault metadata (JSON files)
│   └── {sanitized_address}.json
└── vault_data/                # Vault databases
    └── {vault_address}/
        └── wallet_{network}.sqlite
```

### Key Components

1. **`VaultMetadata`** (`bitvault-common/src/wallet/vault_metadata.rs`)
   - Stores vault metadata
   - Handles save/load/delete operations
   - Provides validation methods

2. **`VaultService`** (`bitvault-common/src/wallet/vault_service.rs`)
   - Manages wallet operations
   - Handles vault setup and loading
   - Integrates with SQLite database

3. **Vault Selection UI** (`bitvault-desktop/src/ui/vault_selection/`)
   - Displays list of available vaults
   - Provides vault management operations
   - Shows validation status

## Usage

### Creating a Vault

1. Navigate to "Create New Vault"
2. Complete vault creation flow
3. Vault is automatically saved with metadata

### Loading a Vault

1. App starts with vault selection screen
2. Select a vault from the list
3. Click "Load Selected Vault"
4. Vault is loaded and user navigates to dashboard

### Switching Vaults

1. From dashboard settings tab, click "Switch Vault"
2. Or from vault detail tab, click "Switch Vault"
3. Select a different vault and load it

### Exporting a Vault

1. Select a vault in vault selection screen
2. Click "Export Vault"
3. Metadata is copied to clipboard as JSON
4. Save the JSON file for backup

### Importing a Vault

1. Click "Import Vault" in vault selection screen
2. Paste vault metadata JSON
3. System validates and saves metadata
4. Manually copy database file to specified path

### Deleting a Vault

1. Select a vault in vault selection screen
2. Click "Delete Vault"
3. Both metadata and database are removed

## Validation

Vaults are validated before loading to ensure:
- All required fields are present
- Database file exists
- Descriptor is available

Visual indicators:
- ✓ = Valid vault
- ⚠ = Database exists but has issues
- ✗ = Database missing

## Error Handling

- Clear error messages for validation failures
- Prevents loading invalid vaults
- Shows validation status in UI

## Implementation Status

✅ **Completed Features:**
- SQLite database persistence
- Vault metadata storage and management
- Vault listing, loading, and switching
- Vault creation with automatic persistence
- Vault renaming and deletion
- Vault import/export (metadata)
- Vault validation and health checks
- Complete vault deletion (metadata + database)
- Transaction history display
- Recent transactions in vault detail view

## Potential Enhancements

- [ ] Automatic database backup/restore
- [ ] Vault encryption at rest
- [ ] Vault sync between devices
- [ ] Batch vault operations
- [ ] Vault search/filter functionality
- [ ] Vault tags/categories
- [ ] Transaction filtering and sorting
- [ ] Vault statistics and analytics

