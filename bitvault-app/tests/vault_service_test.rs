//! Tests for VaultService integration

#[cfg(test)]
mod tests {
    use bdk::bitcoin::Network;
    use bitvault_common::wallet::VaultService;
    use tempfile::TempDir;

    fn create_temp_db_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    #[tokio::test]
    async fn test_vault_service_creation() {
        let service = VaultService::new(Network::Testnet);
        assert!(!service.is_loaded());
        assert_eq!(service.network(), Network::Testnet);
    }

    #[tokio::test]
    async fn test_vault_service_get_address_before_init() {
        let service = VaultService::new(Network::Testnet);
        // get_address should return an error if wallet not initialized
        let result = service.get_address();
        assert!(result.is_err());
        match result.unwrap_err() {
            bitvault_common::BitVaultError::Config(msg) => {
                assert!(msg.contains("Wallet not initialized"));
            }
            e => panic!("Expected Config error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_vault_service_get_balance_before_init() {
        let service = VaultService::new(Network::Testnet);
        // get_balance should return an error if wallet not initialized
        let result = service.get_balance().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            bitvault_common::BitVaultError::Config(msg) => {
                assert!(msg.contains("Wallet not initialized"));
            }
            e => panic!("Expected Config error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_vault_service_network() {
        let service_mainnet = VaultService::new(Network::Bitcoin);
        assert_eq!(service_mainnet.network(), Network::Bitcoin);

        let service_testnet = VaultService::new(Network::Testnet);
        assert_eq!(service_testnet.network(), Network::Testnet);
    }

    #[tokio::test]
    async fn test_vault_service_with_database_path() {
        let temp_dir = create_temp_db_dir();
        let db_path = temp_dir
            .path()
            .join("test_wallet.db")
            .to_string_lossy()
            .to_string();

        let service = VaultService::with_database_path(Network::Testnet, db_path.clone());
        assert_eq!(service.network(), Network::Testnet);
        // Database path should be set (internal state, not directly testable)
    }
}
