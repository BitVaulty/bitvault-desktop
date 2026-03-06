//! TelegramService - Telegram integration
//!
//! Wrapper around ConvenienceService for Telegram notification functionality
//! API surface used by tests and future Telegram notification UI

use bitvault_common::convenience::{ConvenienceService, ConvenienceServiceError};

/// Telegram service
/// Equivalent to Swift's TelegramService
#[allow(dead_code)]
pub struct TelegramService {
    convenience_service: ConvenienceService,
}

#[allow(dead_code)]
impl TelegramService {
    /// Create a new Telegram service
    pub fn new() -> Self {
        Self {
            convenience_service: ConvenienceService::new(None),
        }
    }

    /// Create a new Telegram service with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            convenience_service: ConvenienceService::new(Some(base_url)),
        }
    }

    /// Create a new Telegram service with certificate pinning
    pub fn with_pinning(
        base_url: Option<String>,
        cert_data: Option<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            convenience_service: ConvenienceService::new_with_pinning(base_url, cert_data)?,
        })
    }

    /// Request Telegram registration
    /// POST /tg/init
    /// Equivalent to Swift's requestTelegramRegistration
    pub async fn request_telegram_registration(
        &self,
        address: &str,
        pubkey: &str,
        message: &str,
        signature: &str,
    ) -> Result<String, TelegramServiceError> {
        self.convenience_service
            .request_telegram_registration(address, pubkey, message, signature)
            .await
            .map_err(|e| match e {
                ConvenienceServiceError::NetworkError(msg) => {
                    TelegramServiceError::NetworkError(msg)
                }
                ConvenienceServiceError::ServerError {
                    message,
                    status_code,
                } => TelegramServiceError::ServerError {
                    message,
                    status_code,
                },
                ConvenienceServiceError::ParseError(msg) => TelegramServiceError::ParseError(msg),
            })
    }

    /// Check Telegram registration status
    /// POST /tg/status
    /// Equivalent to Swift's checkTgRegistration
    pub async fn check_tg_registration(
        &self,
        pubkey: &str,
        address: &str,
    ) -> Result<bool, TelegramServiceError> {
        self.convenience_service
            .check_telegram_registration(pubkey, address)
            .await
            .map_err(|e| match e {
                ConvenienceServiceError::NetworkError(msg) => {
                    TelegramServiceError::NetworkError(msg)
                }
                ConvenienceServiceError::ServerError {
                    message,
                    status_code,
                } => TelegramServiceError::ServerError {
                    message,
                    status_code,
                },
                ConvenienceServiceError::ParseError(msg) => TelegramServiceError::ParseError(msg),
            })
    }

    /// Unsubscribe from Telegram notifications
    /// POST /notifications/unsubscribe
    /// Equivalent to Swift's unsubscribe
    pub async fn unsubscribe(
        &self,
        address: &str,
        pubkey: &str,
        message: &str,
        signature: &str,
    ) -> Result<String, TelegramServiceError> {
        self.convenience_service
            .unsubscribe_telegram(address, pubkey, message, signature)
            .await
            .map_err(|e| match e {
                ConvenienceServiceError::NetworkError(msg) => {
                    TelegramServiceError::NetworkError(msg)
                }
                ConvenienceServiceError::ServerError {
                    message,
                    status_code,
                } => TelegramServiceError::ServerError {
                    message,
                    status_code,
                },
                ConvenienceServiceError::ParseError(msg) => TelegramServiceError::ParseError(msg),
            })
    }
}

/// Errors that can occur during Telegram operations
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum TelegramServiceError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Server error ({status_code}): {message}")]
    ServerError { message: String, status_code: u16 },
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl Default for TelegramService {
    fn default() -> Self {
        Self::new()
    }
}
