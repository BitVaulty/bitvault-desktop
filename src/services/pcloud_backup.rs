//! PcloudBackup - pCloud backup integration
//!
//! Wrapper around ConvenienceService for pCloud backup functionality
//! API surface used by tests and pCloud backup UI (backup_management, settings)

use bitvault_common::convenience::{
    ConvenienceService, ConvenienceServiceError, PcloudBackupRequest,
};

/// pCloud backup service
/// Equivalent to Swift's PcloudBackupService
#[allow(dead_code)]
pub struct PcloudBackupService {
    convenience_service: ConvenienceService,
}

#[allow(dead_code)]
impl PcloudBackupService {
    /// Create a new pCloud backup service
    pub fn new() -> Self {
        Self {
            convenience_service: ConvenienceService::new(None),
        }
    }

    /// Create a new pCloud backup service with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            convenience_service: ConvenienceService::new(Some(base_url)),
        }
    }

    /// Create a new pCloud backup service with certificate pinning
    pub fn with_pinning(
        base_url: Option<String>,
        cert_data: Option<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            convenience_service: ConvenienceService::new_with_pinning(base_url, cert_data)?,
        })
    }

    /// Request pCloud backup
    /// POST /backup
    /// Equivalent to Swift's requestPcloudBackup
    pub async fn request_pcloud_backup(
        &self,
        request: PcloudBackupRequest,
    ) -> Result<(), PcloudBackupServiceError> {
        self.convenience_service
            .request_pcloud_backup(request)
            .await
            .map_err(|e| match e {
                ConvenienceServiceError::NetworkError(msg) => {
                    PcloudBackupServiceError::NetworkError(msg)
                }
                ConvenienceServiceError::ServerError {
                    message,
                    status_code,
                } => PcloudBackupServiceError::ServerError {
                    message,
                    status_code,
                },
                ConvenienceServiceError::ParseError(msg) => {
                    PcloudBackupServiceError::ParseError(msg)
                }
            })?;

        Ok(())
    }
}

/// Errors that can occur during pCloud backup operations
#[derive(Debug, thiserror::Error)]
pub enum PcloudBackupServiceError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Server error ({status_code}): {message}")]
    ServerError { message: String, status_code: u16 },
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Initialization failed")]
    InitializationFailed,
}

impl Default for PcloudBackupService {
    fn default() -> Self {
        Self::new()
    }
}
