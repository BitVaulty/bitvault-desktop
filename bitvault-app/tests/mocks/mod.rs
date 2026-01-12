//! Mock infrastructure for testing ConvenienceService integration
//!
//! Provides mock HTTP server and helpers for testing ConvenienceService calls
//! without requiring a real backend server.

use serde::{Deserialize, Serialize};

/// Helper to create a mock ConvenienceService with a test base URL
pub fn create_mock_convenience_service(base_url: String) -> bitvault_common::convenience::ConvenienceService {
    bitvault_common::convenience::ConvenienceService::new(Some(base_url))
}

/// Mock response types matching ConvenienceService responses
#[derive(Debug, Serialize, Deserialize)]
pub struct MockPubkeyResponse {
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockServiceResponse {
    pub message: Option<String>,
    pub error: Option<String>,
    pub txid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockTelegramInitResponse {
    pub link: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockTelegramStatusResponse {
    pub registered: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockTelegramUnsubscribeResponse {
    pub message: String,
}

/// Helper to create a successful pubkey response
pub fn create_pubkey_response(pubkey: &str) -> MockPubkeyResponse {
    MockPubkeyResponse {
        pubkey: pubkey.to_string(),
    }
}

/// Helper to create a successful service response
pub fn create_service_response(message: Option<String>, txid: Option<String>) -> MockServiceResponse {
    MockServiceResponse {
        message,
        error: None,
        txid,
    }
}

/// Helper to create an error service response
pub fn create_error_response(message: String) -> MockServiceResponse {
    MockServiceResponse {
        message: None,
        error: Some(message),
        txid: None,
    }
}

/// Helper to create a Telegram init response
pub fn create_telegram_init_response(link: &str) -> MockTelegramInitResponse {
    MockTelegramInitResponse {
        link: link.to_string(),
    }
}

/// Helper to create a Telegram status response
pub fn create_telegram_status_response(registered: bool) -> MockTelegramStatusResponse {
    MockTelegramStatusResponse { registered }
}

/// Helper to create a Telegram unsubscribe response
pub fn create_telegram_unsubscribe_response(message: &str) -> MockTelegramUnsubscribeResponse {
    MockTelegramUnsubscribeResponse {
        message: message.to_string(),
    }
}
