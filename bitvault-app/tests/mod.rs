//! Test infrastructure for bitvault-app

mod e2e_test;
mod error_handling_test;
mod key_service_test;
mod navigation_test;
mod pin_e2e_test;
mod pin_state_machine_e2e_test;
mod vault_creation_e2e_test;
mod vault_creation_state_machine_e2e_test;
mod transaction_signing_e2e_test;
mod send_transaction_flow_e2e_test;
mod recovery_state_machine_e2e_test;
mod backup_recovery_e2e_test;
mod pcloud_backup_test;
mod services_test;
mod telegram_service_test;
mod tx_storage_test;
mod vault_service_test;
mod convenience_service_integration_test;#[cfg(test)]
mod mocks;